//! Преобразование AST в DocumentCache для редактора.
//!
//! Это единственный модуль mdplus, который зависит от editor::cache и editor::markup::segment.
//! Всё остальное — чистый Rust без внешних зависимостей.

use super::ast::{MarkupDoc, MarkupNode, MarkupStyle};
use crate::editor::cache::DocumentCache;
use crate::editor::markup::segment::{Segment, StyleFlags};

/// Преобразует AST-документ в DocumentCache для редактора.
pub fn to_document_cache(doc: &MarkupDoc) -> DocumentCache {
    // Сначала собираем все сегменты в плоский список с raw-позициями
    let mut segments = Vec::new();
    build_segments(&doc.children, MarkupStyle::PLAIN, &mut segments, 0);

    // Теперь раскладываем по строкам
    // Строки определяются динамически при раскладке

    // Считаем количество строк
    let mut num_lines = 1;
    for seg in &segments {
        num_lines += seg.text.matches('\n').count();
    }

    let mut doc_cache = DocumentCache {
        lines: vec![Default::default(); num_lines],
    };

    // Раскладываем сегменты по строкам
    for seg in segments {
        if seg.text.contains('\n') {
            let parts: Vec<&str> = seg.text.split('\n').collect();
            let n_parts = parts.len();
            let mut raw_offset = seg.raw_start;

            for (pi, part) in parts.iter().enumerate() {
                let line_idx = find_line_by_offset(raw_offset, &doc_cache);
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
            let line_idx = find_line_by_offset(seg.raw_start, &doc_cache);
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
                // Открывающий маркер
                let combined = combine_style(inherited_style, *style);
                let marker_len = marker_open_len(*style);

                // Рекурсивно обрабатываем детей
                let child_start = raw_offset;
                let child_end = build_segments(children, combined, segments, raw_offset);
                raw_offset = child_end;

                // Помечаем первый и последний сегмент маркерами
                if let Some(first) = segments
                    .iter_mut()
                    .rev()
                    .find(|s| s.raw_end <= child_end && s.raw_start >= child_start)
                {
                    first.left_marker_len += marker_len;
                }
                // Эта логика не совсем точная, нужен более аккуратный подход.
                // Пока что маркеры будут обрабатываться на уровне raw текст.
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

/// Находит номер строки по байтовому смещению.
fn find_line_by_offset(offset: usize, doc: &DocumentCache) -> usize {
    let _byte_count = 0;
    for (i, line) in doc.lines.iter().enumerate() {
        if !line.segments.is_empty() {
            let first = &line.segments[0];
            if first.raw_start <= offset && offset <= first.raw_end {
                return i;
            }
        }
    }
    // Fallback: линейный поиск
    for (i, line) in doc.lines.iter().enumerate() {
        for seg in &line.segments {
            if seg.raw_start <= offset && offset < seg.raw_end {
                return i;
            }
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::markup::segment::STYLE_PLAIN;
    use crate::mdplus::ast::{MarkupDoc, MarkupNode};

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
}
