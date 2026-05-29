//! Lowers an [`RmlNode`] tree into an [`Element`] tree.
//!
//! Each recognised tag maps to exactly one Rust builder call sequence.
//! Unknown tags produce an [`RmlError`]. Unsupported-but-known style attributes
//! produce an [`RmlWarning`].

use std::collections::BTreeMap;

use crate::widgets::{
    alert, avatar, badge, button, canvas, card, checkbox, context_menu, divider, icon, image,
    input, link, list, menu, menu_item, modal, popover, progress_bar, radio, scroll_area, select,
    slider, spinner, switch, tab, table, tabs, text, textarea, tooltip, tree, tree_item,
};
use crate::{
    Background, Border, Element, Length, MenuItemSpec, Overflow, Paint, Radius, SelectOption,
    Style, TextStyle, WidgetSpec,
};

use super::value::*;
use super::{RmlChild, RmlError, RmlNode, RmlSpan, RmlWarning};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RmlAttributeStatus {
    Implemented,
    ParsedOnly,
    Warning,
    Error,
}

pub fn rml_attribute_status(tag: &str, attr: &str) -> RmlAttributeStatus {
    match (tag, attr) {
        ("Button", "disabled")
        | ("TextInput", "disabled")
        | ("Textarea", "disabled")
        | ("Checkbox", "disabled")
        | ("Radio", "disabled")
        | ("Select", "disabled")
        | ("Option", "disabled")
        | ("MenuItem", "disabled")
        | ("Modal", "close-on-escape")
        | ("Modal", "close-on-outside-click") => RmlAttributeStatus::Implemented,
        ("Modal", "initial-focus") => RmlAttributeStatus::ParsedOnly,
        (_, "style") => RmlAttributeStatus::Warning,
        _ => RmlAttributeStatus::Implemented,
    }
}

// ── public entry ──────────────────────────────────────────────────────────────

pub fn lower_document(root: &RmlNode, warnings: &mut Vec<RmlWarning>) -> Result<Element, RmlError> {
    warn_duplicate_keys(root, warnings);
    lower_node(root, warnings)
}

fn warn_duplicate_keys(root: &RmlNode, warnings: &mut Vec<RmlWarning>) {
    let mut seen = BTreeMap::<String, RmlSpan>::new();
    collect_duplicate_keys(root, &mut seen, warnings);
}

fn collect_duplicate_keys(
    node: &RmlNode,
    seen: &mut BTreeMap<String, RmlSpan>,
    warnings: &mut Vec<RmlWarning>,
) {
    if let Some(key) = attr_str(&node.attributes, "key") {
        if seen.insert(key.to_string(), node.span).is_some() {
            warnings.push(RmlWarning {
                message: format!("duplicate key `{key}` in RML document"),
                span: Some(node.span),
            });
        }
    }

    for child in &node.children {
        if let RmlChild::Element(child) = child {
            collect_duplicate_keys(child, seen, warnings);
        }
    }
}

// ── node dispatch ─────────────────────────────────────────────────────────────

fn lower_node(node: &RmlNode, warnings: &mut Vec<RmlWarning>) -> Result<Element, RmlError> {
    let span = node.span;
    let el = match node.tag.as_str() {
        // ── Layout ────────────────────────────────────────────────────────────
        "Row" => {
            let mut el = Element::row();
            el = apply_common(el, &node.attributes, span, warnings)?;
            el = append_children(el, &node.children, warnings)?;
            el
        }
        "Column" => {
            let mut el = Element::column();
            el = apply_common(el, &node.attributes, span, warnings)?;
            el = append_children(el, &node.children, warnings)?;
            el
        }
        "Grid" => {
            let mut el = Element::grid();
            el = apply_common(el, &node.attributes, span, warnings)?;
            el = append_children(el, &node.children, warnings)?;
            el
        }
        "Stack" => {
            let mut el = Element::stack();
            el = apply_common(el, &node.attributes, span, warnings)?;
            el = append_children(el, &node.children, warnings)?;
            el
        }
        "Absolute" => {
            let mut el = Element::absolute();
            el = apply_common(el, &node.attributes, span, warnings)?;
            el = append_children(el, &node.children, warnings)?;
            el
        }
        "ScrollArea" => {
            let mut el = scroll_area();
            el = apply_common(el, &node.attributes, span, warnings)?;
            if let Some(axis) = attr_str(&node.attributes, "axis") {
                match axis {
                    "x" => {
                        el.style.overflow_x = Some(Overflow::Scroll);
                        el.style.overflow_y = Some(Overflow::Hidden);
                    }
                    "y" => {
                        el.style.overflow_x = Some(Overflow::Hidden);
                        el.style.overflow_y = Some(Overflow::Scroll);
                    }
                    "both" => {
                        el.style.overflow_x = Some(Overflow::Scroll);
                        el.style.overflow_y = Some(Overflow::Scroll);
                    }
                    other => {
                        return Err(RmlError::at(
                            span.line,
                            span.column,
                            format!("invalid ScrollArea axis `{other}`"),
                        ));
                    }
                }
            }
            el = append_children(el, &node.children, warnings)?;
            el
        }
        "Text" => lower_text(node, warnings)?,
        "Canvas" => lower_canvas(node)?,

        // ── Forms ─────────────────────────────────────────────────────────────
        "Button" => lower_button(node, warnings)?,
        "TextInput" | "Input" => lower_input(node, warnings)?,
        "Checkbox" => lower_checkbox(node, warnings)?,
        "Radio" => lower_radio(node, warnings)?,
        "Select" => lower_select(node, warnings)?,
        "Textarea" => lower_textarea(node, warnings)?,
        "Switch" => lower_switch(node, warnings)?,
        "Slider" => lower_slider(node, warnings)?,

        // ── Collections ───────────────────────────────────────────────────────
        "Tabs" => lower_tabs(node, warnings)?,
        "List" => lower_list(node, warnings)?,
        "Table" => lower_table(node, warnings)?,
        "Tree" => lower_tree(node, warnings)?,
        "Menu" => lower_menu(node, warnings)?,
        "ContextMenu" => lower_context_menu(node, warnings)?,

        // ── Overlays ──────────────────────────────────────────────────────────
        "Popover" => lower_popover(node, warnings)?,
        "Tooltip" => lower_tooltip(node, warnings)?,
        "Modal" => lower_modal(node, warnings)?,

        // ── Primitives ────────────────────────────────────────────────────────
        "Icon" => lower_icon(node, warnings)?,
        "Divider" => {
            let mut el = divider();
            el = apply_common(el, &node.attributes, span, warnings)?;
            el
        }
        "Image" => lower_image(node, warnings)?,
        "Badge" => lower_badge(node, warnings)?,
        "Avatar" => lower_avatar(node, warnings)?,

        // ── Navigation ────────────────────────────────────────────────────────
        "Link" => lower_link(node, warnings)?,

        // ── Feedback ──────────────────────────────────────────────────────────
        "ProgressBar" => lower_progress_bar(node, warnings)?,
        "Spinner" => lower_spinner(node, warnings)?,
        "Alert" => lower_alert(node, warnings)?,

        // ── Layouts ───────────────────────────────────────────────────────────
        "Card" => lower_card(node, warnings)?,

        other => {
            return Err(RmlError::at(
                span.line,
                span.column,
                format!("unknown component tag `<{other}>`"),
            ));
        }
    };
    Ok(el)
}

