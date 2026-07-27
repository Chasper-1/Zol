//! Контекст авто-раскрытия маркеров по положению курсора.

use crate::segment::{MarkerCategory, Segment};

/// Контекст авто-раскрытия маркеров.
#[derive(Clone, Copy)]
pub struct RevealCtx<'a> {
    pub cursor_raw: Option<usize>,
    pub cursor_line: Option<usize>,
    pub block_of_line: &'a [Option<usize>],
}

impl RevealCtx<'_> {
    pub fn empty() -> &'static Self {
        static EMPTY: RevealCtx = RevealCtx {
            cursor_raw: None,
            cursor_line: None,
            block_of_line: &[],
        };
        &EMPTY
    }
}

/// Авто-раскрытие: определена ли позиция курсора рядом с маркерами сегмента.
pub fn segment_is_revealed(seg: &Segment, line_index: usize, ctx: &RevealCtx) -> bool {
    let Some(cursor) = ctx.cursor_raw else {
        return false;
    };
    match seg.category {
        MarkerCategory::Inline => {
            // Показываем маркеры Inline только если курсор на той же строке
            if ctx.cursor_line != Some(line_index) {
                return false;
            }
            let start = seg.raw_start.saturating_sub(seg.left_marker_len);
            let end = seg.raw_end + seg.left_marker_len;
            cursor >= start && cursor <= end
        }
        MarkerCategory::Line => ctx.cursor_line == Some(line_index),
        MarkerCategory::Block => match (ctx.cursor_line, ctx.block_of_line.get(line_index)) {
            (Some(cl), Some(Some(bid))) => {
                ctx.block_of_line.get(cl).copied().flatten() == Some(*bid)
            }
            _ => false,
        },
    }
}
