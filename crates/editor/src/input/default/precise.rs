use super::fast::FastInput;
use super::r#trait::InputModel;

#[derive(Debug)]
pub struct PreciseInput;

/// Делегирует базовые move/delete FastInput,
/// а word-операции использует precise (с учётом is_word_char).
impl InputModel for PreciseInput {
    // ─── Move (delegate basic to FastInput) ────────────────

    fn move_left(&self, content: &str, raw: usize) -> usize {
        FastInput.move_left(content, raw)
    }

    fn move_right(&self, content: &str, raw: usize) -> usize {
        FastInput.move_right(content, raw)
    }

    fn move_home(&self, line_starts: &[usize], line: usize) -> usize {
        FastInput.move_home(line_starts, line)
    }

    fn move_end(&self, content: &str, line_starts: &[usize], line: usize) -> usize {
        FastInput.move_end(content, line_starts, line)
    }

    fn move_up(
        &self,
        content: &str,
        line_starts: &[usize],
        raw: usize,
        line: usize,
        col_visual: f64,
    ) -> (usize, f64) {
        FastInput.move_up(content, line_starts, raw, line, col_visual)
    }

    fn move_down(
        &self,
        content: &str,
        line_starts: &[usize],
        raw: usize,
        line: usize,
        col_visual: f64,
    ) -> (usize, f64) {
        FastInput.move_down(content, line_starts, raw, line, col_visual)
    }

    fn word_left(&self, content: &str, raw: usize) -> usize {
        precise_word_left(content, raw)
    }

    fn word_right(&self, content: &str, raw: usize) -> usize {
        precise_word_right(content, raw)
    }

    // ─── Delete (delegate basic to FastInput) ──────────────

    fn delete_char_before(&self, content: &str, raw: usize) -> usize {
        FastInput.delete_char_before(content, raw)
    }

    fn delete_char_after(&self, content: &str, raw: usize) -> usize {
        FastInput.delete_char_after(content, raw)
    }

    fn delete_word_before(&self, content: &str, raw: usize) -> usize {
        precise_word_left(content, raw)
    }

    fn delete_word_after(&self, content: &str, raw: usize) -> usize {
        precise_word_right(content, raw)
    }

    fn delete_line(
        &self,
        content: &str,
        line_starts: &[usize],
        line: usize,
    ) -> Option<(usize, usize)> {
        FastInput.delete_line(content, line_starts, line)
    }

    fn delete_to_line_end(
        &self,
        content: &str,
        line_starts: &[usize],
        raw: usize,
        line: usize,
    ) -> Option<usize> {
        FastInput.delete_to_line_end(content, line_starts, raw, line)
    }
}

// ─── Word boundary helpers (precise) ────────────────────────

fn precise_word_left(content: &str, raw: usize) -> usize {
    let from = raw.min(content.len());
    if from == 0 || content.is_empty() {
        return 0;
    }
    let mut pos = from;
    for (i, ch) in content[..pos].char_indices().rev() {
        if ch.is_whitespace() {
            pos = i;
        } else {
            break;
        }
    }
    if pos == 0 {
        return 0;
    }
    let ch = content[..pos].chars().next_back().unwrap();
    let is_word = is_word_char(ch);
    let mut start = pos;
    for (i, ch) in content[..pos].char_indices().rev() {
        if is_word_char(ch) == is_word {
            start = i;
        } else {
            break;
        }
    }
    start
}

fn precise_word_right(content: &str, raw: usize) -> usize {
    let len = content.len();
    let pos = raw.min(len);
    if pos >= len {
        return len;
    }
    let mut p = pos;
    for (i, ch) in content[pos..].char_indices() {
        if ch.is_whitespace() {
            p = pos + i + ch.len_utf8();
        } else {
            break;
        }
    }
    if p >= len {
        return len;
    }
    let ch = content[p..].chars().next().unwrap();
    let is_word = is_word_char(ch);
    for (i, ch) in content[p..].char_indices() {
        if is_word_char(ch) != is_word || ch.is_whitespace() {
            return p + i;
        }
    }
    len
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── precise word_left ────────────────────────────

    #[test]
    fn precise_left_mid_word() {
        assert_eq!(precise_word_left("hello world", 4), 0);
    }
    #[test]
    fn precise_left_at_word_end() {
        assert_eq!(precise_word_left("hello world", 5), 0);
    }
    #[test]
    fn precise_left_at_word_start() {
        assert_eq!(precise_word_left("hello world", 6), 0);
    }
    #[test]
    fn precise_left_between_words() {
        assert_eq!(precise_word_left("hello world", 5), 0);
    }

    // ─── precise word_right ───────────────────────────

    #[test]
    fn precise_right_mid_word() {
        assert_eq!(precise_word_right("hello world", 2), 5);
    }
    #[test]
    fn precise_right_at_word_end() {
        assert_eq!(precise_word_right("hello world", 5), 11);
    }
    #[test]
    fn precise_right_at_word_start() {
        assert_eq!(precise_word_right("hello world", 6), 11);
    }
    #[test]
    fn precise_right_mid_second_word() {
        assert_eq!(precise_word_right("hello world", 8), 11);
    }
    #[test]
    fn precise_right_at_end() {
        assert_eq!(precise_word_right("hello", 5), 5);
    }
    #[test]
    fn precise_right_empty() {
        assert_eq!(precise_word_right("", 0), 0);
    }
    #[test]
    fn precise_left_punct() {
        assert_eq!(precise_word_left("hello,world", 7), 6);
    }
    #[test]
    fn precise_right_punct() {
        assert_eq!(precise_word_right("hello,world", 5), 6);
    }
}
