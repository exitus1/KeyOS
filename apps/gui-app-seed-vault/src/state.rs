// SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

#[cfg(not(test))]
use slint_keyos_platform::file_backed::JsonBacked;
use {
    crate::{
        error::VaultError,
        fs_permissions::FileSystemPermissions,
        seed::{Seed, SeedDuplicateReason, SeedEditField, SeedType},
        AccountsParams, Animate, AppWindow, Callbacks, GuiApi, Navigate, NavigateOptions, SeedView,
        SeedViewType,
    },
    bip39::Mnemonic,
    fuzzy_filter::FuzzyFilter,
    ngwallet::{
        bdk_wallet::bitcoin::{
            secp256k1::{Secp256k1, SignOnly},
            Network,
        },
        bip39::{MasterKey, WordCount},
    },
    nostr::{nips::nip06::FromMnemonic, ToBech32},
    ordered_table::{CardSortMode, FilePersistence, OrderedTable, SortableCard},
    slint_keyos_platform::slint::{self, ComponentHandle, Image, ModelRc, VecModel},
    std::{rc::Rc, sync::Arc},
};

security::use_api!();

pub const DATABASE_FILE: &str = "seed_vault_database_v2.json";

#[derive(serde::Serialize, serde::Deserialize)]
struct VaultSettings {
    sort_mode: CardSortMode,
}

impl Default for VaultSettings {
    fn default() -> Self { Self { sort_mode: CardSortMode::Label } }
}

pub struct AppState {
    pub ui: slint::Weak<AppWindow>,
    pub gui: Arc<GuiApi>,
    seed_table: OrderedTable<Seed, FilePersistence<FileSystemPermissions>>,
    pub search_text: String,
    // new_seed: Option<Seed>,
    pub archive_mode: bool,
    secp: Secp256k1<SignOnly>,
    security_api: Security,
    model: Rc<VecModel<SeedView>>,
    #[cfg(not(test))]
    settings: JsonBacked<VaultSettings, FileSystemPermissions>,
    #[cfg(test)]
    sort_mode: CardSortMode,
}

impl AppState {
    pub fn new(gui: Arc<GuiApi>, ui: slint::Weak<AppWindow>) -> Self {
        // All errors encountered here are unrecoverable.
        // The app cannot function without seed_table and settings.
        Self {
            ui,
            gui,
            seed_table: OrderedTable::new()
                .with_persistence(FilePersistence::new(String::from(DATABASE_FILE), fs::Location::AppData))
                .expect("failed to create seed vault database"),
            search_text: String::new(),
            // new_seed: None,
            archive_mode: false,
            secp: Secp256k1::signing_only(),
            security_api: Security::default(),
            model: Rc::new(VecModel::default()),
            #[cfg(not(test))]
            settings: JsonBacked::new("settings.json", fs::Location::AppData).0,
            #[cfg(test)]
            sort_mode: CardSortMode::Label,
        }
    }

    pub fn get_sort_mode(&self) -> CardSortMode {
        #[cfg(not(test))]
        return self.settings.sort_mode.clone();
        #[cfg(test)]
        return self.sort_mode;
    }

    #[cfg(not(test))]
    pub fn set_sort_mode(&mut self, mode: CardSortMode) { self.settings.guard().sort_mode = mode; }

    #[cfg(test)]
    pub fn set_sort_mode(&mut self, mode: CardSortMode) { self.sort_mode = mode; }

    pub fn is_empty(&self) -> bool { self.seed_table.is_empty() }

    pub fn ui(&self) -> AppWindow { self.ui.unwrap() }

    pub fn validate_new_label(&self, label: String) -> Result<(), VaultError> {
        SeedEditField::Label(label.clone()).validate()?;

        if let Some(_s) = self.seed_table.iter().find(|seed| seed.get_label() == &label) {
            return Err(VaultError::from(SeedDuplicateReason::Label(label)));
        }

        Ok(())
    }

    pub fn validate_new_index(&self, seed_type: SeedType) -> Result<(), VaultError> {
        // This works like a find, but avoids re-calculating the dupe_reason
        if let Some(dupe_reason) = self
            .seed_table
            .iter()
            .filter_map(|seed| seed_type.is_duplicate(&seed.seed, seed.label.clone()))
            .next()
        {
            return Err(VaultError::from(dupe_reason));
        }

        Ok(())
    }

    pub fn save(&mut self, seed: Seed) -> Result<(), VaultError> {
        self.seed_table.separate_categories(|s| s.get_category());
        self.seed_table.push_categorized(|s| s.get_category(), seed)?;
        Ok(())
    }

