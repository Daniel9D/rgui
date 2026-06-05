use crate::core::WidgetKind;

/// The umbrella enum that wraps every concrete spec type. The runtime
/// converts an `Element` tree into a `WidgetSpec` during reconciliation.
///
/// ```rust
/// use rgui::widgets::spec::{ButtonSpec, WidgetSpec};
/// let _ = WidgetSpec::Button(ButtonSpec::default());
/// ```
#[derive(Clone, Debug, PartialEq)]
pub enum WidgetSpec {
    Button(ButtonSpec),
    Input(InputSpec),
    Checkbox(CheckboxSpec),
    Radio(RadioSpec),
    Select(SelectSpec),
    Textarea(TextareaSpec),
    Tabs(TabsSpec),
    Tree(TreeSpec),
    Table(TableSpec),
    List(ListSpec),
    Menu(MenuSpec),
    MenuItem(MenuItemSpec),
    Modal(ModalSpec),
    Popover(PopoverSpec),
    Tooltip(TooltipSpec),
    Divider,
    Icon(IconSpec),
    Image(ImageSpec),
    Switch(SwitchSpec),
    Slider(SliderSpec),
    ProgressBar(ProgressBarSpec),
    Spinner(SpinnerSpec),
    Badge(BadgeSpec),
    Avatar(AvatarSpec),
    Link(LinkSpec),
    Alert(AlertSpec),
    Card(CardSpec),
}

