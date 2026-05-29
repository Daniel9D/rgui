use crate::core::{KeyEvent, UiEvent};

pub fn normalize_key(key: impl Into<String>, modifiers: u32, repeat: bool) -> UiEvent {
    UiEvent::KeyDown(KeyEvent {
        key: key.into(),
        modifiers,
        repeat,
    })
}
