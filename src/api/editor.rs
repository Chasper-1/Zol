use crate::editor::state::{EditMode, EditorState};

// Публичный API для будущих плагинов и интеграций
#[allow(dead_code)]
pub fn set_mode(state: &mut EditorState, mode: EditMode) {
    state.mode = mode;
}

#[allow(dead_code)]
pub fn get_mode(state: &EditorState) -> EditMode {
    state.mode
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::state::EditorState;
    use crate::editor::theme::EditorTheme;

    fn make_state() -> EditorState {
        EditorState::new(EditorTheme::default(), String::new())
    }

    #[test]
    fn set_mode_changes_state() {
        let mut state = make_state();
        assert_eq!(state.mode, EditMode::LivePreview); // default
        set_mode(&mut state, EditMode::Preview);
        assert_eq!(state.mode, EditMode::Preview);
    }

    #[test]
    fn get_mode_returns_current() {
        let mut state = make_state();
        set_mode(&mut state, EditMode::Source);
        assert_eq!(get_mode(&state), EditMode::Source);
    }
}
