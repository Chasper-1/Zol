use crate::editor::theme::EditorTheme;

pub struct EditorState {
    pub theme: EditorTheme,
    pub text: String,
}

impl EditorState {
    pub fn new(theme: EditorTheme, text: String) -> Self {
        Self {
            theme,
            text,
        }
    }

    pub fn get_theme(&self) -> &EditorTheme {
        &self.theme
    }
}