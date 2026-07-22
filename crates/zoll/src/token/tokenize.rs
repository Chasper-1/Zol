use crate::ast::MARKERS;
use crate::token::push_text::{flush_text, push_text_token};
use crate::token::types::Token;
use crate::token::find_deep_close::find_deep_close;
use crate::token::helpers::{find_any_marker, is_whitespace_at, is_whitespace_before, next_newline};

/// Токенизирует текст, возвращая список токенов.
pub fn tokenize(text: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let bytes = text.as_bytes();
    let len = text.len();
    let mut pos = 0;

    while pos < len {
        // Escape-последовательность: \X → X (без обратной косой черты)
        if bytes[pos] == b'\\' && pos + 1 < len {
            if let Some(ch) = text[pos + 1..].chars().next() {
                let ch_len = ch.len_utf8();
                push_text_token(&mut tokens, &text[pos + 1..pos + 1 + ch_len]);
                pos += 1 + ch_len;
                continue;
            }
        }

        // Перевод строки
        if bytes[pos] == b'\n' {
            flush_text(&mut tokens);
            tokens.push(Token::Newline);
            pos += 1;
            continue;
        }

        // Поиск маркера
        if let Some(m_idx) = find_any_marker(text, pos) {
            let marker = &MARKERS[m_idx];

            // Проверяем, что после открывающего маркера нет пробела
            let open_end = pos + marker.open.len();
            if open_end <= len && !is_whitespace_at(text, open_end) {
                // Ищем закрывающий маркер
                let search_end = if marker.multiline {
                    len
                } else {
                    next_newline(text, open_end).unwrap_or(len)
                };

                if let Some(close_pos) = find_deep_close(text, open_end..search_end, marker) {
                    // Проверяем, что перед закрывающим маркером нет пробела
                    if close_pos > open_end && !is_whitespace_before(text, close_pos) {
                        flush_text(&mut tokens);
                        tokens.push(Token::Open(marker.style));
                        // Токенизируем содержимое между open и close рекурсивно
                        tokens.extend(tokenize(&text[open_end..close_pos]));
                        tokens.push(Token::Close(marker.style));
                        pos = close_pos + marker.close.len();
                        continue;
                    }
                }
            }
        }

        // Обычный символ — аккумулируем текст до следующего маркера, escape или \n
        let text_start = pos;
        let ch = text[pos..].chars().next().unwrap();
        pos += ch.len_utf8();
        while pos < len {
            if bytes[pos] == b'\\' || bytes[pos] == b'\n' {
                break;
            }
            if find_any_marker(text, pos).is_some() {
                break;
            }
            let c = text[pos..].chars().next().unwrap();
            pos += c.len_utf8();
        }
        push_text_token(&mut tokens, &text[text_start..pos]);
    }

    flush_text(&mut tokens);
    tokens
}
