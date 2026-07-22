//! Преобразование AST в DocumentCache для редактора.
//!
//! Это единственный модуль zoll, который зависит от editor::cache и editor::markup::segment.
//! Всё остальное — чистый Rust без внешних зависимостей.

use super::ast::{MarkupDoc, MarkupNode, MarkupStyle};
use crate::editor::cache::DocumentCache;
use crate::editor::markup::segment::{Segment, StyleFlags};

/// Преобразует AST-документ в DocumentCache для редактора.
pub fn to_document_cache(doc: &MarkupDoc) -> DocumentCache {
    // 1. Собираем все сегменты в плоский список с raw-позициями
    let mut segments = Vec::new();
    build_segments(&doc.children, MarkupStyle::PLAIN, &mut segments, 0);

    // 2. Вычисляем начало каждой строки (line_starts) из raw_start/text сегментов.
    //    line_starts[i] — байтовый offset начала i-й строки в документе.
    let mut line_starts = vec![0usize];
    for seg in &segments {
        let search_offset = seg.raw_start;
        for (i, ch) in seg.text.char_indices() {
            if ch == '\n' {
                line_starts.push(search_offset + i + 1);
            }
        }
    }

    let num_lines = line_starts.len();
    let mut doc_cache = DocumentCache {
        lines: vec![Default::default(); num_lines],
    };

    // 3. Функция: номер строки по байтовому offset в документе.
    //    Использует line_starts для бинарного поиска.
    let line_for_offset = |offset: usize| -> usize {
        match line_starts.binary_search(&offset) {
            Ok(i) => i,
            Err(i) => {
                if i == 0 {
                    0
                } else {
                    i - 1
                }
            }
        }
    };

    // 4. Раскладываем сегменты по строкам
    for seg in segments {
        if seg.text.contains('\n') {
            let parts: Vec<&str> = seg.text.split('\n').collect();
            let n_parts = parts.len();
            let mut raw_offset = seg.raw_start;

            for (pi, part) in parts.iter().enumerate() {
                if part.is_empty() {
                    raw_offset += 1; // +1 for \n
                    continue;
                }
                let line_idx = line_for_offset(raw_offset);
                if line_idx >= doc_cache.lines.len() {
                    break;
                }
                doc_cache.lines[line_idx].segments.push(Segment {
                    text: part.to_string(),
                    style: seg.style,
                    left_marker_len: if pi == 0 { seg.left_marker_len } else { 0 },
                    right_marker_len: if pi == n_parts - 1 {
                        seg.right_marker_len
                    } else {
                        0
                    },
                    raw_start: raw_offset,
                    raw_end: raw_offset + part.len(),
                });
                raw_offset += part.len() + 1; // +1 for \n
            }
        } else {
            let line_idx = line_for_offset(seg.raw_start);
            if line_idx < doc_cache.lines.len() {
                doc_cache.lines[line_idx].segments.push(seg);
            }
        }
    }

    doc_cache
}

/// Рекурсивно обходит AST и собирает сегменты.
fn build_segments(
    nodes: &[MarkupNode],
    inherited_style: MarkupStyle,
    segments: &mut Vec<Segment>,
    mut raw_offset: usize,
) -> usize {
    for node in nodes {
        match node {
            MarkupNode::Text(text) => {
                let combined = combine_style(inherited_style, MarkupStyle::PLAIN);
                segments.push(Segment {
                    text: text.clone(),
                    style: to_style_flags(combined),
                    left_marker_len: 0,
                    right_marker_len: 0,
                    raw_start: raw_offset,
                    raw_end: raw_offset + text.len(),
                });
                raw_offset += text.len();
            }
            MarkupNode::Formatted { style, children } => {
                let combined = combine_style(inherited_style, *style);
                let marker_len = marker_open_len(*style);

                // Пропускаем открывающий маркер
                raw_offset += marker_len;
                let child_start = raw_offset;

                // Рекурсивно обрабатываем детей (они получают корректный raw_offset)
                raw_offset = build_segments(children, combined, segments, raw_offset);

                // Пропускаем закрывающий маркер
                let child_end = raw_offset;
                raw_offset += marker_len;

                // Помечаем первый и последний сегмент маркерами
                if let Some(first) = segments
                    .iter_mut()
                    .rev()
                    .find(|s| s.raw_end <= child_end && s.raw_start >= child_start)
                {
                    first.left_marker_len += marker_len;
                }
            }
        }
    }
    raw_offset
}

