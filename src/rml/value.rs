//! Attribute value parsers for RML.
//!
//! Each function takes a raw string slice and a span for error context, and
//! returns the parsed Rust value or an `RmlError`.

use std::collections::BTreeMap;

use crate::{
    Align, Color, CursorIcon, Display, Edge, FlexDirection, FlexWrap, FontStretch, FontStyle,
    FontWeight, GridPlacement, GridTrack, Justify, Length, Overflow, Position, Shadow, Transform,
};

use super::{RmlError, RmlSpan};

// ── helpers ─────────────────────────────────────────────────────────────────

fn err(msg: impl Into<String>, span: RmlSpan) -> RmlError {
    RmlError {
        message: msg.into(),
        span: Some(span),
    }
}

// ── bool ────────────────────────────────────────────────────────────────────

pub fn parse_bool(s: &str, span: RmlSpan) -> Result<bool, RmlError> {
    match s.trim() {
        "true" | "1" | "yes" => Ok(true),
        "false" | "0" | "no" => Ok(false),
        other => Err(err(
            format!("expected bool (true/false/1/0/yes/no), got `{other}`"),
            span,
        )),
    }
}

// ── usize ───────────────────────────────────────────────────────────────────

pub fn parse_usize(s: &str, span: RmlSpan) -> Result<usize, RmlError> {
    s.trim()
        .parse::<usize>()
        .map_err(|_| err(format!("expected non-negative integer, got `{s}`"), span))
}

// ── f32 ─────────────────────────────────────────────────────────────────────

pub fn parse_f32(s: &str, span: RmlSpan) -> Result<f32, RmlError> {
    s.trim()
        .parse::<f32>()
        .map_err(|_| err(format!("expected number, got `{s}`"), span))
}

// ── Length ───────────────────────────────────────────────────────────────────

pub fn parse_length(s: &str, span: RmlSpan) -> Result<Length, RmlError> {
    let s = s.trim();
    match s {
        "auto" => return Ok(Length::Auto),
        "min-content" => return Ok(Length::MinContent),
        "max-content" => return Ok(Length::MaxContent),
        _ => {}
    }
    // fit-content(…)
    if let Some(inner) = s
        .strip_prefix("fit-content(")
        .and_then(|s| s.strip_suffix(')'))
    {
        let inner = parse_length(inner, span)?;
        return Ok(Length::FitContent(Box::new(inner)));
    }
    // percentage
    if let Some(num) = s.strip_suffix('%') {
        let n = num
            .trim()
            .parse::<f32>()
            .map_err(|_| err(format!("invalid percentage `{s}`"), span))?;
        return Ok(Length::Percent(n / 100.0));
    }
    // fr unit
    if let Some(num) = s.strip_suffix("fr") {
        let n = num
            .trim()
            .parse::<f32>()
            .map_err(|_| err(format!("invalid fr value `{s}`"), span))?;
        return Ok(Length::Fr(n));
    }
    // px (explicit or bare number)
    let num_str = s.strip_suffix("px").unwrap_or(s);
    num_str.trim().parse::<f32>().map(Length::Px).map_err(|_| {
        err(format!("invalid length `{s}` — expected px, %, fr, auto, min-content, max-content, or fit-content(…)"), span)
    })
}

// ── Edge<Length> ─────────────────────────────────────────────────────────────
//
// CSS shorthand: 1 value = all, 2 = vertical horizontal, 4 = top right bottom left.

pub fn parse_edge(s: &str, span: RmlSpan) -> Result<Edge<Length>, RmlError> {
    let parts: Vec<&str> = s.split_whitespace().collect();
    match parts.as_slice() {
        [a] => {
            let v = parse_length(a, span)?;
            Ok(Edge {
                top: v.clone(),
                right: v.clone(),
                bottom: v.clone(),
                left: v,
            })
        }
        [v, h] => {
            let vertical = parse_length(v, span)?;
            let horizontal = parse_length(h, span)?;
            Ok(Edge {
                top: vertical.clone(),
                bottom: vertical,
                right: horizontal.clone(),
                left: horizontal,
            })
        }
        [top, right, bottom, left] => Ok(Edge {
            top: parse_length(top, span)?,
            right: parse_length(right, span)?,
            bottom: parse_length(bottom, span)?,
            left: parse_length(left, span)?,
        }),
        _ => Err(err(
            format!("edge expects 1, 2, or 4 values, got {}: `{s}`", parts.len()),
            span,
        )),
    }
}

// ── Color ────────────────────────────────────────────────────────────────────

