//! Парсер одной строки zoll → `ParsedLine`.
//!
//! Определяет тип строки по первой колонке, парсит inline-маркеры
//! и возвращает готовый кеш-объект с сегментами.

use crate::model::{BlockContainer, BlockRole, MARKERS, MarkerCategory};
use crate::{BlockKind, MarkStyle, ParsedLine, Segment};

// Парсит одну строку (без `\n`) в `ParsedLine`.
pub fn parse_line(line: &str) -> ParsedLine {
    let trimmed = line.trim_start();

    // ── Пустая строка ──
    if trimmed.is_empty() {
        return ParsedLine::empty();
    }

    // ── Block-level: %%% / $$$ / !!! / ``` ──
    // Маркеры берутся из `MARKERS` (категория Block), по длине от длинных к коротким.
    for def in MARKERS {
        if def.category != MarkerCategory::Block {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix(def.open) {
            let container = match def.style {
                MarkStyle::COMMENT => BlockContainer::Comment,
                MarkStyle::FORMULA => BlockContainer::Formula,
                MarkStyle::SPOILER => BlockContainer::Spoiler,
                MarkStyle::CODE => BlockContainer::Code,
                _ => unreachable!("блок-маркер должен иметь свой стиль"),
            };
            let title = match container {
                BlockContainer::Spoiler => parse_spoiler_title(rest),
                BlockContainer::Code => parse_code_lang(rest),
                _ => None,
            };
            return empty_block(line, container, title, BlockRole::Open);
        }
    }

    // ── Line-level: %% / $$ / !! ──
    // `%%` и `!!` с любого места, `$$` только с начала строки.
    for def in MARKERS {
        if def.category != MarkerCategory::Line {
            continue;
        }
        // Позиция маркера в строке.
        let pos = if def.style == MarkStyle::FORMULA {
            // $$ — display formula только с начала строки.
            match trimmed.strip_prefix(def.open) {
                Some(_) => 0,
                None => continue,
            }
        } else {
            match trimmed.find(def.open) {
                Some(p) => p,
                None => continue,
            }
        };
        let after = &trimmed[pos + def.open.len()..];
        let rest = after.trim();
        let kind = match def.style {
            MarkStyle::COMMENT => BlockKind::CommentLine,
            MarkStyle::FORMULA => BlockKind::FormulaLine,
            MarkStyle::SPOILER => {
                // !!title: hidden → заголовок в SpoilerLine, тело в сегментах.
                let (title, body) = split_spoiler(rest);
                let base = base_offset(line, body);
                let segments = parse_inline_to_segments(body, base);
                return ParsedLine::new(line, BlockKind::SpoilerLine(title), segments);
            }
            _ => unreachable!("line-маркер должен иметь свой стиль"),
        };
        let base = base_offset(line, rest);
        let segments = parse_inline_to_segments(rest, base);
        return ParsedLine::new(line, kind, segments);
    }

    // ── #N# Заголовок ──
    if let Some(rest) = trimmed.strip_prefix('#') {
        let level_end = rest.find('#');
        if let Some(end) = level_end {
            if end > 0 {
                let level_str = &rest[..end];
                if let Ok(level) = level_str.parse::<u32>() {
                    let content = rest[end + 1..].trim();
                    let base = base_offset(line, content);
                    let segments = parse_inline_to_segments(content, base);
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
                let base = base_offset(line, content);
                let segments = parse_inline_to_segments(content, base);
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
                let base = base_offset(line, content);
                let segments = parse_inline_to_segments(content, base);
                return ParsedLine::new(line, BlockKind::Ordered(num), segments);
            }
        }
    }

    // ── Цитата: > ──
    if let Some(rest) = trimmed.strip_prefix('>') {
        let content = rest.trim();
        let base = base_offset(line, content);
        let segments = parse_inline_to_segments(content, base);
        return ParsedLine::new(line, BlockKind::Quote, segments);
    }

    // ── Строка таблицы: | a | b | ──
    if trimmed.starts_with('|') {
        return parse_table_row(line, trimmed);
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
    let base = line.len() - trimmed.len();
    let segments = parse_inline_to_segments(trimmed, base);
    ParsedLine::new(line, BlockKind::Paragraph, segments)
}

// Парсит inline-маркеры в тексте и возвращает плоский список сегментов.
//
// Сегменты — диапазоны (start, end) от начала `text`.
// Вложенные маркеры схлопываются: `**//text//**` → один сегмент со стилем BOLD|ITALIC.
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
        if let Some((close_pos, open_len, style)) = match_inline_marker(bytes, pos, len) {
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

// Ищет inline-маркер в `bytes` начиная с `pos`.
// Возвращает `(close_pos, open_len, style)` — позицию закрывающего,
// длину открывающего и стиль. Маркеры берутся из `MARKERS`.
fn match_inline_marker(bytes: &[u8], pos: usize, len: usize) -> Option<(usize, usize, MarkStyle)> {
    for def in MARKERS {
        if def.category != MarkerCategory::Inline {
            continue;
        }
        let open = def.open.as_bytes();
        let open_len = open.len();
        if pos + open_len > len || &bytes[pos..pos + open_len] != open {
            continue;
        }
        let close_pos = find_close(bytes, pos + open_len, def.close.as_bytes(), len)?;
        return Some((close_pos, open_len, def.style));
    }
    None
}

// Ищет закрывающую последовательность `close` начиная с `start`.
fn find_close(bytes: &[u8], start: usize, close: &[u8], len: usize) -> Option<usize> {
    let mut i = start;
    while i + close.len() <= len {
        if &bytes[i..i + close.len()] == close {
            return Some(i);
        }
        i += 1;
    }
    None
}

// Вычислить байтовое смещение подстроки `child` внутри `parent`.
// `child` должен быть подстрокой `parent` (возвращает 0 если нет).
#[inline]
fn base_offset(parent: &str, child: &str) -> usize {
    if child.is_empty() || parent.is_empty() {
        return 0;
    }
    let parent_ptr = parent.as_ptr() as usize;
    let child_ptr = child.as_ptr() as usize;
    if child_ptr >= parent_ptr && child_ptr < parent_ptr + parent.len() {
        child_ptr - parent_ptr
    } else {
        0
    }
}

// Распарсить строку таблицы `| a | b | c |`.
//
// Ячейки — обрезанные тексты без `:` выравнивания. Сегменты — PLAIN-диапазоны
// каждой ячейки внутри исходной строки (от первого `|` до конца).
fn parse_table_row(line: &str, trimmed: &str) -> ParsedLine {
    // Байтовое смещение trimmed внутри line.
    let line_base = line.len() - trimmed.len();

    let mut cells = Vec::new();
    let mut segments = Vec::new();
    // Диапазон ячеек: от первого `|` (включительно) до конца строки.
    let body_start = trimmed.find('|').unwrap_or(0);
    let body = &trimmed[body_start..];
    let mut prev = 1usize; // начало ячейки после разделителя `|`
    let mut idx = prev;
    let bs = body.as_bytes();
    while idx < bs.len() {
        if bs[idx] == b'|' {
            push_cell(
                &mut cells,
                &mut segments,
                body,
                prev,
                idx,
                line_base + body_start,
            );
            prev = idx + 1;
        }
        idx += 1;
    }
    push_cell(
        &mut cells,
        &mut segments,
        body,
        prev,
        body.len(),
        line_base + body_start,
    );

    // Срезаем пустые ячейки-обманки: `| a | b |` не должен давать `""` в конце.
    while cells.last().map_or(false, |c| c.is_empty()) {
        cells.pop();
        segments.pop();
    }

    ParsedLine::new(line, BlockKind::TableRow(cells), segments)
}

// Разобрать одну ячейку `body[from..to]`: обрезать, снять `:`, добавить сегмент.
fn push_cell(
    cells: &mut Vec<String>,
    segments: &mut Vec<Segment>,
    body: &str,
    from: usize,
    to: usize,
    base: usize,
) {
    let raw = &body[from..to];
    let cell = strip_align(raw.trim());
    cells.push(cell);
    if to > from {
        segments.push(Segment::new(base + from, base + to, MarkStyle::PLAIN));
    }
}

// Снять `:` выравнивания с краёв ячейки (`:left:` → `left`).
fn strip_align(cell: &str) -> String {
    let s = cell.strip_prefix(':').unwrap_or(cell);
    s.strip_suffix(':').unwrap_or(s).to_string()
}

// Распарсить спойлер-строку `!!title: hidden` → (заголовок, тело).
// Без `:` — (None, весь текст).
fn split_spoiler(rest: &str) -> (Option<String>, &str) {
    if let Some(end) = rest.find(':') {
        let title = rest[..end].trim();
        let title = if title.is_empty() {
            None
        } else {
            Some(title.to_string())
        };
        let body = rest[end + 1..].trim();
        (title, body)
    } else {
        (None, rest)
    }
}

// Распарсить заголовок спойлера `!!!title:` или `!!!`
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

// Язык код-блока после ``` (например, ` ```rust ` → "rust").
// Пустой/невалидный остаток → None.
fn parse_code_lang(rest: &str) -> Option<String> {
    let s = rest.trim();
    if s.is_empty() {
        None
    } else {
        Some(s.to_string())
    }
}

// Создать пустую блок-строку (только маркер, без контента).
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
        assert_eq!(
            pl.kind,
            BlockKind::TableRow(vec!["a".to_string(), "b".to_string(), "c".to_string()])
        );
    }

    #[test]
    fn table_row_align_colons() {
        let pl = parse_line("|:left: | :center: | right:|");
        assert_eq!(
            pl.kind,
            BlockKind::TableRow(vec![
                "left".to_string(),
                "center".to_string(),
                "right".to_string()
            ])
        );
    }

    #[test]
    fn tag() {
        let pl = parse_line("#:mytag");
        assert_eq!(pl.kind, BlockKind::Tag("mytag".to_string()));
    }
}
