use crate::ast::MARKERS;
use crate::token::find_deep_close::find_deep_close;
use crate::token::helpers::{find_any_marker, is_whitespace_at, is_whitespace_before, next_newline};
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

// ─── Основной токенизатор ─────────────────────────────────────────

fn tokenize_impl(text: &str, range: Range<usize>) -> Vec<SpannedToken> {
    let bytes = text.as_bytes();
    let end = range.end;
    let mut pos = range.start;
    let mut tokens: Vec<SpannedToken> = Vec::new();
    let mut stack: Vec<Frame> = Vec::new();
    // Трекер накопления текста: когда встречаем немаркерный символ,
    // запоминаем начало, идём дальше, при маркере/escape/\n сбрасываем.
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

        // ── Экранирование ──
        if bytes[pos] == b'\\' && pos + 1 < end {
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
        if bytes[pos] == b'\n' {
            flush_text_run(&mut tokens, text, text_start.take(), pos);
            tokens.push(SpannedToken::new(Token::Newline, pos, pos + 1));
            pos += 1;
            continue;
        }

        // ── Поиск маркера ──
        if let Some(m_idx) = find_any_marker(text, pos) {
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
                    if let Some(close_pos) = find_deep_close(text, open_end..search_end, marker) {
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
        let ch = text[pos..].chars().next().unwrap();
        pos += ch.len_utf8();
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
