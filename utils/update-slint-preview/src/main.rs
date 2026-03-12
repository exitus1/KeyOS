// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::Path;
use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use clap::Parser;
use mustache::compile_str;
use serde_json::json;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Folder containing images (can be used multiple times)
    #[arg(short, long)]
    images_folder: Vec<String>,

    /// Folder containing icons
    #[arg(short = 'c', long)]
    icons_folder: String,

    /// Template file path
    #[arg(short, long)]
    template_file: String,

    /// Output file path
    #[arg(short, long)]
    output_file: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut images = String::new();
    let mut nine_slice_images = String::new();

    for folder in args.images_folder {
        let folder_path = Path::new(&folder);
        let folder_name = folder_path
            .file_name()
            .context("Invalid folder name")?
            .to_str()
            .context("Invalid folder name encoding")?;

        for path in read_dir(folder_path)? {
            if let Some(extension) = path.extension() {
                if extension == "png" || extension == "svg" {
                    let file_stem = path
                        .file_stem()
                        .context("Invalid file name")?
                        .to_str()
                        .context("Invalid file name encoding")?;
                    let file_extension = extension.to_str().unwrap();

                    let formatted_string = if let Some((image_name, slice_values)) =
                        file_stem.split_once("__")
                    {
                        let slice_values = slice_values.replace('-', " ");

                        nine_slice_images.push_str(&format!(
                            "    //     if (name == \"{}/{}\") {{\n    //         return @image-url(\"@ui/{}/{}.{}\", nine-slice({slice_values}));\n    //     }}\n",
                            folder_name, image_name, folder_name, file_stem, file_extension
                        ));
                        println!("Added nine-slice image: {folder_name}/{file_stem}.{file_extension}");
                    } else {
                        images.push_str(&format!(
                            "    //     if (name == \"{}/{}\") {{\n    //         return @image-url(\"@ui/{}/{}.{}\");\n    //     }}\n",
                            folder_name, file_stem, folder_name, file_stem, file_extension
                        ));
                        println!("Added regular image: {folder_name}/{file_stem}.{file_extension}");
                    };
                }
            }
        }
    }

    let mut icons = String::new();
    let icons_folder_path = Path::new(&args.icons_folder);
    let icons_folder_name = icons_folder_path
        .file_name()
        .context("Invalid folder name")?
        .to_str()
        .context("Invalid folder name encoding")?;

    for path in read_dir(icons_folder_path)? {
        if path.extension().map(|e| e == "svg").unwrap_or(false) {
            let file_stem = path
                .file_stem()
                .context("Invalid file name")?
                .to_str()
                .context("Invalid file name encoding")?;
            icons.push_str(&
                format!(
                    "    //     if (name == \"{}\") {{\n    //         return @image-url(\"@ui/{}/{}.svg\");\n    //     }}\n",
                    file_stem, icons_folder_name, file_stem
                ));
            println!("Added icon: {file_stem}");
        }
    }

    let template_str = fs::read_to_string(&args.template_file).context("Failed to read template file")?;

    let data = json!({
        "images": images.trim_end(),
        "icons": icons.trim_end(),
        "nine-slice-images": nine_slice_images.trim_end()
    });

    let template = compile_str(&template_str).context("Failed to compile template")?;

    let output = template.render_to_string(&data).context("Failed to render template")?;

    fs::write(&args.output_file, output).context("Failed to write output file")?;

    println!("Successfully generated: {}", args.output_file);

    Ok(())
}

fn read_dir(folder: &Path) -> Result<impl Iterator<Item = PathBuf>> {
    let mut files: Vec<_> = std::fs::read_dir(folder)?
        .filter_map(|e| e.ok())
        .map(|e| e.path().to_owned())
        .filter(|p| p.is_file())
        .collect();
    files.sort_unstable();
    Ok(files.into_iter())
}
