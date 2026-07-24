use crate::ast::MarkerDef;
use crate::token::helpers::{is_whitespace_at, is_whitespace_before};

/// Поиск закрывающего маркера с учётом вложенности (для open == close).
///
/// Оптимизирован: использует байтовый поиск через `position()` для
/// скачка к следующему возможному вхождению маркера.
pub fn find_deep_close(
    text: &str,
    range: std::ops::Range<usize>,
    marker: &MarkerDef,
) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut depth = 1u32;
    let mut pos = range.start;
    let end = range.end.min(bytes.len());

    // Первый байт закрывающего маркера — для batch-skip
    let close_first = marker.close.as_bytes()[0];
    let close_bytes = marker.close.as_bytes();

    // Для open == close нужно различать open/close по пробелам
    let open_bytes = marker.open.as_bytes();
    let open_close_equal = marker.track_depth && marker.open == marker.close;

    while pos < end {
        // Batch-skip: ищем следующее вхождение первого байта маркера
        // (не байта текста — чтобы не проверять каждый байт)
        let tail = &bytes[pos..end];
        let next = if open_close_equal {
            // При open==close ищем байт, с которого может начинаться маркер
            tail.iter().position(|&b| b == close_first)
        } else {
            // Иначе только первый байт close
            tail.iter().position(|&b| b == close_first)
        };

        match next {
            Some(offset) => pos += offset,
            None => return None,
        }

        // Проверяем: совпадает ли полный маркер
        if pos + close_bytes.len() > end {
            return None;
        }

        let after = pos + close_bytes.len();

        // Для open == close: проверяем open (увеличивает глубину)
        if open_close_equal && bytes[pos..].starts_with(open_bytes) {
            if after <= end && !is_whitespace_at(text, after) {
                depth += 1;
                pos = after;
                continue;
            }
        }

        // Проверяем close (уменьшает глубину)
        if bytes[pos..].starts_with(close_bytes) {
            if pos > range.start && !is_whitespace_before(text, pos) {
                depth -= 1;
                if depth == 0 {
                    return Some(pos);
                }
                pos = after;
                continue;
            }
        }

        // Ложное срабатывание (например, `**` внутри слова без пробелов)
        pos += close_bytes.len();
    }

    None
}
