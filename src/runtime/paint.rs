use crate::core::{
    BorderCmd, Color, Element, ElementKind, FontStyle, FontWeight, Paint, PaintCommand,
    PaintCommandSnapshot, Point, Rect, RectCmd, ResolvedStateFlags, ResolvedWidgetStyle, Size,
    TextCmd, Theme, ThemeMode, WidgetKind,
};
use crate::text_engine::TextSystem;

use super::UiNode;

const DEFAULT_TEXT_SIZE: f32 = 14.0;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct VisualState {
    pub checked: bool,
    pub text: Option<String>,
    pub label: Option<String>,
    pub focused: bool,
    pub active: bool,
    pub hovered: bool,
    pub disabled: bool,
    pub open: bool,
    pub primary: bool,
    pub cursor: Option<usize>,
    pub preedit: Option<crate::core::ImePreedit>,
    pub content_size: Size,
    pub scroll_offset: crate::core::Vec2,
    pub tree_expanded: std::collections::HashMap<usize, bool>,
    pub table_selected_row: Option<usize>,
    pub list_selected_index: Option<usize>,
    pub tabs_active_index: Option<usize>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PaintedCommand {
    pub command: PaintCommand,
    pub snapshot: PaintCommandSnapshot,
}

pub(crate) fn visual_state_for_element(element: &Element) -> VisualState {
    VisualState {
        checked: element.checked.unwrap_or(false),
        text: None,
        label: element.semantic.label.clone().or_else(|| {
            element.children.iter().find_map(|child| match &child.kind {
                ElementKind::Text(spec) => Some(spec.text.clone()),
                _ => None,
            })
        }),
        focused: false,
        active: false,
        hovered: false,
        disabled: false,
        open: element.open,
        primary: element
            .variant
            .as_ref()
            .is_some_and(|variant| variant.as_str() == "primary"),
        cursor: None,
        preedit: None,
        content_size: Size::default(),
        scroll_offset: crate::core::Vec2::default(),
        tree_expanded: std::collections::HashMap::new(),
        table_selected_row: None,
        list_selected_index: None,
        tabs_active_index: None,
    }
}

pub fn paint_node(
    node: &UiNode,
    rect: Rect,
    z_index: i32,
    state: &VisualState,
) -> Vec<PaintedCommand> {
    let mut text = TextSystem::default();
    paint_node_with_text(node, rect, z_index, state, &mut text)
}

pub fn paint_node_with_text(
    node: &UiNode,
    rect: Rect,
    z_index: i32,
    state: &VisualState,
    text: &mut TextSystem,
) -> Vec<PaintedCommand> {
    paint_node_themed(node, rect, z_index, state, text, None)
}

pub fn paint_node_themed(
    node: &UiNode,
    rect: Rect,
    z_index: i32,
    state: &VisualState,
    text: &mut TextSystem,
    theme: Option<&Theme>,
) -> Vec<PaintedCommand> {
    match &node.kind {
        ElementKind::Text(spec) => {
            let style = text_style_for_node(node, theme);
            vec![text_command(text, spec.text.clone(), rect, style, z_index)]
        }
        ElementKind::Canvas(spec) => {
            let default_theme;
            let theme = match theme {
                Some(theme) => theme,
                None => {
                    default_theme = Theme::light();
                    &default_theme
                }
            };
            let style = theme.resolve_widget_style(
                WidgetKind::Canvas,
                None,
                &ResolvedStateFlags::default(),
            );
            let metrics = WidgetPaintMetrics {
                metrics: &theme.widgets.metrics,
            };
            canvas_commands(rect, Some(spec.name.as_str()), z_index, &style, metrics)
        }
        ElementKind::Widget(WidgetKind::Modal | WidgetKind::Popover | WidgetKind::Tooltip)
            if !node.open =>
        {
            Vec::new()
        }
        ElementKind::Widget(kind) => {
            paint_widget_themed(node, *kind, rect, z_index, state, text, theme)
        }
        ElementKind::Primitive(crate::core::PrimitiveKind::ScrollArea) => {
            let default_theme;
            let theme = match theme {
                Some(theme) => theme,
                None => {
                    default_theme = Theme::light();
                    &default_theme
                }
            };
            let style = theme.resolve_widget_style(
                WidgetKind::ScrollArea,
                None,
                &ResolvedStateFlags::default(),
            );
            let metrics = WidgetPaintMetrics {
                metrics: &theme.widgets.metrics,
            };
            scroll_area_commands(rect, z_index, state, &style, metrics)
        }
        ElementKind::Primitive(_) if node.parent.is_none() => {
            let default_theme;
            let theme = match theme {
                Some(theme) => theme,
                None => {
                    default_theme = Theme::light();
                    &default_theme
                }
            };
            vec![rect_command(rect, theme.colors.background, 0.0, -1)]
        }
        _ => Vec::new(),
    }
}

fn paint_widget_themed(
    node: &UiNode,
    kind: WidgetKind,
    rect: Rect,
    z_index: i32,
    state: &VisualState,
    text: &mut TextSystem,
    theme: Option<&Theme>,
) -> Vec<PaintedCommand> {
    let mut flags = ResolvedStateFlags::new(state.hovered, state.focused, state.active);
    flags.disabled = state.disabled;
    flags.checked = state.checked;
    flags.open = state.open;
    let variant = if state.primary {
        Some(crate::core::VariantId::new("primary"))
    } else {
        None
    };
    let default_theme;
    let theme = match theme {
        Some(theme) => theme,
        None => {
            default_theme = Theme::light();
            &default_theme
        }
    };
    let style = theme.resolve_widget_style(kind, variant, &flags);
    let metrics = WidgetPaintMetrics {
        metrics: &theme.widgets.metrics,
    };
    let mut ctx = PaintCtx {
        rect,
        z_index,
        state,
        style: &style,
        metrics,
        node,
        theme,
        text,
    };
    widget_painter_for(kind).paint(&mut ctx)
}

// ─── Widget paint metrics ─────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug)]
struct WidgetPaintMetrics<'a> {
    metrics: &'a crate::WidgetMetrics,
}

impl<'a> WidgetPaintMetrics<'a> {
    fn input_text_origin(self, rect: Rect) -> Point {
        Point::new(
            rect.origin.x + self.metrics.input.horizontal_padding,
            rect.origin.y + rect.size.height * 0.5,
        )
    }

    fn input_text_top_left(self, rect: Rect) -> Point {
        Point::new(
            rect.origin.x + self.metrics.input.horizontal_padding,
            rect.origin.y + self.metrics.input.vertical_padding,
        )
    }

    fn input_text_measure_width(self, rect: Rect) -> f32 {
        (rect.size.width - self.metrics.input.horizontal_padding * 2.0).max(0.0)
    }

