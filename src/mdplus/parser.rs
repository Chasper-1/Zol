use crate::editor::cache::DocumentCache;
use crate::editor::markup::segment::{STYLE_PLAIN, Segment, StyleFlags};
use crate::mdplus::marker::{MARKERS, MarkerDef};

pub fn parse_document(text: &str) -> DocumentCache {
    let line_starts: Vec<usize> = std::iter::once(0)
        .chain(text.match_indices('\n').map(|(i, _)| i + 1))
        .collect();
    let n_lines = line_starts.len();

    let segments = parse_region(text, 0, text.len(), STYLE_PLAIN);

    let mut doc = DocumentCache {
        lines: vec![Default::default(); n_lines],
    };

    for seg in segments {
        if seg.text.contains('\n') {
            let parts: Vec<&str> = seg.text.split('\n').collect();
            let n_parts = parts.len();
            let mut raw_offset = seg.raw_start;

            for (pi, part) in parts.iter().enumerate() {
                let line_idx = find_line(&line_starts, raw_offset);
                if line_idx >= doc.lines.len() {
                    break;
                }
                doc.lines[line_idx].segments.push(Segment {
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
            let line_idx = find_line(&line_starts, seg.raw_start);
            doc.lines[line_idx].segments.push(seg);
        }
    }

    doc
}

fn find_line(starts: &[usize], offset: usize) -> usize {
    match starts.binary_search(&offset) {
        Ok(i) => i,
        Err(i) => i.wrapping_sub(1),
    }
}

fn parse_region(text: &str, start: usize, end: usize, parent_style: StyleFlags) -> Vec<Segment> {
    let mut segments = Vec::new();
    let mut pos = start;

    while pos < end {
        let tail = &text[pos..end];

        if tail.starts_with('\\')
            && let Some(ch) = tail[1..].chars().next()
        {
            pos += 1;
            let ch_len = ch.len_utf8();
            segments.push(Segment {
                text: text[pos..pos + ch_len].to_string(),
                style: parent_style,
                left_marker_len: 0,
                right_marker_len: 0,
                raw_start: pos - 1,
                raw_end: pos + ch_len,
            });
            pos += ch_len;
            continue;
        }

        if let Some(m_idx) = find_marker(text, pos, end) {
            let marker = &MARKERS[m_idx];
            let open_end = pos + marker.open.len();

            if is_valid_open(text, open_end) {
                let search_end = if marker.multiline {
                    end
                } else {
                    text[open_end..end]
                        .find('\n')
                        .map(|p| open_end + p)
                        .unwrap_or(end)
                };

                if let Some(close_start) = find_close_nested(text, open_end..search_end, marker) {
                    let close_end = close_start + marker.close.len();

                    if is_valid_close(text, close_start) && close_start > open_end {
                        let combined = parent_style | marker.style;
                        let inner = parse_region(text, open_end, close_start, combined);
                        let mut inner = inner;

                        if let Some(first) = inner.first_mut() {
                            first.left_marker_len += marker.open.len();
                        }
                        if let Some(last) = inner.last_mut() {
                            last.right_marker_len += marker.close.len();
                        }

                        segments.extend(inner);
                        pos = close_end;
                        continue;
                    }
                }
            }
        }

        let text_start = pos;
        let ch = text[pos..].chars().next().unwrap();
        pos += ch.len_utf8();

        while pos < end {
            if text.as_bytes()[pos] == b'\\' {
                break;
            }
            if let Some(idx) = find_marker(text, pos, end) {
                let m = &MARKERS[idx];
                let candidate_end = pos + m.open.len();
                if candidate_end <= end && is_valid_open(text, candidate_end) {
                    break;
                }
            }
            let c = text[pos..].chars().next().unwrap();
            pos += c.len_utf8();
        }

        segments.push(Segment {
            text: text[text_start..pos].to_string(),
            style: parent_style,
            left_marker_len: 0,
            right_marker_len: 0,
            raw_start: text_start,
            raw_end: pos,
        });
    }

    segments
}

fn find_marker(text: &str, pos: usize, end: usize) -> Option<usize> {
    if pos >= end {
        return None;
    }
    let tail = &text[pos..end];
    MARKERS
        .iter()
        .enumerate()
        .find(|(_, m)| tail.starts_with(m.open))
        .map(|(i, _)| i)
}

fn is_valid_open(text: &str, open_end: usize) -> bool {
    if open_end >= text.len() {
        return false;
    }
    text[open_end..]
        .chars()
        .next()
        .is_some_and(|c| !c.is_ascii_whitespace())
}

fn is_valid_close(text: &str, close_start: usize) -> bool {
    if close_start == 0 {
        return false;
    }
    text[..close_start]
        .chars()
        .next_back()
        .is_some_and(|c| !c.is_ascii_whitespace())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::markup::segment::{
        STYLE_BOLD, STYLE_COMMENT, STYLE_DELETION, STYLE_DISPLAY_FORMULA, STYLE_FORMULA,
        STYLE_HIGHLIGHT, STYLE_INSERTION, STYLE_ITALIC, STYLE_PLAIN, STYLE_STRIKETHROUGH,
        STYLE_SUBSCRIPT, STYLE_SUPERSCRIPT, STYLE_UNDERLINE,
    };

    fn check_segments(text: &str, expected: &[(&str, StyleFlags, usize, usize)]) {
        let doc = parse_document(text);
        let segs: Vec<_> = doc.lines.iter().flat_map(|l| &l.segments).collect();
        assert_eq!(
            segs.len(),
            expected.len(),
            "segment count mismatch for: {text}"
        );
        for (i, (seg, (exp_text, exp_style, exp_left, exp_right))) in
            segs.iter().zip(expected.iter()).enumerate()
        {
            assert_eq!(&seg.text, exp_text, "segment {i} text");
            assert_eq!(seg.style, *exp_style, "segment {i} style");
            assert_eq!(seg.left_marker_len, *exp_left, "segment {i} left_marker");
            assert_eq!(seg.right_marker_len, *exp_right, "segment {i} right_marker");
        }
    }

    #[test]
    fn plain_text() {
        check_segments("hello world", &[("hello world", STYLE_PLAIN, 0, 0)]);
    }

    #[test]
    fn bold_basic() {
        check_segments("**bold**", &[("bold", STYLE_BOLD, 2, 2)]);
    }

    #[test]
    fn italic_basic() {
        check_segments("//italic//", &[("italic", STYLE_ITALIC, 2, 2)]);
    }

    #[test]
    fn bold_with_leading_text() {
        check_segments(
            "a **bold** b",
            &[
                ("a ", STYLE_PLAIN, 0, 0),
                ("bold", STYLE_BOLD, 2, 2),
                (" b", STYLE_PLAIN, 0, 0),
            ],
        );
    }

    #[test]
    fn no_close_treated_as_text() {
        check_segments("**bold", &[("**bold", STYLE_PLAIN, 0, 0)]);
    }

    #[test]
    fn space_after_open_invalid() {
        check_segments("** bold**", &[("** bold**", STYLE_PLAIN, 0, 0)]);
    }

    #[test]
    fn space_before_close_invalid() {
        check_segments("**bold **", &[("**bold **", STYLE_PLAIN, 0, 0)]);
    }

    #[test]
    fn nested_bold_italic() {
        check_segments(
            "**a //b// c**",
            &[
                ("a ", STYLE_BOLD, 2, 0),
                ("b", STYLE_BOLD | STYLE_ITALIC, 2, 2),
                (" c", STYLE_BOLD, 0, 2),
            ],
        );
    }

    #[test]
    fn escape_disables_marker() {
        let doc = parse_document(r"\*\*text\*\*");
        let segs: Vec<_> = doc.lines.iter().flat_map(|l| &l.segments).collect();
        for s in &segs {
            assert_eq!(s.style, STYLE_PLAIN);
        }
        let display: String = segs.iter().map(|s| &s.text[..]).collect();
        assert_eq!(display, "**text**");
    }

    #[test]
    fn escape_backslash() {
        check_segments(r"\\", &[("\\", STYLE_PLAIN, 0, 0)]);
    }

    #[test]
    fn empty_content_not_formatted() {
        let doc = parse_document("****");
        let segs: Vec<_> = doc.lines.iter().flat_map(|l| &l.segments).collect();
        assert!(segs.len() >= 1);
        for s in &segs {
            assert_eq!(s.style, STYLE_PLAIN);
        }
        let display: String = segs.iter().map(|s| &s.text[..]).collect();
        assert_eq!(display, "****");
    }

    #[test]
    fn underline() {
        check_segments("__underline__", &[("underline", STYLE_UNDERLINE, 2, 2)]);
    }

    #[test]
    fn strikethrough() {
        check_segments("~~deleted~~", &[("deleted", STYLE_STRIKETHROUGH, 2, 2)]);
    }

    #[test]
    fn superscript() {
        check_segments("''2''", &[("2", STYLE_SUPERSCRIPT, 2, 2)]);
    }

    #[test]
    fn subscript() {
        check_segments(",,2,,", &[("2", STYLE_SUBSCRIPT, 2, 2)]);
    }

    #[test]
    fn highlight() {
        check_segments("==mark==", &[("mark", STYLE_HIGHLIGHT, 2, 2)]);
    }

    #[test]
    fn insertion() {
        check_segments("++insert++", &[("insert", STYLE_INSERTION, 2, 2)]);
    }

    #[test]
    fn deletion() {
        check_segments("--delete--", &[("delete", STYLE_DELETION, 2, 2)]);
    }

    #[test]
    fn formula() {
        check_segments("$x+y$", &[("x+y", STYLE_FORMULA, 1, 1)]);
    }

    #[test]
    fn display_formula() {
        check_segments("$$x^2$$", &[("x^2", STYLE_DISPLAY_FORMULA, 2, 2)]);
    }

    #[test]
    fn priority_longer_marker_first() {
        let doc = parse_document("$$a$b$$");
        let segs: Vec<_> = doc.lines.iter().flat_map(|l| &l.segments).collect();
        assert!(segs.len() >= 1);
        for s in &segs {
            assert_eq!(s.style, STYLE_DISPLAY_FORMULA);
        }
        let display: String = segs.iter().map(|s| &s.text[..]).collect();
        assert_eq!(display, "a$b");
    }

    #[test]
    fn multi_line_comment() {
        let text = "before /*hello\nworld*\\ after";
        let doc = parse_document(text);
        assert_eq!(doc.lines.len(), 2);
        let line0: Vec<_> = doc.lines[0].segments.iter().map(|s| &s.text[..]).collect();
        assert_eq!(line0, ["before ", "hello"]);
        assert_eq!(doc.lines[0].segments[1].style, STYLE_COMMENT);
        assert_eq!(doc.lines[1].segments[0].text, "world");
        assert_eq!(doc.lines[1].segments[0].style, STYLE_COMMENT);
    }

    #[test]
    fn notes_md() {
        let text = "# Flint Notes.\n\nЭто вторая ^строка^.\n\n**1234567890**\n\n~тестовая~ строка *через* пробелы ~~пишу~~ рир _~2~_ ирио ри иор иои рои ори рои оио ир иор иор иои ори орио иро ио иои оир ои орио иро ио\n\nabc**hello**xyz\n\nсува́льда — - \n";
        let doc = parse_document(text.trim_end());
        // Just ensure it parses without panic and produces segments
        assert!(doc.lines.len() > 0);
        for (_i, line) in doc.lines.iter().enumerate() {
            for seg in &line.segments {
                let _ = &seg.text;
                let _ = seg.style;
            }
        }
        // Specifically check that `^строка^` is treated as plain (single char,
        // not in table 1 — actually `^` IS a marker in the old system but we
        // removed it from table 1. In the new parser, `^` is NOT a marker.
        // So `^строка^` should be plain text.
        let line3 = &doc.lines[2];
        assert!(line3.segments.iter().all(|s| s.style == STYLE_PLAIN));
    }

    #[test]
    fn same_type_nesting() {
        check_segments(
            "**a **b** c**",
            &[
                ("a ", STYLE_BOLD, 2, 0),
                ("b", STYLE_BOLD, 2, 2),
                (" c", STYLE_BOLD, 0, 2),
            ],
        );
    }
}

fn find_close_nested(
    text: &str,
    search_range: std::ops::Range<usize>,
    marker: &MarkerDef,
) -> Option<usize> {
    let mut depth = 1u32;
    let mut pos = search_range.start;
    let end = search_range.end;

    while pos < end {
        let tail = &text[pos..end];

        if marker.open == marker.close && tail.starts_with(marker.open) {
            let after = pos + marker.open.len();
            if after <= end {
                let next = text[after..].chars().next();
                if next.is_some_and(|c| !c.is_ascii_whitespace()) {
                    depth += 1;
                    pos += marker.open.len();
                    continue;
                }
            }
        }

        if tail.starts_with(marker.close) && pos > search_range.start {
            let prev = text[..pos].chars().next_back();
            if prev.is_some_and(|c| !c.is_ascii_whitespace()) {
                depth -= 1;
                if depth == 0 {
                    return Some(pos);
                }
                pos += marker.close.len();
                continue;
            }
        }

        let ch = tail.chars().next()?;
        pos += ch.len_utf8();
    }

    None
}
