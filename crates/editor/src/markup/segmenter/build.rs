use zoll::ast::{MarkupNode, MarkupStyle};

use crate::markup::segment::{Segment, StyleFlags};
use crate::markup::segmenter::helpers::{combine_style, marker_open_len, to_style_flags};

/// Рекурсивно обходит AST и собирает сегменты.
pub fn build_segments(
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

                // Рекурсивно обрабатываем детей
                raw_offset = build_segments(children, combined, segments, raw_offset);

                // Пропускаем закрывающий маркер
                let child_end = raw_offset;
                raw_offset += marker_len;

                // Помечаем первый сегмент маркером
                if let Some(first) = segments
                    .iter_mut()
                    .rev()
                    .find(|s| s.raw_end <= child_end && s.raw_start >= child_start)
                {
                    first.left_marker_len += marker_len;
                }
            }

            // Блок-левел ноды: рекурсивно обходим их детей
            MarkupNode::Header { children, .. }
            | MarkupNode::ListItem { children, .. }
            | MarkupNode::Quote(children)
            | MarkupNode::Spoiler { children, .. }
            | MarkupNode::Comment(children)
            | MarkupNode::Formula(children) => {
                raw_offset = build_segments(children, inherited_style, segments, raw_offset);
            }

            MarkupNode::TableRow(cells) => {
                for cell in cells {
                    raw_offset = build_segments(cell, inherited_style, segments, raw_offset);
                }
            }

            MarkupNode::CodeBlock { content, .. } => {
                segments.push(Segment {
                    text: content.clone(),
                    style: to_style_flags(inherited_style),
                    left_marker_len: 0,
                    right_marker_len: 0,
                    raw_start: raw_offset,
                    raw_end: raw_offset + content.len(),
                });
                raw_offset += content.len();
            }

            MarkupNode::ThematicBreak => {
                // Разделитель — пустой сегмент
                segments.push(Segment {
                    text: String::new(),
                    style: to_style_flags(inherited_style),
                    left_marker_len: 0,
                    right_marker_len: 0,
                    raw_start: raw_offset,
                    raw_end: raw_offset,
                });
            }
        }
    }
    raw_offset
}
