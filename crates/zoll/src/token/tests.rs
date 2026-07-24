use super::*;
use crate::ast::MarkupStyle;

/// Создаёт SpannedToken::Text без учёта позиций (для тестов).
fn text(s: &str) -> SpannedToken {
    SpannedToken::new(Token::Text(s.to_string()), 0, 0)
}

fn open(s: MarkupStyle) -> SpannedToken {
    SpannedToken::new(Token::Open(s), 0, 0)
}

fn close(s: MarkupStyle) -> SpannedToken {
    SpannedToken::new(Token::Close(s), 0, 0)
}

fn newline() -> SpannedToken {
    SpannedToken::new(Token::Newline, 0, 0)
}

/// Извлекает токены без позиций для сравнения.
fn strip(tokens: &[SpannedToken]) -> Vec<Token> {
    tokens.iter().map(|st| st.token.clone()).collect()
}

#[test]
fn plain_text() {
    assert_eq!(
        strip(&tokenize("hello")),
        strip(&[text("hello")]),
    );
}

#[test]
fn simple_bold() {
    assert_eq!(
        strip(&tokenize("**bold**")),
        strip(&[open(MarkupStyle::BOLD), text("bold"), close(MarkupStyle::BOLD)]),
    );
}

#[test]
fn bold_with_leading_text() {
    assert_eq!(
        strip(&tokenize("a **bold** b")),
        strip(&[text("a "), open(MarkupStyle::BOLD), text("bold"), close(MarkupStyle::BOLD), text(" b")]),
    );
}

#[test]
fn no_close_treated_as_text() {
    assert_eq!(
        strip(&tokenize("**bold")),
        strip(&[text("**bold")]),
    );
}

#[test]
fn space_after_open_invalid() {
    assert_eq!(
        strip(&tokenize("** bold**")),
        strip(&[text("** bold**")]),
    );
}

#[test]
fn space_before_close_invalid() {
    assert_eq!(
        strip(&tokenize("**bold **")),
        strip(&[text("**bold **")]),
    );
}

#[test]
fn nested_bold_italic() {
    assert_eq!(
        strip(&tokenize("**a //b// c**")),
        strip(&[
            open(MarkupStyle::BOLD),
            text("a "),
            open(MarkupStyle::ITALIC),
            text("b"),
            close(MarkupStyle::ITALIC),
            text(" c"),
            close(MarkupStyle::BOLD),
        ]),
    );
}

#[test]
fn escape_disables_marker() {
    let t = tokenize(r"\*\*text\*\*");
    assert!(t.iter().all(|st| matches!(st.token, Token::Text(_))));
    let result: String = t.iter().map(|st| match &st.token {
        Token::Text(s) => s.as_str(),
        _ => "",
    }).collect();
    assert_eq!(result, "**text**");
}

#[test]
fn newline_separates() {
    assert_eq!(
        strip(&tokenize("line1\nline2")),
        strip(&[text("line1"), newline(), text("line2")]),
    );
}

#[test]
fn spoiler_inline() {
    assert_eq!(
        strip(&tokenize("!!secret!!")),
        strip(&[open(MarkupStyle::SPOILER), text("secret"), close(MarkupStyle::SPOILER)]),
    );
}

#[test]
fn spoiler_block_multiline() {
    assert_eq!(
        strip(&tokenize("!!!hidden\ncontent!!!")),
        strip(&[
            open(MarkupStyle::SPOILER_BLOCK),
            text("hidden"),
            newline(),
            text("content"),
            close(MarkupStyle::SPOILER_BLOCK),
        ]),
    );
}

// ── Тесты позиций SpannedToken ─────────────────────────────────

#[test]
fn spanned_positions_simple() {
    let t = tokenize("**x**");
    assert_eq!(t.len(), 3);
    assert_eq!(t[0].start, 0);
    assert_eq!(t[0].end, 2);
    assert_eq!(t[0].token, Token::Open(MarkupStyle::BOLD));
    assert_eq!(t[1].start, 2);
    assert_eq!(t[1].end, 3);
    assert_eq!(t[1].token, Token::Text("x".to_string()));
    assert_eq!(t[2].start, 3);
    assert_eq!(t[2].end, 5);
    assert_eq!(t[2].token, Token::Close(MarkupStyle::BOLD));
}

#[test]
fn spanned_positions_with_leading_text() {
    // "a **b** c" — байты:
    // 0:'a' 1:' ' 2:'*' 3:'*' 4:'b' 5:'*' 6:'*' 7:' ' 8:'c'
    let t = tokenize("a **b** c");
    assert_eq!(t.len(), 5);
    // Text("a ")
    assert_eq!(t[0].start, 0);
    assert_eq!(t[0].end, 2);
    // Open(BOLD) = "**"
    assert_eq!(t[1].start, 2);
    assert_eq!(t[1].end, 4);
    // Text("b")
    assert_eq!(t[2].start, 4);
    assert_eq!(t[2].end, 5);
    // Close(BOLD) = "**"
    assert_eq!(t[3].start, 5);
    assert_eq!(t[3].end, 7);
    // Text(" c")
    assert_eq!(t[4].start, 7);
    assert_eq!(t[4].end, 9);
}

#[test]
fn spanned_positions_newline() {
    let t = tokenize("ab\ncd");
    assert_eq!(t.len(), 3);
    assert_eq!(t[0].start, 0);
    assert_eq!(t[0].end, 2);
    assert_eq!(t[1].start, 2);
    assert_eq!(t[1].end, 3);
    assert_eq!(t[2].start, 3);
    assert_eq!(t[2].end, 5);
}
