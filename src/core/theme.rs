use std::collections::HashMap;

use crate::style::DEFAULT_FONT_FAMILY;
use crate::{Color, FontWeight, Point, Radius, Rect, Shadow, Size, Style};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ThemeMode {
    Light,
    #[default]
    System,
    Dark,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ColorTokens {
    pub background: Color,
    pub surface: Color,
    pub surface_hover: Color,
    pub text: Color,
    pub text_muted: Color,
    pub primary: Color,
    pub primary_hover: Color,
    pub danger: Color,
    pub border: Color,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SpacingTokens {
    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RadiusTokens {
    pub none: Radius,
    pub sm: Radius,
    pub md: Radius,
    pub lg: Radius,
    pub full: Radius,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TypographyTokens {
    pub families: Vec<String>,
    pub body_size: f32,
    pub heading_size: f32,
    pub normal_weight: FontWeight,
    pub bold_weight: FontWeight,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ShadowTokens {
    pub none: Vec<Shadow>,
    pub sm: Vec<Shadow>,
    pub md: Vec<Shadow>,
    pub lg: Vec<Shadow>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    pub mode: ThemeMode,
    pub colors: ColorTokens,
    pub spacing: SpacingTokens,
    pub radius: RadiusTokens,
    pub typography: TypographyTokens,
    pub shadows: ShadowTokens,
    /// Per-widget sizing baselines and per-widget style
    /// overrides. Bug fix 3.8: this used to be `widgets:
    /// WidgetThemes` with `metrics` and `select` as nested
    /// fields. The intermediate struct was an indirection that
    /// didn't pay for itself (only two fields, both promoted
    /// directly to `Theme`), so it's gone. The `components`
    /// field is a `ComponentThemeMap` for future per-widget
    /// style overrides (see [`ComponentTheme`]).
    pub components: ComponentThemeMap,
    /// Sizing baselines for every widget kind. See
    /// [`WidgetMetrics::min_size_for`] for the canonical
    /// query path.
    pub metrics: WidgetMetrics,
    /// Per-state style overrides for the `Select` widget. The
    /// other widgets get their styles from the resolved style
    /// path; `Select` is special because its part styles
    /// (trigger, dropdown, etc.) are not reachable through
    /// `ResolvedWidgetStyle`.
    pub select: SelectTheme,
}

impl Theme {
    pub fn light() -> Self {
        Self {
            mode: ThemeMode::Light,
            colors: ColorTokens {
                background: Color::rgb(255, 255, 255),
                surface: Color::rgb(247, 248, 250),
                surface_hover: Color::rgb(235, 238, 242),
                text: Color::rgb(20, 23, 28),
                text_muted: Color::rgb(91, 99, 112),
                primary: Color::rgb(35, 99, 235),
                primary_hover: Color::rgb(29, 78, 216),
                danger: Color::rgb(220, 38, 38),
                border: Color::rgb(210, 216, 226),
            },
            spacing: SpacingTokens {
                xs: 4.0,
                sm: 8.0,
                md: 12.0,
                lg: 16.0,
                xl: 24.0,
            },
            radius: RadiusTokens {
                none: Radius::all(0.0),
                sm: Radius::all(3.0),
                md: Radius::all(6.0),
                lg: Radius::all(8.0),
                full: Radius::all(999.0),
            },
            typography: TypographyTokens {
                families: vec![DEFAULT_FONT_FAMILY.into()],
                body_size: 14.0,
                heading_size: 24.0,
                normal_weight: FontWeight::Normal,
                bold_weight: FontWeight::Bold,
            },
            shadows: ShadowTokens {
                none: vec![],
                sm: vec![],
                md: vec![],
                lg: vec![],
            },
            components: ComponentThemeMap::default(),
            metrics: WidgetMetrics::default(),
            select: SelectTheme::default(),
        }
    }

    pub fn dark() -> Self {
        let mut theme = Self::light();
        theme.mode = ThemeMode::Dark;
        theme.colors.background = Color::rgb(14, 16, 20);
        theme.colors.surface = Color::rgb(25, 29, 36);
        theme.colors.surface_hover = Color::rgb(38, 44, 54);
        theme.colors.text = Color::rgb(241, 245, 249);
        theme.colors.text_muted = Color::rgb(148, 163, 184);
        theme.colors.primary = Color::rgb(96, 165, 250);
        theme.colors.primary_hover = Color::rgb(59, 130, 246);
        theme.colors.border = Color::rgb(51, 65, 85);
        theme
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::light()
    }
}

/// Bug fix 3.8: this struct used to be the `widgets` field on
/// [`Theme`]. It only carried two fields (`metrics` and
/// `select`), so the intermediate struct added no value and
/// the `theme.widgets.X` indirection was an unnecessary
/// wrinkle. The fields are now directly on [`Theme`] and this
/// type is gone.
///
/// The `3.8` review note: the alternative was to commit to
/// `theme.components.get(WidgetKind)` as a uniform per-widget
/// theme lookup. That requires every per-widget-metric call
/// site to migrate through a `HashMap` lookup, which is
/// slower and has no upside for a flat namespace like
/// `WidgetMetrics`. Flat fields on `Theme` are the simpler
/// path; `components` is kept as the future home for
/// per-widget *style* overrides, which `WidgetMetrics` does
/// not cover.
#[deprecated(
    since = "0.1.0",
    note = "WidgetThemes was a 2-field wrapper. Use `Theme::metrics` and `Theme::select` directly."
)]
#[derive(Clone, Debug, PartialEq)]
pub struct WidgetThemes {
    pub select: SelectTheme,
    pub metrics: WidgetMetrics,
}

#[allow(deprecated)]
impl Default for WidgetThemes {
    fn default() -> Self {
        Self {
            select: SelectTheme::default(),
            metrics: WidgetMetrics::default(),
        }
    }
}

/// Per-widget sizing baselines (min size, default size, padding,
/// row height, etc.). The struct is a flat namespace of `*Metrics`
/// fields; the canonical way to query "what is the min size of a
/// `Button`" is [`WidgetMetrics::min_size_for`].
///
/// **Access pattern:** callers should go through [`WidgetMetrics::min_size_for`]
/// for size lookups by `WidgetKind`. Direct field access is
/// reserved for fields that are *not* the min size (e.g.
/// `metrics.tabs.tab_height` for the tab-strip height, which is a
/// layout detail independent of the Tabs content's min size). Bug
/// fix 8.4: this single-access pattern is documented but not
/// enforced at compile time.
#[derive(Clone, Debug, PartialEq)]
pub struct WidgetMetrics {
    pub button: ButtonMetrics,
    pub input: InputMetrics,
    pub select: SelectMetrics,
    pub textarea: TextareaMetrics,
    pub tabs: TabsMetrics,
    pub tree: TreeMetrics,
    pub table: TableMetrics,
    pub list: ListMetrics,
    pub menu: MenuMetrics,
    pub menu_item: MenuItemTheme,
    pub icon: IconMetrics,
    pub divider: DividerMetrics,
    pub canvas: CanvasMetrics,
    pub tooltip: TooltipMetrics,
    pub overlay: OverlayMetrics,
    pub scrollbar: ScrollbarMetrics,
    pub card: CardTheme,
    pub alert: AlertTheme,
    pub modal: ModalTheme,
    pub popover: PopoverTheme,
    pub image: ImageTheme,
    pub switch: SwitchTheme,
    pub slider: SliderTheme,
    pub progress_bar: ProgressBarTheme,
    pub spinner: SpinnerTheme,
    pub badge: BadgeTheme,
    pub avatar: AvatarTheme,
    pub link: LinkTheme,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ButtonMetrics {
    pub min_width: f32,
    pub height: f32,
    pub horizontal_padding: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct InputMetrics {
    pub min_size: Size,
    pub horizontal_padding: f32,
    pub vertical_padding: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SelectMetrics {
    pub trigger_min_size: Size,
    pub horizontal_padding: f32,
    pub arrow_slot_width: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TextareaMetrics {
    pub min_size: Size,
    pub horizontal_padding: f32,
    pub vertical_padding: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TabsMetrics {
    pub min_size: Size,
    pub tab_min_width: f32,
    pub tab_height: f32,
    pub tab_gap: f32,
    pub horizontal_padding: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TreeMetrics {
    pub min_size: Size,
    pub row_height: f32,
    pub indent: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TableMetrics {
    pub min_size: Size,
    pub row_height: f32,
    pub cell_padding: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ListMetrics {
    pub min_size: Size,
    pub row_height: f32,
    pub item_padding: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MenuMetrics {
    pub min_size: Size,
    pub item_height: f32,
    pub item_padding: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MenuItemTheme {
    pub default_size: Size,
    pub padding: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct IconMetrics {
    pub default_size: Size,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DividerMetrics {
    pub default_size: Size,
    pub thickness: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CanvasMetrics {
    pub default_size: Size,
    pub padding: f32,
    pub label_baseline_offset: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TooltipMetrics {
    pub min_size: Size,
    pub horizontal_padding: f32,
    pub vertical_padding: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct OverlayMetrics {
    pub min_width: f32,
    pub min_height: f32,
    pub padding: f32,
    pub gap: f32,
    pub max_measure_height: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ScrollbarMetrics {
    pub width: f32,
    pub padding: f32,
    pub min_thumb_height: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CardTheme {
    pub min_size: Size,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AlertTheme {
    pub min_size: Size,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ModalTheme {
    pub min_size: Size,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PopoverTheme {
    pub min_size: Size,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ImageTheme {
    pub default_size: Size,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SwitchTheme {
    pub default_size: Size,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SliderTheme {
    pub default_size: Size,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ProgressBarTheme {
    pub default_size: Size,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SpinnerTheme {
    pub default_size: Size,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BadgeTheme {
    pub default_size: Size,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AvatarTheme {
    pub default_size: Size,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LinkTheme {
    pub default_size: Size,
}

impl ScrollbarMetrics {
    pub fn track_rect(self, rect: Rect) -> Rect {
        Rect::new(
            Point::new(
                rect.max_x() - self.padding * 2.0,
                rect.origin.y + self.padding,
            ),
            Size::new(self.width, (rect.size.height - self.padding * 2.0).max(1.0)),
        )
    }

    pub fn thumb_rect(self, rect: Rect, content_size: Size, scroll_offset: crate::Vec2) -> Rect {
        let track = self.track_rect(rect);
        let ratio = (rect.size.height / content_size.height).clamp(0.0, 1.0);
        let thumb_height = (track.size.height * ratio)
            .max(self.min_thumb_height)
            .min(track.size.height);
        let max_offset = (content_size.height - rect.size.height).max(1.0);
        let thumb_y = track.origin.y
            + (track.size.height - thumb_height) * (scroll_offset.y / max_offset).clamp(0.0, 1.0);

        Rect::new(
            Point::new(track.origin.x, thumb_y),
            Size::new(track.size.width, thumb_height),
        )
    }
}

impl Default for WidgetMetrics {
    fn default() -> Self {
        Self {
            button: ButtonMetrics {
                min_width: 72.0,
                height: 32.0,
                horizontal_padding: 24.0,
            },
            input: InputMetrics {
                min_size: Size::new(160.0, 36.0),
                horizontal_padding: 8.0,
                vertical_padding: 4.0,
            },
            select: SelectMetrics {
                trigger_min_size: Size::new(120.0, 36.0),
                horizontal_padding: 8.0,
                arrow_slot_width: 18.0,
            },
            textarea: TextareaMetrics {
                min_size: Size::new(200.0, 80.0),
                horizontal_padding: 8.0,
                vertical_padding: 4.0,
            },
            tabs: TabsMetrics {
                min_size: Size::new(200.0, 32.0),
                tab_min_width: 48.0,
                tab_height: 20.0,
                tab_gap: 8.0,
                horizontal_padding: 8.0,
            },
            tree: TreeMetrics {
                min_size: Size::new(200.0, 180.0),
                row_height: 20.0,
                indent: 16.0,
            },
            table: TableMetrics {
                min_size: Size::new(300.0, 180.0),
                row_height: 24.0,
                cell_padding: 4.0,
            },
            list: ListMetrics {
                min_size: Size::new(200.0, 180.0),
                row_height: 24.0,
                item_padding: 8.0,
            },
            menu: MenuMetrics {
                min_size: Size::new(150.0, 180.0),
                item_height: 22.0,
                item_padding: 8.0,
            },
            menu_item: MenuItemTheme {
                default_size: Size::new(120.0, 22.0),
                padding: 8.0,
            },
            icon: IconMetrics {
                default_size: Size::new(24.0, 24.0),
            },
            divider: DividerMetrics {
                default_size: Size::new(8.0, 8.0),
                thickness: 1.0,
            },
            canvas: CanvasMetrics {
                default_size: Size::new(200.0, 150.0),
                padding: 8.0,
                label_baseline_offset: 22.0,
            },
            tooltip: TooltipMetrics {
                min_size: Size::new(120.0, 32.0),
                horizontal_padding: 8.0,
                vertical_padding: 6.0,
            },
            overlay: OverlayMetrics {
                min_width: 160.0,
                min_height: 48.0,
                padding: 16.0,
                gap: 6.0,
                max_measure_height: 10000.0,
            },
            scrollbar: ScrollbarMetrics {
                width: 4.0,
                padding: 4.0,
                min_thumb_height: 20.0,
            },
            card: CardTheme {
                min_size: Size::new(200.0, 100.0),
            },
            alert: AlertTheme {
                min_size: Size::new(200.0, 60.0),
            },
            modal: ModalTheme {
                min_size: Size::new(240.0, 120.0),
            },
            popover: PopoverTheme {
                min_size: Size::new(180.0, 80.0),
            },
            image: ImageTheme {
                default_size: Size::new(100.0, 100.0),
            },
            switch: SwitchTheme {
                default_size: Size::new(44.0, 24.0),
            },
            slider: SliderTheme {
                default_size: Size::new(200.0, 24.0),
            },
            progress_bar: ProgressBarTheme {
                default_size: Size::new(200.0, 8.0),
            },
            spinner: SpinnerTheme {
                default_size: Size::new(24.0, 24.0),
            },
            badge: BadgeTheme {
                default_size: Size::new(60.0, 20.0),
            },
            avatar: AvatarTheme {
                default_size: Size::new(40.0, 40.0),
            },
            link: LinkTheme {
                default_size: Size::new(0.0, 16.0),
            },
        }
    }
}

impl WidgetMetrics {
    pub fn min_size_for(&self, kind: WidgetKind) -> Size {
        match kind {
            WidgetKind::Button => Size::new(self.button.min_width, self.button.height),
            WidgetKind::Input => self.input.min_size,
            WidgetKind::Select => self.select.trigger_min_size,
            WidgetKind::Textarea => self.textarea.min_size,
            WidgetKind::Tabs => self.tabs.min_size,
            WidgetKind::Tree => self.tree.min_size,
            WidgetKind::Table => self.table.min_size,
            WidgetKind::List => self.list.min_size,
            WidgetKind::Menu => self.menu.min_size,
            WidgetKind::MenuItem => self.menu_item.default_size,
            WidgetKind::Icon => self.icon.default_size,
            WidgetKind::Divider => self.divider.default_size,
            WidgetKind::Canvas => self.canvas.default_size,
            WidgetKind::Tooltip => self.tooltip.min_size,
            WidgetKind::Card => self.card.min_size,
            WidgetKind::Alert => self.alert.min_size,
            WidgetKind::Modal => self.modal.min_size,
            WidgetKind::Popover => self.popover.min_size,
            WidgetKind::Image => self.image.default_size,
            WidgetKind::Switch => self.switch.default_size,
            WidgetKind::Slider => self.slider.default_size,
            WidgetKind::ProgressBar => self.progress_bar.default_size,
            WidgetKind::Spinner => self.spinner.default_size,
            WidgetKind::Badge => self.badge.default_size,
            WidgetKind::Avatar => self.avatar.default_size,
            WidgetKind::Link => self.link.default_size,
            WidgetKind::ScrollArea
            | WidgetKind::Checkbox
            | WidgetKind::Radio
            | WidgetKind::Text => Size::new(0.0, 0.0),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SelectTheme {
    pub base: crate::widgets::SelectPartStyles,
    pub variants: HashMap<VariantId, crate::widgets::SelectPartStyles>,
}

impl SelectTheme {
    pub fn variant(
        &mut self,
        id: impl Into<String>,
        configure: impl FnOnce(&mut crate::widgets::SelectStylesBuilder<'_>),
    ) {
        let id = VariantId::new(id);
        let mut styles = self.variants.remove(&id).unwrap_or_default();
        {
            let mut builder = crate::widgets::SelectStylesBuilder::new(&mut styles);
            configure(&mut builder);
        }
        self.variants.insert(id, styles);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct VariantId(String);

impl VariantId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WidgetKind {
    Text,
    Button,
    Input,
    Checkbox,
    Radio,
    Select,
    Textarea,
    Tabs,
    Tree,
    Table,
    Modal,
    Popover,
    Tooltip,
    Menu,
    MenuItem,
    ScrollArea,
    List,
    Canvas,
    Icon,
    Divider,
    Image,
    Switch,
    Slider,
    ProgressBar,
    Spinner,
    Badge,
    Avatar,
    Link,
    Alert,
    Card,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ComponentTheme {
    pub base: Style,
    pub variants: HashMap<VariantId, Style>,
    pub states: HashMap<u16, Style>,
}

impl ComponentTheme {
    pub fn with_variant(mut self, id: VariantId, style: Style) -> Self {
        self.variants.insert(id, style);
        self
    }

    pub fn variant(&self, id: &VariantId) -> Option<&Style> {
        self.variants.get(id)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ComponentThemeMap {
    themes: HashMap<WidgetKind, ComponentTheme>,
}

impl ComponentThemeMap {
    pub fn insert(&mut self, kind: WidgetKind, theme: ComponentTheme) {
        self.themes.insert(kind, theme);
    }

    pub fn get(&self, kind: WidgetKind) -> Option<&ComponentTheme> {
        self.themes.get(&kind)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedWidgetStyle {
    pub background: Color,
    pub foreground: Color,
    pub border_color: Color,
    pub border_width: f32,
    pub border_radius: f32,
    pub text_color: Color,
    pub text_muted_color: Color,
    pub font_size: f32,
    pub font_weight: FontWeight,
}

impl ResolvedWidgetStyle {
    pub fn from_theme(theme: &Theme) -> Self {
        Self {
            background: theme.colors.surface,
            foreground: theme.colors.primary,
            border_color: theme.colors.border,
            border_width: 1.5,
            border_radius: 4.0,
            text_color: theme.colors.text,
            text_muted_color: theme.colors.text_muted,
            font_size: theme.typography.body_size,
            font_weight: theme.typography.normal_weight,
        }
    }

    pub fn default_for(_kind: WidgetKind, _state: &ResolvedStateFlags) -> Self {
        Self::from_theme(&Theme::light())
    }
}

impl Theme {
    pub fn resolve_widget_style(
        &self,
        _kind: WidgetKind,
        variant: Option<crate::VariantId>,
        state: &ResolvedStateFlags,
    ) -> ResolvedWidgetStyle {
        let mut style = ResolvedWidgetStyle::from_theme(self);

        if variant.as_ref().is_some_and(|v| v.as_str() == "primary") {
            style.background = self.colors.primary;
            style.text_color = Color::rgb(255, 255, 255);
        }

        if state.hovered {
            style.background = self.colors.surface_hover;
            style.border_color = if self.mode == ThemeMode::Dark {
                Color::rgb(71, 85, 105)
            } else {
                Color::rgb(150, 160, 180)
            };
        }

        if state.focused {
            style.border_color = self.colors.primary;
            if !variant.as_ref().is_some_and(|v| v.as_str() == "primary") {
                style.background = self.colors.background;
            }
        }

        if state.active {
            style.background = self.colors.primary_hover;
        }

        if state.disabled {
            style.background = self.colors.surface_hover;
            style.text_color = self.colors.text_muted;
            style.border_color = self.colors.surface_hover;
        }

        style
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ResolvedStateFlags {
    pub hovered: bool,
    pub focused: bool,
    pub active: bool,
    pub disabled: bool,
    pub checked: bool,
    pub open: bool,
}

impl ResolvedStateFlags {
    pub fn new(hovered: bool, focused: bool, active: bool) -> Self {
        Self {
            hovered,
            focused,
            active,
            disabled: false,
            checked: false,
            open: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ThemeScope {
    theme: Theme,
}

impl ThemeScope {
    pub fn new(theme: Theme) -> Self {
        Self { theme }
    }

    pub fn with_primary(mut self, color: Color) -> Self {
        self.theme.colors.primary = color;
        self
    }

    pub fn theme(&self) -> &Theme {
        &self.theme
    }
}
