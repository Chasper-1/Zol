use unicode_segmentation::GraphemeCursor;

/// Нормализовать позицию до char-границы (не режет multi-byte).
pub fn clamp_to_char_boundary(content: &str, pos: usize) -> usize {
    if content.is_empty() {
        return 0;
    }
    let pos = pos.min(content.len());
    if content.is_char_boundary(pos) {
        return pos;
    }
    let mut prev = 0;
    for (i, _) in content.char_indices() {
        if i > pos {
            break;
        }
        prev = i;
    }
    prev
}

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
