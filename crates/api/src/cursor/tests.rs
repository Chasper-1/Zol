use editor::document::Document;
use super::*;

fn make_doc(text: &str) -> Document {
    Document::new(text)
}

// ── move_left / move_right ───────────────────────────────────────────

#[test]
fn move_left_basic() {
    let mut d = make_doc("abc");
    d.set_cursor_raw( 2);
    move_left(&mut d);
    assert_eq!(d.cursor.raw(), 1);
    assert_eq!(d.cursor.line(), 0);
}

#[test]
fn move_left_at_start() {
    let mut d = make_doc("abc");
    move_left(&mut d);
    assert_eq!(d.cursor.raw(), 0);
}

#[test]
fn move_right_basic() {
    let mut d = make_doc("abc");
    move_right(&mut d);
    assert_eq!(d.cursor.raw(), 1);
}

#[test]
fn move_right_at_end() {
    let mut d = make_doc("abc");
    d.set_cursor_raw( 3);
    move_right(&mut d);
    assert_eq!(d.cursor.raw(), 3);
}

#[test]
fn move_left_grapheme() {
    let mut d = make_doc("e\u{0301}x");
    d.set_cursor_raw( 3);
    move_left(&mut d);
    assert_eq!(d.cursor.raw(), 0);
}

#[test]
fn move_right_grapheme() {
    let mut d = make_doc("e\u{0301}x");
    move_right(&mut d);
    assert_eq!(d.cursor.raw(), 3);
}

// ── move_home / move_end ─────────────────────────────────────────────

#[test]
fn move_home_works() {
    let mut d = make_doc("hello world");
    d.set_cursor_raw( 5);
    move_home(&mut d);
    assert_eq!(d.cursor.raw(), 0);
}

#[test]
fn move_end_works() {
    let mut d = make_doc("hello world");
    move_end(&mut d);
    assert_eq!(d.cursor.raw(), 11);
}

// ── move_up / move_down ──────────────────────────────────────────────

#[test]
fn move_up_to_prev_line() {
    let mut d = make_doc("first\nsecond");
    d.set_cursor_raw( 8);
    move_up(&mut d);
    assert_eq!(d.cursor.line(), 0);
}

#[test]
fn move_down_to_next_line() {
    let mut d = make_doc("first\nsecond");
    move_down(&mut d);
    assert_eq!(d.cursor.line(), 1);
}

// ── move_word_left / move_word_right ─────────────────────────────────

#[test]
fn move_word_left_works() {
    let mut d = make_doc("hello world foo");
    d.set_cursor_raw( 16);
    move_word_left(&mut d);
    assert_eq!(d.cursor.raw(), 12);
}

#[test]
fn move_word_right_works() {
    let mut d = make_doc("hello world");
    move_word_right(&mut d);
    assert_eq!(d.cursor.raw(), 6);
}

// ── Прямые доступоры ─────────────────────────────────────────────────

#[test]
fn cursor_raw_getter() {
    let d = make_doc("abc");
    assert_eq!(cursor_raw(&d), 0);
}

#[test]
fn cursor_raw_setter() {
    let mut d = make_doc("abc\ndef");
    cursor_set_raw(&mut d, 5);
    assert_eq!(cursor_raw(&d), 5);
}

#[test]
fn cursor_line_getter() {
    let mut d = make_doc("a\nb\nc");
    d.set_cursor_raw( 3);
    assert_eq!(cursor_line(&d), 1);
}

#[test]
fn cursor_line_setter() {
    let mut d = make_doc("a\nb\nc");
    cursor_set_line(&mut d, 2);
    assert_eq!(cursor_line(&d), 2);
}

#[test]
fn cursor_col_getter() {
    let mut d = make_doc("hello");
    cursor_set_raw(&mut d, 3);
    assert_eq!(cursor_col(&d), 0.0); // col_visual сброшен set_raw
}

#[test]
fn cursor_col_setter() {
    let mut d = make_doc("hello");
    cursor_set_col(&mut d, 42.0);
    assert_eq!(cursor_col(&d), 42.0);
}

#[test]
fn cursor_reset_col_works() {
    let mut d = make_doc("hello");
    cursor_set_col(&mut d, 99.0);
    cursor_reset_col(&mut d);
    assert_eq!(cursor_col(&d), 0.0);
}

// ── unicode ──────────────────────────────────────────────────────────

#[test]
fn unicode_move_left_right() {
    let mut d = make_doc("Привет");
    d.set_cursor_raw( 12);
    move_left(&mut d);
    assert_eq!(d.cursor.raw(), 10);
    move_right(&mut d);
    assert_eq!(d.cursor.raw(), 12);
}

#[test]
fn unicode_word_nav() {
    let mut d = make_doc("hello\u{A0}world");
    move_word_right(&mut d);
    assert_eq!(d.cursor.raw(), 7);
}

#[test]
fn word_nav_tab() {
    let mut d = make_doc("a\tb");
    move_word_right(&mut d);
    assert_eq!(d.cursor.raw(), 2);
}
