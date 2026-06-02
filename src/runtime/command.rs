/// User-intent commands emitted by the dispatch layer.
///
/// # Toggle vs SetBool
///
/// `Toggle` is the right command for boolean toggles (e.g. a Checkbox
/// pointer-up). The runtime is responsible for reading the current state
/// and converting `Toggle` into a `SetBool` with the correct next value.
///
/// `SetBool` should only be used when the new value is known up front
/// (e.g. radio button selection, form initialization). The dispatch layer
/// must never emit `SetBool { value: true }` for a checkbox — that hardcodes
/// the next value and breaks controlled checkboxes. See bug fix 1.4.
#[derive(Clone, Debug, PartialEq)]
pub enum UiCommand {
    Click {
        key: Option<String>,
        action: Option<String>,
    },
    /// Set a boolean state to an absolute value. Use only when the new
    /// value is known up front.
    SetBool {
        key: String,
        value: bool,
    },
    /// Toggle a boolean state. The runtime is responsible for reading the
    /// current state and emitting a corresponding `SetBool` with the next
    /// value. This is the correct command for checkbox pointer-up.
    Toggle {
        key: String,
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
            UiCommand::Toggle { .. } => "Toggle",
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
