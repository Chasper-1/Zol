use super::cache::MarkupCache;
use super::markers::{marker_style, MARKERS};
use super::segment::{Segment, SegmentStyle};
use super::document_cache::DocumentCache;

pub fn parse_line(line: &str) -> MarkupCache {
    let mut cache = MarkupCache::default();

    let mut remaining = line;

    let mut raw_cursor = 0;
    let mut visible_cursor = 0;
    // TODO(perf):
    // Сейчас используются несколько `find()` для поиска ближайшего маркера.
    // Для текущих объёмов текста этого достаточно, но в будущем стоит заменить
    // на один линейный проход по байтам/символам без повторных поисков.
    // Это уберёт лишние проходы по строке и уменьшит количество сравнений.

    while !remaining.is_empty() {
        let mut found: Option<(usize, &str, usize)> = None;

        // Ищем самый левый маркер
        for &m in &MARKERS {
            if let Some(pos) = remaining.find(m) {
                // Проверяем, что это не часть более длинного маркера
                if m == "*" && remaining[pos..].starts_with("**") {
                    continue;
                }

                let rest = &remaining[pos + m.len()..];

                if let Some(end) = rest.find(m) {
                    let end_pos = pos + m.len() + end;

                    if found.map_or(true, |(p, _, _)| pos < p) {
                        found = Some((pos, m, end_pos));
                    }
                }
            }
        }

        if let Some((start, marker, end)) = found {
            if start > 0 {
                let text = &remaining[..start];

                cache.segments.push(Segment {
                    text: text.to_string(),
                    style: SegmentStyle::Plain,

                    left_marker_len: 0,
                    right_marker_len: 0,

                    raw_start: raw_cursor,
                    raw_end: raw_cursor + text.len(),

                    visible_start: visible_cursor,
                    visible_end: visible_cursor + text.len(),
                });

                raw_cursor += text.len();
                visible_cursor += text.len();
            }

            let content = &remaining[start + marker.len()..end];
            let style = marker_style(marker);

            let raw_len = marker.len() * 2 + content.len();

            cache.segments.push(Segment {
                text: content.to_string(),
                style,

                left_marker_len: marker.len(),
                right_marker_len: marker.len(),

                raw_start: raw_cursor,
                raw_end: raw_cursor + raw_len,

                visible_start: visible_cursor,
                visible_end: visible_cursor + content.len(),
            });

            raw_cursor += raw_len;
            visible_cursor += content.len();

            remaining = &remaining[end + marker.len()..];
        } else {
            cache.segments.push(Segment {
                text: remaining.to_string(),
                style: SegmentStyle::Plain,

                left_marker_len: 0,
                right_marker_len: 0,

                raw_start: raw_cursor,
                raw_end: raw_cursor + remaining.len(),

                visible_start: visible_cursor,
                visible_end: visible_cursor + remaining.len(),
            });

            break;
        }
    }

    cache
}

pub fn parse_document(text: &str) -> DocumentCache {
    let mut document = DocumentCache::default();

    for line in text.lines() {
        document.lines.push(parse_line(line));
    }

    document
}