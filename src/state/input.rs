use crate::core::{TextPosition, TextSelection};
use crate::state::WidgetState;

#[derive(Clone, Debug)]
pub struct InputState {
    pub text: String,
    pub cursor: usize,
    pub selection: TextSelection,
    pub focused: bool,
    pub password_mode: bool,
    pub preedit: Option<crate::core::ImePreedit>,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            text: String::new(),
            cursor: 0,
            selection: TextSelection::caret(TextPosition::new(0)),
            focused: false,
            password_mode: false,
            preedit: None,
        }
    }
}

impl InputState {
    pub fn new(default_value: Option<&str>) -> Self {
        let text = default_value.unwrap_or("").to_string();
        let len = text.len();
        Self {
            cursor: len,
            selection: TextSelection::caret(TextPosition::new(len)),
            text,
            focused: false,
            password_mode: false,
            preedit: None,
        }
    }

    pub fn commit_text(&mut self, value: &str) {
        let range = self.selection.range();
        self.text.replace_range(range.start..range.end, value);
        let caret = range.start + value.len();
        self.cursor = caret;
        self.selection = TextSelection::caret(TextPosition::new(caret));
    }

    pub fn delete_before(&mut self) {
        if self.cursor > 0 {
            self.text.remove(self.cursor - 1);
            self.cursor -= 1;
            self.selection = TextSelection::caret(TextPosition::new(self.cursor));
        }
    }

    pub fn delete_after(&mut self) {
        if self.cursor < self.text.len() {
            self.text.remove(self.cursor);
            self.selection = TextSelection::caret(TextPosition::new(self.cursor));
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.selection = TextSelection::caret(TextPosition::new(self.cursor));
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor < self.text.len() {
            self.cursor += 1;
            self.selection = TextSelection::caret(TextPosition::new(self.cursor));
        }
    }

    pub fn move_cursor_home(&mut self) {
        self.cursor = 0;
        self.selection = TextSelection::caret(TextPosition::new(0));
    }

    pub fn move_cursor_end(&mut self) {
        self.cursor = self.text.len();
        self.selection = TextSelection::caret(TextPosition::new(self.cursor));
    }
}

impl WidgetState for InputState {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
