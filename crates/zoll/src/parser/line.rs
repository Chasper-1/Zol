//! Парсер одной строки zoll → `ParsedLine`.
//!
//! Определяет тип строки по первой колонке, парсит inline-маркеры
//! и возвращает готовый кеш-объект с сегментами.

use crate::model::{BlockContainer, BlockRole};
use crate::{BlockKind, MarkStyle, ParsedLine, Segment};

/// Парсит одну строку (без `\n`) в `ParsedLine`.
pub fn parse_line(line: &str) -> ParsedLine {
    let trimmed = line.trim_start();

    // ── Пустая строка ──
    if trimmed.is_empty() {
        return ParsedLine::empty();
    }

    // ── Block-level: %%% / $$$ / !!! ──
    if let Some(rest) = trimmed.strip_prefix("%%%") {
        let title = if rest.trim().is_empty() { None } else { None };
        let role = BlockRole::Open;
        return empty_block(line, BlockContainer::Comment, title, role);
    }
    if let Some(rest) = trimmed.strip_prefix("$$$") {
        let title = None;
        let role = BlockRole::Open;
        return empty_block(line, BlockContainer::Formula, title, role);
    }
    if let Some(rest) = trimmed.strip_prefix("!!!") {
        let title = parse_spoiler_title(rest);
        let role = BlockRole::Open;
        return empty_block(line, BlockContainer::Spoiler, title, role);
    }

    // ── %% комментарий с любого места ──
    if let Some(pos) = trimmed.find("%%") {
        let after = &trimmed[pos + 2..];
        let content = after.trim();
        let segments = parse_inline_to_segments(content, 0);
        return ParsedLine::new(line, BlockKind::CommentLine, segments);
    }

    // ── $$ Display formula (только с начала строки) ──
    if trimmed.starts_with("$$") {
        let content = trimmed[2..].trim();
        let segments = parse_inline_to_segments(content, 0);
        return ParsedLine::new(line, BlockKind::FormulaLine, segments);
    }

    // ── !! с любого места ── (не !!!, проверено выше)
    if let Some(pos) = trimmed.find("!!") {
        let after = &trimmed[pos + 2..];
        let rest = after.trim();
        let title = if let Some(end) = rest.find(':') {
            Some(rest[..end].trim().to_string())
        } else {
            None
        };
        let content = if title.is_some() {
            let rest2 = &trimmed[pos + 2..];
            let rest_trimmed = rest2.trim();
            if let Some(end) = rest_trimmed.find(':') {
                rest_trimmed[end + 1..].trim().to_string()
            } else {
                String::new()
            }
        } else {
            rest.to_string()
        };
        let segments = parse_inline_to_segments(&content, 0);
        return ParsedLine::new(line, BlockKind::SpoilerLine(title), segments);
    }

    // ── #N# Заголовок ──
    if let Some(rest) = trimmed.strip_prefix('#') {
        let level_end = rest.find('#');
        if let Some(end) = level_end {
            if end > 0 {
                let level_str = &rest[..end];
                if let Ok(level) = level_str.parse::<u32>() {
                    let content = rest[end + 1..].trim();
                    let segments = parse_inline_to_segments(content, 0);
                    return ParsedLine::new(line, BlockKind::Header(level), segments);
                }
            }
        }
    }

    // ── ThematicBreak: --- / ___ / *** ──
    let pure = trimmed.trim();
    if pure == "---" || pure == "___" || pure == "***" {
        return ParsedLine {
            source: line.to_string(),
            kind: BlockKind::ThematicBreak,
            segments: Vec::new(),
        };
    }

    // ── Ненумерованный список: - / * / + ──
    for delim in &['-', '*', '+'] {
        if let Some(rest) = trimmed.strip_prefix(*delim) {
            if rest.is_empty() || rest.starts_with(' ') {
                let content = rest.trim();
                let segments = parse_inline_to_segments(content, 0);
                return ParsedLine::new(line, BlockKind::Bullet, segments);
            }
        }
    }

    // ── Нумерованный список: 1. / 2. / ... ──
    if let Some(end) = trimmed.find(|c: char| !c.is_ascii_digit()) {
        if end > 0 && trimmed.as_bytes().get(end) == Some(&b'.') {
            let num = trimmed[..end].parse::<u32>().unwrap_or(1);
            if trimmed.as_bytes().get(end + 1).map_or(true, |&b| b == b' ') {
                let content = trimmed[end + 1..].trim();
                let segments = parse_inline_to_segments(content, 0);
                return ParsedLine::new(line, BlockKind::Ordered(num), segments);
            }
        }
    }

    // ── Цитата: > ──
    if let Some(rest) = trimmed.strip_prefix('>') {
        let content = rest.trim();
        let segments = parse_inline_to_segments(content, 0);
        return ParsedLine::new(line, BlockKind::Quote, segments);
    }

    // ── Строка таблицы: | ... | ──
    if trimmed.starts_with('|') {
        return ParsedLine::new(line, BlockKind::TableRow, Vec::new());
    }

    // ── Тэг: #:tag ──
    if let Some(rest) = trimmed.strip_prefix("#:") {
        return ParsedLine {
            source: line.to_string(),
            kind: BlockKind::Tag(rest.trim().to_string()),
            segments: Vec::new(),
        };
    }

    // ── Ничего не подошло → обычный параграф ──
    let segments = parse_inline_to_segments(trimmed, 0);
    ParsedLine::new(line, BlockKind::Paragraph, segments)
}

