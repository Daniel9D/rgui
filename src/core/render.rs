use crate::{FontStyle, FontWeight, Point, Rect, SizeU32};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ImageId(u64);

impl ImageId {
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SvgId(u64);

impl SvgId {
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct GlyphKey {
    pub font_id: u64,
    pub glyph_id: u32,
    pub size_bits: u32,
}

/// An sRGB color. Channels are stored as 8-bit values.
///
/// The "no color" / "resolve at paint time from the active theme" sentinel
/// is [`Color::DEFAULT`]. Use [`Color::is_default`] to detect it. The render
/// path is responsible for replacing `DEFAULT` with the appropriate token
/// (e.g. `theme.colors.text`) before lowering to the GPU.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    /// Sentinel meaning "use the active theme to resolve this color at paint
    /// time". Recognized by [`Color::is_default`]. Storing `DEFAULT` directly
    /// in a `Color` field is preferred over a `Option<Color>` so the field
    /// shape stays uniform.
    pub const DEFAULT: Color = Color::rgba(0, 0, 0, 0);

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// `true` if this is the [`Color::DEFAULT`] sentinel.
    pub const fn is_default(self) -> bool {
        self.a == 0
    }

    /// `true` if the alpha channel is fully opaque.
    pub const fn is_opaque(self) -> bool {
        self.a == 255
    }

