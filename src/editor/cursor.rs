use crate::editor::utils::line_utils;
use std::time::{Duration, Instant};
use unicode_segmentation::GraphemeCursor;

/// Позиция курсора в тексте.
///
/// `raw` всегда указывает на валидную **grapheme**-границу (не байт внутри
/// multi-byte char и не середину combining-последовательности).
/// `line` — кешированный номер строки, обновляется после каждой мутации.
#[derive(Debug)]
pub struct Cursor {
    /// Байтовый оффсет от начала текста.
    raw: usize,
    /// Строка, в которой находится `raw`.
    line: usize,
    /// Горизонтальная позиция для move_up/down (в пикселях).
    col_visual: f32,
    /// Время последнего изменения видимости курсора.
    last_blink: Instant,
}

impl Cursor {
    /// Создать курсор в начале текста.
    pub fn new() -> Self {
        Self {
            raw: 0,
            line: 0,
            col_visual: 0.0,
            last_blink: Instant::now(),
        }
    }

    // ── Геттеры ─────────────────────────────────────────────

    /// Байтовый оффсет от начала текста.
    pub fn raw(&self) -> usize {
        self.raw
    }

    /// Строка, в которой находится курсор.
    pub fn line(&self) -> usize {
        self.line
    }

    /// Сохранённая X-позиция для move_up/down.
    pub fn col_visual(&self) -> f32 {
        self.col_visual
    }

    // ── Безопасные мутации ───────────────────────────────────

    /// Установить `raw` с проверкой границ.
    /// Автоматически обновляет `line` и `force_blink()`.
    pub fn set_raw(&mut self, content: &str, new_raw: usize) {
        self.raw = clamp_to_char_boundary(content, new_raw);
        self.line = line_utils::line_of_byte(content, self.raw);
        self.force_blink();
    }

    /// Установить `line` напрямую (для move_up/down).
    pub fn set_line(&mut self, line: usize) {
        self.line = line;
        self.force_blink();
    }

    pub fn set_col_visual(&mut self, x: f32) {
        self.col_visual = x;
    }

    pub fn reset_col_visual(&mut self) {
        self.col_visual = 0.0;
    }

    // ── Навигация (GraphemeCursor, O(cluster), без O(n)) ───

    /// На один grapheme-кластер влево.
    pub fn move_left(&mut self, content: &str) {
        if self.raw == 0 {
            return;
        }
        let prev = prev_grapheme_boundary(content, self.raw).unwrap_or(0);
        self.raw = prev;
        self.line = line_utils::line_of_byte(content, self.raw);
        self.force_blink();
    }

    /// На один grapheme-кластер вправо.
    pub fn move_right(&mut self, content: &str) {
        if self.raw >= content.len() {
            return;
        }
        self.raw = next_grapheme_boundary(content, self.raw).unwrap_or(content.len());
        self.line = line_utils::line_of_byte(content, self.raw);
        self.force_blink();
    }

    /// В начало текущей строки.
    pub fn move_home(&mut self, content: &str) {
        self.raw = line_utils::line_start_byte(content, self.line);
        self.col_visual = 0.0;
        self.force_blink();
    }

    /// В конец текущей строки.
    pub fn move_end(&mut self, content: &str) {
        self.raw = line_utils::line_end_byte(content, self.line);
        self.col_visual = f32::MAX;
        self.force_blink();
    }

    /// На слово влево (по кластерам, char-safe).
    pub fn move_word_left(&mut self, content: &str) {
        self.raw = prev_word_start(content, self.raw);
        self.line = line_utils::line_of_byte(content, self.raw);
        self.reset_col_visual();
        self.force_blink();
    }

    /// На слово вправо (по кластерам, char-safe).
    pub fn move_word_right(&mut self, content: &str) {
        self.raw = next_word_start(content, self.raw);
        self.line = line_utils::line_of_byte(content, self.raw);
        self.reset_col_visual();
        self.force_blink();
    }

    // ── Мигание ─────────────────────────────────────────────

    /// Видим ли курсор сейчас (фазовая мигалка).
    ///
    /// 530ms видим, 530ms скрыт, повтор. `force_blink()` сбрасывает в начало
    /// видимой фазы.
    pub fn should_blink(&self) -> bool {
        let elapsed = Instant::now().duration_since(self.last_blink);
        let period = 1060; // полный цикл в ms
        let phase = elapsed.as_millis() % period;
        phase < 530
    }

    /// Сбросить таймер мигания (курсор видим после действий).
    pub fn force_blink(&mut self) {
        self.last_blink = Instant::now();
    }
}

// ── Графемные / словесные границы ───────────────────────────

