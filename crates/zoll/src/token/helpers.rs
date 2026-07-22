use crate::ast::MARKERS;

/// Нативный поиск следующего символа `'\n'` начиная с байта `from`.
pub fn next_newline(text: &str, from: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut i = from;
    while i < bytes.len() {
        if bytes[i] == b'\n' {
            return Some(i);
        }
        i += 1;
    }
    None
}

/// Находит индекс первого маркера, совпадающего с текущей позицией.
pub fn find_any_marker(text: &str, pos: usize) -> Option<usize> {
    let tail = &text[pos..];
    MARKERS.iter().position(|m| tail.starts_with(m.open))
}

/// Проверка, что в позиции стоит пробельный символ (или конец текста).
pub fn is_whitespace_at(text: &str, pos: usize) -> bool {
    text[pos..]
        .chars()
        .next()
        .map_or(true, |c| c.is_ascii_whitespace())
}

/// Проверка, что перед позицией стоит пробельный символ.
pub fn is_whitespace_before(text: &str, pos: usize) -> bool {
    if pos == 0 {
        return true;
    }
    text[..pos]
        .chars()
        .next_back()
        .map_or(true, |c| c.is_ascii_whitespace())
}
