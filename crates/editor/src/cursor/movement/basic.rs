use crate::cursor::grapheme::{clamp_to_char_boundary, next_grapheme_boundary, prev_grapheme_boundary};
use crate::cursor::types::Cursor;

impl Cursor {
    /// Установить `raw` с проверкой границ.
    pub fn set_raw(&mut self, content: &str, line_starts: &[usize], new_raw: usize) {
        self.raw = clamp_to_char_boundary(content, new_raw);
        self.line = line_of_byte_impl(line_starts, self.raw);
        self.force_blink();
    }

    /// На один grapheme-кластер влево.
    pub fn move_left(&mut self, content: &str, line_starts: &[usize]) {
        if self.raw == 0 { return; }
        self.raw = prev_grapheme_boundary(content, self.raw).unwrap_or(0);
        self.line = line_of_byte_impl(line_starts, self.raw);
        self.force_blink();
    }

    /// На один grapheme-кластер вправо.
    pub fn move_right(&mut self, content: &str, line_starts: &[usize]) {
        if self.raw >= content.len() { return; }
        self.raw = next_grapheme_boundary(content, self.raw).unwrap_or(content.len());
        self.line = line_of_byte_impl(line_starts, self.raw);
        self.force_blink();
    }

    /// В начало текущей строки.
    pub fn move_home(&mut self, _content: &str, line_starts: &[usize]) {
        self.raw = line_starts.get(self.line).copied().unwrap_or(0);
        self.col_visual = 0.0;
        self.force_blink();
    }

    /// В конец текущей строки.
    pub fn move_end(&mut self, content: &str, line_starts: &[usize]) {
        self.raw = line_end_byte_impl(content, line_starts, self.line);
        self.col_visual = f32::MAX;
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

/// O(1) конец строки по индексу.
fn line_end_byte_impl(content: &str, line_starts: &[usize], line: usize) -> usize {
    line_starts
        .get(line + 1)
        .map(|&next| next.saturating_sub(1))
        .unwrap_or(content.len())
}
