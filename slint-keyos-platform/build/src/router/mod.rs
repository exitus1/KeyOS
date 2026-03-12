// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

pub(crate) mod generate;
pub(crate) mod validate;

#[cfg(test)]
mod tests;

use std::path::Path;

use validate::RouterOutput;

use crate::generated_file::GenerateConfig;

pub struct BuildRouterConfig<'a> {
    pub root_slint: &'a Path,
    pub out_dir: &'a Path,

    pub include_router: bool,
}

pub fn build_router(config: BuildRouterConfig) {
    // will always generate an empty router macro so that the include! macro doesn't fail.
    generate::rust_init::gen_empty_router(config.out_dir).unwrap();

    if !config.include_router {
        return;
    }

    let root_slint = config.root_slint;

    println!("cargo:rerun-if-changed={}", root_slint.display());

    let mut router_output = unwrap_fatal({
        RouterOutput::new(root_slint).and_then(|mut output| {
            validate::build_stage_one(&mut output)?;
            Ok(output)
        })
    });

    let gen_config = GenerateConfig { root_slint: root_slint.join("gen"), out_dir: config.out_dir.into() };

    if config.include_router {
        let ctx = (&router_output, &gen_config).into();
        generate::gen_router_stage_one(ctx).expect("router codegen - stage 1");
    }

    unwrap_fatal(validate::build_stage_two(&mut router_output));

    if config.include_router {
        let ctx = (&router_output, &gen_config).into();
        generate::gen_router_stage_two(ctx).expect("router codegen - stage 2");
    }

    let _ = unwrap_fatal(router_output.errors.into_result());
}

fn unwrap_fatal<T, E: Into<miette::Report>>(result: Result<T, E>) -> T {
    match result {
        Ok(pages) => pages,
        Err(err) => {
            let report: miette::Report = err.into();
            eprintln!("{:?}", report);
            std::process::exit(1);
        }
    }
}

impl<'a> From<(&'a RouterOutput, &'a GenerateConfig)> for generate::GenContext<'a> {
    fn from((output, config): (&'a RouterOutput, &'a GenerateConfig)) -> Self {
        generate::GenContext {
            data: generate::GenerateData { pages: &output.valid_pages, props: &output.valid_props },
            config,
        }
    }
}
