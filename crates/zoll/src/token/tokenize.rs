use crate::ast::MARKERS;
use crate::token::find_deep_close::find_deep_close;
use crate::token::helpers::{
    is_special_byte, is_whitespace_at, is_whitespace_before, next_newline, utf8_char_len,
};
use crate::token::push_text::push_text_token;
use crate::token::types::{SpannedToken, Token};
use std::ops::Range;

/// Токенизирует текст, возвращая токены с байтовыми позициями.
pub fn tokenize(text: &str) -> Vec<SpannedToken> {
    tokenize_impl(text, 0..text.len())
}

/// Токенизирует диапазон текста (для инкрементального режима).
pub fn tokenize_range(text: &str, range: Range<usize>) -> Vec<SpannedToken> {
    tokenize_impl(text, range)
}

// ─── Фрейм для стекового обхода ───────────────────────────────────

/// Когда встречаем валидную пару Open/Close, пушим фрейм.
/// При достижении `close_pos` эмитим Close и фрейм снимается.
struct Frame {
    style: crate::ast::MarkupStyle,
    /// Позиция закрывающего маркера (его первый байт).
    close_pos: usize,
    /// Длина закрывающего маркера в байтах.
    close_len: usize,
}

/// Индекс маркера в MARKERS.
/// Используется для быстрого switch/dispatch по первому байту.
/// -1: не маркер.
fn match_marker_idx(bytes: &[u8], pos: usize, end: usize) -> Option<usize> {
    let b = bytes.get(pos)?;
    let remaining = end - pos;
    match *b {
        b'%' if remaining >= 2 && bytes[pos + 1] == b'%' => Some(0),  // COMMENT %%
        b'$' if remaining >= 2 && bytes[pos + 1] == b'$' => Some(1),  // DISPLAY_FORMULA $$
        b'!' if remaining >= 3 && bytes[pos + 1] == b'!' && bytes[pos + 2] == b'!' => Some(2), // SPOILER_BLOCK !!!
        b'!' if remaining >= 2 && bytes[pos + 1] == b'!' => Some(3),  // SPOILER !!
        b'/' if remaining >= 2 && bytes[pos + 1] == b'/' => Some(4),  // ITALIC //
        b'*' if remaining >= 2 && bytes[pos + 1] == b'*' => Some(5),  // BOLD **
        b'_' if remaining >= 2 && bytes[pos + 1] == b'_' => Some(6),  // UNDERLINE __
        b'\'' if remaining >= 2 && bytes[pos + 1] == b'\'' => Some(7), // SUPERSCRIPT ''
        b',' if remaining >= 2 && bytes[pos + 1] == b',' => Some(8),  // SUBSCRIPT ,,
        b'~' if remaining >= 2 && bytes[pos + 1] == b'~' => Some(9),  // STRIKETHROUGH ~~
        b'=' if remaining >= 2 && bytes[pos + 1] == b'=' => Some(10), // HIGHLIGHT ==
        b'+' if remaining >= 2 && bytes[pos + 1] == b'+' => Some(11), // INSERTION ++
        b'-' if remaining >= 2 && bytes[pos + 1] == b'-' => Some(12), // DELETION --
        b'$' => Some(13), // FORMULA $ (одиночный — только если не $$)
        _ => None,
    }
}

// ─── Основной токенизатор ─────────────────────────────────────────