// ── child helpers ─────────────────────────────────────────────────────────────

fn append_children(
    mut el: Element,
    children: &[RmlChild],
    warnings: &mut Vec<RmlWarning>,
) -> Result<Element, RmlError> {
    for child in children {
        match child {
            RmlChild::Element(node) => {
                let child_el = lower_node(node, warnings)?;
                el = el.child(child_el);
            }
            RmlChild::Text(t) => {
                // Raw text inside a layout container becomes a <Text> node.
                el = el.child(text(t.as_str()));
            }
        }
    }
    Ok(el)
}

/// Collect plain text from an element's children (for <Text> and label nodes).
fn collect_text_content(children: &[RmlChild], span: RmlSpan) -> Result<String, RmlError> {
    let mut result = String::new();
    for child in children {
        match child {
            RmlChild::Text(t) => result.push_str(t),
            RmlChild::Element(n) => {
                return Err(RmlError::at(
                    n.span.line,
                    n.span.column,
                    format!(
                        "<Text> must contain only text, found child element `<{}>`",
                        n.tag
                    ),
                ));
            }
        }
    }
    // Check for the `value` attribute as an override
    let _ = span; // span is kept for potential future use
    Ok(result)
}

// ── Text ──────────────────────────────────────────────────────────────────────

fn lower_text(node: &RmlNode, warnings: &mut Vec<RmlWarning>) -> Result<Element, RmlError> {
    let span = node.span;
    // `value` attribute overrides child text
    let content = if let Some(v) = attr_str(&node.attributes, "value") {
        v.to_string()
    } else {
        collect_text_content(&node.children, span)?
    };
    let mut el = text(content);
    el = apply_common(el, &node.attributes, span, warnings)?;
    Ok(el)
}

// ── Canvas ────────────────────────────────────────────────────────────────────

fn lower_canvas(node: &RmlNode) -> Result<Element, RmlError> {
    let span = node.span;
    let name = attr_str(&node.attributes, "name").ok_or_else(|| {
        RmlError::at(
            span.line,
            span.column,
            "<Canvas> requires a `name` attribute",
        )
    })?;
    let mut el = canvas().named(name).build();
    // Apply common style attributes (width, height, key, etc.)
    // Canvas doesn't use the full common set but we allow layout attrs.
    if let Some(k) = attr_str(&node.attributes, "key") {
        el = el.key(k);
    }
    if let Some(w) = attr_str(&node.attributes, "width") {
        el = el.width(parse_length(w, span)?);
    }
    if let Some(h) = attr_str(&node.attributes, "height") {
        el = el.height(parse_length(h, span)?);
    }
    Ok(el)
}

// ── Button ────────────────────────────────────────────────────────────────────

fn lower_button(node: &RmlNode, warnings: &mut Vec<RmlWarning>) -> Result<Element, RmlError> {
    let span = node.span;
    // Label: text child > `label` attr > empty
    let label = collect_button_label(node);
    let mut el = button(label);
    el = apply_common(el, &node.attributes, span, warnings)?;

    // Disabled / loading
    if let Some(d) = attr_bool(&node.attributes, "disabled", span)? {
        if let Some(WidgetSpec::Button(ref mut bs)) = el.widget_spec {
            bs.disabled = d;
        }
    }
    if let Some(l) = attr_bool(&node.attributes, "loading", span)? {
        if let Some(WidgetSpec::Button(ref mut bs)) = el.widget_spec {
            bs.loading = l;
        }
    }

    // Slot children: Popover / ContextMenu
    for child in &node.children {
        if let RmlChild::Element(child_node) = child {
            let slot = attr_str(&child_node.attributes, "slot");
            match (child_node.tag.as_str(), slot) {
                ("Popover", Some("popover")) => {
                    let overlay = lower_popover(child_node, warnings)?;
                    el = el.popover(overlay);
                }
                ("ContextMenu", Some("context-menu")) => {
                    let cm = lower_context_menu(child_node, warnings)?;
                    el = el.context_menu(cm);
                }
                _ => {
                    // Regular child (non-text children are buttons' children for layout)
                    let child_el = lower_node(child_node, warnings)?;
                    el = el.child(child_el);
                }
            }
        }
        // text children were already used as the label; skip
    }
    Ok(el)
}

/// Returns the text label for a Button: first non-whitespace text child, then `label` attr.
fn collect_button_label(node: &RmlNode) -> String {
    for child in &node.children {
        if let RmlChild::Text(t) = child {
            let t = t.trim();
            if !t.is_empty() {
                return t.to_string();
            }
        }
    }
    attr_str(&node.attributes, "label")
        .unwrap_or("")
        .to_string()
}

// ── TextInput / Input ─────────────────────────────────────────────────────────

fn lower_input(node: &RmlNode, warnings: &mut Vec<RmlWarning>) -> Result<Element, RmlError> {
    let span = node.span;
    let mut el = input();
    el = apply_common(el, &node.attributes, span, warnings)?;

    if let Some(p) = attr_str(&node.attributes, "placeholder") {
        el = el.placeholder(p);
    }
    if let Some(v) = attr_str(&node.attributes, "default-value") {
        el = el.default_value(v);
    }
    if let Some(v) = attr_str(&node.attributes, "value") {
        if let Some(WidgetSpec::Input(ref mut spec)) = el.widget_spec {
            spec.value = Some(v.to_string());
        }
    }
    if let Some(d) = attr_bool(&node.attributes, "disabled", span)? {
        if let Some(WidgetSpec::Input(ref mut spec)) = el.widget_spec {
            spec.disabled = d;
        }
    }
    if let Some(p) = attr_bool(&node.attributes, "password", span)? {
        if let Some(WidgetSpec::Input(ref mut spec)) = el.widget_spec {
            spec.password = p;
        }
    }
    if let Some(al) = attr_str(&node.attributes, "aria-label") {
        el = el.aria_label(al);
    }
    Ok(el)
}

// ── Checkbox ──────────────────────────────────────────────────────────────────