    pub fn update_accounts(&mut self) {
        self.model.clear();

        let filter = if self.search_text.is_empty() {
            None
        } else {
            Some(FuzzyFilter::new(self.search_text.as_ref()))
        };

        let entries = self
            .seed_table
            .view_sorted(|a, b| Seed::compare_by(a, b, self.get_sort_mode()))
            .filter(|(_i, entry)| {
                if entry.archived != self.archive_mode {
                    return false;
                }

                match &filter {
                    Some(filter) if !filter.matches(entry.get_label().to_lowercase().as_ref()) => false,
                    _ => true,
                }
            })
            .map(|(i, entry)| SeedView::from_seed(entry).with_index(i as i32))
            .collect::<Vec<SeedView>>();

        self.model.extend(entries);
        self.ui().global::<Callbacks>().set_entries(ModelRc::from(self.model.clone()));
    }

    pub fn nav_accounts(&mut self) {
        let ui = self.ui();
        let ui_nav = ui.global::<Navigate>();
        ui_nav.invoke_backward_animate(Animate::None);
        ui_nav.invoke_accounts(
            AccountsParams::default(),
            NavigateOptions { replace: true, animate: Animate::None },
        );
    }

    pub fn move_position(&mut self, index: i32, up: bool) -> Result<(), VaultError> {
        let destination = usize::try_from(index + if up { -1 } else { 1 })?;
        let index = usize::try_from(index)?;

        // OrderedTable returns errors safely for underflows
        self.seed_table.move_position_categorized(|s| s.get_category(), index, destination)?;
        Ok(())
    }

    pub fn get_next_index(&self, seed_view_type: SeedViewType) -> u32 {
        let mut taken_indices = self
            .seed_table
            .iter()
            .filter_map(|s| match (seed_view_type, s.seed.clone()) {
                (SeedViewType::Bitcoin12, SeedType::Bitcoin12 { index })
                | (SeedViewType::Bitcoin24, SeedType::Bitcoin24 { index })
                | (SeedViewType::NostrKey, SeedType::NostrKey { index }) => Some(index),
                (_, _) => None,
            })
            .collect::<Vec<u32>>();
        taken_indices.sort();

        // Find the first space in the sortted list of taken account indices
        // This should only happen if the user manually adds custom accounts that cause a gap in the range.
        // Otherwise, the next_index will be incremented up to the number of accounts.
        let mut next_index: u32 = 0;
        for i in taken_indices.iter() {
            if next_index != *i {
                return next_index;
            } else {
                next_index += 1;
            }
        }

        next_index
    }

    pub fn validate_edit_label(&mut self, index: i32, new_label: String) -> Result<(), VaultError> {
        let index = usize::try_from(index)?;

        let _ =
            self.seed_table.validate_edit(index, move |s| s.edit(SeedEditField::Label(new_label.clone())))?;
        Ok(())
    }

    pub fn edit_indexed(&mut self, index: i32, new_label: String, new_color: u8) -> Result<(), VaultError> {
        let index = usize::try_from(index)?;

        let _ = self.seed_table.edit(index, move |s| {
            s.edit(SeedEditField::Label(new_label.clone()))?;
            s.color = new_color;
            Ok(())
        })?;

        Ok(())
    }

    pub fn edit_password(
        &mut self,
        index: i32,
        new_label: String,
        new_account: String,
        new_password: String,
        new_color: u8,
    ) -> Result<(), VaultError> {
        let index = usize::try_from(index)?;

        let _ = self.seed_table.edit(index, move |s| {
            s.edit(SeedEditField::Label(new_label.clone()))?;
            s.edit(SeedEditField::Account(new_account.clone()))?;
            s.edit(SeedEditField::Password(new_password.clone()))?;
            s.color = new_color;
            Ok(())
        })?;

        Ok(())
    }

    pub fn set_archived(&mut self, index: i32, archived: bool) -> Result<(), VaultError> {
        let index = usize::try_from(index)?;

        let _ = self.seed_table.edit(index, move |s| {
            s.archived = archived;
            Ok(())
        })?;

        self.seed_table.separate_categories(|s| s.get_category());
        Ok(())
    }

    pub fn delete(&mut self, index: i32) -> Result<(), VaultError> {
        let index = usize::try_from(index)?;

        let _ = self.seed_table.remove(index)?;
        Ok(())
    }