/// Парсит inline-маркеры в тексте и возвращает плоский список сегментов.
///
/// Сегменты — диапазоны (start, end) от начала `text`.
/// Вложенные маркеры схлопываются: `**//text//**` → один сегмент со стилем BOLD|ITALIC.
pub fn parse_inline_to_segments(text: &str, offset: usize) -> Vec<Segment> {
    if text.is_empty() {
        return Vec::new();
    }

    let bytes = text.as_bytes();
    let len = text.len();
    let mut pos = 0;
    let mut segments: Vec<Segment> = Vec::new();
    let mut text_start: Option<usize> = None;

    while pos < len {
        let b = bytes[pos];

        // ── Экранирование ──
        if b == b'\\' && pos + 1 < len {
            flush_text(&mut segments, &mut text_start, pos, offset);
            let ch_len = utf8_char_len(bytes[pos + 1]);
            // escaped char — добавляем как PLAIN текст
            segments.push(Segment::new(
                offset + pos + 1,
                offset + pos + 1 + ch_len,
                MarkStyle::PLAIN,
            ));
            pos += 1 + ch_len;
            continue;
        }

        // ── Поиск inline-маркеров ──
        if let Some((close_pos, style)) = match_inline_marker(bytes, pos, len) {
            let open_len = marker_len(style);
            let open_end = pos + open_len;
            if close_pos > open_end {
                flush_text(&mut segments, &mut text_start, pos, offset);
                let inner_text = &text[open_end..close_pos];
                let inner_segs = parse_inline_to_segments(inner_text, offset + open_end);
                // Если внутренних сегментов нет — создаём один с объединённым стилем
                if inner_segs.is_empty() {
                    segments.push(Segment::new(offset + open_end, offset + close_pos, style));
                } else {
                    for mut seg in inner_segs {
                        seg.style = seg.style | style;
                        segments.push(seg);
                    }
                }
                pos = close_pos + open_len;
                continue;
            }
        }

        // ── Накопление текста ──
        if text_start.is_none() {
            text_start = Some(pos);
        }
        pos += 1;
    }

    flush_text(&mut segments, &mut text_start, pos, offset);
    segments
}

// ─── Помощники ─────────────────────────────────────────────────

fn marker_len(style: MarkStyle) -> usize {
    if style == MarkStyle::FORMULA { 1 } else { 2 }
}

fn utf8_char_len(b: u8) -> usize {
    if b < 128 {
        1
    } else if b & 0xE0 == 0xC0 {
        2
    } else if b & 0xF0 == 0xE0 {
        3
    } else {
        4
    }
}

fn flush_text(segments: &mut Vec<Segment>, start: &mut Option<usize>, end: usize, offset: usize) {
    if let Some(s) = start.take() {
        if end > s {
            segments.push(Segment::new(offset + s, offset + end, MarkStyle::PLAIN));
        }
    }
}

fn match_inline_marker(bytes: &[u8], pos: usize, len: usize) -> Option<(usize, MarkStyle)> {
    if pos + 1 >= len {
        return None;
    }
    let (style, open_len) = match (bytes[pos], bytes[pos + 1]) {
        (b'*', b'*') => (MarkStyle::BOLD, 2),
        (b'/', b'/') => (MarkStyle::ITALIC, 2),
        (b'_', b'_') => (MarkStyle::UNDERLINE, 2),
        (b'~', b'~') => (MarkStyle::STRIKETHROUGH, 2),
        (b'=', b'=') => (MarkStyle::HIGHLIGHT, 2),
        (b'+', b'+') => (MarkStyle::INSERTION, 2),
        (b'-', b'-') => (MarkStyle::DELETION, 2),
        (b'\'', b'\'') => (MarkStyle::SUPERSCRIPT, 2),
        (b',', b',') => (MarkStyle::SUBSCRIPT, 2),
        (b'$', _) => return find_close_for_single(bytes, pos, len),
        _ => return None,
    };
    find_close(bytes, pos, open_len, style)
}