fn lower_checkbox(node: &RmlNode, warnings: &mut Vec<RmlWarning>) -> Result<Element, RmlError> {
    let span = node.span;
    let mut el = checkbox();
    el = apply_common(el, &node.attributes, span, warnings)?;

    // Label: attribute then text child
    let lbl = attr_str(&node.attributes, "label")
        .map(str::to_string)
        .or_else(|| {
            node.children.iter().find_map(|c| match c {
                RmlChild::Text(t) if !t.trim().is_empty() => Some(t.trim().to_string()),
                _ => None,
            })
        });
    if let Some(l) = lbl {
        el = el.label(l);
    }
    if let Some(c) = attr_bool(&node.attributes, "checked", span)? {
        el = el.checked(c);
    }
    if let Some(c) = attr_bool(&node.attributes, "default-checked", span)? {
        el = el.default_checked(c);
    }
    if let Some(d) = attr_bool(&node.attributes, "disabled", span)? {
        if let Some(WidgetSpec::Checkbox(ref mut spec)) = el.widget_spec {
            spec.disabled = d;
        }
    }
    if let Some(i) = attr_bool(&node.attributes, "indeterminate", span)? {
        if let Some(WidgetSpec::Checkbox(ref mut spec)) = el.widget_spec {
            spec.indeterminate = i;
        }
    }
    Ok(el)
}

// ── Radio ─────────────────────────────────────────────────────────────────────

fn lower_radio(node: &RmlNode, warnings: &mut Vec<RmlWarning>) -> Result<Element, RmlError> {
    let span = node.span;
    let mut el = radio();
    el = apply_common(el, &node.attributes, span, warnings)?;

    let lbl = attr_str(&node.attributes, "label")
        .map(str::to_string)
        .or_else(|| {
            node.children.iter().find_map(|c| match c {
                RmlChild::Text(t) if !t.trim().is_empty() => Some(t.trim().to_string()),
                _ => None,
            })
        });
    if let Some(l) = lbl {
        el = el.label(l);
    }
    if let Some(v) = attr_str(&node.attributes, "value") {
        if let Some(WidgetSpec::Radio(ref mut spec)) = el.widget_spec {
            spec.value = Some(v.to_string());
        }
    }
    if let Some(c) = attr_bool(&node.attributes, "checked", span)? {
        el = el.checked(c);
    }
    if let Some(c) = attr_bool(&node.attributes, "default-checked", span)? {
        el = el.default_checked(c);
    }
    if let Some(d) = attr_bool(&node.attributes, "disabled", span)? {
        if let Some(WidgetSpec::Radio(ref mut spec)) = el.widget_spec {
            spec.disabled = d;
        }
    }
    Ok(el)
}

// ── Select ────────────────────────────────────────────────────────────────────

fn lower_select(node: &RmlNode, warnings: &mut Vec<RmlWarning>) -> Result<Element, RmlError> {
    let span = node.span;
    let mut el = select();
    el = apply_common(el, &node.attributes, span, warnings)?;

    if let Some(p) = attr_str(&node.attributes, "placeholder") {
        el = el.placeholder(p);
    }
    if let Some(v) = attr_str(&node.attributes, "default-value") {
        el = el.default_value(v);
    }
    if let Some(si) = attr_str(&node.attributes, "selected-index") {
        el = el.default_selected_index(parse_usize(si, span)?);
    }
    if let Some(d) = attr_bool(&node.attributes, "disabled", span)? {
        if let Some(WidgetSpec::Select(ref mut spec)) = el.widget_spec {
            spec.disabled = d;
        }
    }

    // Collect <Option> / <SelectOption> children
    let mut options: Vec<SelectOption> = Vec::new();
    for child in &node.children {
        if let RmlChild::Element(opt_node) = child {
            if opt_node.tag == "SelectStyle" {
                let part = attr_str(&opt_node.attributes, "part").ok_or_else(|| {
                    RmlError::at(
                        opt_node.span.line,
                        opt_node.span.column,
                        "<SelectStyle> requires a `part` attribute",
                    )
                })?;
                let style = lower_style_only(opt_node, warnings)?;
                if let Some(WidgetSpec::Select(ref mut spec)) = el.widget_spec {
                    match part {
                        "trigger" => spec.styles.trigger = Some(style),
                        "popover" => spec.styles.popover = Some(style),
                        "list" => spec.styles.list = Some(style),
                        "item" => spec.styles.item = Some(style),
                        "item-hovered" => spec.styles.item_hovered = Some(style),
                        "item-selected" => spec.styles.item_selected = Some(style),
                        "item-disabled" => spec.styles.item_disabled = Some(style),
                        other => {
                            return Err(RmlError::at(
                                opt_node.span.line,
                                opt_node.span.column,
                                format!("invalid SelectStyle part `{other}`"),
                            ));
                        }
                    }
                }
            } else if matches!(opt_node.tag.as_str(), "Option" | "SelectOption") {
                let opt_span = opt_node.span;
                let value = attr_str(&opt_node.attributes, "value")
                    .unwrap_or("")
                    .to_string();
                let label = attr_str(&opt_node.attributes, "label")
                    .map(str::to_string)
                    .or_else(|| {
                        opt_node.children.iter().find_map(|c| match c {
                            RmlChild::Text(t) if !t.trim().is_empty() => Some(t.trim().to_string()),
                            _ => None,
                        })
                    })
                    .unwrap_or_else(|| value.clone());
                let disabled =
                    attr_bool(&opt_node.attributes, "disabled", opt_span)?.unwrap_or(false);
                options.push(SelectOption {
                    value,
                    label,
                    disabled,
                });
            } else if opt_node.tag != "SelectStyle" {
                // Unknown child inside <Select>
                warnings.push(RmlWarning {
                    message: format!(
                        "unexpected child `<{}>` inside <Select>; only <Option> is supported",
                        opt_node.tag
                    ),
                    span: Some(opt_node.span),
                });
            }
        }
    }
    if !options.is_empty() {
        el = el.options(options);
    }
    Ok(el)
}

// ── Textarea ──────────────────────────────────────────────────────────────────

fn lower_style_only(node: &RmlNode, warnings: &mut Vec<RmlWarning>) -> Result<Style, RmlError> {
    let el = apply_common(Element::column(), &node.attributes, node.span, warnings)?;
    Ok(el.style)
}

fn lower_textarea(node: &RmlNode, warnings: &mut Vec<RmlWarning>) -> Result<Element, RmlError> {
    let span = node.span;
    let mut el = textarea();
    el = apply_common(el, &node.attributes, span, warnings)?;

    if let Some(p) = attr_str(&node.attributes, "placeholder") {
        el = el.placeholder(p);
    }
    if let Some(v) = attr_str(&node.attributes, "default-value") {
        el = el.default_value(v);
    }
    if let Some(v) = attr_str(&node.attributes, "value") {
        if let Some(WidgetSpec::Textarea(ref mut spec)) = el.widget_spec {
            spec.value = Some(v.to_string());
        }
    }
    if let Some(d) = attr_bool(&node.attributes, "disabled", span)? {
        if let Some(WidgetSpec::Textarea(ref mut spec)) = el.widget_spec {
            spec.disabled = d;
        }
    }
    if let Some(r) = attr_str(&node.attributes, "rows") {
        if let Some(WidgetSpec::Textarea(ref mut spec)) = el.widget_spec {
            spec.rows = Some(parse_usize(r, span)?);
        }
    }
    Ok(el)
}