    fn get_key(&self, words: WordCount, index: u32) -> Result<MasterKey, VaultError> {
        let entropy = self
            .security_api
            .seed()
            .map_err(|e| VaultError::from(anyhow::anyhow!("Could not retrieve bitcoin seed: {:?}", e)))?
            .ok_or(anyhow::anyhow!("No seed or error returned, securam may be corrupt"))?;

        MasterKey::from_entropy(&self.secp, Network::Bitcoin, entropy.bytes(), "", Some((words, index)))
            .map_err(|e| VaultError::from(anyhow::anyhow!("Could not derive bitcoin seed: {e}")))
    }

    pub fn get_words(&self, index: i32) -> Result<String, VaultError> {
        let index = usize::try_from(index)?;
        let seed = self.seed_table.get(index)?;

        let key = match seed.seed {
            SeedType::Bitcoin12 { index: seed_index } => self.get_key(WordCount::Twelve, seed_index)?,
            SeedType::Bitcoin24 { index: seed_index } => self.get_key(WordCount::TwentyFour, seed_index)?,
            _ => return Err(VaultError::from(anyhow::anyhow!("Unable to get words for seed"))),
        };

        Ok(key.mnemonic.clone())
    }

    fn render_bw_qr_code(&self, data: Vec<u8>) -> Image {
        slint_keyos_platform::qrcode::render(
            &data,
            slint::Color::from_rgb_u8(0, 0, 0),       // black
            slint::Color::from_rgb_u8(255, 255, 255), // white
        )
    }

    // TODO: make this a common function in slint_keyos_platform
    pub fn get_standard_seed_qr(&self, index: i32) -> Result<Image, VaultError> {
        let mnemonic = Mnemonic::parse(self.get_words(index)?)
            .map_err(|e| VaultError::from(anyhow::anyhow!("Could not parse derived seed: {:?}", e)))?;
        let indices: String = mnemonic.word_indices().map(|idx| format!("{:04}", idx)).collect();
        Ok(self.render_bw_qr_code(indices.into_bytes()))
    }

    // TODO: make this a common function in slint_keyos_platform
    pub fn get_compact_seed_qr(&self, index: i32) -> Result<Image, VaultError> {
        let mnemonic = Mnemonic::parse(self.get_words(index)?)
            .map_err(|e| VaultError::from(anyhow::anyhow!("Could not parse derived seed: {:?}", e)))?;
        Ok(self.render_bw_qr_code(mnemonic.to_entropy()))
    }

    fn get_nostr_key(&self, index: i32) -> Result<nostr::Keys, VaultError> {
        let index = usize::try_from(index)?;
        let seed = self.seed_table.get(index)?;

        let entropy = self
            .security_api
            .seed()
            .map_err(|e| VaultError::from(anyhow::anyhow!("Could not retrieve bitcoin seed: {:?}", e)))?
            .ok_or(anyhow::anyhow!("No seed or error returned, securam may be corrupt"))?;

        let bitcoin_key = MasterKey::from_entropy(&self.secp, Network::Bitcoin, entropy.bytes(), "", None)
            .map_err(|e| VaultError::from(anyhow::anyhow!("Could not build bitcoin seed: {e}")))?;

        let nostr_index = match seed.seed {
            SeedType::NostrKey { index: seed_index } => seed_index,
            _ => return Err(VaultError::from(anyhow::anyhow!("Unable to get nostr keys for seed"))),
        };

        nostr::Keys::from_mnemonic_with_account(bitcoin_key.mnemonic.clone(), None, Some(nostr_index))
            .map_err(|e| VaultError::from(anyhow::anyhow!("Could not derive nostr key: {:?}", e)))
    }

    pub fn get_npub(&self, index: i32) -> Result<String, VaultError> {
        let key = self.get_nostr_key(index)?;
        key.public_key()
            .to_bech32()
            .map_err(|e| VaultError::from(anyhow::anyhow!("Could not build nostr npub: {:?}", e)))
    }

    pub fn get_npub_qr(&self, index: i32) -> Result<Image, VaultError> {
        let npub = self.get_npub(index)?;
        Ok(self.render_bw_qr_code(npub.into_bytes()))
    }

    pub fn get_nsec(&self, index: i32) -> Result<String, VaultError> {
        let key = self.get_nostr_key(index)?;
        key.secret_key()
            .to_bech32()
            .map_err(|e| VaultError::from(anyhow::anyhow!("Could not build nostr nsec: {:?}", e)))
    }

    pub fn get_nsec_qr(&self, index: i32) -> Result<Image, VaultError> {
        let nsec = self.get_nsec(index)?;
        Ok(self.render_bw_qr_code(nsec.into_bytes()))
    }
}
