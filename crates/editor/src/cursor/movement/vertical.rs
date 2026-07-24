use crate::cursor::types::Cursor;

impl Cursor {
    /// На строку вверх, сохраняя пиксельную X-позицию.
    pub fn move_up(&mut self, content: &str, line_starts: &[usize]) {
        if self.line == 0 {
            self.move_home(content, line_starts);
            return;
        }
        let col_x = self.col_visual;
        let prev_line = self.line - 1;
        let prev_text = line_text_impl(content, line_starts, prev_line);
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

        let start = line_starts.get(prev_line).copied().unwrap_or(0);
        self.raw = (start + byte_offset).min(content.len());
        self.line = prev_line;
        self.col_visual = col_x;
        self.force_blink();
    }

    /// На строку вниз, сохраняя пиксельную X-позицию.
    pub fn move_down(&mut self, content: &str, line_starts: &[usize]) {
        let total = line_starts.len();
        let next_line = self.line + 1;
        if next_line >= total {
            self.move_end(content, line_starts);
            return;
        }

        let col_x = self.col_visual;
        let next_text = line_text_impl(content, line_starts, next_line);
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

        let start = line_starts.get(next_line).copied().unwrap_or(0);
        self.raw = (start + byte_offset).min(content.len());
        self.line = next_line;
        self.col_visual = col_x;
        self.force_blink();
    }
}

/// O(1) получение текста строки (без \n).
fn line_text_impl<'a>(content: &'a str, line_starts: &[usize], line: usize) -> &'a str {
    let start = match line_starts.get(line) {
        Some(&s) => s,
        None => return "",
    };
    let end = match line_starts.get(line + 1) {
        Some(&next) => next.saturating_sub(1),
        None => content.len(),
    };
    if start > end || start >= content.len() {
        return "";
    }
    &content[start..end]
}
