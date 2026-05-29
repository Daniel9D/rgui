#[derive(Clone, Debug, PartialEq)]
pub enum UiCommand {
    Click {
        key: Option<String>,
        action: Option<String>,
    },
    SetBool {
        key: String,
        value: bool,
    },
    SetText {
        key: String,
        value: String,
    },
    OpenOverlay {
        key: String,
    },
    CloseOverlay {
        key: String,
    },
    Focus {
        key: String,
    },
    Blur {
        key: String,
    },
    DragStart {
        key: Option<String>,
        payload: Option<String>,
    },
    DragMove {
        key: Option<String>,
        position: crate::core::Point,
    },
    DragEnd {
        key: Option<String>,
        position: crate::core::Point,
    },
}

impl UiCommand {
    pub fn action(&self) -> Option<&str> {
        match self {
            UiCommand::Click {
                action: Some(action),
                ..
            } => Some(action.as_str()),
            _ => None,
        }
    }

    pub fn kind(&self) -> &'static str {
        match self {
            UiCommand::Click { .. } => "Click",
            UiCommand::SetBool { .. } => "SetBool",
            UiCommand::SetText { .. } => "SetText",
            UiCommand::OpenOverlay { .. } => "OpenOverlay",
            UiCommand::CloseOverlay { .. } => "CloseOverlay",
            UiCommand::Focus { .. } => "Focus",
            UiCommand::Blur { .. } => "Blur",
            UiCommand::DragStart { .. } => "DragStart",
            UiCommand::DragMove { .. } => "DragMove",
            UiCommand::DragEnd { .. } => "DragEnd",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CommandQueue {
    commands: Vec<UiCommand>,
}

impl CommandQueue {
    pub fn push(&mut self, cmd: UiCommand) {
        self.commands.push(cmd);
    }

    pub fn drain(&mut self) -> Vec<UiCommand> {
        std::mem::take(&mut self.commands)
    }

    pub fn count(&self) -> usize {
        self.commands.len()
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    pub fn commands(&self) -> &[UiCommand] {
        &self.commands
    }
}
