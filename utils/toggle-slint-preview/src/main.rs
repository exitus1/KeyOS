// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::env;
use std::fs;
use std::io::{Read, Write};

use regex::Regex;

fn process_block(block: &str, enable: bool, base_indent: usize) -> String {
    block
        .lines()
        .map(|line| {
            let stripped = line.trim();
            if stripped.is_empty() {
                return line.to_string();
            }

            let current_indent = line.len() - line.trim_start().len();

            if enable {
                if line.trim_start().starts_with("//") {
                    // Remove '//' and up to one space after it
                    Regex::new(r"^(\s*)//\s?").unwrap().replace(line, "$1").to_string()
                } else {
                    line.to_string()
                }
            } else {
                if !line.trim_start().starts_with("//") {
                    let extra_indent = current_indent.saturating_sub(base_indent);
                    format!(
                        "{:width$}// {}{}",
                        "",
                        " ".repeat(extra_indent),
                        line.trim_start(),
                        width = base_indent
                    )
                } else {
                    line.to_string()
                }
            }
        })
        .collect::<Vec<String>>()
        .join("\n")
}

fn process_file(filename: &str, enable_preview: bool) -> std::io::Result<()> {
    println!("Processing file: {}", filename);
    println!("Preview mode: {}", enable_preview);

    let mut content = String::new();
    fs::File::open(filename)?.read_to_string(&mut content)?;
    println!("Successfully read file. Content length: {}", content.len());

    let pattern = Regex::new(r"(?m)^([ \t]*)//[ \t]*#IF[ \t]*PREVIEW[ \t]*\n((?:.*\n)*?)([ \t]*)//[ \t]*#ELSE[ \t]*\n((?:.*\n)*?)([ \t]*)//[ \t]*#ENDIF").unwrap();

    let matches: Vec<_> = pattern.captures_iter(&content).collect();
    println!("Found {} conditional blocks", matches.len());

    let modified_content = pattern.replace_all(&content, |caps: &regex::Captures| {
        let base_indent = caps[1].len();
        let preview_block = caps[2].trim_end();
        let else_indent = &caps[3];
        let else_block = caps[4].trim_end();
        let endif_indent = &caps[5];

        let (preview_processed, else_processed) = if enable_preview {
            (process_block(preview_block, true, base_indent), process_block(else_block, false, base_indent))
        } else {
            (process_block(preview_block, false, base_indent), process_block(else_block, true, base_indent))
        };

        format!(
            "{}// #IF PREVIEW\n{}\n{}// #ELSE\n{}\n{}// #ENDIF",
            &caps[1], preview_processed, else_indent, else_processed, endif_indent
        )
    });

    if content != modified_content {
        fs::File::create(filename)?.write_all(modified_content.as_bytes())?;
        println!("{} preview in {}", if enable_preview { "Enabled" } else { "Disabled" }, filename);
    } else {
        println!("No changes made to {}", filename);
    }

    Ok(())
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("Usage: {} [--preview] <file1> <file2> ...", args[0]);
        std::process::exit(1);
    }

    let enable_preview = args[1] == "--preview";
    let files = if enable_preview { &args[2..] } else { &args[1..] };

    for filename in files {
        if let Err(e) = process_file(filename, enable_preview) {
            eprintln!("Error processing file {}: {}", filename, e);
        }
    }

    Ok(())
}