pub fn parse_color(s: &str, span: RmlSpan) -> Result<Color, RmlError> {
    let s = s.trim();
    // rgb(r,g,b) / rgba(r,g,b,a)
    if s.starts_with("rgba(") && s.ends_with(')') {
        let inner = &s[5..s.len() - 1];
        let parts: Vec<&str> = inner.split(',').map(str::trim).collect();
        if parts.len() == 4 {
            let r = parts[0]
                .parse::<u8>()
                .map_err(|_| err(format!("bad rgba `{s}`"), span))?;
            let g = parts[1]
                .parse::<u8>()
                .map_err(|_| err(format!("bad rgba `{s}`"), span))?;
            let b = parts[2]
                .parse::<u8>()
                .map_err(|_| err(format!("bad rgba `{s}`"), span))?;
            let a = parts[3]
                .parse::<f32>()
                .map_err(|_| err(format!("bad rgba alpha `{s}`"), span))?;
            let a8 = (a.clamp(0.0, 1.0) * 255.0).round() as u8;
            return Ok(Color::rgba(r, g, b, a8));
        }
    }
    if s.starts_with("rgb(") && s.ends_with(')') {
        let inner = &s[4..s.len() - 1];
        let parts: Vec<&str> = inner.split(',').map(str::trim).collect();
        if parts.len() == 3 {
            let r = parts[0]
                .parse::<u8>()
                .map_err(|_| err(format!("bad rgb `{s}`"), span))?;
            let g = parts[1]
                .parse::<u8>()
                .map_err(|_| err(format!("bad rgb `{s}`"), span))?;
            let b = parts[2]
                .parse::<u8>()
                .map_err(|_| err(format!("bad rgb `{s}`"), span))?;
            return Ok(Color::rgb(r, g, b));
        }
    }
    // hex
    if let Some(hex) = s.strip_prefix('#') {
        return parse_hex_color(hex, span);
    }
    Err(err(format!("invalid color `{s}`"), span))
}

fn parse_hex_color(hex: &str, span: RmlSpan) -> Result<Color, RmlError> {
    let bad = || err(format!("invalid hex color `#{hex}`"), span);
    match hex.len() {
        3 => {
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).map_err(|_| bad())?;
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).map_err(|_| bad())?;
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).map_err(|_| bad())?;
            Ok(Color::rgb(r, g, b))
        }
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| bad())?;
            let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| bad())?;
            let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| bad())?;
            Ok(Color::rgb(r, g, b))
        }
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| bad())?;
            let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| bad())?;
            let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| bad())?;
            let a = u8::from_str_radix(&hex[6..8], 16).map_err(|_| bad())?;
            Ok(Color::rgba(r, g, b, a))
        }
        _ => Err(bad()),
    }
}

// ── Align ─────────────────────────────────────────────────────────────────────

pub fn parse_align(s: &str, span: RmlSpan) -> Result<Align, RmlError> {
    match s.trim() {
        "start" => Ok(Align::Start),
        "center" => Ok(Align::Center),
        "end" => Ok(Align::End),
        "stretch" => Ok(Align::Stretch),
        other => Err(err(
            format!("expected align (start/center/end/stretch), got `{other}`"),
            span,
        )),
    }
}

// ── Justify ───────────────────────────────────────────────────────────────────

pub fn parse_justify(s: &str, span: RmlSpan) -> Result<Justify, RmlError> {
    match s.trim() {
        "start" => Ok(Justify::Start),
        "center" => Ok(Justify::Center),
        "end" => Ok(Justify::End),
        "space-between" => Ok(Justify::SpaceBetween),
        "space-around" => Ok(Justify::SpaceAround),
        other => Err(err(
            format!(
                "expected justify (start/center/end/space-between/space-around), got `{other}`"
            ),
            span,
        )),
    }
}

// ── Overflow ──────────────────────────────────────────────────────────────────

pub fn parse_overflow(s: &str, span: RmlSpan) -> Result<Overflow, RmlError> {
    match s.trim() {
        "visible" => Ok(Overflow::Visible),
        "hidden" => Ok(Overflow::Hidden),
        "clip" => Ok(Overflow::Clip),
        "scroll" => Ok(Overflow::Scroll),
        "auto" => Ok(Overflow::Auto),
        other => Err(err(
            format!("expected overflow (visible/hidden/clip/scroll/auto), got `{other}`"),
            span,
        )),
    }
}

// ── Display ───────────────────────────────────────────────────────────────────

pub fn parse_display(s: &str, span: RmlSpan) -> Result<Display, RmlError> {
    match s.trim() {
        "flex" => Ok(Display::Flex),
        "grid" => Ok(Display::Grid),
        "block" => Ok(Display::Block),
        "stack" => Ok(Display::Stack),
        "none" => Ok(Display::None),
        other => Err(err(
            format!("expected display (flex/grid/block/stack/none), got `{other}`"),
            span,
        )),
    }
}

