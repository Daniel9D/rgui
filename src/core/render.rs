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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
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
    pub fn z_index(&self) -> i32 {
        match self {
            PaintCommand::DrawRect(cmd) => cmd.z_index,
            PaintCommand::DrawBorder(cmd) => cmd.z_index,
            PaintCommand::DrawText(cmd) => cmd.z_index,
            PaintCommand::DrawImage(cmd) => cmd.z_index,
            PaintCommand::DrawSvg(cmd) => cmd.z_index,
            PaintCommand::DrawPath(cmd) => cmd.z_index,
            PaintCommand::DrawShadow(cmd) => cmd.z_index,
            PaintCommand::PushLayer(spec) => spec.z_index,
            _ => 0,
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
