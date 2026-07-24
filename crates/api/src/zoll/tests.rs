use super::*;
use ::zoll;

#[test]
fn zoll_tokenize_plain_text() {
    let tokens = zoll_tokenize("hello world");
    assert_eq!(tokens.len(), 1);
    assert!(matches!(tokens[0].token, zoll::token::Token::Text(_)));
}

#[test]
fn zoll_tokenize_bold() {
    let tokens = zoll_tokenize("**bold**");
    assert_eq!(tokens.len(), 3);
    assert!(matches!(tokens[0].token, zoll::token::Token::Open(_)));
    assert!(matches!(tokens[1].token, zoll::token::Token::Text(_)));
    assert!(matches!(tokens[2].token, zoll::token::Token::Close(_)));
}

#[test]
fn zoll_parse_plain_text() {
    let ast = zoll_parse("hello");
    assert_eq!(ast.children.len(), 1);
    assert!(matches!(&ast.children[0], zoll::ast::MarkupNode::Text(t) if t == "hello"));
}

#[test]
fn zoll_parse_bold() {
    let ast = zoll_parse("**bold**");
    assert_eq!(ast.children.len(), 1);
    match &ast.children[0] {
        zoll::ast::MarkupNode::Formatted { style, children } => {
            assert!(style.contains(zoll::ast::MarkupStyle::BOLD));
            assert_eq!(children.len(), 1);
            assert!(matches!(&children[0], zoll::ast::MarkupNode::Text(t) if t == "bold"));
        }
        _ => panic!("expected Formatted node"),
    }
}

#[test]
fn zoll_parse_empty() {
    let ast = zoll_parse("");
    assert!(ast.children.is_empty());
}

#[test]
fn zoll_parse_cache_created() {
    let cache = zoll_parse_cache("**bold**");
    assert_eq!(cache.lines.len(), 1);
}

#[test]
fn zoll_parse_cache_empty() {
    let cache = zoll_parse_cache("");
    assert!(cache.lines.len() <= 1);
}

#[test]
fn zoll_tokenize_empty() {
    let tokens = zoll_tokenize("");
    assert!(tokens.is_empty());
}

#[test]
fn zoll_tokenize_incomplete_marker() {
    let tokens = zoll_tokenize("**bold");
    assert!(!tokens.is_empty(), "should have at least one token");
}
