// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

#[cfg(keyos)]
use std::{cell::RefCell, collections::HashMap};

#[cfg(keyos)]
use slint::private_unstable_api::re_exports::StaticTextures;
use slint::{private_unstable_api::re_exports::ImageInner, Image, SharedString};
#[cfg(not(keyos))]
use slint_keyos_platform_common::utils::parse_nine_slice_filename;
#[cfg(keyos)]
use slint_keyos_platform_common::{ArchivedIconSet, IconSet, RawImage};

#[cfg(keyos)]
pub fn load_raw_image<P>(
    fs: &fs::FileSystem<P>,
    cache: &RefCell<HashMap<String, (&'static StaticTextures, Option<[u16; 4]>)>>,
    image_name: SharedString,
    nine_slice: bool,
    is_dark: bool,
) -> Image
where
    P: server::CheckedPermissions,
    P: server::MessageAllowed<fs::messages::MapFileMessage>,
    P: server::MessageAllowed<fs::messages::GetMetadata>,
{
    // Try dark variant first if in dark mode
    let mut actual_image_name = image_name.to_string();
    if is_dark {
        let dark_variant = format!("{}-dark", actual_image_name);
        let dark_path = format!("{dark_variant}.raw");

        // Check if dark variant exists
        if fs.metadata(&dark_path, fs::Location::CommonAssets).is_ok() {
            log::debug!("Using dark variant: {dark_path}");
            actual_image_name = dark_variant;
        }
    }

    let path = format!("{actual_image_name}.raw");
    let (texture, nine_slice_edges) = match cache.borrow_mut().entry(path.clone().into()) {
        std::collections::hash_map::Entry::Occupied(entry) => {
            log::debug!("load_raw_image cache hit on {path}");
            entry.get().clone()
        }
        std::collections::hash_map::Entry::Vacant(entry) => {
            let Some(archived_image) = map_archive::<RawImage, _>(fs, &path) else {
                log::warn!("Could not load image {actual_image_name:?}");
                return Image::from(ImageInner::None);
            };
            let texture = archived_image.into();
            let nine_slice_edges = archived_image.nine_slice.as_ref().map(|edges| edges.map(From::from));
            entry.insert((texture, nine_slice_edges));
            (texture, nine_slice_edges)
        }
    };
    let mut image = Image::from(ImageInner::StaticTextures(texture));
    if nine_slice {
        if let Some(nine_slice_edges) = nine_slice_edges {
            image.set_nine_slice_edges(
                nine_slice_edges[0],
                nine_slice_edges[1],
                nine_slice_edges[2],
                nine_slice_edges[3],
            );
        } else {
            log::warn!("No nine slice info found for {actual_image_name}");
        }
    }
    image
}

#[cfg(keyos)]
#[derive(Default)]
pub struct IconCache {
    icon_set: Option<&'static ArchivedIconSet>,
    cache: HashMap<(usize, String), &'static StaticTextures>,
}

#[cfg(keyos)]
pub fn load_icon<P>(
    fs: &fs::FileSystem<P>,
    cache: &RefCell<IconCache>,
    name: SharedString,
    requested_size: f32,
) -> Image
where
    P: server::CheckedPermissions,
    P: server::MessageAllowed<fs::messages::MapFileMessage>,
{
    if cache.borrow().icon_set.is_none() {
        let Some(icon_set) = map_archive::<IconSet, _>(fs, "icon_set.bin") else {
            return Image::from(ImageInner::None);
        };
        cache.borrow_mut().icon_set = Some(icon_set);
    };
    let icon_set = cache.borrow().icon_set.unwrap();
    let Some(icons) = icon_set.0.get(name.as_str()) else {
        log::warn!("Could not load icon {name:?}");
        return Image::from(ImageInner::None);
    };
    let requested_size = requested_size.round() as u32;
    let icon = icons
        .iter()
        .find(|icon| icon.size.width.to_native() >= requested_size)
        .unwrap_or(icons.last().unwrap());
    let chosen_size = icon.size.width.to_native() as usize;
    if chosen_size == 0 {
        log::debug!("Icon {name} has no valid size, returning an empty image");
        return Image::from(ImageInner::None);
    }

    if let Some(texture) = cache.borrow().cache.get(&(chosen_size, name.to_string())) {
        log::debug!("load_icon cache hit on {name}@{chosen_size}");
        return Image::from(ImageInner::StaticTextures(texture));
    }

    let texture = icon.into();
    cache.borrow_mut().cache.insert((chosen_size, name.to_string()), texture);
    Image::from(ImageInner::StaticTextures(texture))
}

#[cfg(keyos)]
fn map_archive<T, P>(fs: &fs::FileSystem<P>, path: &str) -> Option<&'static T::Archived>
where
    T: rkyv::Archive,
    T::Archived: for<'a> rkyv::bytecheck::CheckBytes<rkyv::api::high::HighValidator<'a, rkyv::rancor::Error>>,
    P: server::CheckedPermissions,
    P: server::MessageAllowed<fs::messages::MapFileMessage>,
{
    log::debug!("Mapping file {path}");
    let mapping = match fs.map_file(fs::Location::CommonAssets, path) {
        Ok(mapping) => mapping,
        Err(e) => {
            log::warn!("Error loading file at path \"{path}\": {e:?}");
            return None;
        }
    };
    // Transmuting to static because we know we are not dropping this memory.
    let mapping = unsafe { core::mem::transmute::<&[u8], &'static [u8]>(mapping.as_slice()) };
    let archived = rkyv::access::<T::Archived, rkyv::rancor::Error>(mapping).ok()?;
    Some(archived)
}

#[cfg(not(keyos))]
fn try_load_image_with_name(
    base_dir: &std::path::Path,
    target_name: &str,
    nine_slice: bool,
) -> Option<Image> {
    for entry in std::fs::read_dir(base_dir).ok()? {
        let Ok(entry) = entry else {
            continue;
        };
        let path = entry.path();
        if !path.extension().map(|e| e == "svg" || e == "png").unwrap_or(false) {
            continue;
        }

        if nine_slice {
            if let Some((ns_name, nine_slice_edges)) = parse_nine_slice_filename(&path) {
                if ns_name == target_name {
                    let Ok(mut image) = Image::load_from_path(&entry.path()) else {
                        continue;
                    };
                    image.set_nine_slice_edges(
                        nine_slice_edges[0],
                        nine_slice_edges[1],
                        nine_slice_edges[2],
                        nine_slice_edges[3],
                    );
                    return Some(image);
                }
            }
        } else {
            if path.file_stem().map(|s| s == target_name).unwrap_or(false) {
                if let Ok(image) = Image::load_from_path(&path) {
                    return Some(image);
                }
            }
        }
    }
    None
}

#[cfg(not(keyos))]
pub fn load_raw_image<FS>(
    _fs: &FS,
    _cache: &(),
    name: SharedString,
    nine_slice: bool,
    is_dark: bool,
) -> Image {
    let image_name = name.split("/").last().unwrap();
    let base_dir = std::path::Path::new("../../ui/ui").join(name.to_string()).parent().unwrap().to_path_buf();

    // Try dark variant first if in dark mode
    if is_dark {
        let dark_image_name = format!("{}-dark", image_name);
        if let Some(image) = try_load_image_with_name(&base_dir, &dark_image_name, nine_slice) {
            log::debug!("Using dark variant: {}", dark_image_name);
            return image;
        }
    }

    // Load regular image
    if let Some(image) = try_load_image_with_name(&base_dir, image_name, nine_slice) {
        return image;
    }

    log::warn!("Could not find image: \"{name}\"");
    Image::from(ImageInner::None)
}

#[cfg(not(keyos))]
pub fn load_icon<FS>(_fs: &FS, _cache: &(), name: SharedString, _requested_size: f32) -> Image {
    let path = std::path::PathBuf::from(format!("../../ui/ui/icons/{name}.svg"));
    match Image::load_from_path(&path) {
        Ok(img) => img,
        Err(e) => {
            log::warn!("Error loading image at path \"{path:?}\": {e:?}");
            Image::from(ImageInner::None)
        }
    }
}