    fn textarea_text_origin(self, rect: Rect) -> Point {
        Point::new(
            rect.origin.x + self.metrics.textarea.horizontal_padding,
            rect.origin.y
                + self.metrics.textarea.vertical_padding
                + self.metrics.input.min_size.height * 0.4444,
        )
    }

    fn textarea_text_top_left(self, rect: Rect) -> Point {
        Point::new(
            rect.origin.x + self.metrics.textarea.horizontal_padding,
            rect.origin.y + self.metrics.textarea.vertical_padding,
        )
    }

    fn textarea_text_measure_width(self, rect: Rect) -> f32 {
        (rect.size.width - self.metrics.textarea.horizontal_padding * 2.0).max(0.0)
    }

    fn divider_thickness(self) -> f32 {
        self.metrics.divider.thickness
    }

    fn tab_text_size(self) -> f32 {
        DEFAULT_TEXT_SIZE
    }

    fn tab_rects(
        self,
        labels: &[String],
        origin: Point,
        text_system: &mut TextSystem,
    ) -> Vec<Rect> {
        calculate_tab_rects_with_metrics(labels, origin, text_system, self.metrics)
    }
}

pub fn debug_resolved_widget_style(kind: WidgetKind, themed: bool) -> ResolvedWidgetStyle {
    let state = ResolvedStateFlags::default();
    if themed {
        Theme::light().resolve_widget_style(kind, None, &state)
    } else {
        ResolvedWidgetStyle::default_for(kind, &state)
    }
}

// ─── Paint context ────────────────────────────────────────────────────────────

/// All data a [`WidgetPainter`] needs to produce its draw commands.
pub struct PaintCtx<'a> {
    pub rect: Rect,
    pub z_index: i32,
    pub state: &'a VisualState,
    pub style: &'a ResolvedWidgetStyle,
    pub metrics: WidgetPaintMetrics<'a>,
    pub node: &'a UiNode,
    pub theme: &'a Theme,
    pub text: &'a mut TextSystem,
}

impl<'a> PaintCtx<'a> {
    /// Rect command at `self.z_index + z_offset`.
    pub fn draw_rect(&self, rect: Rect, color: Color, radius: f32, z_offset: i32) -> PaintedCommand {
        rect_command(rect, color, radius, self.z_index + z_offset)
    }

    /// Border command at `self.z_index + z_offset`.
    pub fn draw_border(
        &self,
        rect: Rect,
        color: Color,
        width: f32,
        z_offset: i32,
    ) -> PaintedCommand {
        border_command(rect, color, width, self.z_index + z_offset)
    }

    /// Text command at `self.z_index + z_offset`.
    pub fn draw_text(
        &self,
        text: String,
        origin: Point,
        color: Color,
        z_offset: i32,
    ) -> PaintedCommand {
        text_at(text, origin, color, self.z_index + z_offset)
    }
}

// ─── WidgetPainter trait ──────────────────────────────────────────────────────

/// Unified rendering contract with a **template method**.
///
/// # Layer contract
/// The default `paint` emits two commands before calling `paint_content`:
/// - **z + 0** — background rect (color from `background_color`, radius from `style`)
/// - **z + 1** — border (skipped when `has_border()` returns `false`)
///
/// `paint_content` must therefore start all commands at **z + 2** or higher.
///
/// Widgets that have a fundamentally different background geometry (e.g. [`CheckboxPainter`],
/// [`RadioPainter`]) may override `paint` entirely and manage their own z-layers.
pub trait WidgetPainter {
    // ── Override hooks ────────────────────────────────────────────────────────

    /// Background fill color. Default: `ctx.style.background`.
    ///
    /// Override to use active-state colors, theme colors, or a fixed tint
    /// without re-implementing the whole background draw.
    fn background_color(&self, ctx: &PaintCtx<'_>) -> Color {
        ctx.style.background
    }

    /// Whether to emit a border from `ctx.style` at `z + 1`. Default: `true`.
    ///
    /// Set to `false` for widgets whose border is part of their unique content
    /// (e.g. Canvas axis lines) or that have no visible border at all.
    fn has_border(&self) -> bool {
        true
    }

    // ── Required: widget-specific content ────────────────────────────────────

