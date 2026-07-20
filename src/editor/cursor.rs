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
        let mut gc = GraphemeCursor::new(self.raw, content.len(), true);
        if let Ok(Some(prev)) = gc.prev_boundary(content, 0) {
            self.raw = prev;
        } else {
            self.raw = 0;
        }
        self.line = line_utils::line_of_byte(content, self.raw);
        self.force_blink();
    }

    /// На один grapheme-кластер вправо.
    pub fn move_right(&mut self, content: &str) {
        if self.raw >= content.len() {
            return;
        }
        let mut gc = GraphemeCursor::new(self.raw, content.len(), true);
        match gc.next_boundary(content, 0) {
            Ok(Some(next)) => self.raw = next,
            _ => self.raw = content.len(),
        }
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

    /// Пора ли мигнуть (>530 мс прошло)?
    pub fn should_blink(&self) -> bool {
        Instant::now().duration_since(self.last_blink) > Duration::from_millis(530)
    }

    /// Сбросить таймер мигания (курсор видим после действий).
    pub fn force_blink(&mut self) {
        self.last_blink = Instant::now();
    }
}

// ── Графемные / словесные границы ───────────────────────────

/// Нормализовать позицию до char-границы (не режет multi-byte).
fn clamp_to_char_boundary(content: &str, pos: usize) -> usize {
    let pos = pos.min(content.len());
    if content.is_char_boundary(pos) {
        pos
    } else {
        content[..pos]
            .char_indices()
            .last()
            .map(|(i, _)| i)
            .unwrap_or(0)
    }
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
