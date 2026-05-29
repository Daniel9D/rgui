use crate::{NodeId, Rect};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Window,
    Group,
    Text,
    Button,
    TextInput,
    Checkbox,
    Radio,
    List,
    ListItem,
    Table,
    Row,
    Cell,
    Dialog,
    Menu,
    MenuItem,
    Tooltip,
    ScrollArea,
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

#[derive(Clone, Debug, PartialEq)]
pub enum SemanticValue {
    Text(String),
    Number(f64),
    Bool(bool),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SemanticStates {
    pub focused: bool,
    pub disabled: bool,
    pub checked: bool,
    pub expanded: Option<bool>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SemanticAction {
    Press,
    Focus,
    SetValue,
    ScrollForward,
    ScrollBackward,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyboardNav {
    None,
    TabStop,
    ArrowGroup,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SemanticNode {
    pub node: NodeId,
    pub key: Option<String>,
    pub role: Role,
    pub label: Option<String>,
    pub description: Option<String>,
    pub value: Option<SemanticValue>,
    pub states: SemanticStates,
    pub actions: Vec<SemanticAction>,
    pub focusable: bool,
    pub focus_order: Option<i32>,
    pub keyboard_navigation: KeyboardNav,
    pub bounds: Rect,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SemanticTree {
    nodes: Vec<SemanticNode>,
}

impl SemanticTree {
    pub fn push(&mut self, node: SemanticNode) {
        self.nodes.push(node);
    }

    pub fn nodes(&self) -> &[SemanticNode] {
        &self.nodes
    }

    pub fn by_key(&self, key: &str) -> Option<&SemanticNode> {
        self.nodes
            .iter()
            .find(|node| node.key.as_deref() == Some(key))
    }
}

pub trait AccessibilityBackend {
    fn update(&mut self, tree: &SemanticTree);
}