    /// Widget-specific rendering, called after the background and optional border.
    /// All commands emitted here should use **`ctx.z_index + 2`** or higher.
    ///
    /// The default implementation is empty — only override for widget-specific output.
    fn paint_content(&self, _ctx: &mut PaintCtx<'_>, _cmds: &mut Vec<PaintedCommand>) {}

    // ── Template method ───────────────────────────────────────────────────────

    /// Emits: `background_color` rect → optional border → `paint_content`.
    ///
    /// Override **only** when the background geometry itself differs
    /// (sub-rect controls, invisible widgets, single-line shapes, etc.).
    fn paint(&self, ctx: &mut PaintCtx<'_>) -> Vec<PaintedCommand> {
        let mut cmds = vec![rect_command(
            ctx.rect,
            self.background_color(ctx),
            ctx.style.border_radius,
            ctx.z_index,
        )];
        if self.has_border() {
            cmds.push(border_command(
                ctx.rect,
                ctx.style.border_color,
                ctx.style.border_width,
                ctx.z_index + 1,
            ));
        }
        self.paint_content(ctx, &mut cmds);
        cmds
    }
}

// ─── Factory ──────────────────────────────────────────────────────────────────

fn widget_painter_for(kind: WidgetKind) -> Box<dyn WidgetPainter> {
    match kind {
        WidgetKind::Button       => Box::new(ButtonPainter),
        WidgetKind::Input        => Box::new(InputPainter),
        WidgetKind::Textarea     => Box::new(TextareaPainter),
        WidgetKind::Checkbox     => Box::new(CheckboxPainter),
        WidgetKind::Radio        => Box::new(RadioPainter),
        WidgetKind::Select       => Box::new(SelectPainter),
        WidgetKind::Tabs         => Box::new(TabsPainter),
        WidgetKind::Tree         => Box::new(TreePainter),
        WidgetKind::Table        => Box::new(TablePainter),
        WidgetKind::List         => Box::new(ListPainter),
        WidgetKind::ScrollArea   => Box::new(ScrollAreaPainter),
        WidgetKind::Menu         => Box::new(MenuPainter),
        WidgetKind::Tooltip      => Box::new(TooltipPainter),
        WidgetKind::Icon         => Box::new(IconPainter),
        WidgetKind::Divider      => Box::new(DividerPainter),
        WidgetKind::Canvas       => Box::new(CanvasPainter),
        WidgetKind::Modal | WidgetKind::Popover => Box::new(InvisiblePainter),
        WidgetKind::Text         => Box::new(TextBackgroundPainter),
        _                        => Box::new(GenericPainter),
    }
}

// ─── Shared helpers ───────────────────────────────────────────────────────────

/// Selection highlight color, consistent across List and Table.
fn selection_color(theme: &Theme) -> Color {
    if theme.mode == ThemeMode::Dark {
        Color::rgba(
            theme.colors.primary.r,
            theme.colors.primary.g,
            theme.colors.primary.b,
            40,
        )
    } else {
        Color::rgb(219, 234, 254)
    }
}

/// Shared text-field rendering: preedit, caret, and placeholder.
///
/// Appends to `cmds` starting at `ctx.z_index + 2` (above the template bg/border).
/// Callers must pass the correct origin/top-left/width from their metrics.
fn paint_text_field(
    ctx: &mut PaintCtx<'_>,
    text_origin: Point,
    text_top_left: Point,
    measure_width: f32,
    cmds: &mut Vec<PaintedCommand>,
) {
    // z+2 = text / placeholder  z+3 = caret / preedit underlines
    let z = ctx.z_index;
    let text_val = ctx.state.text.clone().unwrap_or_default();

    if ctx.state.preedit.is_some() || !text_val.is_empty() {
        let (display_text, caret_idx, preedit_info) = if let Some(preedit) = &ctx.state.preedit {
            let mut disp = text_val.clone();
            let cursor_idx = ctx.state.cursor.unwrap_or(disp.len());
            let clamped = cursor_idx.min(disp.len());
            disp.insert_str(clamped, &preedit.text);
            let c_idx = if let Some((_, end)) = preedit.cursor_byte_range {
                clamped + end
            } else {
                clamped + preedit.text.len()
            };
            (disp, c_idx, Some((clamped, preedit.text.len())))
        } else {
            (text_val, ctx.state.cursor.unwrap_or(0), None)
        };

        let layout = ctx.text.measure(
            &display_text,
            ctx.style.font_size,
            ctx.style.font_weight,
            FontStyle::Normal,
            measure_width,
        );

        if ctx.state.focused {
            let caret_rect = layout.caret_rect(caret_idx, text_top_left);
            cmds.push(rect_command(caret_rect, ctx.style.text_color, 0.0, z + 3));
        }

        if let Some((start, len)) = preedit_info {
            let preedit_range = start..(start + len);
            let preedit_rects = layout.selection_rects(preedit_range, text_top_left);
            for preedit_rect in preedit_rects {
                let underline_rect = Rect::new(
                    Point::new(preedit_rect.origin.x, preedit_rect.max_y() - 1.0),
                    Size::new(preedit_rect.size.width, 1.0),
                );
                cmds.push(rect_command(underline_rect, ctx.style.text_color, 0.0, z + 3));
            }
        }

        cmds.push(text_at(display_text, text_origin, ctx.style.text_color, z + 2));
    } else if let Some(placeholder) = ctx.state.label.as_ref().filter(|l| !l.is_empty()) {
        cmds.push(text_at(placeholder.clone(), text_origin, ctx.style.text_muted_color, z + 2));
        if ctx.state.focused {
            let layout = ctx.text.measure(
                "",
                ctx.style.font_size,
                ctx.style.font_weight,
                FontStyle::Normal,
                measure_width,
            );
            let caret_rect = layout.caret_rect(0, text_top_left);
            cmds.push(rect_command(caret_rect, ctx.style.text_color, 0.0, z + 3));
        }
    } else if ctx.state.focused {
        let layout = ctx.text.measure(
            "",
            ctx.style.font_size,
            ctx.style.font_weight,
            FontStyle::Normal,
            measure_width,
        );
        let caret_rect = layout.caret_rect(0, text_top_left);
        cmds.push(rect_command(caret_rect, ctx.style.text_color, 0.0, z + 3));
    }
}

/// Module-level helper for recursive tree rendering.
/// Lifted from the old inner `fn paint_tree_item` to avoid re-declaration on every call.
fn paint_tree_items(
    items: &[crate::TreeItemSpec],
    depth: usize,
    index: &mut usize,
    rect: Rect,
    z_index: i32,
    cmds: &mut Vec<PaintedCommand>,
    expanded_state: &std::collections::HashMap<usize, bool>,
    metrics: crate::TreeMetrics,
    text_color: Color,
) {
    for item in items {
        let y = rect.origin.y + metrics.indent * 0.5 + *index as f32 * metrics.row_height;
        let item_index = *index;
        let indent = " ".repeat(depth);
        let expanded = expanded_state
            .get(&item_index)
            .copied()
            .unwrap_or(item.expanded);
        let marker = if item.children.is_empty() {
            "  "
        } else if expanded {
            "v "
        } else {
            "> "
        };
        let display_text = format!("{}{}{}", indent, marker, item.label);
        cmds.push(text_at(
            display_text,
            Point::new(
                rect.origin.x + metrics.indent * 0.5,
                y + metrics.row_height * 0.7,
            ),
            text_color,
            z_index + 2,
        ));
        *index += 1;
        if expanded && !item.children.is_empty() {
            paint_tree_items(
                &item.children,
                depth + 1,
                index,
                rect,
                z_index,
                cmds,
                expanded_state,
                metrics,
                text_color,
            );
        }
    }
}

// ─── Select helpers ───────────────────────────────────────────────────────────

fn resolved_select_part_styles(
    node: &UiNode,
    theme: Option<&Theme>,
) -> crate::widgets::SelectPartStyles {
    let mut styles = crate::widgets::SelectPartStyles::default();
    if let Some(theme) = theme {
        styles = merge_select_part_styles(styles, theme.widgets.select.base.clone());
        if let Some(variant) = node.variant.as_ref() {
            if let Some(variant_styles) = theme.widgets.select.variants.get(variant) {
                styles = merge_select_part_styles(styles, variant_styles.clone());
            }
        }
    }
    if let Some(crate::widgets::WidgetSpec::Select(spec)) = node.widget_spec.as_ref() {
        styles = merge_select_part_styles(styles, spec.styles.clone());
    }
    styles
}

fn merge_select_part_styles(
    mut base: crate::widgets::SelectPartStyles,
    next: crate::widgets::SelectPartStyles,
) -> crate::widgets::SelectPartStyles {
    base.trigger = merge_optional_style(base.trigger, next.trigger);
    base.popover = merge_optional_style(base.popover, next.popover);
    base.list = merge_optional_style(base.list, next.list);
    base.item = merge_optional_style(base.item, next.item);
    base.item_hovered = merge_optional_style(base.item_hovered, next.item_hovered);
    base.item_selected = merge_optional_style(base.item_selected, next.item_selected);
    base.item_disabled = merge_optional_style(base.item_disabled, next.item_disabled);
    base
}

fn merge_optional_style(
    base: Option<crate::Style>,
    next: Option<crate::Style>,
) -> Option<crate::Style> {
    match (base, next) {
        (Some(base), Some(next)) => Some(base.merge_over(next)),
        (None, Some(next)) => Some(next),
        (Some(base), None) => Some(base),
        (None, None) => None,
    }
}

fn font_weight_from_style(style: Option<&crate::Style>) -> FontWeight {
    style
        .and_then(|style| style.text.as_ref())
        .map(|text| text.weight)
        .unwrap_or(FontWeight::Normal)
}

// ─── Tab geometry (public) ────────────────────────────────────────────────────

pub fn calculate_tab_rects(
    labels: &[String],
    origin: Point,
    text_system: &mut TextSystem,
) -> Vec<Rect> {
    let metrics = crate::WidgetMetrics::default();
    calculate_tab_rects_with_metrics(labels, origin, text_system, &metrics)
}

pub fn calculate_tab_rects_with_metrics(
    labels: &[String],
    origin: Point,
    text_system: &mut TextSystem,
    metrics: &crate::WidgetMetrics,
) -> Vec<Rect> {
    let mut rects = Vec::new();
    let mut current_x = origin.x + metrics.tabs.horizontal_padding;
    let y_offset = (metrics.tabs.min_size.height - metrics.tabs.tab_height) * 0.5;
    for label in labels {
        let label_width = text_system
            .measure(
                label,
                DEFAULT_TEXT_SIZE,
                crate::core::FontWeight::Normal,
                crate::core::FontStyle::Normal,
                f32::MAX,
            )
            .width;
        let tab_w =
            (label_width + metrics.tabs.horizontal_padding * 2.0).max(metrics.tabs.tab_min_width);
        let tab_rect = Rect::new(
            Point::new(current_x, origin.y + y_offset),
            Size::new(tab_w, metrics.tabs.tab_height),
        );
        rects.push(tab_rect);
        current_x += tab_w + metrics.tabs.tab_gap;
    }
    rects
}

// ─── Widget painters ──────────────────────────────────────────────────────────
//
// Conventions:
//   z + 0  background rect  (template)
//   z + 1  border           (template, when has_border() == true)
//   z + 2+ widget content   (paint_content)
//
// Painters that deviate from the standard background geometry override `paint`.

// ── Button ────────────────────────────────────────────────────────────────────

struct ButtonPainter;
impl WidgetPainter for ButtonPainter {
    /// Active state flips the fill to the foreground accent color.
    fn background_color(&self, ctx: &PaintCtx<'_>) -> Color {
        if ctx.state.active { ctx.style.foreground } else { ctx.style.background }
    }

