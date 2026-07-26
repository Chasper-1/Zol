//! Вспомогательные функции для пропуска маркеров при движении курсора.
//!
//! В Live-режиме маркеры физически отсутствуют в отображаемом тексте,
//! поэтому курсор, двигаясь по исходному тексту, может «застрять» на
//! байте маркера. Эти функции перебрасывают его на ближайший байт
//! содержимого (не маркера).

use crate::cache::MarkupCache;

/// Если `raw` находится внутри маркера какого-либо сегмента,
/// возвращает первый байт содержимого этого сегмента.
/// Иначе возвращает `raw` без изменений.
pub fn snap_forward(raw: usize, line_cache: &MarkupCache) -> usize {
    for seg in &line_cache.segments {
        let seg_start = seg.raw_start;
        let seg_content_start = seg_start + seg.left_marker_len;
        let seg_end = seg.raw_end;
        let seg_content_end = seg_end.saturating_sub(seg.right_marker_len);

        // Если курсор в левом маркере → перескочить на начало контента
        if raw >= seg_start && raw < seg_content_start {
            return seg_content_start;
        }
        // Если курсор в правом маркере → перескочить за конец маркера (конец сегмента)
        if raw >= seg_content_end && raw < seg_end {
            return seg_end;
        }
    }
    raw
}

/// Если `raw` находится внутри маркера, возвращает позицию сразу за
/// концом правого маркера (т.е. на начале следующего за сегментом текста).
pub fn snap_backward(raw: usize, line_cache: &MarkupCache, line_start: usize) -> usize {
    for seg in &line_cache.segments {
        let seg_start = seg.raw_start;
        let seg_content_start = seg_start + seg.left_marker_len;
        let seg_end = seg.raw_end;

        if raw >= seg_start && raw < seg_content_start {
            // В левом маркере → на начало контента ИЛИ на начало сегмента
            return seg_content_start;
        }
        if raw >= seg_content_start && raw < seg_end {
            let seg_content_end = seg_end.saturating_sub(seg.right_marker_len);
            if raw >= seg_content_end {
                // В правом маркере → за конец сегмента
                return seg_end;
            }
        }
    }
    raw
}

/// Как `snap_forward`, но также учитывает заголовки `#N#` (их нет в кеше).
/// `line` — текст строки, `line_start` — её байтовый оффсет в документе.
pub fn snap_forward_line(
    raw: usize,
    line: &str,
    line_cache: &MarkupCache,
    line_start: usize,
) -> usize {
    // Сначала проверяем заголовок #N#
    if let Some(rest) = line.strip_prefix('#') {
        if let Some(level_end) = rest.find('#') {
            if level_end > 0 {
                if rest[..level_end].parse::<u32>().is_ok() {
                    let marker_end = level_end + 2; // длина #N#
                    let head_start = line_start;
                    let head_end = line_start + marker_end;
                    if raw >= head_start && raw < head_end {
                        // Курсор на маркере заголовка → на контент
                        return head_end;
                    }
                }
            }
        }
    }
    // Затем проверяем сегменты из кеша
    snap_forward(raw, line_cache)
}

/// Проверить, находится ли `raw` внутри какого-либо маркера.
pub fn is_on_marker(raw: usize, line_cache: &MarkupCache) -> bool {
    for seg in &line_cache.segments {
        let seg_start = seg.raw_start;
        let seg_content_start = seg_start + seg.left_marker_len;
        let seg_end = seg.raw_end;
        let seg_content_end = seg_end.saturating_sub(seg.right_marker_len);

        if (raw >= seg_start && raw < seg_content_start)
            || (raw >= seg_content_end && raw < seg_end)
        {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markup::segment::Segment;

    fn make_cache(segments: Vec<Segment>) -> MarkupCache {
        MarkupCache { segments }
    }

    fn seg(raw_start: usize, raw_end: usize, left: usize, right: usize) -> Segment {
        Segment {
            text: String::new(),
            style: 0,
            left_marker_len: left,
            right_marker_len: right,
            raw_start,
            raw_end,
        }
    }

    #[test]
    fn snap_from_left_marker() {
        // "**bold**" — левый маркер 2 байта, raw_start=2, raw_end=8
        let cache = make_cache(vec![seg(2, 8, 2, 2)]);
        // Курсор на байте 0 ("*") → должен перескочить на 2 ("b")
        assert_eq!(snap_forward(0, &cache), 2);
        // Курсор на байте 1 ("*") → должен перескочить на 2
        assert_eq!(snap_forward(1, &cache), 2);
    }

    #[test]
    fn snap_from_right_marker() {
        let cache = make_cache(vec![seg(2, 8, 2, 2)]);
        // "**bold**" — правый маркер на байтах 6-7 ("**"), seg_end=8
        // Курсор на байте 6 → должен перескочить на 8 (конец сегмента)
        assert_eq!(snap_forward(6, &cache), 8);
        assert_eq!(snap_forward(7, &cache), 8);
    }

    #[test]
    fn snap_on_content_stays() {
        let cache = make_cache(vec![seg(2, 8, 2, 2)]);
        // "b" (байт 2), "o" (3), "l" (4), "d" (5) — контент, без изменений
        assert_eq!(snap_forward(2, &cache), 2);
        assert_eq!(snap_forward(3, &cache), 3);
        assert_eq!(snap_forward(4, &cache), 4);
        assert_eq!(snap_forward(5, &cache), 5);
    }

    #[test]
    fn is_on_marker_true() {
        let cache = make_cache(vec![seg(2, 8, 2, 2)]);
        assert!(is_on_marker(0, &cache));
        assert!(is_on_marker(1, &cache));
        assert!(is_on_marker(6, &cache));
        assert!(is_on_marker(7, &cache));
    }

    #[test]
    fn is_on_marker_false() {
        let cache = make_cache(vec![seg(2, 8, 2, 2)]);
        assert!(!is_on_marker(2, &cache));
        assert!(!is_on_marker(5, &cache));
        assert!(!is_on_marker(8, &cache));
    }
}
