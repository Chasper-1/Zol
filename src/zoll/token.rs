//! Токенизатор zoll.
//!
//! Преобразует сырой текст в последовательность токенов.
//! Один проход по тексту, без рекурсии.

use super::ast::{MARKERS, MarkerDef, MarkupStyle};

/// Токен разметки.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// Фрагмент обычного текста.
    Text(String),
    /// Открывающий маркер с указанием стиля.
    Open(MarkupStyle),
    /// Закрывающий маркер с указанием стиля.
    Close(MarkupStyle),
    /// Перевод строки.
    Newline,
}

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

        // Обычный символ — аккумулируем текст
        let ch = text[pos..].chars().next().unwrap();
        let ch_len = ch.len_utf8();
        // Собираем текст до следующего маркера, escape или \n
        let text_start = pos;
        pos += ch_len;
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

// —————— внутренние утилиты ——————

/// Нативный поиск следующего символа `'\n'` начиная с байта `from`.
///
/// `'\n'` — ASCII (`0x0A`), поэтому побайтовый проход корректен в UTF-8:
/// этот байт никогда не встречается внутри многобайтового символа.
/// Не использует `str::find`, чтобы не создавать подстроку и не считать
/// относительное смещение (`open_end + p`).
fn next_newline(text: &str, from: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut i = from;
    while i < bytes.len() {
        if bytes[i] == b'\n' {
            return Some(i);
        }
        i += 1;
    }
    None
}

/// Находит индекс первого маркера, совпадающего с текущей позицией.
fn find_any_marker(text: &str, pos: usize) -> Option<usize> {
    let tail = &text[pos..];
    MARKERS.iter().position(|m| tail.starts_with(m.open))
}

/// Проверка, что в позиции стоит пробельный символ (или конец текста).
fn is_whitespace_at(text: &str, pos: usize) -> bool {
    text[pos..]
        .chars()
        .next()
        .map_or(true, |c| c.is_ascii_whitespace())
}

/// Проверка, что перед позицией стоит пробельный символ.
fn is_whitespace_before(text: &str, pos: usize) -> bool {
    if pos == 0 {
        return true;
    }
    text[..pos]
        .chars()
        .next_back()
        .map_or(true, |c| c.is_ascii_whitespace())
}

/// Поиск закрывающего маркера с учётом вложенности (для open == close).
fn find_deep_close(text: &str, range: std::ops::Range<usize>, marker: &MarkerDef) -> Option<usize> {
    let mut depth = 1u32;
    let mut pos = range.start;
    let end = range.end;

    while pos < end {
        let tail = &text[pos..end];

        // Если open совпадает с close И включён трекинг вложенности — увеличиваем глубину
        if marker.track_depth && marker.open == marker.close && tail.starts_with(marker.open) {
            let after = pos + marker.open.len();
            if after <= end && !is_whitespace_at(text, after) {
                depth += 1;
                pos += marker.open.len();
                continue;
            }
        }

        if tail.starts_with(marker.close) {
            if pos > range.start && !is_whitespace_before(text, pos) {
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

fn push_text_token(tokens: &mut Vec<Token>, text: &str) {
    if !text.is_empty() {
        // Если последний токен — текст, добавляем к нему
        if let Some(Token::Text(existing)) = tokens.last_mut() {
            existing.push_str(text);
        } else {
            tokens.push(Token::Text(text.to_string()));
        }
    }
}

/// Принудительно завершает текущий текстовый буфер (разделяет текстовые токены).
fn flush_text(_tokens: &mut Vec<Token>) {
    // Ничего не делаем — текстовые токены и так самосливаются.
    // Этот хук нужен, если мы захотим принудительно разорвать текст.
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::zoll::ast::MarkupStyle;

    fn text(s: &str) -> Token {
        Token::Text(s.to_string())
    }
    fn open(s: MarkupStyle) -> Token {
        Token::Open(s)
    }
    fn close(s: MarkupStyle) -> Token {
        Token::Close(s)
    }

    #[test]
    fn plain_text() {
        assert_eq!(tokenize("hello"), vec![text("hello")]);
    }

    #[test]
    fn simple_bold() {
        let t = tokenize("**bold**");
        assert_eq!(
            t,
            vec![
                open(MarkupStyle::BOLD),
                text("bold"),
                close(MarkupStyle::BOLD)
            ]
        );
    }

    #[test]
    fn bold_with_leading_text() {
        let t = tokenize("a **bold** b");
        assert_eq!(
            t,
            vec![
                text("a "),
                open(MarkupStyle::BOLD),
                text("bold"),
                close(MarkupStyle::BOLD),
                text(" b"),
            ]
        );
    }

    #[test]
    fn no_close_treated_as_text() {
        let t = tokenize("**bold");
        // **bold не имеет закрытия — весь кусок остаётся текстом
        assert_eq!(t, vec![text("**bold")]);
    }

    #[test]
    fn space_after_open_invalid() {
        let t = tokenize("** bold**");
        assert_eq!(t, vec![text("** bold**")]);
    }

    #[test]
    fn space_before_close_invalid() {
        let t = tokenize("**bold **");
        assert_eq!(t, vec![text("**bold **")]);
    }

    #[test]
    fn nested_bold_italic() {
        let t = tokenize("**a //b// c**");
        assert_eq!(
            t,
            vec![
                open(MarkupStyle::BOLD),
                text("a "),
                open(MarkupStyle::ITALIC),
                text("b"),
                close(MarkupStyle::ITALIC),
                text(" c"),
                close(MarkupStyle::BOLD),
            ]
        );
    }

    #[test]
    fn escape_disables_marker() {
        let t = tokenize(r"\*\*text\*\*");
        // Каждый \X — это один текст: \* — два раза, text — текст
        assert!(t.iter().all(|tok| matches!(tok, Token::Text(_))));
        let result: String = t
            .iter()
            .map(|tok| match tok {
                Token::Text(s) => s.as_str(),
                _ => "",
            })
            .collect();
        assert_eq!(result, "**text**");
    }

    #[test]
    fn newline_separates() {
        let t = tokenize("line1\nline2");
        assert_eq!(t, vec![text("line1"), Token::Newline, text("line2")]);
    }

    #[test]
    fn spoiler_inline() {
        let t = tokenize("!!secret!!");
        assert_eq!(
            t,
            vec![
                open(MarkupStyle::SPOILER),
                text("secret"),
                close(MarkupStyle::SPOILER)
            ]
        );
    }

    #[test]
    fn spoiler_block_multiline() {
        let t = tokenize("!!!hidden\ncontent!!!");
        assert_eq!(
            t,
            vec![
                open(MarkupStyle::SPOILER_BLOCK),
                text("hidden"),
                Token::Newline,
                text("content"),
                close(MarkupStyle::SPOILER_BLOCK),
            ]
        );
    }
}
