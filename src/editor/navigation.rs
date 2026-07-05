use crate::editor::cache::MarkupCache;
use crate::editor::markup::Segment;

/// Перевод видимой позиции → raw Markdown позицию
pub fn visible_to_raw(cache: &MarkupCache, visible: usize) -> usize {
    for seg in &cache.segments {
        if visible >= seg.visible_start && visible <= seg.visible_end {
            return seg.raw_start + seg.left_marker_len + (visible - seg.visible_start);
        }
    }

    visible
}

/// Перевод raw Markdown → видимая позиция
pub fn raw_to_visible(cache: &MarkupCache, raw: usize) -> usize {
    for seg in &cache.segments {
        let content_start = seg.raw_start + seg.left_marker_len;
        let content_end = seg.raw_end.saturating_sub(seg.right_marker_len);

        if raw < content_start {
            return seg.visible_start;
        }

        if raw > content_end {
            return seg.visible_end;
        }

        return seg.visible_start + (raw - content_start);
    }

    raw
}

/// Сегмент по raw позиции (используется только для навигации по границам)
pub fn segment_at_raw(cache: &MarkupCache, raw: usize) -> Option<&Segment> {
    cache
        .segments
        .iter()
        .find(|seg| raw >= seg.raw_start && raw <= seg.raw_end)
}

/// === НАВИГАЦИЯ ===

/// Движение курсора вправо (без попадания внутрь маркеров)
pub fn move_right(cache: &MarkupCache, raw: usize) -> usize {
    for seg in &cache.segments {
        let left_end = seg.raw_start + seg.left_marker_len;
        let right_start = seg.raw_end.saturating_sub(seg.right_marker_len);

        // Если курсор внутри открывающего маркера → прыжок в конец контента
        if raw >= seg.raw_start && raw < left_end {
            return right_start + seg.right_marker_len;
        }
    }

    raw + 1
}

/// Движение курсора влево (без попадания внутрь маркеров)
pub fn move_left(cache: &MarkupCache, raw: usize) -> usize {
    for seg in &cache.segments {
        let right_start = seg.raw_end.saturating_sub(seg.right_marker_len);

        // Если курсор внутри закрывающего маркера → прыжок в начало сегмента
        if raw > right_start && raw <= seg.raw_end {
            return seg.raw_start;
        }
    }

    raw.saturating_sub(1)
}