/// Комбинирует стили (наследование).
fn combine_style(parent: MarkupStyle, child: MarkupStyle) -> MarkupStyle {
    MarkupStyle(parent.bits() | child.bits())
}

/// Преобразует AST-стиль в StyleFlags редактора.
fn to_style_flags(style: MarkupStyle) -> StyleFlags {
    // Прямое соответствие битов
    style.bits()
}

/// Длина открывающего маркера по стилю.
fn marker_open_len(style: MarkupStyle) -> usize {
    for m in super::ast::MARKERS {
        if m.style == style {
            return m.open.len();
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::markup::segment::STYLE_PLAIN;
    use crate::zoll::ast::{MarkupDoc, MarkupNode};

    #[test]
    fn plain_text_segments() {
        let doc = MarkupDoc {
            children: vec![MarkupNode::Text("hello".to_string())],
        };
        let cache = to_document_cache(&doc);
        assert_eq!(cache.lines.len(), 1);
        assert_eq!(cache.lines[0].segments.len(), 1);
        assert_eq!(cache.lines[0].segments[0].text, "hello");
        assert_eq!(cache.lines[0].segments[0].style, STYLE_PLAIN);
    }

    #[test]
    fn multiline_formatted_correct_line_assignment() {
        // Текст: "a\n**bold**\nc"
        use crate::zoll::ast::MarkupStyle;
        let doc = MarkupDoc {
            children: vec![
                MarkupNode::Text("a\n".to_string()),
                MarkupNode::Formatted {
                    style: MarkupStyle::BOLD,
                    children: vec![MarkupNode::Text("bold".to_string())],
                },
                MarkupNode::Text("\nc".to_string()),
            ],
        };
        let cache = to_document_cache(&doc);
        // Должно быть 3 строки: "a", "bold", "c"
        assert_eq!(cache.lines.len(), 3, "должно быть 3 строки");
        // Строка 0: "a"
        assert_eq!(cache.lines[0].segments.len(), 1);
        assert_eq!(cache.lines[0].segments[0].text, "a");
        // Строка 1: "bold" (стиль BOLD)
        assert_eq!(cache.lines[1].segments.len(), 1);
        assert_eq!(cache.lines[1].segments[0].text, "bold");
        assert_ne!(
            cache.lines[1].segments[0].style & crate::editor::markup::segment::STYLE_BOLD,
            0,
            "строка 1 должна быть BOLD"
        );
        // Строка 2: "c"
        assert_eq!(cache.lines[2].segments.len(), 1);
        assert_eq!(cache.lines[2].segments[0].text, "c");
    }

    #[test]
    fn bold_segment_raw_positions() {
        // Только "**bold**" — один сегмент с маркерами
        use crate::zoll::ast::MarkupStyle;
        let doc = MarkupDoc {
            children: vec![MarkupNode::Formatted {
                style: MarkupStyle::BOLD,
                children: vec![MarkupNode::Text("bold".to_string())],
            }],
        };
        let cache = to_document_cache(&doc);
        assert_eq!(cache.lines.len(), 1);
        assert_eq!(cache.lines[0].segments.len(), 1);
        let seg = &cache.lines[0].segments[0];
        assert_eq!(seg.text, "bold");
        // bold начинается с байта 2 (после "**")
        assert_eq!(seg.raw_start, 2);
        // bold заканчивается на байте 6 (2 + 4 байта)
        assert_eq!(seg.raw_end, 6);
    }
}
