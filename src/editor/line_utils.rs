//! Утилиты для работы со строками текста.
//!
//! Все функции работают с байтовыми индексами и корректно обрабатывают UTF-8.
//! Это единый источник правды для навигации по строкам — никакого дублирования.

/// Результат поиска границ строки.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LineBounds {
    /// Байтовое смещение начала строки (включительно).
    pub start: usize,
    /// Байтовое смещение конца строки (исключительно).
    /// Указывает на символ `\n` или на длину текста.
    pub end: usize,
}

/// Безопасное извлечение подстроки.
/// Если индексы не на границах UTF-8, они корректируются к ближайшей нижней границе.
/// Никогда не паникует.
pub fn safe_slice(content: &str, start: usize, end: usize) -> &str {
    let len = content.len();
    let start = start.min(len);
    let end = end.min(len);
    let start = if content.is_char_boundary(start) {
        start
    } else {
        safe_prev_boundary(content, start)
    };
    let end = if content.is_char_boundary(end) {
        end
    } else {
        safe_prev_boundary(content, end)
    };
    if start >= end {
        return &content[start..start]; // пустой срез
    }
    &content[start..end]
}

#[inline]
fn safe_prev_boundary(content: &str, byte: usize) -> usize {
    let mut b = byte.min(content.len());
    while b > 0 && !content.is_char_boundary(b) {
        b -= 1;
    }
    b
}

/// Количество строк в тексте.
/// Пустой текст считается одной (пустой) строкой.
pub fn count_lines(content: &str) -> usize {
    if content.is_empty() {
        return 1;
    }
    content.bytes().filter(|&b| b == b'\n').count() + 1
}

/// Границы строки по индексу (0-based).
///
/// Возвращает `None`, если строки с таким индексом не существует.
/// При пустом тексте существует только строка 0 (пустая).
pub fn line_bounds(content: &str, line: usize) -> Option<LineBounds> {
    if content.is_empty() {
        return if line == 0 {
            Some(LineBounds { start: 0, end: 0 })
        } else {
            None
        };
    }

    let mut current = 0usize;
    let mut start = 0usize;

    for (i, c) in content.char_indices() {
        if current == line && c == '\n' {
            return Some(LineBounds { start, end: i });
        }
        // Продолжаем — это контент искомой строки, просто идём дальше
        if c == '\n' {
            current += 1;
            start = i + 1;
        }
    }

    // Если мы дошли до конца, current указывает на последнюю строку
    if current == line {
        Some(LineBounds {
            start,
            end: content.len(),
        })
    } else {
        None
    }
}

/// Текст строки по индексу (0-based).
///
/// Возвращает `None`, если строки с таким индексом не существует.
pub fn line_text(content: &str, line: usize) -> Option<&str> {
    line_bounds(content, line).map(|bounds| {
        // SAFETY: bounds заведомо на границах char — мы их получили из char_indices()
        unsafe { content.get_unchecked(bounds.start..bounds.end) }
    })
}

/// Байтовое смещение начала строки (0-based).
///
/// Возвращает `content.len()` для последней строки (её начало — конец предыдущей),
/// `0` для несуществующей строки (legacy-поведение).
pub fn line_start_byte(content: &str, line: usize) -> usize {
    line_bounds(content, line).map(|b| b.start).unwrap_or(0)
}

/// Байтовое смещение конца строки (0-based).
///
/// Возвращает `content.len()` для несуществующей строки (legacy-поведение).
pub fn line_end_byte(content: &str, line: usize) -> usize {
    line_bounds(content, line)
        .map(|b| b.end)
        .unwrap_or_else(|| content.len())
}

/// Определяет номер строки, содержащей указанный байтовый индекс.
///
/// Если `byte >= content.len()`, возвращает номер последней строки.
pub fn line_of_byte(content: &str, byte: usize) -> usize {
    if content.is_empty() || byte == 0 {
        return 0;
    }
    // Приводим к безопасной границе UTF-8, чтобы не было паники при slice
    let byte = byte.min(content.len());
    // Ищем предыдущую границу char, если byte указывает в середину символа
    let safe_byte = if content.is_char_boundary(byte) {
        byte
    } else {
        // Ищем предыдущую границу char
        let mut adjusted = byte;
        while adjusted > 0 && !content.is_char_boundary(adjusted) {
            adjusted -= 1;
        }
        adjusted
    };
    // Считаем \n до указанной позиции
    content[..safe_byte].bytes().filter(|&b| b == b'\n').count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_text() {
        assert_eq!(count_lines(""), 1);
        assert_eq!(line_bounds("", 0), Some(LineBounds { start: 0, end: 0 }));
        assert_eq!(line_bounds("", 1), None);
        assert_eq!(line_text("", 0), Some(""));
        assert_eq!(line_text("", 1), None);
        assert_eq!(line_of_byte("", 0), 0);
    }

    #[test]
    fn single_line() {
        assert_eq!(count_lines("hello"), 1);
        assert_eq!(
            line_bounds("hello", 0),
            Some(LineBounds { start: 0, end: 5 })
        );
        assert_eq!(line_text("hello", 0), Some("hello"));
        assert_eq!(line_bounds("hello", 1), None);
    }

    #[test]
    fn two_lines() {
        assert_eq!(count_lines("abc\ndef"), 2);
        assert_eq!(
            line_bounds("abc\ndef", 0),
            Some(LineBounds { start: 0, end: 3 })
        );
        assert_eq!(
            line_bounds("abc\ndef", 1),
            Some(LineBounds { start: 4, end: 7 })
        );
        assert_eq!(line_text("abc\ndef", 0), Some("abc"));
        assert_eq!(line_text("abc\ndef", 1), Some("def"));
    }

    #[test]
    fn trailing_newline() {
        assert_eq!(count_lines("a\n"), 2);
        assert_eq!(line_text("a\n", 0), Some("a"));
        assert_eq!(line_text("a\n", 1), Some(""));
    }

    #[test]
    fn only_newlines() {
        assert_eq!(count_lines("\n"), 2);
        assert_eq!(line_text("\n", 0), Some(""));
        assert_eq!(line_text("\n", 1), Some(""));
    }

    #[test]
    fn line_of_byte_works() {
        let text = "abc\ndef\nghi";
        assert_eq!(line_of_byte(text, 0), 0);
        assert_eq!(line_of_byte(text, 1), 0);
        assert_eq!(line_of_byte(text, 3), 0);
        assert_eq!(line_of_byte(text, 4), 1);
        assert_eq!(line_of_byte(text, 7), 1);
        assert_eq!(line_of_byte(text, 8), 2);
    }

    #[test]
    fn unicode_text() {
        let text = "Привет\nМир";
        assert_eq!(count_lines(text), 2);
        assert_eq!(line_text(text, 0), Some("Привет"));
        assert_eq!(line_text(text, 1), Some("Мир"));
        // Проверка, что байтовые индексы корректны
        let bounds0 = line_bounds(text, 0).unwrap();
        assert_eq!(&text[bounds0.start..bounds0.end], "Привет");
    }
}
