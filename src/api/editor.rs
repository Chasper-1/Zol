use crate::editor::state::{EditMode, EditorState};

pub fn set_mode(state: &mut EditorState, mode: EditMode) {
    state.mode = mode;
}

pub fn get_mode(state: &EditorState) -> EditMode {
    state.mode
}