fn tokenize_impl(text: &str, range: Range<usize>) -> Vec<SpannedToken> {
    let bytes = text.as_bytes();
    let end = range.end.min(bytes.len());
    let mut pos = range.start;
    // Pre-alloc: для текста со множеством маркеров снижает реаллокации
    let estimated_tokens = (end - pos) / 8 + 16;
    let mut tokens: Vec<SpannedToken> = Vec::with_capacity(estimated_tokens);
    let mut stack: Vec<Frame> = Vec::new();
    let mut text_start: Option<usize> = None;

    while pos < end {
        // ── Шаг 0: проверка, не дошли ли до Close активного фрейма ──
        if let Some(frame) = stack.last() {
            if pos == frame.close_pos {
                flush_text_run(&mut tokens, text, text_start.take(), pos);
                tokens.push(SpannedToken::new(
                    Token::Close(frame.style),
                    pos,
                    pos + frame.close_len,
                ));
                pos += frame.close_len;
                stack.pop();
                continue;
            }
        }

        let b = bytes[pos];

        // ── Шаг 0.5: batch-skip plain text ──
        // Если байт не специальный — весь блок до следующего special
        // накапливаем как текст и прыгаем сразу туда.
        if !is_special_byte(b) {
            if text_start.is_none() {
                text_start = Some(pos);
            }
            // Ищем следующий special-байт (включая \n, \\, первые байты маркеров)
            let skip = bytes[pos + 1..end]
                .iter()
                .position(|&b| is_special_byte(b))
                .map(|offset| offset + 1) // +1 потому что мы с pos+1 начали
                .unwrap_or(end - pos);
            pos += skip;
            continue;
        }

        // ── Экранирование ──
        if b == b'\\' && pos + 1 < end {
            if let Some(ch) = text[pos + 1..].chars().next() {
                let ch_len = ch.len_utf8();
                flush_text_run(&mut tokens, text, text_start.take(), pos);
                tokens.push(SpannedToken::new(
                    Token::Text(ch.to_string()),
                    pos + 1,
                    pos + 1 + ch_len,
                ));
                pos += 1 + ch_len;
                continue;
            }
        }

        // ── Перевод строки ──
        if b == b'\n' {
            flush_text_run(&mut tokens, text, text_start.take(), pos);
            tokens.push(SpannedToken::new(Token::Newline, pos, pos + 1));
            pos += 1;
            continue;
        }

        // ── Поиск маркера ──
        if let Some(m_idx) = match_marker_idx(bytes, pos, end) {
            let marker = &MARKERS[m_idx];
            let open_end = pos + marker.open.len();

            if open_end <= end && !is_whitespace_at(text, open_end) {
                // Граница поиска close: не дальше конца строки (для inline)
                // и не дальше close_pos активного фрейма (чтобы не вылезти наружу).
                let search_end = if let Some(f) = stack.last() {
                    let line_end = if marker.multiline {
                        end
                    } else {
                        next_newline(text, open_end).unwrap_or(end)
                    };
                    line_end.min(f.close_pos)
                } else {
                    if marker.multiline {
                        end
                    } else {
                        next_newline(text, open_end).unwrap_or(end)
                    }
                };

                if search_end > open_end {
                    if let Some(close_pos) =
                        find_deep_close(text, open_end..search_end, marker)
                    {
                        if close_pos > open_end && !is_whitespace_before(text, close_pos) {
                            // Нашли валидную пару — эмитим Open, пушим фрейм
                            flush_text_run(&mut tokens, text, text_start.take(), pos);
                            tokens.push(SpannedToken::new(
                                Token::Open(marker.style),
                                pos,
                                open_end,
                            ));
                            stack.push(Frame {
                                style: marker.style,
                                close_pos,
                                close_len: marker.close.len(),
                            });
                            pos = open_end;
                            continue;
                        }
                    }
                }
            }
        }

        // ── Накопление обычного текста ──
        if text_start.is_none() {
            text_start = Some(pos);
        }
        pos += utf8_char_len(bytes[pos]);
    }

    // ── Финализация ──
    flush_text_run(&mut tokens, text, text_start.take(), pos);

    // Незакрытые маркеры на стеке: их Open-токены остаются как есть,
    // парсер сам обработает их как обычный текст (через marker_text_for_close).
    tokens
}

// ─── Утилиты ──────────────────────────────────────────────────────

/// Сбрасывает накопленный текст (от `start` до `end`) в один токен.
fn flush_text_run(
    tokens: &mut Vec<SpannedToken>,
    text: &str,
    start: Option<usize>,
    end: usize,
) {
    let start = match start {
        Some(s) => s,
        None => return,
    };
    if end > start {
        push_text_token(tokens, &text[start..end], start, end);
    }
}