    /// Centered label text.
    fn paint_content(&self, ctx: &mut PaintCtx<'_>, cmds: &mut Vec<PaintedCommand>) {
        let text_color = if ctx.state.primary || ctx.state.active {
            Color::rgb(255, 255, 255)
        } else {
            ctx.style.text_color
        };
        if let Some(label) = ctx.state.label.as_deref().filter(|v| !v.is_empty()) {
            let label_width = ctx
                .text
                .measure(
                    label,
                    ctx.style.font_size,
                    ctx.style.font_weight,
                    FontStyle::Normal,
                    ctx.rect.size.width,
                )
                .width;
            let text_x = ctx.rect.origin.x + (ctx.rect.size.width - label_width) * 0.5;
            let text_y =
                ctx.rect.origin.y + ctx.rect.size.height * 0.5 + ctx.style.font_size * 0.3;
            cmds.push(text_at_with_size_and_weight(
                label.to_string(),
                Point::new(text_x, text_y),
                text_color,
                ctx.z_index + 2,
                ctx.style.font_size,
                ctx.style.font_weight,
            ));
        }
    }
}

// ── Input ─────────────────────────────────────────────────────────────────────

struct InputPainter;
impl WidgetPainter for InputPainter {
    /// Single-line text with preedit, caret, and placeholder support.
    fn paint_content(&self, ctx: &mut PaintCtx<'_>, cmds: &mut Vec<PaintedCommand>) {
        let text_origin = ctx.metrics.input_text_origin(ctx.rect);
        let text_top_left = ctx.metrics.input_text_top_left(ctx.rect);
        let measure_width = ctx.metrics.input_text_measure_width(ctx.rect);
        paint_text_field(ctx, text_origin, text_top_left, measure_width, cmds);
    }
}

// ── Textarea ──────────────────────────────────────────────────────────────────

struct TextareaPainter;
impl WidgetPainter for TextareaPainter {
    /// Multi-line text with preedit, caret, and placeholder support.
    fn paint_content(&self, ctx: &mut PaintCtx<'_>, cmds: &mut Vec<PaintedCommand>) {
        let text_origin = ctx.metrics.textarea_text_origin(ctx.rect);
        let text_top_left = ctx.metrics.textarea_text_top_left(ctx.rect);
        let measure_width = ctx.metrics.textarea_text_measure_width(ctx.rect);
        paint_text_field(ctx, text_origin, text_top_left, measure_width, cmds);
    }
}

// ── Checkbox ──────────────────────────────────────────────────────────────────

/// Overrides `paint` — the control box is a sub-rect of the widget rect, not the full rect.
struct CheckboxPainter;
impl WidgetPainter for CheckboxPainter {
    fn paint(&self, ctx: &mut PaintCtx<'_>) -> Vec<PaintedCommand> {
        let box_rect =
            Rect::new(ctx.rect.origin, Size::new(ctx.rect.size.height, ctx.rect.size.height));
        let mut cmds = vec![rect_command(
            box_rect,
            ctx.style.background,
            ctx.style.border_radius,
            ctx.z_index,
        )];
        if ctx.state.checked {
            let inset_val = (box_rect.size.width * 0.208).round();
            cmds.push(rect_command(
                inset(box_rect, inset_val, inset_val),
                ctx.style.text_color,
                (box_rect.size.width * 0.083).round(),
                ctx.z_index + 1,
            ));
        }
        cmds.push(border_command(
            box_rect,
            ctx.style.border_color,
            ctx.style.border_width,
            ctx.z_index + 2,
        ));
        if let Some(label) = ctx.state.label.as_ref().filter(|l| !l.is_empty()) {
            cmds.push(text_at_with_size_and_weight(
                label.clone(),
                Point::new(
                    box_rect.max_x() + 6.0,
                    box_rect.origin.y + box_rect.size.height * 0.5 + ctx.style.font_size * 0.3,
                ),
                ctx.style.text_color,
                ctx.z_index + 3,
                ctx.style.font_size,
                ctx.style.font_weight,
            ));
        }
        cmds
    }
}

// ── Radio ─────────────────────────────────────────────────────────────────────

/// Overrides `paint` — the control is a circle sub-rect of the widget rect.
struct RadioPainter;
impl WidgetPainter for RadioPainter {
    fn paint(&self, ctx: &mut PaintCtx<'_>) -> Vec<PaintedCommand> {
        let box_rect =
            Rect::new(ctx.rect.origin, Size::new(ctx.rect.size.height, ctx.rect.size.height));
        let center = Point::new(
            box_rect.origin.x + box_rect.size.width * 0.5,
            box_rect.origin.y + box_rect.size.height * 0.5,
        );
        let inner = box_rect.size.width * 0.125;
        let radius = box_rect.size.height * 0.5;
        let mut cmds = vec![
            rect_command(box_rect, ctx.style.background, radius, ctx.z_index),
            border_command(
                box_rect,
                ctx.style.border_color,
                ctx.style.border_width,
                ctx.z_index + 1,
            ),
        ];
        if ctx.state.checked {
            cmds.push(rect_command(
                Rect::new(
                    Point::new(center.x - inner, center.y - inner),
                    Size::new(inner * 2.0, inner * 2.0),
                ),
                ctx.style.foreground,
                inner,
                ctx.z_index + 2,
            ));
        }
        if let Some(label) = ctx.state.label.as_ref().filter(|l| !l.is_empty()) {
            cmds.push(text_at_with_size_and_weight(
                label.clone(),
                Point::new(
                    box_rect.max_x() + 6.0,
                    box_rect.origin.y + box_rect.size.height * 0.5 + ctx.style.font_size * 0.3,
                ),
                ctx.style.text_color,
                ctx.z_index + 3,
                ctx.style.font_size,
                ctx.style.font_weight,
            ));
        }
        cmds
    }
}

// ── Select ────────────────────────────────────────────────────────────────────

struct SelectPainter;
impl WidgetPainter for SelectPainter {
    /// Selected-value label + dropdown arrow glyph.
    fn paint_content(&self, ctx: &mut PaintCtx<'_>, cmds: &mut Vec<PaintedCommand>) {
        let part_styles = resolved_select_part_styles(ctx.node, Some(ctx.theme));
        let label = ctx.state.label.as_deref().unwrap_or("Select...");
        let selected_weight = font_weight_from_style(part_styles.item_selected.as_ref());
        let text_origin = ctx.metrics.input_text_origin(ctx.rect);
        cmds.push(text_at_weight(
            label.to_string(),
            text_origin,
            ctx.style.text_color,
            ctx.z_index + 2,
            selected_weight,
        ));
        cmds.push(text_at(
            "v".to_string(),
            Point::new(
                ctx.rect.max_x() - ctx.metrics.metrics.select.arrow_slot_width,
                text_origin.y,
            ),
            ctx.style.text_muted_color,
            ctx.z_index + 3,
        ));
    }
}

// ── Tabs ──────────────────────────────────────────────────────────────────────

struct TabsPainter;
impl WidgetPainter for TabsPainter {
    /// Tab strip: inactive tabs use surface_hover, active tab uses background.
    fn paint_content(&self, ctx: &mut PaintCtx<'_>, cmds: &mut Vec<PaintedCommand>) {
        let mut spec_tabs: &Vec<String> = &vec![];
        if let Some(crate::WidgetSpec::Tabs(ref tabs_spec)) = ctx.node.widget_spec {
            spec_tabs = &tabs_spec.tabs;
        }
        let default_tabs = vec!["Tab1".to_string(), "Tab2".to_string(), "Tab3".to_string()];
        let tabs: &Vec<String> = if spec_tabs.is_empty() { &default_tabs } else { spec_tabs };

        let active_index = ctx.state.tabs_active_index.unwrap_or(0);
        let rects = ctx.metrics.tab_rects(tabs, ctx.rect.origin, ctx.text);

        for (i, (tab_label, tab_rect)) in tabs.iter().zip(rects).enumerate() {
            let is_active = i == active_index;
            cmds.push(rect_command(
                tab_rect,
                if is_active {
                    ctx.theme.colors.background
                } else {
                    ctx.theme.colors.surface_hover
                },
                ctx.style.border_radius - 1.0,
                ctx.z_index + 2,
            ));
            let label_width = ctx
                .text
                .measure(
                    tab_label,
                    ctx.metrics.tab_text_size(),
                    crate::core::FontWeight::Normal,
                    crate::core::FontStyle::Normal,
                    f32::MAX,
                )
                .width;
            let text_x = tab_rect.origin.x + (tab_rect.size.width - label_width) / 2.0;
            let text_y =
                tab_rect.origin.y + tab_rect.size.height * 0.5 + ctx.style.font_size * 0.3;
            cmds.push(text_at(
                tab_label.clone(),
                Point::new(text_x, text_y),
                ctx.style.text_color,
                ctx.z_index + 3,
            ));
        }
    }
}

// ── Tree ──────────────────────────────────────────────────────────────────────

struct TreePainter;
impl WidgetPainter for TreePainter {
    /// Tree uses the theme background, not the widget-style background.
    fn background_color(&self, ctx: &PaintCtx<'_>) -> Color {
        ctx.theme.colors.background
    }

