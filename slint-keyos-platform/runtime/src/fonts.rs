// SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundationdevices.com>
// SPDX-License-Identifier: GPL-3.0-or-later

const DEFAULT_FONT: &str = "Montserrat";

#[cfg(keyos)]
pub fn register_fonts<P>(fs: &fs::FileSystem<P>)
where
    P: server::CheckedPermissions,
    P: server::MessageAllowed<fs::messages::OpenDirMessage>,
    P: server::MessageAllowed<fs::messages::CloseDir>,
    P: server::MessageAllowed<fs::messages::NextEntry>,
    P: server::MessageAllowed<fs::messages::MapFileMessage>,
{
    let fonts_dir =
        fs.open_dir("fonts", fs::Location::CommonAssets).expect("Could not open common fonts dir");
    while let Some(font_entry) = fonts_dir.next_entry().expect("Could not read fonts dir") {
        if font_entry.is_file {
            let mapping = fs
                .map_file(fs::Location::CommonAssets, format!("fonts/{}", font_entry.name))
                .expect("Could not load font");
            // Transmuting to static because we know we are not dropping this memory.
            let mapping = unsafe { core::mem::transmute::<&[u8], &'static [u8]>(mapping.as_slice()) };
            i_slint_common::sharedfontdb::register_font_from_memory(mapping)
                .expect("Could not register individual font");
        }
    }
    i_slint_common::sharedfontdb::set_default_font_family(DEFAULT_FONT);
}

#[cfg(not(keyos))]
pub fn register_fonts<FS>(_fs: &FS) {
    for font_file in std::fs::read_dir("../../ui/ui/fonts").unwrap() {
        let font_data = std::fs::read(&font_file.unwrap().path()).unwrap().leak();
        i_slint_common::sharedfontdb::register_font_from_memory(font_data)
            .expect("Could not register individual font");
    }
    i_slint_common::sharedfontdb::set_default_font_family(DEFAULT_FONT);
}