// ── Switch ────────────────────────────────────────────────────────────────────

fn lower_switch(node: &RmlNode, warnings: &mut Vec<RmlWarning>) -> Result<Element, RmlError> {
    let span = node.span;
    let mut el = switch();
    el = apply_common(el, &node.attributes, span, warnings)?;
    if let Some(d) = attr_bool(&node.attributes, "disabled", span)? {
        if let Some(WidgetSpec::Switch(ref mut spec)) = el.widget_spec {
            spec.disabled = d;
        }
    }
    if let Some(c) = attr_bool(&node.attributes, "checked", span)? {
        if let Some(WidgetSpec::Switch(ref mut spec)) = el.widget_spec {
            spec.checked = c;
        }
    }
    if let Some(l) = collect_text_content(&node.children, span).ok() {
        if !l.is_empty() {
            el = el.label(l);
        }
    }
    Ok(el)
}

// ── Slider ────────────────────────────────────────────────────────────────────

fn lower_slider(node: &RmlNode, warnings: &mut Vec<RmlWarning>) -> Result<Element, RmlError> {
    let span = node.span;
    let mut el = slider();
    el = apply_common(el, &node.attributes, span, warnings)?;
    if let Some(v) = attr_str(&node.attributes, "min") {
        if let Some(WidgetSpec::Slider(ref mut spec)) = el.widget_spec {
            spec.min = parse_f32(v, span)?;
        }
    }
    if let Some(v) = attr_str(&node.attributes, "max") {
        if let Some(WidgetSpec::Slider(ref mut spec)) = el.widget_spec {
            spec.max = parse_f32(v, span)?;
        }
    }
    if let Some(v) = attr_str(&node.attributes, "step") {
        if let Some(WidgetSpec::Slider(ref mut spec)) = el.widget_spec {
            spec.step = Some(parse_f32(v, span)?);
        }
    }
    if let Some(v) = attr_str(&node.attributes, "value") {
        if let Some(WidgetSpec::Slider(ref mut spec)) = el.widget_spec {
            spec.value = parse_f32(v, span)?;
        }
    }
    if let Some(d) = attr_bool(&node.attributes, "disabled", span)? {
        if let Some(WidgetSpec::Slider(ref mut spec)) = el.widget_spec {
            spec.disabled = d;
        }
    }
    Ok(el)
}

// ── Tabs ──────────────────────────────────────────────────────────────────────

fn lower_tabs(node: &RmlNode, warnings: &mut Vec<RmlWarning>) -> Result<Element, RmlError> {
    let span = node.span;
    let mut el = tabs();
    el = apply_common(el, &node.attributes, span, warnings)?;

    // Compact form: tabs="General,Advanced"
    if let Some(tabs_str) = attr_str(&node.attributes, "tabs") {
        let tab_labels: Vec<&str> = tabs_str.split(',').map(str::trim).collect();
        el = el.tabs(tab_labels.iter().copied());
        for label in tab_labels {
            el = el.child(tab(label));
        }
    }

    // active-index / default-active-index
    let ai = attr_str(&node.attributes, "active-index")
        .or_else(|| attr_str(&node.attributes, "default-active-index"));
    if let Some(s) = ai {
        el = el.default_active_index(parse_usize(s, span)?);
    }

    // <Tab> children
    let mut tab_labels_from_children: Vec<String> = Vec::new();
    for child in &node.children {
        if let RmlChild::Element(tab_node) = child {
            if tab_node.tag == "Tab" {
                let label = attr_str(&tab_node.attributes, "label")
                    .map(str::to_string)
                    .or_else(|| {
                        tab_node.children.iter().find_map(|c| match c {
                            RmlChild::Text(t) if !t.trim().is_empty() => Some(t.trim().to_string()),
                            _ => None,
                        })
                    })
                    .unwrap_or_default();
                tab_labels_from_children.push(label.clone());
                el = el.child(tab(label));
            } else {
                warnings.push(RmlWarning {
                    message: format!("unexpected child `<{}>` inside <Tabs>", tab_node.tag),
                    span: Some(tab_node.span),
                });
            }
        }
    }
    if !tab_labels_from_children.is_empty() {
        el = el.tabs(tab_labels_from_children.iter().map(String::as_str));
    }
    Ok(el)
}

// ── List ──────────────────────────────────────────────────────────────────────

fn lower_list(node: &RmlNode, warnings: &mut Vec<RmlWarning>) -> Result<Element, RmlError> {
    let span = node.span;
    let mut el = list();
    el = apply_common(el, &node.attributes, span, warnings)?;

    // Compact: items="Inbox,Today,Done"
    let mut item_labels: Vec<String> = if let Some(items_str) = attr_str(&node.attributes, "items")
    {
        items_str.split(',').map(|s| s.trim().to_string()).collect()
    } else {
        Vec::new()
    };

    // selected-index / default-selected-index
    let si = attr_str(&node.attributes, "selected-index")
        .or_else(|| attr_str(&node.attributes, "default-selected-index"));
    if let Some(s) = si {
        el = el.default_selected_index(parse_usize(s, span)?);
    }

    // <ListItem> children
    for child in &node.children {
        if let RmlChild::Element(item_node) = child {
            if item_node.tag == "ListItem" {
                let label = item_node
                    .children
                    .iter()
                    .find_map(|c| match c {
                        RmlChild::Text(t) if !t.trim().is_empty() => Some(t.trim().to_string()),
                        _ => None,
                    })
                    .unwrap_or_default();
                item_labels.push(label);
            } else {
                warnings.push(RmlWarning {
                    message: format!("unexpected child `<{}>` inside <List>", item_node.tag),
                    span: Some(item_node.span),
                });
            }
        }
    }
    if !item_labels.is_empty() {
        el = el.items(item_labels);
    }
    Ok(el)
}

// ── Table ─────────────────────────────────────────────────────────────────────

