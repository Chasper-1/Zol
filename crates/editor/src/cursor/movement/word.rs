use crate::cursor::types::Cursor;
use crate::cursor::word::{prev_word_start, next_word_start};

impl Cursor {
    /// На слово влево.
    pub fn move_word_left(&mut self, content: &str, line_starts: &[usize]) {
        self.raw = prev_word_start(content, self.raw);
        self.line = line_of_byte_impl(line_starts, self.raw);
        self.reset_col_visual();
        self.force_blink();
    }

    /// На слово вправо.
    pub fn move_word_right(&mut self, content: &str, line_starts: &[usize]) {
        self.raw = next_word_start(content, self.raw);
        self.line = line_of_byte_impl(line_starts, self.raw);
        self.reset_col_visual();
        self.force_blink();
    }
}

/// O(log n) бинарный поиск строки по байтовой позиции.
fn line_of_byte_impl(line_starts: &[usize], byte: usize) -> usize {
    if line_starts.is_empty() || byte == 0 {
        return 0;
    }
    match line_starts.binary_search(&byte) {
        Ok(i) => i,
        Err(0) => 0,
        Err(i) => i - 1,
    }
}
