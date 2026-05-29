use crate::core::WidgetKind;

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
pub struct ButtonSpec {
    pub label: Option<String>,
    pub disabled: bool,
    pub loading: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct InputSpec {
    pub placeholder: Option<String>,
    pub default_value: Option<String>,
    pub value: Option<String>,
    pub disabled: bool,
    pub password: bool,
    pub aria_label: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct CheckboxSpec {
    pub label: Option<String>,
    pub disabled: bool,
    pub indeterminate: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct RadioSpec {
    pub label: Option<String>,
    pub disabled: bool,
    pub value: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
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
pub struct TextareaSpec {
    pub placeholder: Option<String>,
    pub default_value: Option<String>,
    pub value: Option<String>,
    pub disabled: bool,
    pub rows: Option<usize>,
}

#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct TabsSpec {
    pub tabs: Vec<String>,
    pub active_index: Option<usize>,
}

#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct TreeSpec {
    pub items: Vec<TreeItemSpec>,
}

#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct TreeItemSpec {
    pub label: String,
    pub expanded: bool,
    pub children: Vec<TreeItemSpec>,
}

#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
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
pub struct ListSpec {
    pub items: Vec<String>,
    pub selected_index: Option<usize>,
}

#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct MenuSpec {
    pub items: Vec<MenuItemSpec>,
}

#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct MenuItemSpec {
    pub label: String,
    pub action: Option<String>,
    pub disabled: bool,
    pub shortcut: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct ModalSpec {
    pub title: Option<String>,
    pub close_on_escape: bool,
    pub close_on_outside_click: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct PopoverSpec {
    pub content_label: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct TooltipSpec {
    pub text: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
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
pub struct ImageSpec {
    pub src: Option<String>,
    pub alt: Option<String>,
    pub fit: ImageFit,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ImageFit {
    #[default]
    Cover,
    Contain,
    Fill,
    None,
}

#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct SwitchSpec {
    pub label: Option<String>,
    pub disabled: bool,
    pub checked: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct SliderSpec {
    pub min: f32,
    pub max: f32,
    pub step: Option<f32>,
    pub value: f32,
    pub disabled: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct ProgressBarSpec {
    pub value: f32,
    pub max: f32,
    pub indeterminate: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct SpinnerSpec {
    pub label: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct BadgeSpec {
    pub text: String,
    pub variant: BadgeVariant,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
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
pub struct AvatarSpec {
    pub src: Option<String>,
    pub initials: Option<String>,
    pub size: AvatarSize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AvatarSize {
    #[default]
    Md,
    Sm,
    Lg,
    Xl,
}

#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct LinkSpec {
    pub href: Option<String>,
    pub label: Option<String>,
    pub disabled: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct AlertSpec {
    pub title: Option<String>,
    pub variant: AlertVariant,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AlertVariant {
    #[default]
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct CardSpec {
    pub title: Option<String>,
}