fn find_close(
    bytes: &[u8],
    open_pos: usize,
    open_len: usize,
    style: MarkStyle,
) -> Option<(usize, MarkStyle)> {
    let start = open_pos + open_len;
    let mut i = start;
    while i + open_len <= bytes.len() {
        if bytes[i] == bytes[open_pos] && bytes[i + 1] == bytes[open_pos + 1] {
            return Some((i, style));
        }
        i += 1;
    }
    None
}

fn find_close_for_single(bytes: &[u8], open_pos: usize, len: usize) -> Option<(usize, MarkStyle)> {
    let start = open_pos + 1;
    for i in start..len {
        if bytes[i] == b'$' {
            return Some((i, MarkStyle::FORMULA));
        }
    }
    None
}

/// Распарсить заголовок спойлера `!!!title:` или `!!!`
fn parse_spoiler_title(rest: &str) -> Option<String> {
    let s = rest.trim();
    if let Some(end) = s.find(':') {
        let title = s[..end].trim();
        if !title.is_empty() {
            return Some(title.to_string());
        }
    }
    None
}

/// Создать пустую блок-строку (только маркер, без контента).
fn empty_block(
    line: &str,
    kind: BlockContainer,
    title: Option<String>,
    role: BlockRole,
) -> ParsedLine {
    ParsedLine {
        source: line.to_string(),
        kind: BlockKind::Block { kind, title, role },
        segments: Vec::new(),
    }
}

