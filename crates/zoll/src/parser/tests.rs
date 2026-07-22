use super::*;
use crate::token::tokenize;
use crate::ast::{MarkupDoc, MarkupNode, MarkupStyle};

fn flatten_text(node: &MarkupNode) -> String {
    match node {
        MarkupNode::Text(t) => t.clone(),
        MarkupNode::Formatted { children, .. } => children.iter().map(flatten_text).collect(),
    }
}

fn collect_text(doc: &MarkupDoc) -> String {
    doc.children.iter().map(flatten_text).collect()
}

#[test]
fn plain_text_roundtrip() {
    let tokens = tokenize("hello world");
    let doc = parse(&tokens);
    assert_eq!(collect_text(&doc), "hello world");
}

#[test]
fn bold_roundtrip() {
    let tokens = tokenize("**bold**");
    let doc = parse(&tokens);
    assert_eq!(collect_text(&doc), "bold");
}

#[test]
fn bold_with_text() {
    let tokens = tokenize("a **bold** b");
    let doc = parse(&tokens);
    assert_eq!(collect_text(&doc), "a bold b");
    assert_eq!(doc.children.len(), 3);
    assert!(matches!(doc.children[0], MarkupNode::Text(_)));
    assert!(matches!(doc.children[1], MarkupNode::Formatted { .. }));
    assert!(matches!(doc.children[2], MarkupNode::Text(_)));
}

#[test]
fn nested_bold_italic() {
    let tokens = tokenize("**a //b// c**");
    let doc = parse(&tokens);
    assert_eq!(collect_text(&doc), "a b c");
    if let MarkupNode::Formatted { style, children } = &doc.children[0] {
        assert_eq!(*style, MarkupStyle::BOLD);
        assert_eq!(children.len(), 3);
        if let MarkupNode::Formatted { style, .. } = &children[1] {
            assert_eq!(*style, MarkupStyle::ITALIC);
        } else {
            panic!("expected italic");
        }
    } else {
        panic!("expected bold");
    }
}

#[test]
fn no_close_returns_plain() {
    let tokens = tokenize("**bold");
    let doc = parse(&tokens);
    assert_eq!(collect_text(&doc), "**bold");
}

#[test]
fn same_type_nesting() {
    let tokens = tokenize("**a **b** c**");
    let doc = parse(&tokens);
    assert_eq!(collect_text(&doc), "a b c");
    if let MarkupNode::Formatted { style, children } = &doc.children[0] {
        assert_eq!(*style, MarkupStyle::BOLD);
        assert_eq!(children.len(), 3);
        if let MarkupNode::Formatted { style, .. } = &children[1] {
            assert_eq!(*style, MarkupStyle::BOLD);
        } else {
            panic!("expected nested bold");
        }
    } else {
        panic!("expected bold");
    }
}

#[test]
fn spoiler_inline() {
    let tokens = tokenize("!!secret!!");
    let doc = parse(&tokens);
    assert_eq!(collect_text(&doc), "secret");
    if let MarkupNode::Formatted { style, .. } = &doc.children[0] {
        assert_eq!(*style, MarkupStyle::SPOILER);
    } else {
        panic!("expected spoiler");
    }
}

#[test]
fn comment_is_marked() {
    let tokens = tokenize("before %%comment%% after");
    let doc = parse(&tokens);
    assert_eq!(collect_text(&doc), "before comment after");
}