// ── Position ──────────────────────────────────────────────────────────────────

pub fn parse_position(s: &str, span: RmlSpan) -> Result<Position, RmlError> {
    match s.trim() {
        "relative" => Ok(Position::Relative),
        "absolute" => Ok(Position::Absolute),
        "fixed" => Ok(Position::Fixed),
        other => Err(err(
            format!("expected position (relative/absolute/fixed), got `{other}`"),
            span,
        )),
    }
}

// ── FontWeight ────────────────────────────────────────────────────────────────

pub fn parse_font_weight(s: &str, span: RmlSpan) -> Result<FontWeight, RmlError> {
    match s.trim() {
        "thin" => return Ok(FontWeight::Thin),
        "extra-light" => return Ok(FontWeight::ExtraLight),
        "light" => return Ok(FontWeight::Light),
        "normal" => return Ok(FontWeight::Normal),
        "medium" => return Ok(FontWeight::Medium),
        "semibold" => return Ok(FontWeight::Semibold),
        "bold" => return Ok(FontWeight::Bold),
        "extra-bold" => return Ok(FontWeight::ExtraBold),
        "black" => return Ok(FontWeight::Black),
        _ => {}
    }
    // numeric 100-900
    if let Ok(n) = s.trim().parse::<u16>() {
        return Ok(FontWeight::Number(n));
    }
    Err(err(format!("invalid font-weight `{s}`"), span))
}

// ── FontStyle ─────────────────────────────────────────────────────────────────

pub fn parse_font_style(s: &str, span: RmlSpan) -> Result<FontStyle, RmlError> {
    match s.trim() {
        "normal" => Ok(FontStyle::Normal),
        "italic" => Ok(FontStyle::Italic),
        "oblique" => Ok(FontStyle::Oblique),
        other => Err(err(
            format!("expected font-style (normal/italic/oblique), got `{other}`"),
            span,
        )),
    }
}

// ── FontStretch ───────────────────────────────────────────────────────────────

pub fn parse_font_stretch(s: &str, span: RmlSpan) -> Result<FontStretch, RmlError> {
    match s.trim() {
        "condensed" => Ok(FontStretch::Condensed),
        "normal" => Ok(FontStretch::Normal),
        "expanded" => Ok(FontStretch::Expanded),
        other => {
            // Try to parse as percentage e.g. "75%"
            if let Some(num) = other.strip_suffix('%') {
                let n = num
                    .trim()
                    .parse::<f32>()
                    .map_err(|_| err(format!("invalid font-stretch `{s}`"), span))?;
                return Ok(FontStretch::Percent(n));
            }
            Err(err(format!("invalid font-stretch `{s}`"), span))
        }
    }
}

// ── CursorIcon ────────────────────────────────────────────────────────────────

pub fn parse_cursor(s: &str, span: RmlSpan) -> Result<CursorIcon, RmlError> {
    match s.trim() {
        "default" => Ok(CursorIcon::Default),
        "pointer" => Ok(CursorIcon::Pointer),
        "text" => Ok(CursorIcon::Text),
        "resize-horizontal" => Ok(CursorIcon::ResizeHorizontal),
        "resize-vertical" => Ok(CursorIcon::ResizeVertical),
        other => Err(err(
            format!(
                "expected cursor (default/pointer/text/resize-horizontal/resize-vertical), got `{other}`"
            ),
            span,
        )),
    }
}

// ── GridTrack list ────────────────────────────────────────────────────────────

pub fn parse_shadow_list(s: &str, span: RmlSpan) -> Result<Vec<Shadow>, RmlError> {
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() < 5 {
        return Err(err(
            format!("shadow expects `offset-x offset-y blur spread color`, got `{s}`"),
            span,
        ));
    }

    let offset_x = parse_f32(parts[0], span)?;
    let offset_y = parse_f32(parts[1], span)?;
    let blur = parse_f32(parts[2], span)?;
    let spread = parse_f32(parts[3], span)?;
    let color = parse_color(&parts[4..].join(" "), span)?;

    Ok(vec![Shadow {
        color,
        offset_x,
        offset_y,
        blur,
        spread,
    }])
}

pub fn parse_transform(s: &str, span: RmlSpan) -> Result<Transform, RmlError> {
    let mut transform = Transform::default();

    for token in s
        .split(')')
        .map(str::trim)
        .filter(|token| !token.is_empty())
    {
        let token = format!("{token})");
        if let Some(inner) = token
            .strip_prefix("translate(")
            .and_then(|v| v.strip_suffix(')'))
        {
            let (x, y) = parse_pair(inner, span, "translate")?;
            transform.translate_x = x;
            transform.translate_y = y;
        } else if let Some(inner) = token
            .strip_prefix("scale(")
            .and_then(|v| v.strip_suffix(')'))
        {
            let (x, y) = parse_pair(inner, span, "scale")?;
            transform.scale_x = x;
            transform.scale_y = y;
        } else if let Some(inner) = token
            .strip_prefix("rotate(")
            .and_then(|v| v.strip_suffix(')'))
        {
            transform.rotate_radians = parse_f32(inner, span)?;
        } else {
            return Err(err(format!("unsupported transform `{token}`"), span));
        }
    }

    Ok(transform)
}

