use crate::{
    Align, Color, Display, Edge, GridPlacement, GridTrack, Length, Overflow, Paint, Position,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DefaultStyleMode {
    Full,
    Minimal,
    Reset,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StateFlags(u16);

impl StateFlags {
    pub const EMPTY: Self = Self(0);
    pub const HOVER: Self = Self(1 << 0);
    pub const FOCUS: Self = Self(1 << 1);
    pub const ACTIVE: Self = Self(1 << 2);
    pub const DISABLED: Self = Self(1 << 3);
    pub const CHECKED: Self = Self(1 << 4);
    pub const OPEN: Self = Self(1 << 5);
    pub const SELECTED: Self = Self(1 << 6);
    pub const INVALID: Self = Self(1 << 7);

    pub const fn contains(self, flag: Self) -> bool {
        self.0 & flag.0 == flag.0
    }
}

impl std::ops::BitOr for StateFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FontWeight {
    Thin,
    ExtraLight,
    Light,
    Normal,
    Medium,
    Semibold,
    Bold,
    ExtraBold,
    Black,
    Number(u16),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FontStyle {
    Normal,
    Italic,
    Oblique,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FontStretch {
    Condensed,
    Normal,
    Expanded,
    Percent(f32),
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextStyle {
    pub family: Vec<String>,
    pub size: Length,
    pub weight: FontWeight,
    pub style: FontStyle,
    pub stretch: FontStretch,
    pub color: Color,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            family: vec!["system-ui".to_string()],
            size: Length::Px(14.0),
            weight: FontWeight::Normal,
            style: FontStyle::Normal,
            stretch: FontStretch::Normal,
            color: Color::rgb(0, 0, 0),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Background {
    Paint(Paint),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Border {
    pub color: Color,
    pub width: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Radius {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_right: f32,
    pub bottom_left: f32,
}

impl Radius {
    pub const fn all(value: f32) -> Self {
        Self {
            top_left: value,
            top_right: value,
            bottom_right: value,
            bottom_left: value,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shadow {
    pub color: Color,
    pub offset_x: f32,
    pub offset_y: f32,
    pub blur: f32,
    pub spread: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform {
    pub translate_x: f32,
    pub translate_y: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub rotate_radians: f32,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translate_x: 0.0,
            translate_y: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            rotate_radians: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CursorIcon {
    Default,
    Pointer,
    Text,
    ResizeHorizontal,
    ResizeVertical,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Style {
    pub display: Option<Display>,
    pub position: Option<Position>,
    pub width: Option<Length>,
    pub height: Option<Length>,
    pub min_width: Option<Length>,
    pub max_width: Option<Length>,
    pub min_height: Option<Length>,
    pub max_height: Option<Length>,
    pub margin: Option<Edge<Length>>,
    pub padding: Option<Edge<Length>>,
    pub gap: Option<Length>,
    pub align_items: Option<Align>,
    pub align_self: Option<Align>,
    pub align_content: Option<Align>,
    pub justify_content: Option<crate::Justify>,
    pub flex_direction: Option<crate::FlexDirection>,
    pub flex_wrap: Option<crate::FlexWrap>,
    pub flex_grow: Option<f32>,
    pub flex_shrink: Option<f32>,
    pub flex_basis: Option<Length>,
    pub aspect_ratio: Option<f32>,
    pub inset: Option<Edge<Length>>,
    pub overflow_x: Option<Overflow>,
    pub overflow_y: Option<Overflow>,
    pub grid_template_columns: Option<Vec<GridTrack>>,
    pub grid_template_rows: Option<Vec<GridTrack>>,
    pub grid_column: Option<GridPlacement>,
    pub grid_row: Option<GridPlacement>,
    pub background: Option<Background>,
    pub border: Option<Border>,
    pub radius: Option<Radius>,
    pub shadow: Option<Vec<Shadow>>,
    pub opacity: Option<f32>,
    pub transform: Option<Transform>,
    pub z_index: Option<i32>,
    pub text: Option<TextStyle>,
    pub cursor: Option<CursorIcon>,
}

impl Style {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn display(mut self, value: Display) -> Self {
        self.display = Some(value);
        self
    }

    pub fn width(mut self, value: f32) -> Self {
        self.width = Some(Length::Px(value));
        self
    }

    pub fn height(mut self, value: f32) -> Self {
        self.height = Some(Length::Px(value));
        self
    }

    pub fn opacity(mut self, value: f32) -> Self {
        self.opacity = Some(value);
        self
    }

    pub fn z_index(mut self, value: i32) -> Self {
        self.z_index = Some(value);
        self
    }

    pub fn padding(mut self, value: f32) -> Self {
        self.padding = Some(Edge::all(Length::Px(value)));
        self
    }

    pub fn padding_edge(mut self, value: Edge<Length>) -> Self {
        self.padding = Some(value);
        self
    }

    pub fn font_weight(mut self, value: FontWeight) -> Self {
        let mut text = self.text.unwrap_or_default();
        text.weight = value;
        self.text = Some(text);
        self
    }

    pub fn background(mut self, value: Color) -> Self {
        self.background = Some(Background::Paint(Paint::Solid(value)));
        self
    }

    pub fn merge_over(mut self, next: Style) -> Self {
        merge(&mut self, next);
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedStyle {
    pub display: Display,
    pub position: Position,
    pub opacity: f32,
    pub z_index: i32,
    pub text: TextStyle,
    pub transform: Transform,
}

pub struct StyleResolver {
    mode: DefaultStyleMode,
}

impl StyleResolver {
    pub const fn new(mode: DefaultStyleMode) -> Self {
        Self { mode }
    }

    pub fn resolve_layers<I>(&self, layers: I) -> ResolvedStyle
    where
        I: IntoIterator<Item = Style>,
    {
        let mut merged = self.base_style();
        for layer in layers {
            merge(&mut merged, layer);
        }
        ResolvedStyle {
            display: merged.display.unwrap_or(Display::Block),
            position: merged.position.unwrap_or(Position::Relative),
            opacity: merged.opacity.unwrap_or(1.0),
            z_index: merged.z_index.unwrap_or(0),
            text: merged.text.unwrap_or_default(),
            transform: merged.transform.unwrap_or_default(),
        }
    }

    fn base_style(&self) -> Style {
        match self.mode {
            DefaultStyleMode::Full => Style::default().display(Display::Block).opacity(1.0),
            DefaultStyleMode::Minimal => Style::default().display(Display::Block),
            DefaultStyleMode::Reset => Style::default(),
        }
    }
}

fn merge(base: &mut Style, next: Style) {
    if next.display.is_some() {
        base.display = next.display;
    }
    if next.position.is_some() {
        base.position = next.position;
    }
    if next.width.is_some() {
        base.width = next.width;
    }
    if next.height.is_some() {
        base.height = next.height;
    }
    if next.min_width.is_some() {
        base.min_width = next.min_width;
    }
    if next.max_width.is_some() {
        base.max_width = next.max_width;
    }
    if next.min_height.is_some() {
        base.min_height = next.min_height;
    }
    if next.max_height.is_some() {
        base.max_height = next.max_height;
    }
    if next.align_self.is_some() {
        base.align_self = next.align_self;
    }
    if next.align_content.is_some() {
        base.align_content = next.align_content;
    }
    if next.flex_direction.is_some() {
        base.flex_direction = next.flex_direction;
    }
    if next.flex_wrap.is_some() {
        base.flex_wrap = next.flex_wrap;
    }
    if next.flex_grow.is_some() {
        base.flex_grow = next.flex_grow;
    }
    if next.flex_shrink.is_some() {
        base.flex_shrink = next.flex_shrink;
    }
    if next.flex_basis.is_some() {
        base.flex_basis = next.flex_basis;
    }
    if next.aspect_ratio.is_some() {
        base.aspect_ratio = next.aspect_ratio;
    }
    if next.inset.is_some() {
        base.inset = next.inset;
    }
    if next.grid_template_columns.is_some() {
        base.grid_template_columns = next.grid_template_columns;
    }
    if next.grid_template_rows.is_some() {
        base.grid_template_rows = next.grid_template_rows;
    }
    if next.grid_column.is_some() {
        base.grid_column = next.grid_column;
    }
    if next.grid_row.is_some() {
        base.grid_row = next.grid_row;
    }
    if next.margin.is_some() {
        base.margin = next.margin;
    }
    if next.padding.is_some() {
        base.padding = next.padding;
    }
    if next.gap.is_some() {
        base.gap = next.gap;
    }
    if next.align_items.is_some() {
        base.align_items = next.align_items;
    }
    if next.justify_content.is_some() {
        base.justify_content = next.justify_content;
    }
    if next.overflow_x.is_some() {
        base.overflow_x = next.overflow_x;
    }
    if next.overflow_y.is_some() {
        base.overflow_y = next.overflow_y;
    }
    if next.background.is_some() {
        base.background = next.background;
    }
    if next.border.is_some() {
        base.border = next.border;
    }
    if next.radius.is_some() {
        base.radius = next.radius;
    }
    if next.shadow.is_some() {
        base.shadow = next.shadow;
    }
    if next.opacity.is_some() {
        base.opacity = next.opacity;
    }
    if next.transform.is_some() {
        base.transform = next.transform;
    }
    if next.z_index.is_some() {
        base.z_index = next.z_index;
    }
    if next.text.is_some() {
        base.text = next.text;
    }
    if next.cursor.is_some() {
        base.cursor = next.cursor;
    }
}
