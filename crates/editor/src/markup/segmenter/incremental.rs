//! Преобразование `IncrementalDoc` → `DocumentCache` для редактора.
//!
//! В отличие от `to_doc.rs` (который берёт `MarkupDoc`), эта функция
//! работает напрямую с `IncrementalDoc.line_asts` — построчно.
//! Это позволяет в будущем генерировать сегменты только для видимых строк.

use zoll::ast::LineAST;
use zoll::incremental::IncrementalDoc;

use crate::cache::DocumentCache;
use crate::markup::segment::Segment;
use crate::markup::segmenter::build::build_segments_from_nodes;
use crate::markup::segmenter::helpers::to_style_flags;

/// Преобразовать `IncrementalDoc` в `DocumentCache`.
///
/// Проходит по всем `line_asts` и генерирует сегменты для каждой строки.
pub fn incremental_to_cache(inc: &IncrementalDoc) -> DocumentCache {
    let num_lines = inc.line_starts.len();
    let mut cache = DocumentCache {
        lines: vec![Default::default(); num_lines],
    };

    for i in 0..inc.line_asts.len() {
        let line_text = get_line_text(inc, i);
        let line_start = inc.line_starts[i];
        cache.lines[i].segments = line_ast_to_segments(&inc.line_asts[i], line_text, line_start);
    }

    cache
}

/// Получить текст строки из IncrementalDoc (без аллокации).
fn get_line_text(inc: &IncrementalDoc, idx: usize) -> &str {
    if idx >= inc.line_starts.len() {
        return "";
    }
    let start = inc.line_starts[idx];
    let end = if idx + 1 < inc.line_starts.len() {
        inc.line_starts[idx + 1]
    } else {
        inc.source.len()
    };
    let line = &inc.source[start..end];
    // Отрезаем \n и \r\n без аллокации
    if let Some(stripped) = line.strip_suffix('\n') {
        stripped.strip_suffix('\r').unwrap_or(stripped)
    } else {
        line
    }
}

/// Преобразовать `LineAST` одной строки в `Vec<Segment>`.
///
/// Каждый вариант `LineAST` превращается в соответствующие сегменты,
/// с учётом raw-позиций в исходном тексте.
fn line_ast_to_segments(line_ast: &LineAST, _line_text: &str, line_start: usize) -> Vec<Segment> {
    match line_ast {
        LineAST::Empty => vec![],

        LineAST::Paragraph(children)
        | LineAST::Header(_, children)
        | LineAST::ListItem(_, _, children)
        | LineAST::Quote(children)
        | LineAST::SpoilerLine(children)
        | LineAST::Spoiler(_, children) => {
            build_segments_from_nodes(children, line_start)
        }

        LineAST::Comment(children) => {
            // Комментарий — серая маска с текстом
            build_segments_from_nodes(children, line_start)
        }

        LineAST::Formula(children) => {
            build_segments_from_nodes(children, line_start)
        }

        LineAST::Tag(_) | LineAST::ThematicBreak => {
            vec![]
        }

        LineAST::TableRow(cells) => {
            let mut segments = Vec::new();
            let mut offset = line_start;
            for cell in cells {
                let cell_segs = build_segments_from_nodes(cell, offset);
                if let Some(last) = cell_segs.last() {
                    offset = last.raw_end;
                }
                segments.extend(cell_segs);
            }
            segments
        }

        LineAST::BlockMarker(_)
        | LineAST::SpoilerBlockOpen(_) => {
            // Блок-маркеры — сами не отображаются, их контент обрабатывается
            // внутри соответствующих children
            vec![]
        }

        LineAST::CodeLine(content) => {
            vec![Segment {
                text: content.clone(),
                style: to_style_flags(zoll::ast::MarkupStyle::PLAIN),
                left_marker_len: 0,
                right_marker_len: 0,
                raw_start: line_start,
                raw_end: line_start + content.len(),
            }]
        }

        LineAST::CommentLine(content) => {
            vec![Segment {
                text: content.clone(),
                style: to_style_flags(zoll::ast::MarkupStyle::PLAIN),
                left_marker_len: 0,
                right_marker_len: 0,
                raw_start: line_start,
                raw_end: line_start + content.len(),
            }]
        }

        LineAST::FormulaLine(content) => {
            vec![Segment {
                text: content.clone(),
                style: to_style_flags(zoll::ast::MarkupStyle::PLAIN),
                left_marker_len: 0,
                right_marker_len: 0,
                raw_start: line_start,
                raw_end: line_start + content.len(),
            }]
        }
    }
}
