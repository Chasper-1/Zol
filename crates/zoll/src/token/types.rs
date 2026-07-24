use crate::ast::MarkupStyle;

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

/// Токен с привязкой к исходному тексту (байтовые позиции).
///
/// Нужен для инкрементального парсинга: позволяет понять, какой токен
/// соответствует какому участку исходного текста.
#[derive(Debug, Clone)]
pub struct SpannedToken {
    pub token: Token,
    /// Байтовое смещение начала токена в исходном тексте.
    pub start: usize,
    /// Байтовое смещение конца токена (не включая).
    pub end: usize,
}

impl SpannedToken {
    /// Создать spanned-токен с позициями.
    pub fn new(token: Token, start: usize, end: usize) -> Self {
        Self { token, start, end }
    }

    /// Ссылка на внутренний токен (для парсера, которому позиции не нужны).
    pub fn as_token(&self) -> &Token {
        &self.token
    }
}