fn lower_table(node: &RmlNode, warnings: &mut Vec<RmlWarning>) -> Result<Element, RmlError> {
    let span = node.span;
    let mut el = table();
    el = apply_common(el, &node.attributes, span, warnings)?;

    // Compact: columns="Name,Status"
    let mut col_labels: Vec<String> = if let Some(cols_str) = attr_str(&node.attributes, "columns")
    {
        cols_str.split(',').map(|s| s.trim().to_string()).collect()
    } else {
        Vec::new()
    };

    // selected-row / default-selected-row
    let sr = attr_str(&node.attributes, "selected-row")
        .or_else(|| attr_str(&node.attributes, "default-selected-row"));
    if let Some(s) = sr {
        el = el.default_selected_row(parse_usize(s, span)?);
    }

    let mut data_rows: Vec<Vec<String>> = Vec::new();

    for child in &node.children {
        if let RmlChild::Element(child_node) = child {
            match child_node.tag.as_str() {
                "Columns" => {
                    for col_child in &child_node.children {
                        if let RmlChild::Element(col_def) = col_child {
                            if col_def.tag == "ColumnDef" {
                                let label = col_def
                                    .children
                                    .iter()
                                    .find_map(|c| match c {
                                        RmlChild::Text(t) if !t.trim().is_empty() => {
                                            Some(t.trim().to_string())
                                        }
                                        _ => None,
                                    })
                                    .unwrap_or_default();
                                col_labels.push(label);
                            }
                        }
                    }
                }
                "TableRow" => {
                    // Compact form: values="Runtime,Ready"
                    if let Some(vals) = attr_str(&child_node.attributes, "values") {
                        data_rows.push(vals.split(',').map(|s| s.trim().to_string()).collect());
                    } else {
                        let row: Vec<String> = child_node
                            .children
                            .iter()
                            .filter_map(|c| match c {
                                RmlChild::Element(cell) if cell.tag == "Cell" => Some(
                                    cell.children
                                        .iter()
                                        .find_map(|cc| match cc {
                                            RmlChild::Text(t) => Some(t.trim().to_string()),
                                            _ => None,
                                        })
                                        .unwrap_or_default(),
                                ),
                                _ => None,
                            })
                            .collect();
                        data_rows.push(row);
                    }
                }
                // <Row> inside <Table> is an alias for <TableRow>
                "Row" => {
                    let row: Vec<String> = child_node
                        .children
                        .iter()
                        .filter_map(|c| match c {
                            RmlChild::Element(cell) if cell.tag == "Cell" => Some(
                                cell.children
                                    .iter()
                                    .find_map(|cc| match cc {
                                        RmlChild::Text(t) => Some(t.trim().to_string()),
                                        _ => None,
                                    })
                                    .unwrap_or_default(),
                            ),
                            _ => None,
                        })
                        .collect();
                    data_rows.push(row);
                }
                other => {
                    warnings.push(RmlWarning {
                        message: format!("unexpected child `<{other}>` inside <Table>"),
                        span: Some(child_node.span),
                    });
                }
            }
        }
    }

    if !col_labels.is_empty() {
        el = el.columns(col_labels);
    }
    if !data_rows.is_empty() {
        if let Some(WidgetSpec::Table(ref mut spec)) = el.widget_spec {
            spec.rows = data_rows;
        }
    }
    Ok(el)
}

// ── Tree ──────────────────────────────────────────────────────────────────────

fn lower_tree(node: &RmlNode, warnings: &mut Vec<RmlWarning>) -> Result<Element, RmlError> {
    let span = node.span;
    let mut el = tree();
    el = apply_common(el, &node.attributes, span, warnings)?;

    let items: Result<Vec<_>, _> = node
        .children
        .iter()
        .filter_map(|c| match c {
            RmlChild::Element(n) if n.tag == "TreeItem" => Some(lower_tree_item(n)),
            RmlChild::Element(n) => {
                warnings.push(RmlWarning {
                    message: format!("unexpected child `<{}>` inside <Tree>", n.tag),
                    span: Some(n.span),
                });
                None
            }
            RmlChild::Text(_) => None,
        })
        .collect();
    el = el.items(items?);
    Ok(el)
}

fn lower_tree_item(node: &RmlNode) -> Result<crate::TreeItemSpec, RmlError> {
    let span = node.span;
    let label = attr_str(&node.attributes, "label")
        .map(str::to_string)
        .or_else(|| {
            // Only use text child if there are no child TreeItems
            let has_child_nodes = node
                .children
                .iter()
                .any(|c| matches!(c, RmlChild::Element(_)));
            if !has_child_nodes {
                node.children.iter().find_map(|c| match c {
                    RmlChild::Text(t) if !t.trim().is_empty() => Some(t.trim().to_string()),
                    _ => None,
                })
            } else {
                None
            }
        })
        .unwrap_or_default();
    let expanded = attr_bool(&node.attributes, "expanded", span)?.unwrap_or(false);
    let mut item = tree_item(label).expanded(expanded);
    for child in &node.children {
        if let RmlChild::Element(child_node) = child {
            if child_node.tag == "TreeItem" {
                item = item.child(lower_tree_item(child_node)?);
            }
        }
    }
    Ok(item)
}

// ── Menu ──────────────────────────────────────────────────────────────────────

fn lower_menu(node: &RmlNode, warnings: &mut Vec<RmlWarning>) -> Result<Element, RmlError> {
    let span = node.span;
    let mut el = menu();
    el = apply_common(el, &node.attributes, span, warnings)?;
    for child in &node.children {
        if let RmlChild::Element(item_node) = child {
            if item_node.tag == "MenuItem" {
                let item_spec = lower_menu_item_spec(item_node)?;
                let item_el = lower_menu_item(item_node, warnings)?;
                if let Some(WidgetSpec::Menu(ref mut spec)) = el.widget_spec {
                    spec.items.push(item_spec);
                }
                el = el.child(item_el);
            } else {
                warnings.push(RmlWarning {
                    message: format!("unexpected child `<{}>` inside <Menu>", item_node.tag),
                    span: Some(item_node.span),
                });
            }
        }
    }
    Ok(el)
}

fn lower_context_menu(node: &RmlNode, warnings: &mut Vec<RmlWarning>) -> Result<Element, RmlError> {
    let span = node.span;
    let mut el = context_menu();
    el = apply_common(el, &node.attributes, span, warnings)?;
    for child in &node.children {
        if let RmlChild::Element(item_node) = child {
            if item_node.tag == "MenuItem" {
                let item_spec = lower_menu_item_spec(item_node)?;
                let item_el = lower_menu_item(item_node, warnings)?;
                if let Some(WidgetSpec::Menu(ref mut spec)) = el.widget_spec {
                    spec.items.push(item_spec);
                }
                el = el.child(item_el);
            } else {
                warnings.push(RmlWarning {
                    message: format!(
                        "unexpected child `<{}>` inside <ContextMenu>",
                        item_node.tag
                    ),
                    span: Some(item_node.span),
                });
            }
        }
    }
    Ok(el)
}

fn lower_menu_item_spec(node: &RmlNode) -> Result<MenuItemSpec, RmlError> {
    let span = node.span;
    let label = menu_item_label(node);
    let action =
        attr_str(&node.attributes, "action").or_else(|| attr_str(&node.attributes, "on-click"));
    Ok(MenuItemSpec {
        label,
        action: action.map(str::to_string),
        disabled: attr_bool(&node.attributes, "disabled", span)?.unwrap_or(false),
        shortcut: attr_str(&node.attributes, "shortcut").map(str::to_string),
    })
}

