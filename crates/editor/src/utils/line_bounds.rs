use super::bounds::LineBounds;

/// Границы строки по индексу (0-based).
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
        if c == '\n' {
            current += 1;
            start = i + 1;
        }
    }

    if current == line {
        Some(LineBounds { start, end: content.len() })
    } else {
        None
    }
}
