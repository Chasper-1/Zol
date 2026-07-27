use super::r#trait::InputModel;
use crate::cursor::grapheme::next_grapheme_boundary;
use crate::cursor::grapheme::prev_grapheme_boundary;

#[derive(Debug)]
pub struct FastInput;

impl InputModel for FastInput {
    // ─── Move ──────────────────────────────────────────────

    fn move_left(&self, content: &str, raw: usize) -> usize {
        if raw == 0 || content.is_empty() {
            return 0;
        }
        prev_grapheme_boundary(content, raw).unwrap_or(0)
    }

    fn move_right(&self, content: &str, raw: usize) -> usize {
        if raw >= content.len() || content.is_empty() {
            return raw;
        }
        next_grapheme_boundary(content, raw).unwrap_or(content.len())
    }

    fn move_home(&self, line_starts: &[usize], line: usize) -> usize {
        line_starts.get(line).copied().unwrap_or(0)
    }

    fn move_end(&self, content: &str, line_starts: &[usize], line: usize) -> usize {
        line_starts
            .get(line + 1)
            .map(|&next| next.saturating_sub(1))
            .unwrap_or(content.len())
    }

    fn move_up(
        &self,
        content: &str,
        line_starts: &[usize],
        raw: usize,
        line: usize,
        col_visual: f64,
    ) -> (usize, f64) {
        let raw = raw.min(content.len());
        if line == 0 {
            let home = self.move_home(line_starts, line);
            return if raw == home { (raw, col_visual) } else { (home, 0.0) };
        }
        let prev_line = line - 1;
        let prev_text = line_text(content, line_starts, prev_line);
        let target_char = if col_visual.is_infinite() {
            prev_text.chars().count()
        } else {
            let char_count = prev_text.chars().count();
            let approx = (col_visual / 10.0).round() as usize;
            approx.min(char_count)
        };
        let byte_offset = prev_text
            .char_indices()
            .nth(target_char)
            .map(|(b, _)| b)
            .unwrap_or(prev_text.len());
        let start = line_starts.get(prev_line).copied().unwrap_or(0);
        let new_raw = (start + byte_offset).min(content.len());
        (new_raw, col_visual)
    }

    fn move_down(
        &self,
        content: &str,
        line_starts: &[usize],
        raw: usize,
        line: usize,
        col_visual: f64,
    ) -> (usize, f64) {
        let raw = raw.min(content.len());
        let total = line_starts.len();
        let next_line = line + 1;
        if next_line >= total {
            let end = self.move_end(content, line_starts, line);
            return if raw == end { (raw, col_visual) } else { (end, f64::INFINITY) };
        }
        let next_text = line_text(content, line_starts, next_line);
        let target_char = if col_visual.is_infinite() {
            next_text.chars().count()
        } else {
            let char_count = next_text.chars().count();
            let approx = (col_visual / 10.0).round() as usize;
            approx.min(char_count)
        };
        let byte_offset = next_text
            .char_indices()
            .nth(target_char)
            .map(|(b, _)| b)
            .unwrap_or(next_text.len());
        let start = line_starts.get(next_line).copied().unwrap_or(0);
        let new_raw = (start + byte_offset).min(content.len());
        (new_raw, col_visual)
    }

    fn word_left(&self, content: &str, raw: usize) -> usize {
        prev_word_start(content, raw)
    }

    fn word_right(&self, content: &str, raw: usize) -> usize {
        next_word_start(content, raw)
    }

    // ─── Delete ────────────────────────────────────────────

    fn delete_char_before(&self, content: &str, raw: usize) -> usize {
        if raw == 0 || content.is_empty() {
            return raw;
        }
        prev_grapheme_boundary(content, raw).unwrap_or(0)
    }

    fn delete_char_after(&self, content: &str, raw: usize) -> usize {
        if raw >= content.len() || content.is_empty() {
            return raw;
        }
        next_grapheme_boundary(content, raw).unwrap_or(content.len())
    }

    fn delete_word_before(&self, content: &str, raw: usize) -> usize {
        prev_word_start(content, raw)
    }

    fn delete_word_after(&self, content: &str, raw: usize) -> usize {
        next_word_start(content, raw)
    }

    fn delete_line(
        &self,
        content: &str,
        line_starts: &[usize],
        line: usize,
    ) -> Option<(usize, usize)> {
        let start = line_starts.get(line).copied()?;
        let end = match line_starts.get(line + 1) {
            Some(&next) => next,
            None => content.len(),
        };
        Some((start, end))
    }

