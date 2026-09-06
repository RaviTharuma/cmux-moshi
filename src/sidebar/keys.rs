//! Key bindings for the optional workspace picker.
//!
//! `Esc` clears the query and never exits: cmux owns the prefix-escape chord
//! that leaves sidebar focus. `Ctrl-C` is the clean exit.

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

/// Outcome of a key press.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeyAction {
    /// Exit the process cleanly.
    Quit,
    /// Clear the filter query. Does not exit.
    ClearQuery,
    /// Move the selection by `delta` rows.
    Move(isize),
    /// Activate the selected workspace via cmux-client.
    Activate,
    /// Append a character to the query.
    Type(char),
    /// Delete the last query character.
    Backspace,
    /// Ignore (release events, unused chords).
    Ignore,
}

/// Interprets a crossterm key event.
pub fn interpret_key(key: KeyEvent) -> KeyAction {
    if key.kind != KeyEventKind::Press {
        return KeyAction::Ignore;
    }

    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return KeyAction::Quit;
    }

    match key.code {
        KeyCode::Esc => KeyAction::ClearQuery,
        KeyCode::Enter => KeyAction::Activate,
        KeyCode::Up => KeyAction::Move(-1),
        KeyCode::Down => KeyAction::Move(1),
        KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => KeyAction::Move(-1),
        KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => KeyAction::Move(1),
        KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) => KeyAction::Move(-1),
        KeyCode::Char('j') if key.modifiers.contains(KeyModifiers::CONTROL) => KeyAction::Move(1),
        KeyCode::Backspace => KeyAction::Backspace,
        KeyCode::Delete => KeyAction::ClearQuery,
        KeyCode::Char(ch) if key.modifiers.is_empty() || key.modifiers == KeyModifiers::SHIFT => {
            KeyAction::Type(ch)
        }
        _ => KeyAction::Ignore,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn press(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, modifiers)
    }

    #[test]
    fn esc_clears_query_and_does_not_quit() {
        assert_eq!(
            interpret_key(press(KeyCode::Esc, KeyModifiers::NONE)),
            KeyAction::ClearQuery
        );
    }

    #[test]
    fn ctrl_c_quits() {
        assert_eq!(
            interpret_key(press(KeyCode::Char('c'), KeyModifiers::CONTROL)),
            KeyAction::Quit
        );
    }

    #[test]
    fn enter_activates_and_arrows_move() {
        assert_eq!(
            interpret_key(press(KeyCode::Enter, KeyModifiers::NONE)),
            KeyAction::Activate
        );
        assert_eq!(
            interpret_key(press(KeyCode::Up, KeyModifiers::NONE)),
            KeyAction::Move(-1)
        );
        assert_eq!(
            interpret_key(press(KeyCode::Down, KeyModifiers::NONE)),
            KeyAction::Move(1)
        );
    }

    #[test]
    fn release_events_are_ignored() {
        let mut key = press(KeyCode::Esc, KeyModifiers::NONE);
        key.kind = KeyEventKind::Release;
        assert_eq!(interpret_key(key), KeyAction::Ignore);
    }
}
