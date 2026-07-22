use super::*;
use crate::ast::MarkupStyle;

fn text(s: &str) -> Token { Token::Text(s.to_string()) }
fn open(s: MarkupStyle) -> Token { Token::Open(s) }
fn close(s: MarkupStyle) -> Token { Token::Close(s) }

#[test]
fn plain_text() {
    assert_eq!(tokenize("hello"), vec![text("hello")]);
}

#[test]
fn simple_bold() {
    let t = tokenize("**bold**");
    assert_eq!(t, vec![open(MarkupStyle::BOLD), text("bold"), close(MarkupStyle::BOLD)]);
}

#[test]
fn bold_with_leading_text() {
    let t = tokenize("a **bold** b");
    assert_eq!(t, vec![text("a "), open(MarkupStyle::BOLD), text("bold"), close(MarkupStyle::BOLD), text(" b")]);
}

#[test]
fn no_close_treated_as_text() {
    let t = tokenize("**bold");
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
    assert_eq!(t, vec![
        open(MarkupStyle::BOLD), text("a "),
        open(MarkupStyle::ITALIC), text("b"), close(MarkupStyle::ITALIC),
        text(" c"), close(MarkupStyle::BOLD),
    ]);
}

#[test]
fn escape_disables_marker() {
    let t = tokenize(r"\*\*text\*\*");
    assert!(t.iter().all(|tok| matches!(tok, Token::Text(_))));
    let result: String = t.iter().map(|tok| match tok {
        Token::Text(s) => s.as_str(),
        _ => "",
    }).collect();
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
    assert_eq!(t, vec![open(MarkupStyle::SPOILER), text("secret"), close(MarkupStyle::SPOILER)]);
}

#[test]
fn spoiler_block_multiline() {
    let t = tokenize("!!!hidden\ncontent!!!");
    assert_eq!(t, vec![
        open(MarkupStyle::SPOILER_BLOCK),
        text("hidden"), Token::Newline, text("content"),
        close(MarkupStyle::SPOILER_BLOCK),
    ]);
}
