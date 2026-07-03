use crate::editor::theme::EditorTheme;

pub struct Cursor {
    pub line: usize,
    pub column: usize,
}

pub struct EditorState {
    pub theme: EditorTheme,
    pub lines: Vec<String>,
    pub cursor: Cursor,
}

impl EditorState {
    pub fn new(theme: EditorTheme, text: String) -> Self {
        Self {
            theme,
            lines: text.lines().map(|s| s.to_string()).collect(),
            cursor: Cursor { line: 0, column: 0 },
        }
    }

    pub fn get_theme(&self) -> &EditorTheme {
        &self.theme
    }
}

impl EditorState {
    pub fn move_left(&mut self) { /* логика */ }
    pub fn move_right(&mut self) { /* логика */ }
    pub fn move_up(&mut self) { /* логика */ }
    pub fn move_down(&mut self) { /* логика */ }

    pub fn backspace(&mut self) { /* логика */ }
    pub fn newline(&mut self) { /* логика */ }
    pub fn insert_char(&mut self, _ch: char) {}
}