    /// Recursive item rows with expand/collapse markers.
    fn paint_content(&self, ctx: &mut PaintCtx<'_>, cmds: &mut Vec<PaintedCommand>) {
        let mut spec_items: &Vec<crate::TreeItemSpec> = &vec![];
        if let Some(crate::WidgetSpec::Tree(ref tree_spec)) = ctx.node.widget_spec {
            spec_items = &tree_spec.items;
        }
        let default_items = vec![crate::TreeItemSpec {
            label: "Root".to_string(),
            expanded: true,
            children: vec![
                crate::TreeItemSpec {
                    label: "Branch A".to_string(),
                    expanded: true,
                    children: vec![crate::TreeItemSpec {
                        label: "Leaf 1".to_string(),
                        expanded: false,
                        children: vec![],
                    }],
                },
                crate::TreeItemSpec {
                    label: "Branch B".to_string(),
                    expanded: false,
                    children: vec![],
                },
            ],
        }];
        let items: &Vec<crate::TreeItemSpec> =
            if spec_items.is_empty() { &default_items } else { spec_items };

        let mut index = 0;
        paint_tree_items(
            items,
            0,
            &mut index,
            ctx.rect,
            ctx.z_index,
            cmds,
            &ctx.state.tree_expanded,
            ctx.metrics.metrics.tree,
            ctx.style.text_color,
        );
    }
}

// ── Table ─────────────────────────────────────────────────────────────────────

struct TablePainter;
impl WidgetPainter for TablePainter {
    /// Table uses the theme background, not the widget-style background.
    fn background_color(&self, ctx: &PaintCtx<'_>) -> Color {
        ctx.theme.colors.background
    }

