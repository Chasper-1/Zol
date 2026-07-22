use crate::utils;

/// Границы строки в байтах для позиционирования курсора.
pub fn cursor_line_bounds(content: &str, line: usize) -> (usize, usize) {
    utils::line_bounds(content, line)
        .map(|b| (b.start, b.end))
        .unwrap_or((0, 0))
}
