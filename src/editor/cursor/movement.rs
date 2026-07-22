use crate::editor::cursor::grapheme::{clamp_to_char_boundary, next_grapheme_boundary, prev_grapheme_boundary};
use crate::editor::cursor::word::{prev_word_start, next_word_start};
use crate::editor::cursor::types::Cursor;
use crate::editor::utils;

impl Cursor {
    /// Установить `raw` с проверкой границ.
    pub fn set_raw(&mut self, content: &str, new_raw: usize) {
        self.raw = clamp_to_char_boundary(content, new_raw);
        self.line = utils::line_of_byte(content, self.raw);
        self.force_blink();
    }

    /// На один grapheme-кластер влево.
    pub fn move_left(&mut self, content: &str) {
        if self.raw == 0 { return; }
        self.raw = prev_grapheme_boundary(content, self.raw).unwrap_or(0);
        self.line = utils::line_of_byte(content, self.raw);
        self.force_blink();
    }

    /// На один grapheme-кластер вправо.
    pub fn move_right(&mut self, content: &str) {
        if self.raw >= content.len() { return; }
        self.raw = next_grapheme_boundary(content, self.raw).unwrap_or(content.len());
        self.line = utils::line_of_byte(content, self.raw);
        self.force_blink();
    }

    /// В начало текущей строки.
    pub fn move_home(&mut self, content: &str) {
        self.raw = utils::line_start_byte(content, self.line);
        self.col_visual = 0.0;
        self.force_blink();
    }

    /// В конец текущей строки.
    pub fn move_end(&mut self, content: &str) {
        self.raw = utils::line_end_byte(content, self.line);
        self.col_visual = f32::MAX;
        self.force_blink();
    }

    /// На слово влево.
    pub fn move_word_left(&mut self, content: &str) {
        self.raw = prev_word_start(content, self.raw);
        self.line = utils::line_of_byte(content, self.raw);
        self.reset_col_visual();
        self.force_blink();
    }

    /// На слово вправо.
    pub fn move_word_right(&mut self, content: &str) {
        self.raw = next_word_start(content, self.raw);
        self.line = utils::line_of_byte(content, self.raw);
        self.reset_col_visual();
        self.force_blink();
    }

    /// На строку вверх, сохраняя пиксельную X-позицию.
    pub fn move_up(&mut self, content: &str) {
        if self.line == 0 {
            self.move_home(content);
            return;
        }
        let col_x = self.col_visual;
        let prev_line = self.line - 1;
        let prev_text = utils::line_text(content, prev_line).unwrap_or("");
        let target_char = if col_x.is_infinite() {
            prev_text.chars().count()
        } else {
            let char_count = prev_text.chars().count();
            let approx = (col_x / 10.0).round() as usize;
            approx.min(char_count)
        };

        let byte_offset = prev_text
            .char_indices()
            .nth(target_char)
            .map(|(b, _)| b)
            .unwrap_or(prev_text.len());

        let start = utils::line_start_byte(content, prev_line);
        self.raw = (start + byte_offset).min(content.len());
        self.line = prev_line;
        self.col_visual = col_x;
        self.force_blink();
    }

    /// На строку вниз, сохраняя пиксельную X-позицию.
    pub fn move_down(&mut self, content: &str) {
        let total = utils::count_lines(content);
        let next_line = self.line + 1;
        if next_line >= total {
            self.move_end(content);
            return;
        }

        let col_x = self.col_visual;
        let next_text = utils::line_text(content, next_line).unwrap_or("");
        let target_char = if col_x.is_infinite() {
            next_text.chars().count()
        } else {
            let char_count = next_text.chars().count();
            let approx = (col_x / 10.0).round() as usize;
            approx.min(char_count)
        };

        let byte_offset = next_text
            .char_indices()
            .nth(target_char)
            .map(|(b, _)| b)
            .unwrap_or(next_text.len());

        let start = utils::line_start_byte(content, next_line);
        self.raw = (start + byte_offset).min(content.len());
        self.line = next_line;
        self.col_visual = col_x;
        self.force_blink();
    }
}
