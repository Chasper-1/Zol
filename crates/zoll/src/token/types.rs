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