/// Нормализовать позицию до char-границы (не режет multi-byte).
fn clamp_to_char_boundary(content: &str, pos: usize) -> usize {
    if content.is_empty() {
        return 0;
    }
    let pos = pos.min(content.len());
    if content.is_char_boundary(pos) {
        return pos;
    }
    // Ищем предыдущую char boundary без slicing (pos может быть внутри multi-byte)
    let mut prev = 0;
    for (i, _) in content.char_indices() {
        if i > pos {
            break;
        }
        prev = i;
    }
    prev
}

/// Начало предыдущего слова (char-safe, is_whitespace).
fn prev_word_start(content: &str, from: usize) -> usize {
    let from = from.min(content.len());
    if from == 0 || content.is_empty() {
        return 0;
    }

    let mut pos = from;

    // 1. Пропустить пробелы назад (весь Unicode)
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

    // 2. Пропустить непробелы назад (текущее слово)
    let mut start = pos;
    for (i, ch) in content[..pos].char_indices().rev() {
        if !ch.is_whitespace() {
            start = i;
        } else {
            break;
        }
    }

    // Если не сдвинулись — ищем предыдущее слово
    if start == from || start == pos {
        // пропускаем текущее слово
        let mut p = from;
        for (i, ch) in content[..p].char_indices().rev() {
            if !ch.is_whitespace() {
                p = i;
            } else {
                break;
            }
        }
        // пропускаем пробелы
        let mut after_space = p;
        for (i, ch) in content[..p].char_indices().rev() {
            if ch.is_whitespace() {
                after_space = i;
            } else {
                break;
            }
        }
        // начало предыдущего слова
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

/// Начало следующего слова (char-safe, is_whitespace).
fn next_word_start(content: &str, from: usize) -> usize {
    let len = content.len();
    let mut pos = from.min(len);
    if pos >= len {
        return len;
    }

    // 1. Если на непробельном — пропускаем слово
    if let Some(ch) = content[pos..].chars().next() {
        if !ch.is_whitespace() {
            for (i, c) in content[pos..].char_indices() {
                if c.is_whitespace() {
                    pos += i;
                    break;
                }
            }
        }
    }

    // 2. Пропускаем пробелы к началу следующего слова
    for (i, c) in content[pos..].char_indices() {
        if !c.is_whitespace() {
            pos += i;
            return pos;
        }
    }

    len
}

// ── Публичный хелпер для delete_before/after ────────────────

/// Найти предыдущую grapheme-границу (для внешних модулей).
pub fn prev_grapheme_boundary(content: &str, raw: usize) -> Option<usize> {
    if raw == 0 {
        return None;
    }
    let mut gc = GraphemeCursor::new(raw, content.len(), true);
    gc.prev_boundary(content, 0).ok()?
}

/// Найти следующую grapheme-границу (для внешних модулей).
pub fn next_grapheme_boundary(content: &str, raw: usize) -> Option<usize> {
    if raw >= content.len() {
        return None;
    }
    let mut gc = GraphemeCursor::new(raw, content.len(), true);
    gc.next_boundary(content, 0).ok()?
}

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------
    // helpers
    // ------------------------------------------------------------------

    fn cursor_at(raw: usize, line: usize, col_visual: f32) -> Cursor {
        Cursor {
            raw,
            line,
            col_visual,
            last_blink: Instant::now(),
        }
    }

    // ------------------------------------------------------------------
    // new
    // ------------------------------------------------------------------

    #[test]
    fn new_initializes_at_zero() {
        let c = Cursor::new();
        assert_eq!(c.raw(), 0);
        assert_eq!(c.line(), 0);
        assert_eq!(c.col_visual(), 0.0);
    }

    // ------------------------------------------------------------------
    // set_raw
    // ------------------------------------------------------------------

    #[test]
    fn set_raw_moves_to_valid_byte() {
        let mut c = Cursor::new();
        c.set_raw("hello\nworld", 6);
        assert_eq!(c.raw(), 6);
        assert_eq!(c.line(), 1);
    }

    #[test]
    fn set_raw_clamps_to_content_len() {
        let mut c = Cursor::new();
        c.set_raw("abc", 100);
        assert_eq!(c.raw(), 3);
    }

    #[test]
    fn set_raw_clamps_to_char_boundary() {
        // "привет" — кириллица 2 байта на символ
        let mut c = Cursor::new();
        c.set_raw("привет", 3); // байт 3 = внутри 'р' (байты 2-3)
        assert_eq!(c.raw(), 2); // должен откатиться к началу 'р'
    }

    #[test]
    fn set_raw_line_updates() {
        let mut c = Cursor::new();
        c.set_raw("a\nb\nc", 2);
        assert_eq!(c.line(), 1); // b
        c.set_raw("a\nb\nc", 4);
        assert_eq!(c.line(), 2); // c
    }

    // ------------------------------------------------------------------
    // move_left / move_right
    // ------------------------------------------------------------------

    #[test]
    fn move_left_at_start_stays() {
        let mut c = Cursor::new();
        c.move_left("");
        assert_eq!(c.raw(), 0);
        c.move_left("abc");
        assert_eq!(c.raw(), 0);
    }

    #[test]
    fn move_right_at_end_stays() {
        let mut c = cursor_at(3, 0, 0.0);
        c.move_right("abc");
        assert_eq!(c.raw(), 3);
    }

    #[test]
    fn move_left_right_ascii() {
        let mut c = cursor_at(2, 0, 0.0);
        c.move_left("abcd");
        assert_eq!(c.raw(), 1);
        c.move_right("abcd");
        assert_eq!(c.raw(), 2);
    }

    #[test]
    fn move_right_updates_line() {
        let mut c = cursor_at(3, 0, 0.0);
        c.move_right("abc\ndef");
        assert_eq!(c.raw(), 4); // \n
        assert_eq!(c.line(), 1);
        c.move_right("abc\ndef");
        assert_eq!(c.raw(), 5); // d
        assert_eq!(c.line(), 1);
    }

    #[test]
    fn move_left_multibyte() {
        // "a👨‍👩‍👧‍👦b" — сложный emoji ZWJ sequence (11 байт)
        let text = "a👨‍👩‍👧‍👦b";
        let mut c = cursor_at(text.len(), 0, 0.0);
        c.move_left(text);
        // должно перескочить через весь кластер
        assert!(c.raw() < text.len());
        assert_eq!(&text[c.raw()..], "b"); // должен быть перед 'b'
    }

    #[test]
    fn move_left_line_updates() {
        // "ab\ncd" → a=0,b=1,\n=2,c=3,d=4
        let mut c = cursor_at(4, 1, 0.0); // 'd'
        c.move_left("ab\ncd"); // → 'c'
        assert_eq!(c.raw(), 3);
        assert_eq!(c.line(), 1); // 'c' still on line 1
        c.move_left("ab\ncd"); // → '\n'
        assert_eq!(c.raw(), 2);
        assert_eq!(c.line(), 0); // '\n' on line 0
    }

    // ------------------------------------------------------------------
    // move_home / move_end
    // ------------------------------------------------------------------

    #[test]
    fn move_home_to_line_start() {
        let mut c = cursor_at(6, 1, 42.0);
        c.move_home("abc\ndef");
        assert_eq!(c.raw(), 4);
        assert_eq!(c.col_visual(), 0.0);
    }

    #[test]
    fn move_end_to_line_end() {
        let mut c = cursor_at(0, 0, 0.0);
        c.move_end("abc\ndef");
        assert_eq!(c.raw(), 3);
        assert_eq!(c.col_visual(), f32::MAX);
    }

    #[test]
    fn move_home_on_first_line() {
        let mut c = cursor_at(2, 0, 10.0);
        c.move_home("hello");
        assert_eq!(c.raw(), 0);
    }

    #[test]
    fn move_end_last_line() {
        let mut c = cursor_at(0, 0, 0.0);
        c.move_end("hello");
        assert_eq!(c.raw(), 5);
    }

    // ------------------------------------------------------------------
    // move_word_left / move_word_right
    // ------------------------------------------------------------------

    #[test]
    fn move_word_left_from_middle() {
        let mut c = cursor_at(6, 0, 0.0);
        c.move_word_left("abc def ghi");
        assert_eq!(c.raw(), 4); // начало def
    }

    #[test]
    fn move_word_left_from_start() {
        let mut c = Cursor::new();
        c.move_word_left("abc");
        assert_eq!(c.raw(), 0);
    }

    #[test]
    fn move_word_right_from_middle() {
        let mut c = cursor_at(0, 0, 0.0);
        c.move_word_right("abc def ghi");
        assert_eq!(c.raw(), 4); // начало def
    }

    #[test]
    fn move_word_right_from_end() {
        let mut c = cursor_at(11, 0, 0.0);
        c.move_word_right("abc def ghi");
        assert_eq!(c.raw(), 11);
    }

    #[test]
    fn move_word_left_skips_whitespace() {
        // "abc   def" → a=0,b=1,c=2,' '=3,' '=4,' '=5,d=6,e=7,f=8
        let mut c = cursor_at(8, 0, 0.0);
        c.move_word_left("abc   def");
        assert_eq!(c.raw(), 6); // начало 'def'
    }

    #[test]
    fn move_word_right_skips_whitespace() {
        // "abc   def" → a=0,b=1,c=2,' '=3,' '=4,' '=5,d=6,e=7,f=8
        let mut c = cursor_at(3, 0, 0.0);
        c.move_word_right("abc   def");
        assert_eq!(c.raw(), 6); // начало 'def'
    }

    #[test]
    fn move_word_left_empty_content() {
        let mut c = Cursor::new();
        c.move_word_left("");
        assert_eq!(c.raw(), 0);
    }

    #[test]
    fn move_word_right_empty_content() {
        let mut c = Cursor::new();
        c.move_word_right("");
        assert_eq!(c.raw(), 0);
    }

    // ------------------------------------------------------------------
    // col_visual
    // ------------------------------------------------------------------

    #[test]
    fn col_visual_set_reset() {
        let mut c = Cursor::new();
        c.set_col_visual(123.0);
        assert_eq!(c.col_visual(), 123.0);
        c.reset_col_visual();
        assert_eq!(c.col_visual(), 0.0);
    }

    // ------------------------------------------------------------------
    // blink (фазовая: 530ms visible, 530ms hidden, repeat)
    // ------------------------------------------------------------------

    #[test]
    fn should_blink_initially_visible() {
        let c = Cursor::new();
        // last_blink = Instant::now(), фаза 0 → visible
        assert!(c.should_blink(), "cursor should be visible right after creation");
    }

    #[test]
    fn force_blink_resets_to_visible() {
        let mut c = Cursor::new();
        // после паузы курсор должен быть visible (phase 0)
        assert!(c.should_blink());
        c.force_blink();
        assert!(c.should_blink(), "force_blink should reset to visible phase");
    }

    // ------------------------------------------------------------------
    // set_line
    // ------------------------------------------------------------------

    #[test]
    fn set_line_works() {
        let mut c = Cursor::new();
        c.set_line(5);
        assert_eq!(c.line(), 5);
    }

    // ------------------------------------------------------------------
    // edge cases
    // ------------------------------------------------------------------

    #[test]
    fn move_on_empty_content_does_nothing() {
        let mut c = Cursor::new();
        c.move_left("");
        c.move_right("");
        c.move_home("");
        c.move_end("");
        c.move_word_left("");
        c.move_word_right("");
        // не паникует, raw = 0
        assert_eq!(c.raw(), 0);
    }

    #[test]
    fn set_raw_on_empty_string() {
        let mut c = Cursor::new();
        c.set_raw("", 0);
        assert_eq!(c.raw(), 0);
        assert_eq!(c.line(), 0);
    }

    #[test]
    fn move_left_then_right_roundtrip() {
        let text = "abcdef";
        let mut c = cursor_at(3, 0, 0.0);
        let original = c.raw();
        c.move_left(text);
        c.move_left(text);
        c.move_right(text);
        c.move_right(text);
        assert_eq!(c.raw(), original);
    }

    // ------------------------------------------------------------------
    // grapheme boundary helpers
    // ------------------------------------------------------------------

    #[test]
    fn prev_grapheme_boundary_at_start() {
        assert_eq!(prev_grapheme_boundary("abc", 0), None);
    }

    #[test]
    fn prev_grapheme_boundary_ascii() {
        assert_eq!(prev_grapheme_boundary("abc", 2), Some(1));
    }

    #[test]
    fn next_grapheme_boundary_at_end() {
        assert_eq!(next_grapheme_boundary("abc", 3), None);
    }

    #[test]
    fn next_grapheme_boundary_ascii() {
        assert_eq!(next_grapheme_boundary("abc", 1), Some(2));
    }

    #[test]
    fn clamp_to_char_boundary_already_valid() {
        assert_eq!(clamp_to_char_boundary("hello", 3), 3);
    }

    #[test]
    fn clamp_to_char_boundary_mid_multi_byte() {
        assert_eq!(clamp_to_char_boundary("héllo", 1), 1); // é is 2 bytes, pos 1 is inside it
    }

    #[test]
    fn clamp_to_char_boundary_past_end() {
        assert_eq!(clamp_to_char_boundary("hi", 100), 2);
    }

    #[test]
    fn clamp_to_char_boundary_empty() {
        assert_eq!(clamp_to_char_boundary("", 0), 0);
    }

    #[test]
    fn set_raw_clamps_past_end() {
        let mut c = Cursor::new();
        c.set_raw("abc", 10);
        assert_eq!(c.raw(), 3);
    }
}
