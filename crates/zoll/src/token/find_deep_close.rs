use crate::ast::MarkerDef;
use crate::token::helpers::is_whitespace_at;
use crate::token::helpers::is_whitespace_before;

/// Поиск закрывающего маркера с учётом вложенности (для open == close).
pub fn find_deep_close(
    text: &str,
    range: std::ops::Range<usize>,
    marker: &MarkerDef,
) -> Option<usize> {
    let mut depth = 1u32;
    let mut pos = range.start;
    let end = range.end;

    while pos < end {
        let tail = &text[pos..end];

        // Если open совпадает с close И включён трекинг вложенности — увеличиваем глубину
        if marker.track_depth && marker.open == marker.close && tail.starts_with(marker.open) {
            let after = pos + marker.open.len();
            if after <= end && !is_whitespace_at(text, after) {
                depth += 1;
                pos += marker.open.len();
                continue;
            }
        }

        if tail.starts_with(marker.close) {
            if pos > range.start && !is_whitespace_before(text, pos) {
                depth -= 1;
                if depth == 0 {
                    return Some(pos);
                }
                pos += marker.close.len();
                continue;
            }
        }

        let ch = tail.chars().next()?;
        pos += ch.len_utf8();
    }

    None
}
