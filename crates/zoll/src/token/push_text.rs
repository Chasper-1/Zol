use crate::token::types::Token;

/// Добавляет текст в конец списка токенов, склеивая с последним текстовым токеном.
pub fn push_text_token(tokens: &mut Vec<Token>, text: &str) {
    if !text.is_empty() {
        if let Some(Token::Text(existing)) = tokens.last_mut() {
            existing.push_str(text);
        } else {
            tokens.push(Token::Text(text.to_string()));
        }
    }
}

/// Принудительно завершает текущий текстовый буфер.
/// Сейчас — заглушка; текстовые токены сливаются автоматически.
pub fn flush_text(_tokens: &mut Vec<Token>) {}
