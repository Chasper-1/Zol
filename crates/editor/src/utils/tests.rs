use super::*;

fn ls(text: &str) -> Vec<usize> {
    let mut v = vec![0usize];
    for (i, c) in text.char_indices() {
        if c == '\n' {
            v.push(i + 1);
        }
    }
    v
}

#[test]
fn empty_text() {
    assert_eq!(count_lines(""), 1);
    assert_eq!(line_bounds("", &[], 0), Some(LineBounds { start: 0, end: 0 }));
    assert_eq!(line_bounds("", &[], 1), None);
    assert_eq!(line_text("", &[], 0), Some(""));
    assert_eq!(line_text("", &[], 1), None);
    assert_eq!(line_of_byte("", &[], 0), 0);
}

#[test]
fn single_line() {
    let text = "hello";
    assert_eq!(count_lines(text), 1);
    assert_eq!(line_bounds(text, &ls(text), 0), Some(LineBounds { start: 0, end: 5 }));
    assert_eq!(line_text(text, &ls(text), 0), Some("hello"));
    assert_eq!(line_bounds(text, &ls(text), 1), None);
}

#[test]
fn two_lines() {
    let text = "abc\ndef";
    assert_eq!(count_lines(text), 2);
    assert_eq!(line_bounds(text, &ls(text), 0), Some(LineBounds { start: 0, end: 3 }));
    assert_eq!(line_bounds(text, &ls(text), 1), Some(LineBounds { start: 4, end: 7 }));
    assert_eq!(line_text(text, &ls(text), 0), Some("abc"));
    assert_eq!(line_text(text, &ls(text), 1), Some("def"));
}

#[test]
fn trailing_newline() {
    let text = "a\n";
    assert_eq!(count_lines(text), 2);
    assert_eq!(line_text(text, &ls(text), 0), Some("a"));
    assert_eq!(line_text(text, &ls(text), 1), Some(""));
}

#[test]
fn only_newlines() {
    let text = "\n";
    assert_eq!(count_lines(text), 2);
    assert_eq!(line_text(text, &ls(text), 0), Some(""));
    assert_eq!(line_text(text, &ls(text), 1), Some(""));
}

#[test]
fn line_of_byte_works() {
    let text = "abc\ndef\nghi";
    let l = ls(text);
    assert_eq!(line_of_byte(text, &l, 0), 0);
    assert_eq!(line_of_byte(text, &l, 1), 0);
    assert_eq!(line_of_byte(text, &l, 3), 0);
    assert_eq!(line_of_byte(text, &l, 4), 1);
    assert_eq!(line_of_byte(text, &l, 7), 1);
    assert_eq!(line_of_byte(text, &l, 8), 2);
}

#[test]
fn unicode_text() {
    let text = "Привет\nМир";
    let l = ls(text);
    assert_eq!(count_lines(text), 2);
    assert_eq!(line_text(text, &l, 0), Some("Привет"));
    assert_eq!(line_text(text, &l, 1), Some("Мир"));
    let bounds0 = line_bounds(text, &l, 0).unwrap();
    assert_eq!(&text[bounds0.start..bounds0.end], "Привет");
}