fn lower_menu_item(node: &RmlNode, warnings: &mut Vec<RmlWarning>) -> Result<Element, RmlError> {
    let span = node.span;
    let label = menu_item_label(node);
    let mut el = menu_item(label);
    el = apply_common(el, &node.attributes, span, warnings)?;
    // action / on-click
    let action =
        attr_str(&node.attributes, "action").or_else(|| attr_str(&node.attributes, "on-click"));
    if let Some(a) = action {
        el = el.on_click(a);
    }
    if let Some(d) = attr_bool(&node.attributes, "disabled", span)? {
        if let Some(WidgetSpec::Button(ref mut spec)) = el.widget_spec {
            spec.disabled = d;
        }
    }
    Ok(el)
}

fn menu_item_label(node: &RmlNode) -> String {
    attr_str(&node.attributes, "label")
        .map(str::to_string)
        .or_else(|| {
            node.children.iter().find_map(|c| match c {
                RmlChild::Text(t) if !t.trim().is_empty() => Some(t.trim().to_string()),
                _ => None,
            })
        })
        .unwrap_or_default()
}

// ── Popover ───────────────────────────────────────────────────────────────────

fn lower_popover(node: &RmlNode, warnings: &mut Vec<RmlWarning>) -> Result<Element, RmlError> {
    let span = node.span;
    let mut el = popover();
    el = apply_common(el, &node.attributes, span, warnings)?;
    if let Some(cl) = attr_str(&node.attributes, "content-label") {
        if let Some(WidgetSpec::Popover(ref mut spec)) = el.widget_spec {
            spec.content_label = Some(cl.to_string());
        }
    }
    el = append_children(el, &node.children, warnings)?;
    Ok(el)
}

// ── Tooltip ───────────────────────────────────────────────────────────────────

fn lower_tooltip(node: &RmlNode, warnings: &mut Vec<RmlWarning>) -> Result<Element, RmlError> {
    let span = node.span;
    let mut el = tooltip();
    el = apply_common(el, &node.attributes, span, warnings)?;
    if let Some(t) = attr_str(&node.attributes, "text") {
        if let Some(WidgetSpec::Tooltip(ref mut spec)) = el.widget_spec {
            spec.text = Some(t.to_string());
        }
    }
    Ok(el)
}

// ── Modal ─────────────────────────────────────────────────────────────────────

fn lower_modal(node: &RmlNode, warnings: &mut Vec<RmlWarning>) -> Result<Element, RmlError> {
    let span = node.span;
    let mut el = modal();
    el = apply_common(el, &node.attributes, span, warnings)?;
    if let Some(title) = attr_str(&node.attributes, "title") {
        if let Some(WidgetSpec::Modal(ref mut spec)) = el.widget_spec {
            spec.title = Some(title.to_string());
        }
    }
    if let Some(v) = attr_bool(&node.attributes, "close-on-escape", span)? {
        if let Some(WidgetSpec::Modal(ref mut spec)) = el.widget_spec {
            spec.close_on_escape = v;
        }
    }
    if let Some(v) = attr_bool(&node.attributes, "close-on-outside-click", span)? {
        if let Some(WidgetSpec::Modal(ref mut spec)) = el.widget_spec {
            spec.close_on_outside_click = v;
        }
    }
    if node.attributes.contains_key("initial-focus") {
        warnings.push(RmlWarning {
            message: "`initial-focus` on <Modal> is parsed but not behaviorally implemented yet"
                .to_string(),
            span: Some(span),
        });
    }
    if let Some(open) = attr_bool(&node.attributes, "open", span)? {
        el = el.open(open);
    }
    el = append_children(el, &node.children, warnings)?;
    Ok(el)
}

// ── Icon ──────────────────────────────────────────────────────────────────────

fn lower_icon(node: &RmlNode, warnings: &mut Vec<RmlWarning>) -> Result<Element, RmlError> {
    let span = node.span;
    let name = attr_str(&node.attributes, "name").ok_or_else(|| {
        RmlError::at(span.line, span.column, "<Icon> requires a `name` attribute")
    })?;
    let mut el = icon(name);
    el = apply_common(el, &node.attributes, span, warnings)?;
    Ok(el)
}

// ── Image ─────────────────────────────────────────────────────────────────────

fn lower_image(node: &RmlNode, warnings: &mut Vec<RmlWarning>) -> Result<Element, RmlError> {
    let span = node.span;
    let src = attr_str(&node.attributes, "src").ok_or_else(|| {
        RmlError::at(span.line, span.column, "<Image> requires a `src` attribute")
    })?;
    let mut el = image(src);
    el = apply_common(el, &node.attributes, span, warnings)?;
    if let Some(alt) = attr_str(&node.attributes, "alt") {
        if let Some(WidgetSpec::Image(ref mut spec)) = el.widget_spec {
            spec.alt = Some(alt.to_string());
        }
    }
    Ok(el)
}

// ── Badge ─────────────────────────────────────────────────────────────────────

fn lower_badge(node: &RmlNode, warnings: &mut Vec<RmlWarning>) -> Result<Element, RmlError> {
    let span = node.span;
    let text = collect_text_content(&node.children, span)?;
    let mut el = badge(text);
    el = apply_common(el, &node.attributes, span, warnings)?;
    Ok(el)
}

// ── Avatar ────────────────────────────────────────────────────────────────────

fn lower_avatar(node: &RmlNode, warnings: &mut Vec<RmlWarning>) -> Result<Element, RmlError> {
    let span = node.span;
    let mut el = avatar();
    el = apply_common(el, &node.attributes, span, warnings)?;
    if let Some(src) = attr_str(&node.attributes, "src") {
        if let Some(WidgetSpec::Avatar(ref mut spec)) = el.widget_spec {
            spec.src = Some(src.to_string());
        }
    }
    if let Some(initials) = attr_str(&node.attributes, "initials") {
        if let Some(WidgetSpec::Avatar(ref mut spec)) = el.widget_spec {
            spec.initials = Some(initials.to_string());
        }
    }
    Ok(el)
}

// ── Link ──────────────────────────────────────────────────────────────────────

fn lower_link(node: &RmlNode, warnings: &mut Vec<RmlWarning>) -> Result<Element, RmlError> {
    let span = node.span;
    let label = collect_text_content(&node.children, span)?;
    let mut el = link(label);
    el = apply_common(el, &node.attributes, span, warnings)?;
    if let Some(href) = attr_str(&node.attributes, "href") {
        if let Some(WidgetSpec::Link(ref mut spec)) = el.widget_spec {
            spec.href = Some(href.to_string());
        }
    }
    Ok(el)
}

// ── ProgressBar ───────────────────────────────────────────────────────────────

fn lower_progress_bar(
    node: &RmlNode,
    warnings: &mut Vec<RmlWarning>,
) -> Result<Element, RmlError> {
    let span = node.span;
    let mut el = progress_bar();
    el = apply_common(el, &node.attributes, span, warnings)?;
    if let Some(v) = attr_str(&node.attributes, "value") {
        if let Some(WidgetSpec::ProgressBar(ref mut spec)) = el.widget_spec {
            spec.value = parse_f32(v, span)?;
        }
    }
    if let Some(v) = attr_str(&node.attributes, "max") {
        if let Some(WidgetSpec::ProgressBar(ref mut spec)) = el.widget_spec {
            spec.max = parse_f32(v, span)?;
        }
    }
    if let Some(v) = attr_bool(&node.attributes, "indeterminate", span)? {
        if let Some(WidgetSpec::ProgressBar(ref mut spec)) = el.widget_spec {
            spec.indeterminate = v;
        }
    }
    Ok(el)
}

