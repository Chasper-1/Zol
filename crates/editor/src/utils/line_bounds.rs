use super::bounds::LineBounds;

/// Границы строки по индексу (0-based).
///
/// Использует `line_starts` для O(1) доступа (вместо O(n) сканирования).
/// `content` нужен только для определения конца последней строки.
pub fn line_bounds(content: &str, line_starts: &[usize], line: usize) -> Option<LineBounds> {
    // Для пустого контента с пустым line_starts — особая обработка
    if line_starts.is_empty() {
        return if line == 0 {
            Some(LineBounds { start: 0, end: content.len() })
        } else {
            None
        };
    }
    let start = *line_starts.get(line)?;
    let end = line_starts
        .get(line + 1)
        .map(|&next| next.saturating_sub(1)) // позиция \n
        .unwrap_or(content.len());
    Some(LineBounds { start, end })
}
