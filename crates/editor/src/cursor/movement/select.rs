//! Selection-extended movement: Shift+Arrow и т.д.
//!
//! Каждый метод начинает выделение (если его нет) перед движением.

use crate::cursor::types::Cursor;

impl Cursor {
    /// Влево с выделением.
    pub fn move_left_select(&mut self, content: &str, line_starts: &[usize]) {
        self.begin_selection();
        self.move_left(content, line_starts);
    }

    /// Вправо с выделением.
    pub fn move_right_select(&mut self, content: &str, line_starts: &[usize]) {
        self.begin_selection();
        self.move_right(content, line_starts);
    }

    /// В начало строки с выделением.
    pub fn move_home_select(&mut self, line_starts: &[usize]) {
        self.begin_selection();
        self.move_home(line_starts);
    }

    /// В конец строки с выделением.
    pub fn move_end_select(&mut self, content: &str, line_starts: &[usize]) {
        self.begin_selection();
        self.move_end(content, line_starts);
    }

    /// На слово влево с выделением.
    pub fn move_word_left_select(&mut self, content: &str, line_starts: &[usize]) {
        self.begin_selection();
        self.move_word_left(content, line_starts);
    }

    /// На слово вправо с выделением.
    pub fn move_word_right_select(&mut self, content: &str, line_starts: &[usize]) {
        self.begin_selection();
        self.move_word_right(content, line_starts);
    }

    /// На строку вверх с выделением.
    pub fn move_up_select(&mut self, content: &str, line_starts: &[usize]) {
        self.begin_selection();
        self.move_up(content, line_starts);
    }

    /// На строку вниз с выделением.
    pub fn move_down_select(&mut self, content: &str, line_starts: &[usize]) {
        self.begin_selection();
        self.move_down(content, line_starts);
    }
}
