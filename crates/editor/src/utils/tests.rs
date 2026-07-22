use super::*;

#[test]
fn empty_text() {
    assert_eq!(count_lines(""), 1);
    assert_eq!(line_bounds("", 0), Some(LineBounds { start: 0, end: 0 }));
    assert_eq!(line_bounds("", 1), None);
    assert_eq!(line_text("", 0), Some(""));
    assert_eq!(line_text("", 1), None);
    assert_eq!(line_of_byte("", 0), 0);
}

#[test]
fn single_line() {
    assert_eq!(count_lines("hello"), 1);
    assert_eq!(line_bounds("hello", 0), Some(LineBounds { start: 0, end: 5 }));
    assert_eq!(line_text("hello", 0), Some("hello"));
    assert_eq!(line_bounds("hello", 1), None);
}

#[test]
fn two_lines() {
    assert_eq!(count_lines("abc\ndef"), 2);
    assert_eq!(line_bounds("abc\ndef", 0), Some(LineBounds { start: 0, end: 3 }));
    assert_eq!(line_bounds("abc\ndef", 1), Some(LineBounds { start: 4, end: 7 }));
    assert_eq!(line_text("abc\ndef", 0), Some("abc"));
    assert_eq!(line_text("abc\ndef", 1), Some("def"));
}

#[test]
fn trailing_newline() {
    assert_eq!(count_lines("a\n"), 2);
    assert_eq!(line_text("a\n", 0), Some("a"));
    assert_eq!(line_text("a\n", 1), Some(""));
}

#[test]
fn only_newlines() {
    assert_eq!(count_lines("\n"), 2);
    assert_eq!(line_text("\n", 0), Some(""));
    assert_eq!(line_text("\n", 1), Some(""));
}

#[test]
fn line_of_byte_works() {
    let text = "abc\ndef\nghi";
    assert_eq!(line_of_byte(text, 0), 0);
    assert_eq!(line_of_byte(text, 1), 0);
    assert_eq!(line_of_byte(text, 3), 0);
    assert_eq!(line_of_byte(text, 4), 1);
    assert_eq!(line_of_byte(text, 7), 1);
    assert_eq!(line_of_byte(text, 8), 2);
}

#[test]
fn unicode_text() {
    let text = "Привет\nМир";
    assert_eq!(count_lines(text), 2);
    assert_eq!(line_text(text, 0), Some("Привет"));
    assert_eq!(line_text(text, 1), Some("Мир"));
    let bounds0 = line_bounds(text, 0).unwrap();
    assert_eq!(&text[bounds0.start..bounds0.end], "Привет");
}