    /// Grid of rows and columns with header highlight and row selection.
    fn paint_content(&self, ctx: &mut PaintCtx<'_>, cmds: &mut Vec<PaintedCommand>) {
        let mut spec_cols: &Vec<String> = &vec![];
        let mut spec_rows: &Vec<Vec<String>> = &vec![];
        if let Some(crate::WidgetSpec::Table(ref table_spec)) = ctx.node.widget_spec {
            spec_cols = &table_spec.columns;
            spec_rows = &table_spec.rows;
        }

        let cols_count = spec_cols.len().max(1);
        let rows_count = (spec_rows.len() + 1).max(1);
        let col_width = ctx.rect.size.width / cols_count as f32;
        let row_height = ctx.metrics.metrics.table.row_height;
        let selection_bg = selection_color(ctx.theme);

        for r in 0..rows_count {
            let y = ctx.rect.origin.y
                + ctx.metrics.metrics.table.cell_padding
                + r as f32 * row_height;
            if r == 0 {
                cmds.push(rect_command(
                    Rect::new(
                        Point::new(ctx.rect.origin.x, y),
                        Size::new(ctx.rect.size.width, row_height),
                    ),
                    ctx.theme.colors.surface_hover,
                    0.0,
                    ctx.z_index + 2,
                ));
            } else if ctx.state.table_selected_row == Some(r - 1) {
                cmds.push(rect_command(
                    Rect::new(
                        Point::new(ctx.rect.origin.x, y),
                        Size::new(ctx.rect.size.width, row_height),
                    ),
                    selection_bg,
                    0.0,
                    ctx.z_index + 2,
                ));
            }

            for c in 0..cols_count {
                let x = ctx.rect.origin.x + c as f32 * col_width;
                let text_val = if r == 0 {
                    spec_cols
                        .get(c)
                        .cloned()
                        .unwrap_or_else(|| format!("Col{}", c + 1))
                } else {
                    spec_rows
                        .get(r - 1)
                        .and_then(|row| row.get(c).cloned())
                        .unwrap_or_else(|| format!("R{}C{}", r, c + 1))
                };
                let text_y = y + row_height * 0.5 + ctx.style.font_size * 0.3;
                cmds.push(text_at(
                    text_val,
                    Point::new(x + ctx.metrics.metrics.table.cell_padding, text_y),
                    ctx.style.text_color,
                    ctx.z_index + 3,
                ));
                if c > 0 {
                    cmds.push(rect_command(
                        Rect::new(Point::new(x, y), Size::new(1.0, row_height)),
                        ctx.theme.colors.border,
                        0.0,
                        ctx.z_index + 2,
                    ));
                }
            }

            if r > 0 {
                cmds.push(rect_command(
                    Rect::new(
                        Point::new(ctx.rect.origin.x, y - 1.0),
                        Size::new(ctx.rect.size.width, 1.0),
                    ),
                    ctx.theme.colors.border,
                    0.0,
                    ctx.z_index + 2,
                ));
            }
        }
    }
}

// ── List ──────────────────────────────────────────────────────────────────────

struct ListPainter;
impl WidgetPainter for ListPainter {
    /// List uses the theme background, not the widget-style background.
    fn background_color(&self, ctx: &PaintCtx<'_>) -> Color {
        ctx.theme.colors.background
    }

    /// Item rows with selection highlight and alternating text colors.
    fn paint_content(&self, ctx: &mut PaintCtx<'_>, cmds: &mut Vec<PaintedCommand>) {
        let mut spec_items: &Vec<String> = &vec![];
        if let Some(crate::WidgetSpec::List(ref list_spec)) = ctx.node.widget_spec {
            spec_items = &list_spec.items;
        }
        let default_items = vec![
            "Item One".to_string(),
            "Item Two".to_string(),
            "Item Three".to_string(),
            "Item Four".to_string(),
        ];
        let items: &Vec<String> =
            if spec_items.is_empty() { &default_items } else { spec_items };
        let selection_bg = selection_color(ctx.theme);

        for (i, label) in items.iter().enumerate() {
            let y = ctx.rect.origin.y
                + ctx.metrics.metrics.list.item_padding
                + i as f32 * ctx.metrics.metrics.list.row_height;
            if ctx.state.list_selected_index == Some(i) {
                cmds.push(rect_command(
                    Rect::new(
                        Point::new(
                            ctx.rect.origin.x + ctx.metrics.metrics.list.item_padding,
                            y - 2.0,
                        ),
                        Size::new(
                            (ctx.rect.size.width - ctx.metrics.metrics.list.item_padding * 2.0)
                                .max(0.0),
                            (ctx.metrics.metrics.list.row_height - 4.0).max(1.0),
                        ),
                    ),
                    selection_bg,
                    (ctx.style.border_radius - 1.0).max(0.0),
                    ctx.z_index + 2,
                ));
            }
            let text_y =
                y + ctx.metrics.metrics.list.row_height * 0.5 + ctx.style.font_size * 0.3;
            cmds.push(text_at(
                format!("  {}", label),
                Point::new(ctx.rect.origin.x + ctx.metrics.metrics.list.item_padding, text_y),
                if i % 2 == 0 { ctx.style.text_color } else { ctx.style.text_muted_color },
                ctx.z_index + 3,
            ));
        }
    }
}

// ── ScrollArea ────────────────────────────────────────────────────────────────

struct ScrollAreaPainter;
impl WidgetPainter for ScrollAreaPainter {
    /// Scrollbar track and thumb, only when content overflows.
    fn paint_content(&self, ctx: &mut PaintCtx<'_>, cmds: &mut Vec<PaintedCommand>) {
        if ctx.state.content_size.height > ctx.rect.size.height {
            let track = ctx.metrics.metrics.scrollbar.track_rect(ctx.rect);
            let thumb = ctx.metrics.metrics.scrollbar.thumb_rect(
                ctx.rect,
                ctx.state.content_size,
                ctx.state.scroll_offset,
            );
            cmds.push(rect_command(track, ctx.style.border_color, 2.0, ctx.z_index + 2));
            cmds.push(rect_command(thumb, ctx.style.text_muted_color, 2.0, ctx.z_index + 3));
        }
    }
}

/// Shared implementation used by the `ElementKind::Primitive(ScrollArea)` branch
/// in `paint_node_themed` (which is not a widget-kind dispatch).
fn scroll_area_commands(
    rect: Rect,
    z_index: i32,
    state: &VisualState,
    style: &ResolvedWidgetStyle,
    metrics: WidgetPaintMetrics<'_>,
) -> Vec<PaintedCommand> {
    let mut cmds = vec![
        rect_command(rect, style.background, style.border_radius, z_index),
        border_command(rect, style.border_color, style.border_width, z_index + 1),
    ];
    if state.content_size.height > rect.size.height {
        let track = metrics.metrics.scrollbar.track_rect(rect);
        let thumb =
            metrics
                .metrics
                .scrollbar
                .thumb_rect(rect, state.content_size, state.scroll_offset);
        cmds.push(rect_command(track, style.border_color, 2.0, z_index + 2));
        cmds.push(rect_command(thumb, style.text_muted_color, 2.0, z_index + 3));
    }
    cmds
}

// ── Menu ──────────────────────────────────────────────────────────────────────

struct MenuPainter;
impl WidgetPainter for MenuPainter {
    /// Menu uses the theme background, not the widget-style background.
    fn background_color(&self, ctx: &PaintCtx<'_>) -> Color {
        ctx.theme.colors.background
    }