// ── Spinner ───────────────────────────────────────────────────────────────────

fn lower_spinner(node: &RmlNode, warnings: &mut Vec<RmlWarning>) -> Result<Element, RmlError> {
    let span = node.span;
    let mut el = spinner();
    el = apply_common(el, &node.attributes, span, warnings)?;
    if let Some(l) = attr_str(&node.attributes, "label") {
        if let Some(WidgetSpec::Spinner(ref mut spec)) = el.widget_spec {
            spec.label = Some(l.to_string());
        }
    }
    Ok(el)
}

// ── Alert ─────────────────────────────────────────────────────────────────────

fn lower_alert(node: &RmlNode, warnings: &mut Vec<RmlWarning>) -> Result<Element, RmlError> {
    let span = node.span;
    let mut el = alert();
    el = apply_common(el, &node.attributes, span, warnings)?;
    if let Some(t) = attr_str(&node.attributes, "title") {
        if let Some(WidgetSpec::Alert(ref mut spec)) = el.widget_spec {
            spec.title = Some(t.to_string());
        }
    }
    el = append_children(el, &node.children, warnings)?;
    Ok(el)
}

// ── Card ──────────────────────────────────────────────────────────────────────

fn lower_card(node: &RmlNode, warnings: &mut Vec<RmlWarning>) -> Result<Element, RmlError> {
    let span = node.span;
    let mut el = card();
    el = apply_common(el, &node.attributes, span, warnings)?;
    if let Some(t) = attr_str(&node.attributes, "title") {
        if let Some(WidgetSpec::Card(ref mut spec)) = el.widget_spec {
            spec.title = Some(t.to_string());
        }
    }
    el = append_children(el, &node.children, warnings)?;
    Ok(el)
}

// ── Common attribute application ──────────────────────────────────────────────

fn apply_common(
    mut el: Element,
    attrs: &BTreeMap<String, String>,
    span: RmlSpan,
    warnings: &mut Vec<RmlWarning>,
) -> Result<Element, RmlError> {
    warn_unsupported_common_attrs(attrs, span, warnings);

    // key
    if let Some(k) = attr_str(attrs, "key") {
        el = el.key(k);
    }
    // width / height
    if let Some(w) = attr_str(attrs, "width") {
        el = el.width(parse_length(w, span)?);
    }
    if let Some(h) = attr_str(attrs, "height") {
        el = el.height(parse_length(h, span)?);
    }
    // min/max width/height  (go through style)
    if let Some(v) = attr_str(attrs, "min-width") {
        el.style.min_width = Some(parse_length(v, span)?);
    }
    if let Some(v) = attr_str(attrs, "max-width") {
        el.style.max_width = Some(parse_length(v, span)?);
    }
    if let Some(v) = attr_str(attrs, "min-height") {
        el.style.min_height = Some(parse_length(v, span)?);
    }
    if let Some(v) = attr_str(attrs, "max-height") {
        el.style.max_height = Some(parse_length(v, span)?);
    }
    // padding
    if let Some(p) = attr_str(attrs, "padding") {
        el.style.padding = Some(parse_edge(p, span)?);
    }
    // margin
    if let Some(m) = attr_str(attrs, "margin") {
        el.style.margin = Some(parse_edge(m, span)?);
    }
    // gap
    if let Some(g) = attr_str(attrs, "gap").or_else(|| attr_str(attrs, "align")) {
        // `align` is an alias for gap on Row/Column per spec §3.1
        let _ = g;
    }
    if let Some(g) = attr_str(attrs, "gap") {
        el = el.gap(length_to_f32(parse_length(g, span)?));
    }
    // align-items / align (alias)
    let ai = attr_str(attrs, "align-items").or_else(|| attr_str(attrs, "align"));
    if let Some(a) = ai {
        el.style.align_items = Some(parse_align(a, span)?);
    }
    // align-self
    if let Some(a) = attr_str(attrs, "align-self") {
        el.style.align_self = Some(parse_align(a, span)?);
    }
    // align-content
    if let Some(a) = attr_str(attrs, "align-content") {
        el.style.align_content = Some(parse_align(a, span)?);
    }
    // justify-content / justify (alias)
    let jc = attr_str(attrs, "justify-content").or_else(|| attr_str(attrs, "justify"));
    if let Some(j) = jc {
        el.style.justify_content = Some(parse_justify(j, span)?);
    }
    // flex-*
    if let Some(v) = attr_str(attrs, "flex-direction") {
        el.style.flex_direction = Some(parse_flex_direction(v, span)?);
    }
    if let Some(v) = attr_str(attrs, "flex-wrap") {
        el.style.flex_wrap = Some(parse_flex_wrap(v, span)?);
    }
    if let Some(v) = attr_str(attrs, "flex-grow") {
        el.style.flex_grow = Some(parse_f32(v, span)?);
    }
    if let Some(v) = attr_str(attrs, "flex-shrink") {
        el.style.flex_shrink = Some(parse_f32(v, span)?);
    }
    if let Some(v) = attr_str(attrs, "flex-basis") {
        el.style.flex_basis = Some(parse_length(v, span)?);
    }
    // aspect-ratio
    if let Some(v) = attr_str(attrs, "aspect-ratio") {
        el.style.aspect_ratio = Some(parse_f32(v, span)?);
    }
    // inset
    if let Some(v) = attr_str(attrs, "inset") {
        el.style.inset = Some(parse_edge(v, span)?);
    }
    // overflow / overflow-x / overflow-y
    if let Some(v) = attr_str(attrs, "overflow") {
        let ov = parse_overflow(v, span)?;
        el.style.overflow_x = Some(ov);
        el.style.overflow_y = Some(ov);
    }
    if let Some(v) = attr_str(attrs, "overflow-x") {
        el.style.overflow_x = Some(parse_overflow(v, span)?);
    }
    if let Some(v) = attr_str(attrs, "overflow-y") {
        el.style.overflow_y = Some(parse_overflow(v, span)?);
    }
    // display
    if let Some(v) = attr_str(attrs, "display") {
        el.style.display = Some(parse_display(v, span)?);
    }
    // position
    if let Some(v) = attr_str(attrs, "position") {
        el.style.position = Some(parse_position(v, span)?);
    }
    // z-index
    if let Some(v) = attr_str(attrs, "z-index") {
        el =
            el.z_index(v.trim().parse::<i32>().map_err(|_| {
                RmlError::at(span.line, span.column, format!("invalid z-index `{v}`"))
            })?);
    }
    // grid
    if let Some(v) = attr_str(attrs, "grid-template-columns") {
        el.style.grid_template_columns = Some(parse_grid_track_list(v, span)?);
    }
    if let Some(v) = attr_str(attrs, "grid-template-rows") {
        el.style.grid_template_rows = Some(parse_grid_track_list(v, span)?);
    }
    if let Some(v) = attr_str(attrs, "grid-column") {
        el.style.grid_column = Some(parse_grid_placement(v, span)?);
    }
    if let Some(v) = attr_str(attrs, "grid-row") {
        el.style.grid_row = Some(parse_grid_placement(v, span)?);
    }
    // background
    if let Some(v) = attr_str(attrs, "background") {
        let color = parse_color(v, span)?;
        el.style.background = Some(Background::Paint(Paint::Solid(color)));
    }
    // border (MVP: "width color" or just a color)
    if let Some(v) = attr_str(attrs, "border") {
        let parts: Vec<&str> = v.split_whitespace().collect();
        match parts.as_slice() {
            [width_str, color_str] => {
                let width_str = width_str.strip_suffix("px").unwrap_or(width_str);
                let width = width_str.parse::<f32>().map_err(|_| {
                    RmlError::at(span.line, span.column, format!("invalid border `{v}`"))
                })?;
                let color = parse_color(color_str, span)?;
                el.style.border = Some(Border { color, width });
            }
            [color_str] => {
                let color = parse_color(color_str, span)?;
                el.style.border = Some(Border { color, width: 1.0 });
            }
            _ => {
                warnings.push(RmlWarning {
                    message: format!("unsupported border shorthand `{v}` (use `<width> <color>`)"),
                    span: Some(span),
                });
            }
        }
    }
    // radius
    if let Some(v) = attr_str(attrs, "radius") {
        let parts: Vec<&str> = v.split_whitespace().collect();
        let r = match parts.as_slice() {
            [a] => {
                let n = a
                    .strip_suffix("px")
                    .unwrap_or(a)
                    .parse::<f32>()
                    .map_err(|_| {
                        RmlError::at(span.line, span.column, format!("invalid radius `{v}`"))
                    })?;
                Radius::all(n)
            }
            [tl, tr, br, bl] => Radius {
                top_left: parse_radius_val(tl, span)?,
                top_right: parse_radius_val(tr, span)?,
                bottom_right: parse_radius_val(br, span)?,
                bottom_left: parse_radius_val(bl, span)?,
            },
            _ => {
                warnings.push(RmlWarning {
                    message: format!("unsupported radius shorthand `{v}`"),
                    span: Some(span),
                });
                Radius::all(0.0)
            }
        };
        el.style.radius = Some(r);
    }
    // opacity
    if let Some(v) = attr_str(attrs, "opacity") {
        el.style.opacity = Some(parse_f32(v, span)?);
    }
    // effects
    if let Some(v) = attr_str(attrs, "shadow") {
        el.style.shadow = Some(parse_shadow_list(v, span)?);
    }
    if let Some(v) = attr_str(attrs, "transform") {
        el.style.transform = Some(parse_transform(v, span)?);
    }
    // cursor
    if let Some(v) = attr_str(attrs, "cursor") {
        el.style.cursor = Some(parse_cursor(v, span)?);
    }
    // text style
    apply_text_style(&mut el, attrs, span)?;
    // variant / primary
    if let Some(v) = attr_str(attrs, "variant") {
        el = el.variant(v);
    }
    if attr_bool(attrs, "primary", span)?.unwrap_or(false) {
        el = el.primary();
    }
    if attr_bool(attrs, "heading", span)?.unwrap_or(false) {
        el = el.heading();
    }
    // on-click
    if let Some(a) = attr_str(attrs, "on-click") {
        el = el.on_click(a);
    }
    // draggable
    if let Some(p) = attr_str(attrs, "draggable") {
        el = el.draggable(p);
    }
    // open
    if let Some(o) = attr_bool(attrs, "open", span)? {
        el = el.open(o);
    }
    // aria-label
    if let Some(al) = attr_str(attrs, "aria-label") {
        el = el.aria_label(al);
    }
    // label (generic)
    if let Some(l) = attr_str(attrs, "label") {
        el = el.label(l);
    }
    // checked / default-checked
    if let Some(c) = attr_bool(attrs, "checked", span)? {
        el = el.checked(c);
    }
    if let Some(c) = attr_bool(attrs, "default-checked", span)? {
        el = el.default_checked(c);
    }

    Ok(el)
}

