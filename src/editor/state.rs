use crate::editor::theme::EditorTheme;

pub struct EditorState {
    pub theme: EditorTheme,
}

impl EditorState {
    pub fn new(theme: EditorTheme) -> Self {
        Self { theme }
    }

    // Исправляем ошибку: добавляем отсутствующий метод
    pub fn get_theme(&self) -> &EditorTheme {
        &self.theme
    }
}