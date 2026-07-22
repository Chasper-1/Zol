use ::zoll;

/// Токенизирует текст zoll.
pub fn zoll_tokenize(text: &str) -> Vec<zoll::token::Token> {
    zoll::token::tokenize(text)
}

/// Парсит текст zoll в AST.
pub fn zoll_parse(text: &str) -> zoll::ast::MarkupDoc {
    let tokens = zoll::token::tokenize(text);
    zoll::parser::parse(&tokens)
}

/// Парсит текст zoll в DocumentCache редактора.
pub fn zoll_parse_cache(text: &str) -> editor::cache::DocumentCache {
    editor::markup::parse_document(text)
}
