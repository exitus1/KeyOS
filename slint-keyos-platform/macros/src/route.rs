// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use darling::{ast::NestedMeta, Error, FromDeriveInput, FromMeta};
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub fn expand(attr: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let source_input = input.clone();
    let derive_input = syn::parse_macro_input!(input as DeriveInput);

    let mut errors = darling::Error::accumulator();

    let meta = errors
        .handle_in(|| NestedMeta::parse_meta_list(attr.into()).map_err(Error::from))
        .and_then(|meta| errors.handle_in(|| model::RouteArgs::from_list(&meta)));

    let route = errors.handle_in(|| model::RouteProperties::from_derive_input(&derive_input));

    if let Err(err) = errors.finish() {
        return err.write_errors().into();
    }

    let (meta, route) = (meta.unwrap(), route.unwrap());

    let result = match expand_with_err(meta, route) {
        // Append the original input to the generated code.
        Ok(tokens) => {
            let input: TokenStream = source_input.into();
            quote! {
                #input
                #tokens
            }
        }
        Err(e) => e.to_compile_error(),
    };

    result.into()
}

fn expand_with_err(meta: model::RouteArgs, route: model::RouteProperties) -> syn::Result<TokenStream> {
    let fields: Vec<model::RouteField> = route
        .data
        .take_struct()
        .expect("struct fields")
        .into_iter()
        .map(|field| model::RouteField::from(field))
        .collect();

    let validate::AnalyzedRoute { path, query } = validate::parse_path(&meta.path, &fields)?;

    let route = &route.ident;

    let path_segments = path.iter().map(|segment| match segment {
        validate::Segment::Capture(_) => {
            quote! {
                PathSegment::Capture
            }
        }
        validate::Segment::Static(static_segment) => {
            quote! {
                PathSegment::Static(std::borrow::Cow::Borrowed(#static_segment))
            }
        }
    });

    let query_parameters = query.iter().map(|query| {
        let query = &query.ident.as_str();
        quote! {
            QueryParameter { key: std::borrow::Cow::Borrowed(#query) }
        }
    });

    let route_entry = quote! {
        impl RouteEntry for #route {
            fn metadata() -> RouteMetadata {
                RouteMetadata::new(vec![#(#path_segments),*], vec![#(#query_parameters),*])
            }
        }
    };

    let serde_impl = impl_codec(&meta, route, path, query);

    let result = quote! {
        #[automatically_derived]
        const _: () = {
            #serde_impl
            #route_entry
        };
    };

    Ok(result)
}

fn impl_codec(
    meta: &model::RouteArgs,
    struct_ident: &syn::Ident,
    path_segments: Vec<validate::Segment>,
    query_captures: Vec<validate::Capture>,
) -> TokenStream {
    let struct_to_string = struct_ident.to_string();
    let struct_to_str = struct_to_string.trim_start_matches("r#");

    let path_serializer_ident =
        syn::Ident::new(format!("{struct_to_str}PathRef").as_str(), struct_ident.span());
    let query_serializer_ident =
        syn::Ident::new(format!("{struct_to_str}QueryRef").as_str(), struct_ident.span());

    let path_deserializer_ident =
        syn::Ident::new(format!("{struct_to_str}Path").as_str(), struct_ident.span());
    let query_deserializer_ident =
        syn::Ident::new(format!("{struct_to_str}Query").as_str(), struct_ident.span());

    let path_fields = path_segments
        .iter()
        .filter_map(|segment| match segment {
            validate::Segment::Capture(capture) => Some(capture),
            validate::Segment::Static(_) => None,
        })
        .collect::<Vec<_>>();

    let path_field_names = path_fields.iter().map(|capture| &capture.ident).collect::<Vec<_>>();
    let query_field_names = query_captures.iter().map(|capture| &capture.ident).collect::<Vec<_>>();
    let query_required = meta.required.is_present();

    let serialize = {
        let path_serializer = {
            let num_segments = path_segments.len();
            let segments = path_segments.iter().map(|segment| match segment {
                validate::Segment::Capture(capture) => {
                    let ident = &capture.ident;

                    quote! {
                        s.serialize_element(&self.0.#ident)?;
                    }
                }
                validate::Segment::Static(static_segment) => {
                    quote! {
                        s.serialize_element(&#static_segment)?;
                    }
                }
            });

            quote! {
                pub struct #path_serializer_ident<'a>(&'a #struct_ident);

                impl<'a> serde::ser::Serialize for #path_serializer_ident<'a> {
                    fn serialize<Ser>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error>
                    where
                        Ser: serde::ser::Serializer,
                    {
                        use serde::ser::SerializeSeq;
                        let mut s = serializer.serialize_seq(Some(#num_segments))?;
                        #(#segments)*
                        s.end()
                    }
                }
            }
        };

        let query_serializer = {
            let entries = query_captures.iter().map(|query| {
                let ident = &query.ident;
                let field_name = {
                    let s = query.ident.as_str();
                    s.strip_prefix("r#").unwrap_or(s)
                };
                quote! {
                    s.serialize_field(#field_name, &self.0.#ident)?;
                }
            });

            let len = query_captures.len();
            quote! {
                pub struct #query_serializer_ident<'a>(&'a #struct_ident);

                impl<'a> serde::ser::Serialize for #query_serializer_ident<'a> {
                    fn serialize<Ser>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error>
                    where
                        Ser: serde::ser::Serializer,
                    {
                        use serde::ser::SerializeStruct;
                        let mut s = serializer.serialize_struct("QueryParams", #len)?;
                        #(#entries)*
                        s.end()
                    }
                }
            }
        };

        quote! {
            #path_serializer
            #query_serializer
        }
    };

    let deserialize = {
        let num_segments = path_segments.len();

        let path_deserializers = path_segments.iter().map(|segment| match segment {
            validate::Segment::Capture(capture) => {
                let field_ident = &capture.ident;
                quote! {
                    let #field_ident = seq.next_element()?.ok_or_else(|| serde::de::Error::invalid_length(#num_segments, &self))?;
                }
            }
            validate::Segment::Static(static_segment) => {
                // TODO: is there a way to avoid creating owned string?
                quote! {
                    let static_segment: String = seq.next_element()?.ok_or_else(|| serde::de::Error::invalid_length(#num_segments, &self))?;
                    let static_segment = static_segment.as_str();
                    if static_segment != #static_segment {
                        return Err(serde::de::Error::invalid_value(serde::de::Unexpected::Str(static_segment), &#static_segment));
                    }
                }
            }
        });

        let path_deserializer = {
            let path_fields = path_fields.iter().map(|capture| {
                let field_ident = &capture.ident;
                let ty = &capture.ty;
                quote! {
                    #field_ident: #ty,
                }
            });

            quote! {
                pub struct #path_deserializer_ident {
                    #(#path_fields)*
                }

                impl <'de> serde::de::Deserialize<'de> for #path_deserializer_ident {
                    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                    where
                        D: serde::de::Deserializer<'de>,
                    {
                        struct PathVisitor;

                        impl<'de> serde::de::Visitor<'de> for PathVisitor {
                            type Value = #path_deserializer_ident;

                            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                                formatter.write_str(concat!("struct ", stringify!(#path_deserializer_ident)))
                            }

                            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                            where
                                A: serde::de::SeqAccess<'de>,
                            {
                                #(#path_deserializers)*

                                Ok(#path_deserializer_ident {
                                    #(#path_field_names),*
                                })
                            }
                        }

                        deserializer.deserialize_seq(PathVisitor)
                    }
                }
            }
        };

        // Only de-serialize query parameters.
        let query_deserializer = {
            let query_fields = query_captures.iter().map(|query| {
                let field_ident = &query.ident;
                let ty = &query.ty;

                if query_required {
                    quote! {
                        #field_ident: #ty,
                    }
                } else {
                    quote! {
                        #field_ident: Option<#ty>,
                    }
                }
            });

            quote! {
                #[derive(serde::Deserialize)]
                pub struct #query_deserializer_ident {
                    #(#query_fields)*
                }
            }
        };

        quote! {
            #path_deserializer
            #query_deserializer
        }
    };

    let from_parts = {
        if path_fields.is_empty() && query_captures.is_empty() {
            quote! {
                Self {}
            }
        } else {
            let all_names = path_field_names.iter().chain(query_field_names.iter());
            let query_fields = if query_required {
                quote! {
                    let Self::Query { #(#query_field_names),* } = query;
                }
            } else {
                quote! {
                    let Self::Query { #(#query_field_names),* } = query;
                    #(let #query_field_names = #query_field_names.unwrap_or_default();)*
                }
            };
            quote! {
                let Self::Path { #(#path_field_names),* } = path;
                #query_fields
                Self {
                    #(#all_names),*
                }
            }
        }
    };

    let query_into_parts = if query_required {
        quote! {
            #(#query_field_names: self.#query_field_names),*
        }
    } else {
        quote! {
            #(#query_field_names: Some(self.#query_field_names)),*
        }
    };

    quote! {
        #serialize
        #deserialize

        impl RouteCodec for #struct_ident {
            type Path = #path_deserializer_ident;
            type Query = #query_deserializer_ident;

            type PathRef<'a> = #path_serializer_ident<'a>;
            type QueryRef<'a> = #query_serializer_ident<'a>;

            fn from_parts(path: Self::Path, query: Self::Query) -> Self {
                #from_parts
            }

            fn path_ref(&self) -> Self::PathRef<'_> {
                #path_serializer_ident(self)
            }

            fn query_ref(&self) -> Self::QueryRef<'_> {
                #query_serializer_ident(self)
            }

            fn into_parts(self) -> (Self::Path, Self::Query) {
                (
                    #path_deserializer_ident {
                        #(#path_field_names: self.#path_field_names),*
                    },
                    #query_deserializer_ident {
                        #query_into_parts
                    }
                )
            }
        }
    }
}

mod validate {
    use darling::util::IdentString;

    use super::model;

    #[derive(Debug)]
    pub struct AnalyzedRoute {
        pub path: Vec<Segment>,
        pub query: Vec<Capture>,
    }

    #[derive(Debug, PartialEq)]
    pub enum Segment {
        Capture(Capture),
        Static(String),
    }

    #[derive(Debug, PartialEq)]
    pub struct Capture {
        pub ty: syn::Type,
        pub ident: IdentString,
    }

    pub fn parse_path(path: &syn::LitStr, fields: &[model::RouteField]) -> syn::Result<AnalyzedRoute> {
        use slint_keyos_platform_common::analyze_path::{PathError, Segment as CommonSegment};

        let fields_names: Vec<String> = fields.iter().map(|field| field.ident.to_string()).collect();
        let result = match slint_keyos_platform_common::analyze_path::validate(path.value(), fields_names) {
            Ok(result) => result,
            Err(err) => {
                let span = match &err {
                    PathError::MissingFields(missing) => fields
                        .iter()
                        .find(|field| {
                            let field = field.ident.as_str();
                            missing.iter().any(|missing| missing == field)
                        })
                        .map(|field| field.ident.span())
                        .unwrap_or_else(|| path.span()),
                    _ => path.span(),
                };
                let err = syn::Error::new(span, err.to_string());
                return Err(err);
            }
        };

        let path: Vec<Segment> = result
            .path
            .iter()
            .map(|segment| match segment {
                CommonSegment::Capture(capture) => Segment::Capture(find_capture(fields, capture)),
                CommonSegment::Static(static_segment) => Segment::Static(static_segment.clone()),
            })
            .collect();

        let query: Vec<Capture> = result.query.iter().map(|capture| find_capture(fields, capture)).collect();

        Ok(AnalyzedRoute { path, query })
    }

    fn find_capture(fields: &[model::RouteField], capture: &str) -> Capture {
        let field = fields
            .iter()
            .find(|field| field.ident.as_str().trim_start_matches("r#") == capture)
            .expect("Field exists");

        Capture { ty: field.ty.clone(), ident: field.ident.clone() }
    }

    #[test]
    fn test_raw_ident() {
        let span = proc_macro2::Span::call_site();
        let path = syn::LitStr::new("/user/{user_id}", span.clone());
        let fields = vec![model::RouteField {
            ident: IdentString::new(syn::Ident::new_raw("user_id", span)),
            ty: syn::parse_quote!(String),
        }];

        let result = parse_path(&path, &fields).unwrap();

        assert_eq!(
            result.path,
            vec![
                Segment::Static("user".to_string()),
                Segment::Capture(Capture {
                    ty: syn::parse_quote!(String),
                    ident: IdentString::new(syn::Ident::new_raw("user_id", span))
                }),
            ]
        );
        assert!(result.query.is_empty());
    }

    #[test]
    fn test_slint_prefix_included() {
        let span = proc_macro2::Span::call_site();
        let path = syn::LitStr::new("/user/{r#user}", span.clone());
        let fields = vec![model::RouteField {
            ident: IdentString::new(syn::Ident::new_raw("user", span)),
            ty: syn::parse_quote!(String),
        }];

        let err = parse_path(&path, &fields).unwrap_err();

        assert_eq!(err.to_string(), "Field missing from path: [\"user\"]");
    }

    #[test]
    fn kebab_case_raw() {
        let span = proc_macro2::Span::call_site();
        let path = syn::LitStr::new("/user/{user-id}", span.clone());
        let fields = vec![model::RouteField {
            ident: IdentString::new(syn::Ident::new_raw("user_id", span)),
            ty: syn::parse_quote!(String),
        }];

        let result = parse_path(&path, &fields);

        assert!(result.is_ok())
    }
}

#[allow(dead_code)]
mod model {
    use darling::util::{Flag, IdentString};
    use darling::{ast, FromDeriveInput, FromField, FromMeta};
    use syn::LitStr;

    #[derive(Debug, FromMeta)]
    pub struct RouteArgs {
        pub path: LitStr,
        pub default: Flag,
        pub dev: Flag,
        pub required: Flag,
    }

    #[derive(Debug, FromDeriveInput)]
    #[darling(supports(struct_named))]
    pub struct RouteProperties {
        pub ident: syn::Ident,
        pub data: ast::Data<(), RouteFieldRaw>,
    }

    #[derive(Debug, FromField)]
    // #[darling(attributes(param))]
    pub struct RouteFieldRaw {
        pub ty: syn::Type,
        pub ident: Option<syn::Ident>,
    }

    impl From<RouteFieldRaw> for RouteField {
        fn from(field: RouteFieldRaw) -> Self {
            let ident = field.ident.expect("Field must be named");
            RouteField { ident: IdentString::new(ident), ty: field.ty }
        }
    }
    pub struct RouteField {
        pub ident: IdentString,
        pub ty: syn::Type,
    }
}