fn parse_pair(s: &str, span: RmlSpan, name: &str) -> Result<(f32, f32), RmlError> {
    let parts: Vec<&str> = s
        .split([',', ' '])
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect();

    match parts.as_slice() {
        [one] => {
            let v = parse_f32(one, span)?;
            Ok((v, v))
        }
        [x, y] => Ok((parse_f32(x, span)?, parse_f32(y, span)?)),
        _ => Err(err(
            format!("{name} expects one or two numbers, got `{s}`"),
            span,
        )),
    }
}

pub fn parse_grid_track_list(s: &str, span: RmlSpan) -> Result<Vec<GridTrack>, RmlError> {
    s.split_whitespace()
        .map(|token| parse_grid_track(token, span))
        .collect()
}

fn parse_grid_track(s: &str, span: RmlSpan) -> Result<GridTrack, RmlError> {
    match s {
        "auto" => Ok(GridTrack::Auto),
        _ if s.ends_with("fr") => {
            let n = s[..s.len() - 2]
                .parse::<f32>()
                .map_err(|_| err(format!("invalid grid track `{s}`"), span))?;
            Ok(GridTrack::Fraction(n))
        }
        _ => {
            let len = parse_length(s, span)?;
            Ok(GridTrack::Fixed(len))
        }
    }
}

// ── GridPlacement ─────────────────────────────────────────────────────────────
//
// Supports: "2", "2 / 4", "span 2"

pub fn parse_grid_placement(s: &str, span: RmlSpan) -> Result<GridPlacement, RmlError> {
    let s = s.trim();
    if let Some(n) = s.strip_prefix("span ") {
        let span_val = n
            .trim()
            .parse::<u32>()
            .map_err(|_| err(format!("invalid grid span `{s}`"), span))?;
        return Ok(GridPlacement::span(span_val));
    }
    if let Some((start, end)) = s.split_once('/') {
        let start = start
            .trim()
            .parse::<i32>()
            .map_err(|_| err(format!("invalid grid placement start `{s}`"), span))?;
        let end = end
            .trim()
            .parse::<i32>()
            .map_err(|_| err(format!("invalid grid placement end `{s}`"), span))?;
        return Ok(GridPlacement {
            start: Some(start),
            end: Some(end),
            span: None,
        });
    }
    let line = s
        .parse::<i32>()
        .map_err(|_| err(format!("invalid grid placement `{s}`"), span))?;
    Ok(GridPlacement::start(line))
}

// ── FlexDirection ─────────────────────────────────────────────────────────────

pub fn parse_flex_direction(s: &str, span: RmlSpan) -> Result<FlexDirection, RmlError> {
    match s.trim() {
        "row" => Ok(FlexDirection::Row),
        "row-reverse" => Ok(FlexDirection::RowReverse),
        "column" => Ok(FlexDirection::Column),
        "column-reverse" => Ok(FlexDirection::ColumnReverse),
        other => Err(err(format!("invalid flex-direction `{other}`"), span)),
    }
}

// ── FlexWrap ──────────────────────────────────────────────────────────────────

pub fn parse_flex_wrap(s: &str, span: RmlSpan) -> Result<FlexWrap, RmlError> {
    match s.trim() {
        "nowrap" | "no-wrap" => Ok(FlexWrap::NoWrap),
        "wrap" => Ok(FlexWrap::Wrap),
        "wrap-reverse" => Ok(FlexWrap::WrapReverse),
        other => Err(err(format!("invalid flex-wrap `{other}`"), span)),
    }
}

// ── Attribute map helpers ─────────────────────────────────────────────────────

/// Returns `true` if the attribute is present (any value) or its value parses as truthy.
/// Shorthand attributes (`<Button disabled />`) are stored as `"true"` by the parser.
pub fn attr_bool(
    attrs: &BTreeMap<String, String>,
    name: &str,
    span: RmlSpan,
) -> Result<Option<bool>, RmlError> {
    match attrs.get(name) {
        None => Ok(None),
        Some(v) => parse_bool(v, span).map(Some),
    }
}

pub fn attr_str<'a>(attrs: &'a BTreeMap<String, String>, name: &str) -> Option<&'a str> {
    attrs.get(name).map(String::as_str)
}
