use super::line_bounds::line_bounds;

/// Текст строки по индексу (0-based).
pub fn line_text<'a>(content: &'a str, line_starts: &[usize], line: usize) -> Option<&'a str> {
    line_bounds(content, line_starts, line).map(|b| {
        unsafe { content.get_unchecked(b.start..b.end) }
    })
}

/// Байтовое смещение начала строки.
pub fn line_start_byte(_content: &str, line_starts: &[usize], line: usize) -> usize {
    line_starts.get(line).copied().unwrap_or(0)
}

/// Байтовое смещение конца строки.
pub fn line_end_byte(content: &str, line_starts: &[usize], line: usize) -> usize {
    line_starts
        .get(line + 1)
        .map(|&next| next.saturating_sub(1))
        .unwrap_or(content.len())
}
