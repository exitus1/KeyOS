// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

pub(crate) mod rust_init;
pub(crate) mod slint;

use std::fmt::Write;

use slint_keyos_platform_common::analyze_path::Segment;

use super::validate::{PageSlintData, RouteProps, RouterPage};
use crate::{generated_file::GenerateConfig, source::uwrite};

pub fn gen_router_stage_one(ctx: GenContext) -> Result<(), std::io::Error> {
    slint::generate_and_write_navigate(ctx)?;
    rust_init::generate_and_write(ctx)?;

    Ok(())
}

pub fn gen_router_stage_two(ctx: GenContext) -> Result<(), std::io::Error> {
    slint::generate_and_write_router(ctx)?;

    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub struct GenContext<'a> {
    pub data: GenerateData<'a>,
    pub config: &'a GenerateConfig,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct GenerateData<'a> {
    pub pages: &'a [RouterPage],
    pub props: &'a [RouteProps],
}

impl<'a> GenerateData<'a> {
    pub fn complete_pages(&self) -> Vec<(&'a RouterPage, &'a PageSlintData)> {
        self.pages.iter().filter_map(|p| p.slint_data.as_ref().map(|sd| (p, sd))).collect()
    }
}

impl RouterPage {
    pub fn example_route(&self, writer: &mut impl Write) {
        for path_segment in self.props.iter().flat_map(|p| &p.analyzed_path.path) {
            match path_segment {
                Segment::Capture(capture) => uwrite!(writer, "/{{{capture}}}"),
                Segment::Static(static_segment) => uwrite!(writer, "/{}", static_segment),
            }
        }

        for (ii, query_param) in self.props.iter().flat_map(|p| &p.analyzed_path.query).enumerate() {
            if ii == 0 {
                uwrite!(writer, "?");
            } else {
                uwrite!(writer, "&");
            }
            uwrite!(writer, "{{{query_param}}}");
        }
    }

    pub fn slint_callback_decl(&self, writer: &mut impl Write) {
        if self.is_static_route() {
            uwrite!(writer, "callback {name}(NavigateOptions);", name = self.names.kebab)
        } else {
            uwrite!(writer, "callback {name}(", name = self.names.kebab);
            self.slint_navigation_struct(writer);
            uwrite!(writer, ", NavigateOptions);");
        }
    }

    pub fn slint_path_property(&self, writer: &mut impl Write) {
        if self.is_static_route() {
            uwrite!(writer, "out property <string> {name} : ", name = self.names.kebab);
            uwrite!(writer, "\"");
            self.example_route(writer);
            uwrite!(writer, "\";");
        } else {
            uwrite!(writer, "pure callback {name}(", name = self.names.kebab);
            self.slint_navigation_struct(writer);
            uwrite!(writer, ") -> string;");
        }
    }

    pub fn rust_closure_input(&self, w: &mut impl Write) {
        if self.is_static_route() {
            uwrite!(w, "move |options|");
        } else {
            uwrite!(w, "move |");
            self.deconstruct_params(w);
            uwrite!(w, ", options|")
        }
    }

    pub fn deconstruct_params(&self, writer: &mut impl Write) {
        self.slint_navigation_struct(writer);
        uwrite!(writer, "{{");

        for field in self.props.iter().flat_map(|p| &p.fields) {
            uwrite!(writer, "{},", field.key.snake);
        }

        uwrite!(writer, "}}")
    }

    // flattens all props into a single slint struct.
    // Includes implicit props.
    fn slint_navigation_struct(&self, writer: &mut impl Write) {
        uwrite!(writer, "{}Params", self.names.pascal)
    }

    pub fn rust_tuple(&self) -> String {
        let result = self.props.iter().fold(String::new(), |mut acc, p| {
            write!(acc, "{}, ", &p.export_name.name).unwrap();
            acc
        });

        format!("({result})")
    }
}