    /// Menu items with optional keyboard shortcut hints.
    fn paint_content(&self, ctx: &mut PaintCtx<'_>, cmds: &mut Vec<PaintedCommand>) {
        let mut spec_items: &Vec<crate::MenuItemSpec> = &vec![];
        if let Some(crate::WidgetSpec::Menu(ref menu_spec)) = ctx.node.widget_spec {
            spec_items = &menu_spec.items;
        }
        let default_items = vec![
            crate::MenuItemSpec {
                label: "Open".to_string(),
                action: None,
                disabled: false,
                shortcut: None,
            },
            crate::MenuItemSpec {
                label: "Save".to_string(),
                action: None,
                disabled: false,
                shortcut: None,
            },
            crate::MenuItemSpec {
                label: "Exit".to_string(),
                action: None,
                disabled: false,
                shortcut: None,
            },
        ];
        let items: &Vec<crate::MenuItemSpec> =
            if spec_items.is_empty() { &default_items } else { spec_items };

        for (i, item) in items.iter().enumerate() {
            let y = ctx.rect.origin.y
                + ctx.metrics.metrics.menu.item_padding
                + i as f32 * ctx.metrics.metrics.menu.item_height;
            let text_color =
                if item.disabled { ctx.style.text_muted_color } else { ctx.style.text_color };
            let text_y =
                y + ctx.metrics.metrics.menu.item_height * 0.5 + ctx.style.font_size * 0.3;
            cmds.push(text_at(
                item.label.clone(),
                Point::new(ctx.rect.origin.x + ctx.metrics.metrics.menu.item_padding, text_y),
                text_color,
                ctx.z_index + 2,
            ));
            if let Some(ref shortcut) = item.shortcut {
                cmds.push(text_at(
                    shortcut.clone(),
                    Point::new(
                        ctx.rect.max_x() - ctx.metrics.metrics.menu.item_padding * 6.0,
                        text_y,
                    ),
                    ctx.style.text_muted_color,
                    ctx.z_index + 2,
                ));
            }
        }
    }
}

// ── Tooltip ───────────────────────────────────────────────────────────────────

struct TooltipPainter;
impl WidgetPainter for TooltipPainter {
    /// Fixed dark background regardless of theme mode.
    fn background_color(&self, _ctx: &PaintCtx<'_>) -> Color {
        Color::rgb(30, 41, 59)
    }

    /// Tooltip has no surrounding border — its shape is self-contained.
    fn has_border(&self) -> bool {
        false
    }

    /// Label text centered vertically.
    fn paint_content(&self, ctx: &mut PaintCtx<'_>, cmds: &mut Vec<PaintedCommand>) {
        let label = ctx.state.label.as_deref().unwrap_or("Tooltip");
        let tooltip_fg = if ctx.theme.mode == ThemeMode::Dark {
            ctx.theme.colors.text
        } else {
            Color::rgb(241, 245, 249)
        };
        let padding = ctx.theme.spacing.sm;
        let text_y = ctx.rect.origin.y + ctx.rect.size.height * 0.5 + ctx.style.font_size * 0.3;
        cmds.push(text_at(
            label.to_string(),
            Point::new(ctx.rect.origin.x + padding, text_y),
            tooltip_fg,
            ctx.z_index + 2,
        ));
    }
}

// ── Icon ──────────────────────────────────────────────────────────────────────

fn icon_symbol(name: &str) -> String {
    name.chars()
        .find(|ch| ch.is_ascii_alphanumeric())
        .map(|ch| ch.to_ascii_uppercase().to_string())
        .unwrap_or_else(|| "?".to_string())
}

struct IconPainter;
impl WidgetPainter for IconPainter {
    /// Centered glyph derived from the icon name.
    fn paint_content(&self, ctx: &mut PaintCtx<'_>, cmds: &mut Vec<PaintedCommand>) {
        if let Some(name) = ctx.state.label.as_deref().filter(|v| !v.is_empty()) {
            let symbol = icon_symbol(name);
            let text_x = ctx.rect.origin.x + (ctx.rect.size.width - 12.0) * 0.5;
            let text_y =
                ctx.rect.origin.y + ctx.rect.size.height * 0.5 + ctx.style.font_size * 0.3;
            cmds.push(text_at(
                symbol,
                Point::new(text_x, text_y),
                ctx.style.text_color,
                ctx.z_index + 2,
            ));
        }
    }
}

// ── Divider ───────────────────────────────────────────────────────────────────

/// Overrides `paint` — a divider is a single horizontal line with no fill or border.
struct DividerPainter;
impl WidgetPainter for DividerPainter {
    fn paint(&self, ctx: &mut PaintCtx<'_>) -> Vec<PaintedCommand> {
        let thickness = ctx.metrics.divider_thickness();
        let y = ctx.rect.origin.y + (ctx.rect.size.height - thickness) / 2.0;
        vec![rect_command(
            Rect::new(
                Point::new(ctx.rect.origin.x, y),
                Size::new(ctx.rect.size.width, thickness),
            ),
            ctx.style.border_color,
            0.0,
            ctx.z_index,
        )]
    }
}

// ── Canvas ────────────────────────────────────────────────────────────────────

struct CanvasPainter;
impl WidgetPainter for CanvasPainter {
    /// Canvas uses axis lines as its "border" — no surrounding box border.
    fn has_border(&self) -> bool {
        false
    }