    /// Returns a new color with the same RGB but a different alpha.
    pub const fn with_alpha(self, a: u8) -> Self {
        Self { r: self.r, g: self.g, b: self.b, a }
    }
}

impl Default for Color {
    fn default() -> Self {
        Color::DEFAULT
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Paint {
    Solid(Color),
    LinearGradient {
        start: Point,
        end: Point,
        stops: Vec<(f32, Color)>,
    },
    Image(ImageId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LayerKind {
    Document,
    Floating,
    Popover,
    Tooltip,
    ContextMenu,
    Modal,
    Debug,
}

impl LayerKind {
    /// Stable ordering used by both the GPU render path (`item::sort_by_key`)
    /// and the event dispatch hit-test path. Lower values are drawn first /
    /// hit-tested first.
    pub const fn order(self) -> i32 {
        match self {
            LayerKind::Document => 0,
            LayerKind::Floating => 1,
            LayerKind::Popover => 2,
            LayerKind::Tooltip => 3,
            LayerKind::ContextMenu => 4,
            LayerKind::Modal => 5,
            LayerKind::Debug => 6,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LayerSpec {
    pub kind: LayerKind,
    pub opacity: f32,
    pub z_index: i32,
}

impl LayerSpec {
    pub const fn new(kind: LayerKind) -> Self {
        Self {
            kind,
            opacity: 1.0,
            z_index: 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClipSpec {
    pub rect: Rect,
    pub radius: f32,
}

impl ClipSpec {
    pub const fn rect(rect: Rect) -> Self {
        Self { rect, radius: 0.0 }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RectCmd {
    pub rect: Rect,
    pub paint: Paint,
    pub radius: f32,
    pub opacity: f32,
    pub z_index: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BorderCmd {
    pub rect: Rect,
    pub color: Color,
    pub width: f32,
    pub radius: f32,
    pub z_index: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextCmd {
    pub text: String,
    pub rect: Rect,
    pub color: Color,
    pub size: f32,
    pub font_weight: FontWeight,
    pub font_style: FontStyle,
    pub line_height: Option<f32>,
    pub z_index: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ImageCmd {
    pub id: ImageId,
    pub rect: Rect,
    pub opacity: f32,
    pub z_index: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SvgCmd {
    pub id: SvgId,
    pub rect: Rect,
    pub opacity: f32,
    pub z_index: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PathCmd {
    pub points: Vec<Point>,
    pub color: Color,
    pub width: f32,
    pub z_index: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ShadowCmd {
    pub rect: Rect,
    pub color: Color,
    pub blur_radius: f32,
    pub offset: Point,
    pub z_index: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PaintCommand {
    PushLayer(LayerSpec),
    PopLayer,
    PushClip(ClipSpec),
    PopClip,
    DrawRect(RectCmd),
    DrawBorder(BorderCmd),
    DrawText(TextCmd),
    DrawImage(ImageCmd),
    DrawSvg(SvgCmd),
    DrawPath(PathCmd),
    DrawShadow(ShadowCmd),
}

impl PaintCommand {
    /// Z-index used for paint ordering. Stack-management commands
    /// (`PushLayer` / `PopLayer` / `PushClip` / `PopClip`) return `i32::MIN`
    /// so they sort before any draw command. Bug fix 2.13: previously
    /// `PushLayer` returned `spec.z_index` and the rest returned `0`, which
    /// silently corrupted the sort order when stack commands had non-zero
    /// z-indices.
    pub fn z_index(&self) -> i32 {
        match self {
            PaintCommand::DrawRect(cmd) => cmd.z_index,
            PaintCommand::DrawBorder(cmd) => cmd.z_index,
            PaintCommand::DrawText(cmd) => cmd.z_index,
            PaintCommand::DrawImage(cmd) => cmd.z_index,
            PaintCommand::DrawSvg(cmd) => cmd.z_index,
            PaintCommand::DrawPath(cmd) => cmd.z_index,
            PaintCommand::DrawShadow(cmd) => cmd.z_index,
            // Stack-management commands sort first. `i32::MIN` is well
            // below the documented `OVERLAY_PANEL_Z_BASE = 1000`, so
            // this is safe for the existing z-base constants.
            PaintCommand::PushLayer(_)
            | PaintCommand::PopLayer
            | PaintCommand::PushClip(_)
            | PaintCommand::PopClip => i32::MIN,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DisplayList {
    commands: Vec<PaintCommand>,
}

impl DisplayList {
    pub fn push(&mut self, command: PaintCommand) {
        self.commands.push(command);
    }

    pub fn commands(&self) -> &[PaintCommand] {
        &self.commands
    }

    pub fn validate(&self) -> Result<(), String> {
        let mut layers = 0usize;
        let mut clips = 0usize;
        for command in &self.commands {
            match command {
                PaintCommand::PushLayer(spec) => {
                    layers += 1;
                    validate_non_negative(spec.opacity, "layer opacity")?;
                }
                PaintCommand::PopLayer => {
                    layers = layers.checked_sub(1).ok_or("layer stack underflow")?;
                }
                PaintCommand::PushClip(spec) => {
                    clips += 1;
                    validate_rect(spec.rect)?;
                }
                PaintCommand::PopClip => {
                    clips = clips.checked_sub(1).ok_or("clip stack underflow")?;
                }
                PaintCommand::DrawRect(cmd) => {
                    validate_rect(cmd.rect)?;
                    validate_non_negative(cmd.radius, "rect radius")?;
                    validate_non_negative(cmd.opacity, "rect opacity")?;
                }
                PaintCommand::DrawBorder(cmd) => {
                    validate_rect(cmd.rect)?;
                    validate_non_negative(cmd.width, "border width")?;
                    validate_non_negative(cmd.radius, "border radius")?;
                }
                PaintCommand::DrawText(cmd) => {
                    validate_point(cmd.rect.origin).map_err(|err| format!("text {err}"))?;
                    validate_positive(cmd.size, "text size")?;
                }
                PaintCommand::DrawImage(cmd) => {
                    validate_rect(cmd.rect)?;
                    validate_non_negative(cmd.opacity, "image opacity")?;
                }
                PaintCommand::DrawSvg(cmd) => {
                    validate_rect(cmd.rect)?;
                    validate_non_negative(cmd.opacity, "svg opacity")?;
                }
                PaintCommand::DrawPath(cmd) => {
                    if cmd.points.len() < 2 {
                        return Err("path must contain at least two points".to_string());
                    }
                    for point in &cmd.points {
                        validate_point(*point).map_err(|err| format!("path {err}"))?;
                    }
                    validate_non_negative(cmd.width, "path width")?;
                }
                PaintCommand::DrawShadow(cmd) => {
                    validate_rect(cmd.rect)?;
                    validate_non_negative(cmd.blur_radius, "shadow blur radius")?;
                    validate_point(cmd.offset).map_err(|err| format!("shadow offset {err}"))?;
                }
            }
        }
        if layers != 0 {
            return Err(format!("layer stack has {layers} unclosed entries"));
        }
        if clips != 0 {
            return Err(format!("clip stack has {clips} unclosed entries"));
        }
        Ok(())
    }
}

fn validate_point(point: Point) -> Result<(), String> {
    if !point.x.is_finite() || !point.y.is_finite() {
        return Err("point coordinates must be finite".to_string());
    }
    Ok(())
}

fn validate_non_negative(value: f32, name: &str) -> Result<(), String> {
    if !value.is_finite() || value < 0.0 {
        return Err(format!("{name} must be finite and non-negative"));
    }
    Ok(())
}

fn validate_positive(value: f32, name: &str) -> Result<(), String> {
    if !value.is_finite() || value <= 0.0 {
        return Err(format!("{name} must be finite and positive"));
    }
    Ok(())
}

fn validate_rect(rect: Rect) -> Result<(), String> {
    if !rect.origin.x.is_finite() || !rect.origin.y.is_finite() {
        return Err("rect origin must be finite".to_string());
    }
    if !rect.size.width.is_finite() || rect.size.width < 0.0 {
        return Err("rect width must be finite and non-negative".to_string());
    }
    if !rect.size.height.is_finite() || rect.size.height < 0.0 {
        return Err("rect height must be finite and non-negative".to_string());
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum AtlasEntryKind {
    Glyph(GlyphKey),
    Image(ImageId),
    Svg(SvgId),
}

#[derive(Clone, Debug, PartialEq)]
pub struct AtlasEntry {
    pub uv: Rect,
    pub size: SizeU32,
    pub generation: u64,
    pub kind: AtlasEntryKind,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ResourceStore {
    pub atlas_entries: Vec<AtlasEntry>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderStats {
    pub command_count: usize,
    pub batch_count: usize,
    pub atlas_upload_bytes: usize,
    pub render_item_count: usize,
    pub text_item_count: usize,
    pub clip_batch_count: usize,
    pub glyphon_enabled: bool,
    pub text_area_count: usize,
    pub clipped_text_area_count: usize,
    pub skipped_text_area_count: usize,
    pub glyph_count: usize,
    pub fallback_used: bool,
}

pub trait RendererBackend {
    fn resize(&mut self, size: SizeU32);
    fn render(&mut self, display_list: &DisplayList, resources: &ResourceStore) -> RenderStats;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_default_sentinel_is_recognized() {
        assert!(Color::DEFAULT.is_default());
        assert!(!Color::rgb(255, 0, 0).is_default());
        assert!(Color::rgb(0, 0, 0).is_opaque());
        assert!(!Color::DEFAULT.is_opaque());
    }

    #[test]
    fn color_with_alpha_returns_new_color() {
        let c = Color::rgb(10, 20, 30).with_alpha(128);
        assert_eq!(c.r, 10);
        assert_eq!(c.g, 20);
        assert_eq!(c.b, 30);
        assert_eq!(c.a, 128);
    }

    #[test]
    fn paint_command_z_index_for_stack_ops_is_min() {
        // Bug fix 2.13: PushLayer / PopLayer / PushClip / PopClip must sort
        // before any draw command.
        let list = DisplayList::default();
        let push_layer = PaintCommand::PushLayer(LayerSpec::new(LayerKind::Document));
        let pop_layer = PaintCommand::PopLayer;
        let push_clip = PaintCommand::PushClip(ClipSpec::rect(Rect::new(
            Point::new(0.0, 0.0),
            crate::core::Size::new(10.0, 10.0),
        )));
        let pop_clip = PaintCommand::PopClip;
        assert_eq!(push_layer.z_index(), i32::MIN);
        assert_eq!(pop_layer.z_index(), i32::MIN);
        assert_eq!(push_clip.z_index(), i32::MIN);
        assert_eq!(pop_clip.z_index(), i32::MIN);
        // Draw command with z=0 sorts after the stack ops.
        let draw_rect = PaintCommand::DrawRect(RectCmd {
            rect: Rect::new(
                Point::new(0.0, 0.0),
                crate::core::Size::new(1.0, 1.0),
            ),
            paint: Paint::Solid(Color::rgb(0, 0, 0)),
            radius: 0.0,
            opacity: 1.0,
            z_index: 0,
        });
        assert!(draw_rect.z_index() > push_layer.z_index());
        // Suppress unused warning for the empty list.
        let _ = list;
    }

    #[test]
    fn layer_kind_order_is_a_total_order() {
        use std::collections::HashSet;
        let mut orders = HashSet::new();
        for kind in [
            LayerKind::Document,
            LayerKind::Floating,
            LayerKind::Popover,
            LayerKind::Tooltip,
            LayerKind::ContextMenu,
            LayerKind::Modal,
            LayerKind::Debug,
        ] {
            assert!(orders.insert(kind.order()), "duplicate order for {kind:?}");
        }
    }
}