fn warn_unsupported_common_attrs(
    attrs: &BTreeMap<String, String>,
    span: RmlSpan,
    warnings: &mut Vec<RmlWarning>,
) {
    if attrs.contains_key("style") {
        warnings.push(RmlWarning {
            message: "`style` is recognized by RML but is not implemented yet".to_string(),
            span: Some(span),
        });
    }
}

fn apply_text_style(
    el: &mut Element,
    attrs: &BTreeMap<String, String>,
    span: RmlSpan,
) -> Result<(), RmlError> {
    let has_text = attrs.contains_key("font-family")
        || attrs.contains_key("font-size")
        || attrs.contains_key("font-weight")
        || attrs.contains_key("font-style")
        || attrs.contains_key("font-stretch")
        || attrs.contains_key("color");
    if !has_text {
        return Ok(());
    }
    let mut ts = el.style.text.take().unwrap_or_else(TextStyle::default);
    if let Some(v) = attr_str(attrs, "font-family") {
        ts.family = v.split(',').map(|s| s.trim().to_string()).collect();
    }
    if let Some(v) = attr_str(attrs, "font-size") {
        ts.size = parse_length(v, span)?;
    }
    if let Some(v) = attr_str(attrs, "font-weight") {
        ts.weight = parse_font_weight(v, span)?;
    }
    if let Some(v) = attr_str(attrs, "font-style") {
        ts.style = parse_font_style(v, span)?;
    }
    if let Some(v) = attr_str(attrs, "font-stretch") {
        ts.stretch = parse_font_stretch(v, span)?;
    }
    if let Some(v) = attr_str(attrs, "color") {
        ts.color = parse_color(v, span)?;
    }
    el.style.text = Some(ts);
    Ok(())
}

// ── small helpers ──────────────────────────────────────────────────────────────

/// Returns the px value of a Length, or 0.0 for non-px variants.
fn length_to_f32(l: Length) -> f32 {
    match l {
        Length::Px(v) => v,
        Length::Percent(v) => v,
        Length::Fr(v) => v,
        _ => 0.0,
    }
}

fn parse_radius_val(s: &str, span: RmlSpan) -> Result<f32, RmlError> {
    s.strip_suffix("px")
        .unwrap_or(s)
        .parse::<f32>()
        .map_err(|_| {
            RmlError::at(
                span.line,
                span.column,
                format!("invalid radius value `{s}`"),
            )
        })
}
