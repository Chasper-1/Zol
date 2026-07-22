use super::line_bounds::line_bounds;

/// Текст строки по индексу (0-based).
pub fn line_text(content: &str, line: usize) -> Option<&str> {
    line_bounds(content, line).map(|b| {
        unsafe { content.get_unchecked(b.start..b.end) }
    })
}

/// Байтовое смещение начала строки.
pub fn line_start_byte(content: &str, line: usize) -> usize {
    line_bounds(content, line).map(|b| b.start).unwrap_or(0)
}

/// Байтовое смещение конца строки.
pub fn line_end_byte(content: &str, line: usize) -> usize {
    line_bounds(content, line)
        .map(|b| b.end)
        .unwrap_or_else(|| content.len())
}
