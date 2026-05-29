//! RML — rgui Markup Language
//!
//! A small, typed, XML-like declarative syntax that maps directly to the
//! `rgui::Element` and widget builder API.
//!
//! # Quick example
//!
//! ```rust,no_run
//! # #[cfg(feature = "rml")]
//! # {
//! use rgui::rml;
//!
//! let src = r#"
//!   <Column key="settings" padding="16" gap="8">
//!     <Text heading>Settings</Text>
//!     <Button key="save" primary on-click="save">Save</Button>
//!   </Column>
//! "#;
//!
//! let result = rml::parse(src).unwrap();
//! let element = result.element;
//! // element is now an Element::column() tree identical to what the Rust
//! // builder API would produce.
//! # }
//! ```
//!
//! # Warnings
//!
//! Non-fatal issues (unsupported style attributes, duplicate keys, etc.) are
//! returned in [`ParseResult::warnings`] rather than causing errors.
//!
//! # Feature flag
//!
//! This module requires the `rml` Cargo feature, which enables the
//! `quick-xml` dependency.
//!
//! Full language reference: `docs/rml.md` in this repository.

mod lower;
mod parse;
pub mod value;

pub use lower::{RmlAttributeStatus, rml_attribute_status};

use std::collections::BTreeMap;

use crate::Element;

// ── Public types ──────────────────────────────────────────────────────────────

/// The result of a successful [`parse`] call.
#[derive(Debug)]
pub struct ParseResult {
    /// The lowered element tree.
    pub element: Element,
    /// Non-fatal warnings produced during parsing or lowering.
    pub warnings: Vec<RmlWarning>,
}

/// A non-fatal issue found during parsing or lowering.
#[derive(Debug, Clone, PartialEq)]
pub struct RmlWarning {
    pub message: String,
    pub span: Option<RmlSpan>,
}

/// A fatal parse / lowering error.
#[derive(Debug, Clone, PartialEq)]
pub struct RmlError {
    pub message: String,
    pub span: Option<RmlSpan>,
}

impl RmlError {
    pub(crate) fn at(line: usize, column: usize, message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            span: if line == 0 && column == 0 {
                None
            } else {
                Some(RmlSpan { line, column })
            },
        }
    }
}

impl std::fmt::Display for RmlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.span {
            Some(s) => write!(f, "rml error at {}:{}: {}", s.line, s.column, self.message),
            None => write!(f, "rml error: {}", self.message),
        }
    }
}

impl std::error::Error for RmlError {}

/// Source location (1-based line and column).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RmlSpan {
    pub line: usize,
    pub column: usize,
}

// ── Internal IR ───────────────────────────────────────────────────────────────

/// Parsed RML document with a single root node.
pub(crate) struct RmlDocument {
    pub root: RmlNode,
}

/// A single element in the RML tree.
pub(crate) struct RmlNode {
    pub tag: String,
    pub attributes: BTreeMap<String, String>,
    pub children: Vec<RmlChild>,
    pub span: RmlSpan,
}

/// A child of an [`RmlNode`].
pub(crate) enum RmlChild {
    Element(RmlNode),
    Text(String),
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Parse an RML string and lower it to an [`Element`] tree.
///
/// Returns a [`ParseResult`] containing the element and any non-fatal
/// [`RmlWarning`]s, or an [`RmlError`] if the input is invalid.
pub fn parse(input: &str) -> Result<ParseResult, RmlError> {
    let doc = parse::parse_document(input)?;
    let mut warnings = Vec::new();
    let element = lower::lower_document(&doc.root, &mut warnings)?;
    Ok(ParseResult { element, warnings })
}