    fn delete_to_line_end(
        &self,
        content: &str,
        line_starts: &[usize],
        raw: usize,
        line: usize,
    ) -> Option<usize> {
        let line_end = line_starts
            .get(line + 1)
            .map(|&next| next.saturating_sub(1))
            .unwrap_or(content.len());
        if line_end > raw { Some(line_end) } else { None }
    }
}

// ─── Helper: текст строки (без \n) ──────────────────────────

fn line_text<'a>(content: &'a str, line_starts: &[usize], line: usize) -> &'a str {
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

// ─── Word boundary helpers ──────────────────────────────────

fn prev_word_start(content: &str, from: usize) -> usize {
    let from = from.min(content.len());
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
    let mut start = pos;
    for (i, ch) in content[..pos].char_indices().rev() {
        if !ch.is_whitespace() {
            start = i;
        } else {
            break;
        }
    }
    if start == from || start == pos {
        let mut p = from;
        for (i, ch) in content[..p].char_indices().rev() {
            if !ch.is_whitespace() {
                p = i;
            } else {
                break;
            }
        }
        let mut after_space = p;
        for (i, ch) in content[..p].char_indices().rev() {
            if ch.is_whitespace() {
                after_space = i;
            } else {
                break;
            }
        }
        let mut word_start = after_space;
        for (i, ch) in content[..after_space].char_indices().rev() {
            if !ch.is_whitespace() {
                word_start = i;
            } else {
                break;
            }
        }
        return word_start;
    }
    start
}

