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
        // Bug fix RT-5: if a selection is active, delete the
        // selection range first (standard text-editor behavior).
        // Without this, Backspace with a multi-char selection
        // only deletes one character before the cursor.
        if self.delete_selection() {
            return;
        }
        if self.cursor > 0 {
            self.text.remove(self.cursor - 1);
            self.cursor -= 1;
            self.selection = TextSelection::caret(TextPosition::new(self.cursor));
        }
    }

    pub fn delete_after(&mut self) {
        // Bug fix RT-5: same as above — delete the selection
        // range first.
        if self.delete_selection() {
            return;
        }
        if self.cursor < self.text.len() {
            self.text.remove(self.cursor);
            self.selection = TextSelection::caret(TextPosition::new(self.cursor));
        }
    }

    /// Delete the active selection (the text between the
    /// selection's anchor and head). Returns `true` if a
    /// non-empty selection was deleted, `false` if there was
    /// no selection to delete.
    ///
    /// Bug fix RT-5: previously `delete_before` / `delete_after`
    /// only deleted a single character even when a multi-char
    /// selection was active. Standard text-editor behavior is to
    /// delete the selection first; only when no selection is
    /// active should the cursor-based delete fire.
    pub fn delete_selection(&mut self) -> bool {
        let range = self.selection.range();
        if range.start >= range.end {
            return false;
        }
        self.text.replace_range(range.start..range.end, "");
        self.cursor = range.start;
        self.selection = TextSelection::caret(TextPosition::new(range.start));
        true
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

#[cfg(test)]
mod tests {
    use super::*;

    // Bug fix RT-5: `delete_before` and `delete_after` must
    // honor the active selection, not just the cursor.
    #[test]
    fn delete_before_removes_active_selection() {
        let mut state = InputState::default();
        state.text = "hello world".to_string();
        // Select "world" (bytes 6..11)
        state.cursor = 11;
        state.selection = crate::core::TextSelection {
            anchor: crate::core::TextPosition::new(6),
            head: crate::core::TextPosition::new(11),
        };
        state.delete_before();
        assert_eq!(state.text, "hello ");
        assert_eq!(state.cursor, 6);
    }

    #[test]
    fn delete_after_removes_active_selection() {
        let mut state = InputState::default();
        state.text = "hello world".to_string();
        // Select "hello" (bytes 0..5)
        state.cursor = 0;
        state.selection = crate::core::TextSelection {
            anchor: crate::core::TextPosition::new(0),
            head: crate::core::TextPosition::new(5),
        };
        state.delete_after();
        assert_eq!(state.text, " world");
        assert_eq!(state.cursor, 0);
    }

    #[test]
    fn delete_before_falls_back_to_single_char_when_no_selection() {
        let mut state = InputState::default();
        state.text = "abc".to_string();
        state.cursor = 3;
        state.delete_before();
        assert_eq!(state.text, "ab");
        assert_eq!(state.cursor, 2);
    }

    #[test]
    fn delete_selection_returns_false_on_empty_selection() {
        let mut state = InputState::default();
        state.text = "abc".to_string();
        state.cursor = 1;
        state.selection =
            crate::core::TextSelection::caret(crate::core::TextPosition::new(1));
        assert!(!state.delete_selection());
        assert_eq!(state.text, "abc");
    }
}