    /// L-shaped axis lines and optional name label.
    fn paint_content(&self, ctx: &mut PaintCtx<'_>, cmds: &mut Vec<PaintedCommand>) {
        let canvas = ctx.metrics.metrics.canvas;
        cmds.push(rect_command(
            Rect::new(
                Point::new(
                    ctx.rect.origin.x + canvas.padding,
                    ctx.rect.origin.y + ctx.rect.size.height - canvas.padding * 2.0,
                ),
                Size::new((ctx.rect.size.width - canvas.padding * 2.0).max(1.0), 1.0),
            ),
            ctx.style.border_color,
            0.0,
            ctx.z_index + 2,
        ));
        cmds.push(rect_command(
            Rect::new(
                Point::new(
                    ctx.rect.origin.x + canvas.padding,
                    ctx.rect.origin.y + canvas.padding,
                ),
                Size::new(1.0, (ctx.rect.size.height - canvas.padding * 3.0).max(1.0)),
            ),
            ctx.style.border_color,
            0.0,
            ctx.z_index + 2,
        ));
        if let Some(label) = ctx.state.label.as_deref().filter(|v| !v.is_empty()) {
            cmds.push(text_at(
                label.to_string(),
                Point::new(
                    ctx.rect.origin.x + canvas.padding * 1.5,
                    ctx.rect.origin.y + canvas.label_baseline_offset,
                ),
                ctx.style.text_color,
                ctx.z_index + 3,
            ));
        }
    }
}

/// Shared implementation used by the `ElementKind::Canvas` branch in `paint_node_themed`.
fn canvas_commands(
    rect: Rect,
    label: Option<&str>,
    z_index: i32,
    style: &ResolvedWidgetStyle,
    metrics: WidgetPaintMetrics<'_>,
) -> Vec<PaintedCommand> {
    let canvas = metrics.metrics.canvas;
    let mut cmds = vec![
        rect_command(rect, style.background, style.border_radius, z_index),
        rect_command(
            Rect::new(
                Point::new(
                    rect.origin.x + canvas.padding,
                    rect.origin.y + rect.size.height - canvas.padding * 2.0,
                ),
                Size::new((rect.size.width - canvas.padding * 2.0).max(1.0), 1.0),
            ),
            style.border_color,
            0.0,
            z_index + 1,
        ),
        rect_command(
            Rect::new(
                Point::new(rect.origin.x + canvas.padding, rect.origin.y + canvas.padding),
                Size::new(1.0, (rect.size.height - canvas.padding * 3.0).max(1.0)),
            ),
            style.border_color,
            0.0,
            z_index + 1,
        ),
    ];
    if let Some(label) = label.filter(|value| !value.is_empty()) {
        cmds.push(text_at(
            label.to_string(),
            Point::new(
                rect.origin.x + canvas.padding * 1.5,
                rect.origin.y + canvas.label_baseline_offset,
            ),
            style.text_color,
            z_index + 2,
        ));
    }
    cmds
}

// ── Invisible ─────────────────────────────────────────────────────────────────

/// Modal and Popover are rendered by the overlay/portal pass — nothing here.
struct InvisiblePainter;
impl WidgetPainter for InvisiblePainter {
    fn paint(&self, _ctx: &mut PaintCtx<'_>) -> Vec<PaintedCommand> {
        Vec::new()
    }
}

// ── Text background ───────────────────────────────────────────────────────────

/// Renders only the theme background behind an inline text widget — no border, no content.
struct TextBackgroundPainter;
impl WidgetPainter for TextBackgroundPainter {
    fn background_color(&self, ctx: &PaintCtx<'_>) -> Color {
        ctx.theme.colors.background
    }

    fn has_border(&self) -> bool {
        false
    }
}

// ── Generic fallback ──────────────────────────────────────────────────────────

/// Background-only fallback for new or unimplemented widget kinds.
struct GenericPainter;
impl WidgetPainter for GenericPainter {
    fn has_border(&self) -> bool {
        false
    }
}

// ─── Low-level command builders ───────────────────────────────────────────────

fn rect_command(rect: Rect, color: Color, radius: f32, z_index: i32) -> PaintedCommand {
    PaintedCommand {
        command: PaintCommand::DrawRect(RectCmd {
            rect,
            paint: Paint::Solid(color),
            radius,
            opacity: 1.0,
            z_index,
        }),
        snapshot: PaintCommandSnapshot {
            kind: "DrawRect".to_string(),
            z_index,
        },
    }
}

fn text_at(text: String, origin: Point, color: Color, z_index: i32) -> PaintedCommand {
    text_at_weight(text, origin, color, z_index, FontWeight::Normal)
}

fn text_at_weight(
    text: String,
    origin: Point,
    color: Color,
    z_index: i32,
    weight: FontWeight,
) -> PaintedCommand {
    text_at_with_size_and_weight(text, origin, color, z_index, 14.0, weight)
}

fn text_at_with_size_and_weight(
    text: String,
    origin: Point,
    color: Color,
    z_index: i32,
    size: f32,
    weight: FontWeight,
) -> PaintedCommand {
    let line_height = (size * 1.2).ceil();
    PaintedCommand {
        command: PaintCommand::DrawText(TextCmd {
            text,
            rect: Rect::new(
                Point::new(origin.x, origin.y - (size * 0.8).ceil()),
                Size::new(0.0, line_height),
            ),
            color,
            size,
            font_weight: weight,
            font_style: FontStyle::Normal,
            line_height: Some(line_height),
            z_index,
        }),
        snapshot: PaintCommandSnapshot {
            kind: "DrawText".to_string(),
            z_index,
        },
    }
}

fn border_command(rect: Rect, color: Color, width: f32, z_index: i32) -> PaintedCommand {
    PaintedCommand {
        command: PaintCommand::DrawBorder(BorderCmd {
            rect,
            color,
            width,
            radius: 0.0,
            z_index,
        }),
        snapshot: PaintCommandSnapshot {
            kind: "DrawBorder".to_string(),
            z_index,
        },
    }
}

fn text_command(
    text_system: &mut TextSystem,
    text: String,
    rect: Rect,
    style: RuntimeTextStyle,
    z_index: i32,
) -> PaintedCommand {
    let layout = text_system.measure(
        &text,
        style.font_size,
        style.weight,
        style.style,
        rect.size.width.max(style.font_size),
    );
    let origin = Point::new(
        rect.origin.x,
        rect.origin.y + (rect.size.height - layout.height).max(0.0) * 0.5 + layout.baseline,
    );
    PaintedCommand {
        command: PaintCommand::DrawText(TextCmd {
            text,
            rect: layout.rect_for_baseline_origin(origin),
            color: style.color,
            size: style.font_size,
            font_weight: style.weight,
            font_style: style.style,
            line_height: Some(layout.line_height),
            z_index,
        }),
        snapshot: PaintCommandSnapshot {
            kind: "DrawText".to_string(),
            z_index,
        },
    }
}

fn inset(rect: Rect, x: f32, y: f32) -> Rect {
    Rect::new(
        Point::new(rect.origin.x + x, rect.origin.y + y),
        Size::new(
            (rect.size.width - x * 2.0).max(0.0),
            (rect.size.height - y * 2.0).max(0.0),
        ),
    )
}

#[derive(Clone, Copy, Debug)]
struct RuntimeTextStyle {
    font_size: f32,
    weight: FontWeight,
    style: FontStyle,
    color: Color,
}

fn text_style_for_node(node: &UiNode, theme: Option<&Theme>) -> RuntimeTextStyle {
    let default_theme;
    let theme = match theme {
        Some(theme) => theme,
        None => {
            default_theme = Theme::light();
            &default_theme
        }
    };
    let text_style = node.style.text.as_ref();
    RuntimeTextStyle {
        font_size: text_style
            .and_then(|style| style.size.resolve(14.0))
            .filter(|size| *size > 0.0)
            .unwrap_or(DEFAULT_TEXT_SIZE),
        weight: text_style
            .map(|style| style.weight)
            .unwrap_or(FontWeight::Normal),
        style: text_style
            .map(|style| style.style)
            .unwrap_or(FontStyle::Normal),
        color: text_style
            .map(|style| style.color)
            .unwrap_or(theme.colors.text),
    }
}