fn next_word_start(content: &str, from: usize) -> usize {
    let len = content.len();
    let mut pos = from.min(len);
    if pos >= len {
        return len;
    }
    if content.as_bytes()[pos] == b'\n' {
        let rest = &content[pos..];
        match rest.char_indices().find(|(_, c)| !c.is_whitespace()) {
            Some((skip, _)) => {
                let word_start = pos + skip;
                match content[word_start..]
                    .char_indices()
                    .find(|(_, c)| c.is_whitespace())
                {
                    Some((end, _)) => return word_start + end,
                    None => return len,
                }
            }
            None => return len,
        }
    }
    if let Some(ch) = content[pos..].chars().next() {
        if !ch.is_whitespace() {
            match content[pos..]
                .char_indices()
                .find(|(_, c)| c.is_whitespace())
            {
                Some((i, _)) => pos += i,
                None => pos = len,
            }
        }
    }
    for (i, c) in content[pos..].char_indices() {
        if c == '\n' {
            break;
        }
        if !c.is_whitespace() {
            pos += i;
            return pos;
        }
    }
    pos
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── word_left ─────────────────────────────────────

    #[test]
    fn word_left_mid_second_word() {
        assert_eq!(prev_word_start("hello world", 6), 0);
    }
    #[test]
    fn word_left_after_first_word() {
        assert_eq!(prev_word_start("hello world", 5), 0);
    }
    #[test]
    fn word_left_mid_first_word() {
        assert_eq!(prev_word_start("hello", 3), 0);
    }
    #[test]
    fn word_left_empty() {
        assert_eq!(prev_word_start("", 0), 0);
    }
    #[test]
    fn word_left_spaced() {
        assert_eq!(prev_word_start("  spaced  ", 10), 2);
    }
    #[test]
    fn word_left_three_words() {
        assert_eq!(prev_word_start("hello world foo", 15), 12);
    }

    // ─── word_right ────────────────────────────────────

    #[test]
    fn word_right_from_start() {
        assert_eq!(next_word_start("hello world", 0), 6);
    }
    #[test]
    fn word_right_single_word() {
        assert_eq!(next_word_start("hello", 0), 5);
    }
    #[test]
    fn word_right_from_second_word() {
        assert_eq!(next_word_start("hello world", 6), 11);
    }
    #[test]
    fn word_right_empty() {
        assert_eq!(next_word_start("", 0), 0);
    }
    #[test]
    fn word_right_multi_spaces() {
        assert_eq!(next_word_start("a   b", 0), 4);
    }
    #[test]
    fn word_right_at_end() {
        assert_eq!(next_word_start("hello", 5), 5);
    }

    // ─── move_left / move_right ────────────────────────

    #[test]
    fn move_left_basic() {
        let f = FastInput;
        assert_eq!(f.move_left("abc", 1), 0);
    }
    #[test]
    fn move_left_at_start() {
        let f = FastInput;
        assert_eq!(f.move_left("abc", 0), 0);
    }
    #[test]
    fn move_left_empty() {
        let f = FastInput;
        assert_eq!(f.move_left("", 0), 0);
    }
    #[test]
    fn move_right_basic() {
        let f = FastInput;
        assert_eq!(f.move_right("abc", 0), 1);
    }
    #[test]
    fn move_right_at_end() {
        let f = FastInput;
        assert_eq!(f.move_right("abc", 3), 3);
    }
    #[test]
    fn move_right_empty() {
        let f = FastInput;
        assert_eq!(f.move_right("", 0), 0);
    }

    // ─── move_home / move_end ──────────────────────────

    #[test]
    fn move_home_mid_line() {
        let f = FastInput;
        let ls = vec![0, 10, 20];
        assert_eq!(f.move_home(&ls, 1), 10);
    }
    #[test]
    fn move_end_mid_line() {
        let f = FastInput;
        let content = "hello\nworld\nfoo";
        let ls = vec![0, 6, 12];
        assert_eq!(f.move_end(content, &ls, 0), 5); // line 0 → "hello\0" = 4? no: "hello" len=5, next start=6, 6-1=5
    }
    #[test]
    fn move_end_last_line() {
        let f = FastInput;
        let content = "hello\nworld";
        let ls = vec![0, 6];
        assert_eq!(f.move_end(content, &ls, 1), 11);
    }

    // ─── move_up / move_down ───────────────────────────

    #[test]
    fn move_up_first_line_goes_home() {
        let f = FastInput;
        let content = "hello";
        let ls = vec![0];
        let (raw, col) = f.move_up(content, &ls, 3, 0, 0.0);
        assert_eq!(raw, 0);
        assert_eq!(col, 0.0);
    }
    #[test]
    fn move_down_last_line_goes_end() {
        let f = FastInput;
        let content = "hello";
        let ls = vec![0];
        let (raw, col) = f.move_down(content, &ls, 0, 0, 0.0);
        assert_eq!(raw, 5);
        assert!(col.is_infinite());
    }
    #[test]
    fn move_up_basic() {
        let f = FastInput;
        let content = "abc\ndefg";
        let ls = vec![0, 4]; // line 0: 0..3, line 1: 4..8
        let (raw, col) = f.move_up(content, &ls, 5, 1, 10.0); // col_visual 10.0 → approx 1 char
        assert_eq!(raw, 1); // col 10 → char 1 → byte 1 in "abc"
        assert_eq!(col, 10.0);
    }
    #[test]
    fn move_down_basic() {
        let f = FastInput;
        let content = "abc\ndefg";
        let ls = vec![0, 4];
        let (raw, col) = f.move_down(content, &ls, 1, 0, 10.0);
        assert_eq!(raw, 5); // col 10 → char 1 → byte 1 in "defg" → 4+1=5
        assert_eq!(col, 10.0);
    }

    // ─── delete_char_before / delete_char_after ────────

    #[test]
    fn delete_char_before_basic() {
        let f = FastInput;
        assert_eq!(f.delete_char_before("abc", 1), 0);
    }
    #[test]
    fn delete_char_before_at_zero() {
        let f = FastInput;
        assert_eq!(f.delete_char_before("abc", 0), 0);
    }
    #[test]
    fn delete_char_after_basic() {
        let f = FastInput;
        assert_eq!(f.delete_char_after("abc", 0), 1);
    }
    #[test]
    fn delete_char_after_at_end() {
        let f = FastInput;
        assert_eq!(f.delete_char_after("abc", 3), 3);
    }

    // ─── delete_line ───────────────────────────────────

    #[test]
    fn delete_line_middle() {
        let f = FastInput;
        let content = "a\nb\nc";
        let ls = vec![0, 2, 4];
        assert_eq!(f.delete_line(content, &ls, 1), Some((2, 4)));
    }
    #[test]
    fn delete_line_last() {
        let f = FastInput;
        let content = "a\nb\nc";
        let ls = vec![0, 2, 4];
        assert_eq!(f.delete_line(content, &ls, 2), Some((4, 5)));
    }
    #[test]
    fn delete_line_single_line() {
        let f = FastInput;
        let content = "abc";
        let ls = vec![0];
        assert_eq!(f.delete_line(content, &ls, 0), Some((0, 3)));
    }

    // ─── delete_to_line_end ────────────────────────────

    #[test]
    fn delete_to_line_end_mid() {
        let f = FastInput;
        let content = "hello\nworld";
        let ls = vec![0, 6];
        assert_eq!(f.delete_to_line_end(content, &ls, 3, 0), Some(5));
    }
    #[test]
    fn delete_to_line_end_at_end() {
        let f = FastInput;
        let content = "hello\nworld";
        let ls = vec![0, 6];
        assert_eq!(f.delete_to_line_end(content, &ls, 5, 0), None);
    }
}
