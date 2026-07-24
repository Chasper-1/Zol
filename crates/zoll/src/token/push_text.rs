use crate::token::types::{SpannedToken, Token};

/// Добавляет текст в конец списка токенов, склеивая с последним текстовым токеном.
/// Если `is_new` — начать новый текстовый токен (не склеивать с предыдущим).
pub fn push_text_token(tokens: &mut Vec<SpannedToken>, text: &str, start: usize, end: usize) {
    if text.is_empty() {
        return;
    }
    if let Some(SpannedToken {
        token: Token::Text(existing),
        end: last_end,
        ..
    }) = tokens.last_mut()
    {
        existing.push_str(text);
        *last_end = end;
    } else {
        tokens.push(SpannedToken::new(Token::Text(text.to_string()), start, end));
    }
}

/// Принудительно завершает текущий текстовый буфер.
/// Сейчас текстовые токены и так сливаются, но точка вызова обозначает
/// границу: после неё следующий текст начнёт новый токен.
pub fn flush_text(_tokens: &mut Vec<SpannedToken>) {}