// ─── Тесты ─────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_text() {
        let pl = parse_line("hello world");
        assert_eq!(pl.kind, BlockKind::Paragraph);
        assert_eq!(pl.segments.len(), 1);
        assert_eq!(pl.segments[0].style, MarkStyle::PLAIN);
        assert_eq!(pl.segments[0].text(&pl.source), "hello world");
    }

    #[test]
    fn bold_text() {
        let pl = parse_line("**hello**");
        assert_eq!(pl.kind, BlockKind::Paragraph);
        assert_eq!(pl.segments.len(), 1);
        assert_eq!(pl.segments[0].style, MarkStyle::BOLD);
        assert_eq!(pl.segments[0].text(&pl.source), "hello");
    }

    #[test]
    fn italic_text() {
        let pl = parse_line("//hello//");
        assert_eq!(pl.kind, BlockKind::Paragraph);
        assert_eq!(pl.segments.len(), 1);
        assert_eq!(pl.segments[0].style, MarkStyle::ITALIC);
        assert_eq!(pl.segments[0].text(&pl.source), "hello");
    }

    #[test]
    fn nested_formatting() {
        let pl = parse_line("**//bold italic//**");
        assert_eq!(pl.kind, BlockKind::Paragraph);
        assert_eq!(pl.segments.len(), 1);
        assert!(pl.segments[0].style.contains(MarkStyle::BOLD));
        assert!(pl.segments[0].style.contains(MarkStyle::ITALIC));
        assert_eq!(pl.segments[0].text(&pl.source), "bold italic");
    }

    #[test]
    fn mixed_text_and_formatting() {
        let pl = parse_line("hello **world**");
        assert_eq!(pl.kind, BlockKind::Paragraph);
        assert_eq!(pl.segments.len(), 2);
        assert_eq!(pl.segments[0].style, MarkStyle::PLAIN);
        assert_eq!(pl.segments[0].text(&pl.source), "hello ");
        assert_eq!(pl.segments[1].style, MarkStyle::BOLD);
        assert_eq!(pl.segments[1].text(&pl.source), "world");
    }

    #[test]
    fn header() {
        let pl = parse_line("#1# Title");
        assert_eq!(pl.kind, BlockKind::Header(1));
        assert_eq!(pl.segments.len(), 1);
        assert_eq!(pl.segments[0].text(&pl.source), "Title");
    }

    #[test]
    fn header_level_3() {
        let pl = parse_line("#3# Sub Section");
        assert_eq!(pl.kind, BlockKind::Header(3));
    }

    #[test]
    fn comment_line() {
        let pl = parse_line("%% this is hidden");
        assert_eq!(pl.kind, BlockKind::CommentLine);
    }

    #[test]
    fn comment_mid_line() {
        let pl = parse_line("visible %% hidden");
        assert_eq!(pl.kind, BlockKind::CommentLine);
    }

    #[test]
    fn formula_line() {
        let pl = parse_line("$$ x = 5");
        assert_eq!(pl.kind, BlockKind::FormulaLine);
        assert_eq!(pl.segments.len(), 1);
        assert_eq!(pl.segments[0].text(&pl.source), "x = 5");
    }

    #[test]
    fn spoiler_line() {
        let pl = parse_line("!! hidden content");
        assert_eq!(pl.kind, BlockKind::SpoilerLine(None));
    }

    #[test]
    fn spoiler_with_title() {
        let pl = parse_line("!!title: hidden");
        assert_eq!(pl.kind, BlockKind::SpoilerLine(Some("title".to_string())));
    }

    #[test]
    fn thematic_break() {
        assert_eq!(parse_line("---").kind, BlockKind::ThematicBreak);
        assert_eq!(parse_line("___").kind, BlockKind::ThematicBreak);
        assert_eq!(parse_line("***").kind, BlockKind::ThematicBreak);
    }

    #[test]
    fn unordered_list() {
        let pl = parse_line("- item");
        assert_eq!(pl.kind, BlockKind::Bullet);
        assert_eq!(pl.segments[0].text(&pl.source), "item");
    }

    #[test]
    fn unordered_list_with_star() {
        let pl = parse_line("* item");
        assert_eq!(pl.kind, BlockKind::Bullet);
    }

    #[test]
    fn ordered_list() {
        let pl = parse_line("1. first");
        assert_eq!(pl.kind, BlockKind::Ordered(1));
    }

    #[test]
    fn star_not_confused_with_bold() {
        let pl = parse_line("**bold**");
        assert_eq!(pl.kind, BlockKind::Paragraph);
    }

    #[test]
    fn block_marker_comment() {
        assert_eq!(
            parse_line("%%%").kind,
            BlockKind::Block {
                kind: BlockContainer::Comment,
                title: None,
                role: BlockRole::Open
            }
        );
    }

    #[test]
    fn block_marker_formula() {
        assert_eq!(
            parse_line("$$$").kind,
            BlockKind::Block {
                kind: BlockContainer::Formula,
                title: None,
                role: BlockRole::Open
            }
        );
    }

    #[test]
    fn block_marker_spoiler() {
        assert_eq!(
            parse_line("!!!").kind,
            BlockKind::Block {
                kind: BlockContainer::Spoiler,
                title: None,
                role: BlockRole::Open
            }
        );
    }

    #[test]
    fn quote() {
        let pl = parse_line("> quoted text");
        assert_eq!(pl.kind, BlockKind::Quote);
        assert_eq!(pl.segments[0].text(&pl.source), "quoted text");
    }

    #[test]
    fn empty_line() {
        let pl = parse_line("");
        assert_eq!(pl.kind, BlockKind::Empty);
        assert!(pl.segments.is_empty());
        assert!(pl.source.is_empty());
    }

    #[test]
    fn escape_char() {
        let pl = parse_line(r"\**not bold**");
        assert_eq!(pl.kind, BlockKind::Paragraph);
        // escaped * становится PLAIN текстом, остальное тоже PLAIN
        assert!(pl.segments.iter().all(|s| s.style == MarkStyle::PLAIN));
    }

    #[test]
    fn spoiler_block_open_with_title() {
        let pl = parse_line("!!!spoiler block:");
        assert_eq!(
            pl.kind,
            BlockKind::Block {
                kind: BlockContainer::Spoiler,
                title: Some("spoiler block".to_string()),
                role: BlockRole::Open
            }
        );
    }

    #[test]
    fn mixed_on_line() {
        let pl = parse_line("text %% comment");
        assert_eq!(pl.kind, BlockKind::CommentLine);
    }

    #[test]
    fn plain_segment_covers_whole() {
        let pl = parse_line("just plain text");
        assert_eq!(pl.segments.len(), 1);
        assert_eq!(pl.segments[0].text(&pl.source), "just plain text");
    }

    #[test]
    fn table_row() {
        let pl = parse_line("| a | b | c |");
        assert_eq!(pl.kind, BlockKind::TableRow);
    }

    #[test]
    fn tag() {
        let pl = parse_line("#:mytag");
        assert_eq!(pl.kind, BlockKind::Tag("mytag".to_string()));
    }
}
