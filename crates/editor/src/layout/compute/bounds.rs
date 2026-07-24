use crate::utils;

/// Границы строки в байтах для позиционирования курсора.
pub fn cursor_line_bounds(content: &str, line_starts: &[usize], line: usize) -> (usize, usize) {
    utils::line_bounds(content, line_starts, line)
        .map(|b| (b.start, b.end))
        .unwrap_or((0, 0))
}
