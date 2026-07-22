use zoll::ast::MarkupDoc;

use crate::cache::DocumentCache;
use crate::markup::segment::Segment;
use crate::markup::segmenter::build::build_segments;

/// Преобразует AST-документ zoll в DocumentCache для редактора.
pub fn to_document_cache(doc: &MarkupDoc) -> DocumentCache {
    // 1. Собираем все сегменты в плоский список с raw-позициями
    let mut segments = Vec::new();
    build_segments(
        &doc.children,
        zoll::ast::MarkupStyle::PLAIN,
        &mut segments,
        0,
    );

    // 2. Вычисляем line_starts
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

    // 3. Функция: номер строки по байтовому offset в документе
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
                    raw_offset += 1;
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
                raw_offset += part.len() + 1;
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
