//! XML pull-parser that turns RML source text into an [`RmlDocument`].
//!
//! Uses `quick-xml` as the tokeniser. The output is a simple tree of
//! [`RmlNode`] / [`RmlChild`] values that the lowering pass then converts
//! into [`crate::Element`].

use std::collections::BTreeMap;

use quick_xml::{
    Reader,
    events::{BytesStart, Event},
};

use super::{RmlChild, RmlDocument, RmlError, RmlNode, RmlSpan};

// ── public entry point ────────────────────────────────────────────────────────

pub fn parse_document(input: &str) -> Result<RmlDocument, RmlError> {
    let mut reader = Reader::from_str(input);
    reader.config_mut().trim_text(false);

    let mut stack: Vec<RmlNode> = Vec::new();
    let mut roots: Vec<RmlNode> = Vec::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let node = start_event_to_node(e, &reader)?;
                stack.push(node);
            }
            Ok(Event::Empty(ref e)) => {
                let node = start_event_to_node(e, &reader)?;
                push_child(&mut stack, &mut roots, RmlChild::Element(node));
            }
            Ok(Event::End(_)) => {
                let finished = stack
                    .pop()
                    .ok_or_else(|| RmlError::at(0, 0, "unexpected closing tag"))?;
                push_child(&mut stack, &mut roots, RmlChild::Element(finished));
            }
            Ok(Event::Text(ref e)) => {
                let text = e
                    .unescape()
                    .map_err(|err| RmlError::at(0, 0, format!("XML text error: {err}")))?;
                let text = text.to_string();
                if !text.trim().is_empty() {
                    push_child(
                        &mut stack,
                        &mut roots,
                        RmlChild::Text(text.trim().to_string()),
                    );
                }
            }
            Ok(Event::CData(ref e)) => {
                let text = String::from_utf8_lossy(e.as_ref()).to_string();
                if !text.trim().is_empty() {
                    push_child(
                        &mut stack,
                        &mut roots,
                        RmlChild::Text(text.trim().to_string()),
                    );
                }
            }
            // Skip comments, processing instructions, DOCTYPE, etc.
            Ok(Event::Comment(_))
            | Ok(Event::PI(_))
            | Ok(Event::Decl(_))
            | Ok(Event::DocType(_)) => {}
            Ok(Event::Eof) => break,
            Err(e) => {
                let (line, col) = byte_pos_to_line_col(input, reader.error_position() as usize);
                return Err(RmlError::at(line, col, format!("XML parse error: {e}")));
            }
        }
    }

    match roots.len() {
        0 => Err(RmlError::at(
            0,
            0,
            "RML document must have exactly one root element (got none)",
        )),
        1 => Ok(RmlDocument {
            root: roots.remove(0),
        }),
        n => Err(RmlError::at(
            0,
            0,
            format!("RML document must have exactly one root element (got {n})"),
        )),
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn start_event_to_node(e: &BytesStart<'_>, reader: &Reader<&[u8]>) -> Result<RmlNode, RmlError> {
    let (line, col) = byte_pos_to_line_col("", reader.buffer_position() as usize);
    let span = RmlSpan { line, column: col };

    let tag = std::str::from_utf8(e.local_name().as_ref())
        .map_err(|_| RmlError::at(line, col, "tag name is not valid UTF-8"))?
        .to_string();

    let mut attributes: BTreeMap<String, String> = BTreeMap::new();

    // Use Attributes::html() so that shorthand boolean attributes like
    // `<Button disabled />` are accepted. Standalone (valueless) attributes
    // come through with an empty value slice, which we map to "true".
    use quick_xml::events::attributes::Attributes;
    let raw_bytes = e.attributes_raw();
    let raw_str = std::str::from_utf8(raw_bytes)
        .map_err(|_| RmlError::at(line, col, "tag attributes contain non-UTF-8 bytes"))?;
    // pos is only used for error position reporting; since attributes_raw()
    // already excludes the tag name bytes, we start at offset 0.
    let html_attrs = Attributes::html(raw_str, 0);

    for attr_result in html_attrs {
        let attr = attr_result
            .map_err(|err| RmlError::at(line, col, format!("attribute error: {err}")))?;

        let key = std::str::from_utf8(attr.key.local_name().as_ref())
            .map_err(|_| RmlError::at(line, col, "attribute name not valid UTF-8"))?
            .to_string();

        // A standalone attribute (e.g. `disabled`) has an empty value byte slice.
        // We normalise that to "true" so all downstream parsing sees consistent booleans.
        let value = if attr.value.is_empty() {
            "true".to_string()
        } else {
            attr.unescape_value()
                .map_err(|err| RmlError::at(line, col, format!("attribute value error: {err}")))?
                .to_string()
        };

        attributes.insert(key, value);
    }

    Ok(RmlNode {
        tag,
        attributes,
        children: Vec::new(),
        span,
    })
}

/// Push a completed child either onto the top of the stack or into the root list.
fn push_child(stack: &mut Vec<RmlNode>, roots: &mut Vec<RmlNode>, child: RmlChild) {
    if let Some(parent) = stack.last_mut() {
        parent.children.push(child);
    } else {
        match child {
            RmlChild::Element(node) => roots.push(node),
            // Bare text outside any element is silently discarded.
            RmlChild::Text(_) => {}
        }
    }
}

/// Convert a byte offset into (1-based line, 1-based column) by scanning `src`.
/// Falls back to (0, 0) when `src` is empty.
fn byte_pos_to_line_col(src: &str, byte_pos: usize) -> (usize, usize) {
    if src.is_empty() {
        return (0, 0);
    }
    let clamped = byte_pos.min(src.len());
    let before = &src[..clamped];
    let line = before.bytes().filter(|&b| b == b'\n').count() + 1;
    let col = before
        .rfind('\n')
        .map(|p| clamped - p)
        .unwrap_or(clamped + 1);
    (line, col)
}