impl WidgetSpec {
    pub fn kind(&self) -> WidgetKind {
        match self {
            WidgetSpec::Button(_) => WidgetKind::Button,
            WidgetSpec::Input(_) => WidgetKind::Input,
            WidgetSpec::Checkbox(_) => WidgetKind::Checkbox,
            WidgetSpec::Radio(_) => WidgetKind::Radio,
            WidgetSpec::Select(_) => WidgetKind::Select,
            WidgetSpec::Textarea(_) => WidgetKind::Textarea,
            WidgetSpec::Tabs(_) => WidgetKind::Tabs,
            WidgetSpec::Tree(_) => WidgetKind::Tree,
            WidgetSpec::Table(_) => WidgetKind::Table,
            WidgetSpec::List(_) => WidgetKind::List,
            WidgetSpec::Menu(_) => WidgetKind::Menu,
            WidgetSpec::MenuItem(_) => WidgetKind::MenuItem,
            WidgetSpec::Modal(_) => WidgetKind::Modal,
            WidgetSpec::Popover(_) => WidgetKind::Popover,
            WidgetSpec::Tooltip(_) => WidgetKind::Tooltip,
            WidgetSpec::Divider => WidgetKind::Divider,
            WidgetSpec::Icon(_) => WidgetKind::Icon,
            WidgetSpec::Image(_) => WidgetKind::Image,
            WidgetSpec::Switch(_) => WidgetKind::Switch,
            WidgetSpec::Slider(_) => WidgetKind::Slider,
            WidgetSpec::ProgressBar(_) => WidgetKind::ProgressBar,
            WidgetSpec::Spinner(_) => WidgetKind::Spinner,
            WidgetSpec::Badge(_) => WidgetKind::Badge,
            WidgetSpec::Avatar(_) => WidgetKind::Avatar,
            WidgetSpec::Link(_) => WidgetKind::Link,
            WidgetSpec::Alert(_) => WidgetKind::Alert,
            WidgetSpec::Card(_) => WidgetKind::Card,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
/// Configuration for a `Button` widget.
///
/// ```rust
/// use rgui::widgets::spec::ButtonSpec;
/// let _ = ButtonSpec::default();
/// ```
pub struct ButtonSpec {
    pub label: Option<String>,
    pub disabled: bool,
    pub loading: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
/// Configuration for an `Input` text-entry widget.
///
/// ```rust
/// use rgui::widgets::spec::InputSpec;
/// let _ = InputSpec::default();
/// ```
pub struct InputSpec {
    pub placeholder: Option<String>,
    pub default_value: Option<String>,
    pub value: Option<String>,
    pub disabled: bool,
    pub password: bool,
    pub aria_label: Option<String>,
    /// Phase 2 / Plan 02-04: opt-in to IME composition events. When
    /// `true`, the runtime routes `ImePreedit` / `ImeCommit` events
    /// to this `Input` so CJK (and other IME-using) users can type.
    /// Defaults to `false`: Latin-keyboard users get the simpler
    /// direct-key-event path. v1 covers the CJK preedit-then-commit
    /// model; complex-script composition (Hindi, Thai, Khmer) is
    /// deferred to v1.x.
    pub ime_enabled: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
/// Configuration for a `Checkbox` widget.
///
/// ```rust
/// use rgui::widgets::spec::CheckboxSpec;
/// let _ = CheckboxSpec::default();
/// ```
pub struct CheckboxSpec {
    pub label: Option<String>,
    pub disabled: bool,
    pub indeterminate: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
/// Configuration for a `Radio` widget.
///
/// ```rust
/// use rgui::widgets::spec::RadioSpec;
/// let _ = RadioSpec::default();
/// ```
pub struct RadioSpec {
    pub label: Option<String>,
    pub disabled: bool,
    pub value: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
/// A single `<option>` inside a `Select` widget.
///
/// ```rust
/// use rgui::widgets::spec::SelectOption;
/// let opt = SelectOption::new("value", "Label");
/// assert_eq!(opt.value, "value");
/// assert_eq!(opt.label, "Label");
/// ```
pub struct SelectOption {
    pub value: String,
    pub label: String,
    pub disabled: bool,
}

impl SelectOption {
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
/// Per-part style overrides for a `Select` widget. Each field is
/// `Some(Style)` to apply, `None` to inherit from the theme.
///
/// ```rust
/// use rgui::widgets::spec::SelectPartStyles;
/// let _ = SelectPartStyles::default();
/// ```
pub struct SelectPartStyles {
    pub trigger: Option<crate::Style>,
    pub popover: Option<crate::Style>,
    pub list: Option<crate::Style>,
    pub item: Option<crate::Style>,
    pub item_hovered: Option<crate::Style>,
    pub item_selected: Option<crate::Style>,
    pub item_disabled: Option<crate::Style>,
}

#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
/// Configuration for a `Select` dropdown widget.
///
/// ```rust
/// use rgui::widgets::spec::{SelectOption, SelectSpec};
/// let mut spec = SelectSpec::default();
/// spec.options.push(SelectOption::new("v", "L"));
/// let _ = spec;
/// ```
pub struct SelectSpec {
    pub placeholder: Option<String>,
    pub disabled: bool,
    pub options: Vec<SelectOption>,
    pub selected_index: Option<usize>,
    pub default_value: Option<String>,
    pub styles: SelectPartStyles,
}

#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
/// Configuration for a multi-line `Textarea` widget.
///
/// ```rust
/// use rgui::widgets::spec::TextareaSpec;
/// let _ = TextareaSpec::default();
/// ```
pub struct TextareaSpec {
    pub placeholder: Option<String>,
    pub default_value: Option<String>,
    pub value: Option<String>,
    pub disabled: bool,
    pub rows: Option<usize>,
}

#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
/// Configuration for a `Tabs` widget.
///
/// ```rust
/// use rgui::widgets::spec::TabsSpec;
/// let _ = TabsSpec::default();
/// ```
pub struct TabsSpec {
    pub tabs: Vec<String>,
    pub active_index: Option<usize>,
}

#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
/// Configuration for a hierarchical `Tree` widget.
///
/// ```rust
/// use rgui::widgets::spec::TreeSpec;
/// let _ = TreeSpec::default();
/// ```
pub struct TreeSpec {
    pub items: Vec<TreeItemSpec>,
}

#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
/// A single node inside a `Tree` widget.
///
/// ```rust
/// use rgui::widgets::spec::TreeItemSpec;
/// let _ = TreeItemSpec::default();
/// ```
pub struct TreeItemSpec {
    pub label: String,
    pub expanded: bool,
    pub children: Vec<TreeItemSpec>,
}

#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
/// Configuration for a `Table` widget.
///
/// ```rust
/// use rgui::widgets::spec::TableSpec;
/// let _ = TableSpec::default();
/// ```
pub struct TableSpec {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub selected_row: Option<usize>,
}

impl TableSpec {
    /// Returns `true` if every row has exactly as many cells as there are
    /// column headers. An empty table (no columns, no rows) is considered valid.
    pub fn is_valid(&self) -> bool {
        self.rows.iter().all(|row| row.len() == self.columns.len())
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
/// Configuration for a `List` widget.
///
/// ```rust
/// use rgui::widgets::spec::ListSpec;
/// let _ = ListSpec::default();
/// ```
pub struct ListSpec {
    pub items: Vec<String>,
    pub selected_index: Option<usize>,
}

#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
/// Configuration for a `Menu` widget (top-level menu container).
///
/// ```rust
/// use rgui::widgets::spec::MenuSpec;
/// let _ = MenuSpec::default();
/// ```
pub struct MenuSpec {}

#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
/// Configuration for a single item inside a `Menu` widget.
///
/// ```rust
/// use rgui::widgets::spec::MenuItemSpec;
/// let _ = MenuItemSpec::default();
/// ```
pub struct MenuItemSpec {
    pub label: String,
    pub action: Option<String>,
    pub disabled: bool,
    pub shortcut: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
/// Configuration for a `Modal` overlay widget.
///
/// ```rust
/// use rgui::widgets::spec::ModalSpec;
/// let _ = ModalSpec::default();
/// ```
pub struct ModalSpec {
    pub title: Option<String>,
    pub close_on_escape: bool,
    pub close_on_outside_click: bool,
    /// Phase 2 / Plan 02-01: when `true`, Tab traversal is
    /// restricted to the modal's subtree. Tab cycles inside the
    /// modal; it does not move focus to nodes outside. Defaults to
    /// `false` for the simple modal use case.
    pub trap_focus: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
/// Configuration for a `Popover` floating widget.
///
/// ```rust
/// use rgui::widgets::spec::PopoverSpec;
/// let _ = PopoverSpec::default();
/// ```
pub struct PopoverSpec {
    pub content_label: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
/// Configuration for a `Tooltip` widget.
///
/// ```rust
/// use rgui::widgets::spec::TooltipSpec;
/// let _ = TooltipSpec::default();
/// ```
pub struct TooltipSpec {
    pub text: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
/// Configuration for an `Icon` widget.
///
/// ```rust
/// use rgui::widgets::spec::IconSpec;
/// let _ = IconSpec::default();
/// ```
pub struct IconSpec {
    pub name: String,
}

impl IconSpec {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
/// Configuration for an `Image` widget.
///
/// ```rust
/// use rgui::widgets::spec::ImageSpec;
/// let _ = ImageSpec::default();
/// ```
pub struct ImageSpec {
    pub src: Option<String>,
    pub alt: Option<String>,
    pub fit: ImageFit,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
/// How an `Image` widget scales its source to fit the layout box.
///
/// ```rust
/// use rgui::widgets::spec::ImageFit;
/// let _ = (ImageFit::Cover, ImageFit::Contain, ImageFit::Fill);
/// ```
pub enum ImageFit {
    #[default]
    Cover,
    Contain,
    Fill,
    None,
}

#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
/// Configuration for a `Switch` toggle widget.
///
/// ```rust
/// use rgui::widgets::spec::SwitchSpec;
/// let _ = SwitchSpec::default();
/// ```
pub struct SwitchSpec {
    pub label: Option<String>,
    pub disabled: bool,
    pub checked: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
/// Configuration for a `Slider` widget.
///
/// ```rust
/// use rgui::widgets::spec::SliderSpec;
/// let _ = SliderSpec::default();
/// ```
pub struct SliderSpec {
    pub min: f32,
    pub max: f32,
    pub step: Option<f32>,
    pub value: f32,
    pub disabled: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
/// Configuration for a `ProgressBar` widget.
///
/// ```rust
/// use rgui::widgets::spec::ProgressBarSpec;
/// let _ = ProgressBarSpec::default();
/// ```
pub struct ProgressBarSpec {
    pub value: f32,
    pub max: f32,
    pub indeterminate: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
/// Configuration for a `Spinner` widget.
///
/// ```rust
/// use rgui::widgets::spec::SpinnerSpec;
/// let _ = SpinnerSpec::default();
/// ```
pub struct SpinnerSpec {
    pub label: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
/// Configuration for a `Badge` widget.
///
/// ```rust
/// use rgui::widgets::spec::BadgeSpec;
/// let _ = BadgeSpec::default();
/// ```
pub struct BadgeSpec {
    pub text: String,
    pub variant: BadgeVariant,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
/// Color/intent variant for a `Badge` widget.
///
/// ```rust
/// use rgui::widgets::spec::BadgeVariant;
/// let _ = (BadgeVariant::Default, BadgeVariant::Primary, BadgeVariant::Success);
/// ```
pub enum BadgeVariant {
    #[default]
    Default,
    Primary,
    Success,
    Warning,
    Danger,
}

#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
/// Configuration for an `Avatar` widget.
///
/// ```rust
/// use rgui::widgets::spec::AvatarSpec;
/// let _ = AvatarSpec::default();
/// ```
pub struct AvatarSpec {
    pub src: Option<String>,
    pub initials: Option<String>,
    pub size: AvatarSize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
/// Size preset for an `Avatar` widget.
///
/// ```rust
/// use rgui::widgets::spec::AvatarSize;
/// let _ = (AvatarSize::Sm, AvatarSize::Md, AvatarSize::Lg);
/// ```
pub enum AvatarSize {
    #[default]
    Md,
    Sm,
    Lg,
    Xl,
}

#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
/// Configuration for a `Link` widget.
///
/// ```rust
/// use rgui::widgets::spec::LinkSpec;
/// let _ = LinkSpec::default();
/// ```
pub struct LinkSpec {
    pub href: Option<String>,
    pub label: Option<String>,
    pub disabled: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
/// Configuration for an `Alert` banner widget.
///
/// ```rust
/// use rgui::widgets::spec::AlertSpec;
/// let _ = AlertSpec::default();
/// ```
pub struct AlertSpec {
    pub title: Option<String>,
    pub variant: AlertVariant,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
/// Severity/intent variant for an `Alert` widget.
///
/// ```rust
/// use rgui::widgets::spec::AlertVariant;
/// let _ = (AlertVariant::Info, AlertVariant::Success, AlertVariant::Error);
/// ```
pub enum AlertVariant {
    #[default]
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
/// Configuration for a `Card` container widget.
///
/// ```rust
/// use rgui::widgets::spec::CardSpec;
/// let _ = CardSpec::default();
/// ```
pub struct CardSpec {
    pub title: Option<String>,
}

pub fn spec_label(spec: &WidgetSpec) -> Option<String> {
    match spec {
        WidgetSpec::Button(bs) => bs.label.clone(),
        WidgetSpec::Checkbox(cs) => cs.label.clone(),
        WidgetSpec::Radio(rs) => rs.label.clone(),
        WidgetSpec::Switch(sw) => sw.label.clone(),
        WidgetSpec::Input(is) => is.aria_label.clone(),
        WidgetSpec::Textarea(ts) => ts.placeholder.clone(),
        WidgetSpec::Select(ss) => ss.placeholder.clone(),
        WidgetSpec::Tooltip(ts) => ts.text.clone(),
        WidgetSpec::Icon(is) => Some(is.name.clone()),
        WidgetSpec::Image(is) => is.alt.clone(),
        WidgetSpec::Modal(ms) => ms.title.clone(),
        WidgetSpec::Popover(ps) => ps.content_label.clone(),
        WidgetSpec::MenuItem(mi) => Some(mi.label.clone()),
        WidgetSpec::Spinner(sp) => sp.label.clone(),
        WidgetSpec::Badge(b) => Some(b.text.clone()),
        WidgetSpec::Avatar(a) => a.initials.clone(),
        WidgetSpec::Link(l) => l.label.clone().or_else(|| l.href.clone()),
        WidgetSpec::Alert(a) => a.title.clone(),
        WidgetSpec::Card(c) => c.title.clone(),
        _ => None,
    }
}

pub fn spec_label_str(spec: &WidgetSpec) -> Option<&str> {
    match spec {
        WidgetSpec::Button(bs) => bs.label.as_deref(),
        WidgetSpec::Checkbox(cs) => cs.label.as_deref(),
        WidgetSpec::Radio(rs) => rs.label.as_deref(),
        WidgetSpec::Switch(sw) => sw.label.as_deref(),
        WidgetSpec::Input(is) => is.aria_label.as_deref(),
        WidgetSpec::Textarea(ts) => ts.placeholder.as_deref(),
        WidgetSpec::Select(ss) => ss.placeholder.as_deref(),
        WidgetSpec::Tooltip(ts) => ts.text.as_deref(),
        WidgetSpec::Icon(is) => Some(is.name.as_str()),
        WidgetSpec::Image(is) => is.alt.as_deref(),
        WidgetSpec::Modal(ms) => ms.title.as_deref(),
        WidgetSpec::Popover(ps) => ps.content_label.as_deref(),
        WidgetSpec::MenuItem(mi) => Some(mi.label.as_str()),
        WidgetSpec::Spinner(sp) => sp.label.as_deref(),
        WidgetSpec::Badge(b) => Some(b.text.as_str()),
        WidgetSpec::Avatar(a) => a.initials.as_deref(),
        WidgetSpec::Link(l) => l.label.as_deref().or_else(|| l.href.as_deref()),
        WidgetSpec::Alert(a) => a.title.as_deref(),
        WidgetSpec::Card(c) => c.title.as_deref(),
        _ => None,
    }
}

/// Compute a stable 64-bit signature of the spec **kind** for use by
/// the reconciler.
///
/// Two specs with the same signature can be treated as a *patch* by the
/// reconciler (state preserved, dirty flags set on style/text changes).
/// Different signatures mean the reconciler should unmount the old and
/// mount a fresh node (state reset).
///
/// The signature intentionally hashes **only** the `WidgetKind` — not the
/// spec's text content (`label`, `value`, etc.) — because for v1 the
/// reconciler treats label changes as a *patch* (state preserved, just
/// the displayed text is updated). State lives in the runtime's
/// `BoolState` / `Value` / etc., keyed by `NodeId`, so a label change
/// that preserves the same `NodeId` keeps the state. (A future
/// spec-shape-change detection could re-include shape fields here.)
pub fn spec_signature(spec: &WidgetSpec) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    (spec.kind() as u8).hash(&mut h);
    h.finish()
}
