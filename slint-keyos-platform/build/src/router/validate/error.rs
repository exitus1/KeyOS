// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{path::Path, rc::Rc, sync::Arc, vec};

use i_slint_compiler::{object_tree::Component, parser::SyntaxNode};
use miette::{Diagnostic, LabeledSpan, NamedSource, SourceOffset, SourceSpan};
use thiserror::Error;

use super::common::make_source_offset;

// NOTE: we are using Arc<String> instead of Rc<String> intentionally
// because Miette requires that Errors must be Send + Sync

#[derive(Clone, Debug, Default, Error, Diagnostic)]
#[error("Route validation errors")]
pub struct RouteErrorList {
    #[related]
    pub errors: Vec<RouteError>,
}

impl RouteErrorList {
    pub fn into_result(self) -> Result<(), RouteErrorList> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self)
        }
    }

    pub fn extend(&mut self, error: Self) { self.errors.extend(error.errors) }

    pub fn push(&mut self, error: RouteError) { self.errors.push(error); }
}

impl<T> From<T> for RouteErrorList
where
    T: Into<RouteError>,
{
    fn from(error: T) -> Self { RouteErrorList { errors: vec![error.into()] } }
}

impl<E> FromIterator<E> for RouteErrorList
where
    E: Into<RouteError>,
{
    fn from_iter<I: IntoIterator<Item = E>>(iter: I) -> Self {
        let errors: Vec<RouteError> = iter.into_iter().map(Into::into).collect();
        RouteErrorList { errors }
    }
}

#[derive(Clone, Debug, Error, Diagnostic)]
pub enum RouteError {
    #[error("Unexpected: {0:?}")]
    #[diagnostic(transparent)]
    Unexpected(Arc<miette::Report>),

    #[error(transparent)]
    #[diagnostic(transparent)]
    Single(#[from] SourceError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    MultiFile(#[from] MultiFileErrorList),
}

impl PartialEq for RouteError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (RouteError::Unexpected(_), RouteError::Unexpected(_)) => false,
            (RouteError::Single(a), RouteError::Single(b)) => {
                a.src == b.src && a.message == b.message && a.span == b.span
            }
            (RouteError::MultiFile(_), RouteError::MultiFile(_)) => false,
            _ => false,
        }
    }
}

impl From<miette::Report> for RouteError {
    fn from(report: miette::Report) -> RouteError { RouteError::Unexpected(Arc::new(report)) }
}

#[derive(Clone, Diagnostic, Debug, Error)]
#[error("{message}")]
#[diagnostic()]
pub struct MultiFileErrorList {
    message: String,
    #[help]
    help: Option<String>,
    #[related]
    errors: Vec<SourceError>,
}

impl MultiFileErrorList {
    pub fn new(error: impl Into<String>, help: Option<impl Into<String>>, errors: Vec<SourceError>) -> Self {
        Self { message: error.into(), help: help.map(Into::into), errors }
    }

    pub(crate) fn push(&mut self, error: SourceError) { self.errors.push(error) }
}

#[derive(Clone, Diagnostic, Debug, Error)]
#[error("{message}")]
#[diagnostic()]
pub struct SourceError {
    message: String,
    #[label(collection)]
    span: Vec<LabeledSpan>,
    #[source_code]
    src: NamedSource<Arc<String>>,
    #[help]
    help: Option<String>,
}

impl SourceError {
    pub fn single(
        error: impl Into<String>,
        src: NamedSource<Arc<String>>,
        span: LabeledSpan,
        help: Option<String>,
    ) -> Self {
        Self { message: error.into(), src, span: vec![span], help }
    }

    pub(crate) fn page(
        page: Rc<Component>,
        src: Arc<String>,
        path: impl AsRef<Path>,
        message: impl Into<String>,
        label: Option<impl Into<String>>,
        help: Option<impl Into<String>>,
    ) -> Self {
        // For some reason component.root_element.span() is not present.
        // Need to get the span of the component from the syntax node.
        let offset = make_source_offset(&*src, page.node.as_ref().expect("page node"));

        let len = "component ".len() + page.id.len();
        let span = SourceSpan::new(offset, len);
        let span = vec![LabeledSpan::new_with_span(label.map(Into::into), span)];

        let src = NamedSource::new(path.as_ref().to_string_lossy(), src);

        Self { message: message.into(), span, src, help: help.map(Into::into) }
    }

    pub(crate) fn props(
        name: &i_slint_compiler::object_tree::ExportedName,
        path: impl AsRef<Path>,
        src: Arc<String>,
        message: impl Into<String>,
        label: Option<impl Into<String>>,
        help: Option<impl Into<String>>,
    ) -> Self {
        let offset = make_source_offset(&*src, &name.name_ident);
        let len = name.len();
        let span = SourceSpan::new(offset, len);
        let span = vec![LabeledSpan::new_with_span(label.map(Into::into), span)];

        let src = NamedSource::new(path.as_ref().to_string_lossy(), src);

        Self { message: message.into(), span, src, help: help.map(Into::into) }
    }

    pub(crate) fn from_node(
        node: &SyntaxNode,
        src: Arc<String>,
        path: impl AsRef<Path>,
        message: impl Into<String>,
        label: Option<impl Into<String>>,
        help: Option<impl Into<String>>,
    ) -> Self {
        let offset = make_source_offset(&*src, node);
        let span = SourceSpan::new(offset, 0);
        let span = vec![LabeledSpan::new_with_span(label.map(Into::into), span)];

        let src = NamedSource::new(path.as_ref().to_string_lossy(), src);

        Self { message: message.into(), span, src, help: help.map(Into::into) }
    }

    pub(crate) fn missing_export(
        message: impl Into<String>,
        path: impl AsRef<Path>,
        src: Arc<String>,
        label: Option<impl Into<String>>,
        help: Option<impl Into<String>>,
    ) -> Self {
        let last_line = src.lines().map(|_| 1).sum();

        let offset = SourceOffset::from_location(&*src, last_line, 0);
        let span = SourceSpan::new(offset, 0);
        let span = vec![LabeledSpan::new_with_span(label.map(Into::into), span)];

        let src = NamedSource::new(path.as_ref().to_string_lossy(), src);

        Self { message: message.into(), src, span, help: help.map(Into::into) }
    }
}
