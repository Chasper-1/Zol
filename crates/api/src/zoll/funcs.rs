use ::zoll;

/// Токенизирует текст zoll, возвращает токены с позициями.
/// 
/// Устаревший API. Сохранён для совместимости, но теперь
/// использует `parse_full` под капотом.
pub fn zoll_tokenize(text: &str) -> Vec<zoll::ast::MarkupNode> {
    let ast = zoll::parser::parse_full(text);
    ast.children
}

/// Парсит текст zoll в AST.
pub fn zoll_parse(text: &str) -> zoll::ast::MarkupDoc {
    zoll::parser::parse_full(text)
}

/// Парсит текст zoll в DocumentCache редактора.
pub fn zoll_parse_cache(text: &str) -> editor::cache::DocumentCache {
    editor::markup::parse_document(text)
}
