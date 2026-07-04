use crate::editor::theme::EditorTheme;

pub struct EditorState {
    pub theme: EditorTheme,
    pub content: String, // Теперь храним весь текст единой строкой, egui это любит
}

impl EditorState {
    pub fn new(theme: EditorTheme, text: String) -> Self {
        Self {
            theme,
            content: text,
        }
    }

    pub fn get_theme(&self) -> &EditorTheme {
        &self.theme
    }
}
