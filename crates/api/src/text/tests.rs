use crate::cursor::cursor_raw;
use editor::document::Document;
use super::*;

fn make_doc(text: &str) -> Document {
    Document::new(text)
}

// ── insert_at_cursor ─────────────────────────────────────────────────

#[test]
fn insert_at_cursor_end() {
    let mut d = make_doc("hello");
    d.set_cursor_raw( 5);
    insert_at_cursor(&mut d, " world");
    assert_eq!(d.content(), "hello world");
    assert_eq!(d.cursor.raw(), "hello world".len());
}

#[test]
fn insert_at_cursor_mid() {
    let mut d = make_doc("helo");
    d.set_cursor_raw( 3);
    insert_at_cursor(&mut d, "l");
    assert_eq!(d.content(), "hello");
}

#[test]
fn insert_at_cursor_empty() {
    let mut d = make_doc("");
    insert_at_cursor(&mut d, "abc");
    assert_eq!(d.content(), "abc");
    assert_eq!(d.cursor.raw(), 3);
}

#[test]
fn insert_at_cursor_sets_dirty() {
    let mut d = make_doc("x");
    d.dirty = false;
    insert_at_cursor(&mut d, "y");
    assert!(d.dirty);
}

// ── delete_before ────────────────────────────────────────────────────

#[test]
fn delete_before_removes_char() {
    let mut d = make_doc("hello");
    d.set_cursor_raw( 5);
    delete_before(&mut d);
    assert_eq!(d.content(), "hell");
    assert_eq!(d.cursor.raw(), 4);
}

#[test]
fn delete_before_at_start() {
    let mut d = make_doc("hello");
    delete_before(&mut d);
    assert_eq!(d.content(), "hello");
}

#[test]
fn delete_before_empty() {
    let mut d = make_doc("");
    delete_before(&mut d);
    assert_eq!(d.content(), "");
}

#[test]
fn delete_before_sets_dirty() {
    let mut d = make_doc("ab");
    d.set_cursor_raw( 2);
    d.dirty = false;
    delete_before(&mut d);
    assert!(d.dirty);
}

#[test]
fn delete_before_grapheme() {
    let mut d = make_doc("e\u{0301}x");
    d.set_cursor_raw( 4);
    delete_before(&mut d);
    assert_eq!(d.content(), "e\u{0301}");
}

// ── delete_after ─────────────────────────────────────────────────────

#[test]
fn delete_after_removes_char() {
    let mut d = make_doc("hello");
    delete_after(&mut d);
    assert_eq!(d.content(), "ello");
}

#[test]
fn delete_after_at_end() {
    let mut d = make_doc("hello");
    d.set_cursor_raw( 5);
    delete_after(&mut d);
    assert_eq!(d.content(), "hello");
}

#[test]
fn delete_after_empty() {
    let mut d = make_doc("");
    delete_after(&mut d);
    assert_eq!(d.content(), "");
}

#[test]
fn delete_after_sets_dirty() {
    let mut d = make_doc("ab");
    d.dirty = false;
    delete_after(&mut d);
    assert!(d.dirty);
}

#[test]
fn delete_after_grapheme() {
    let mut d = make_doc("e\u{0301}x");
    d.set_cursor_raw( 0);
    delete_after(&mut d);
    assert_eq!(d.content(), "x");
}

// ── newline ──────────────────────────────────────────────────────────

#[test]
fn newline_inserts_break() {
    let mut d = make_doc("ab");
    d.set_cursor_raw( 1);
    newline(&mut d);
    assert_eq!(d.content(), "a\nb");
    assert_eq!(d.cursor.raw(), 2);
}

#[test]
fn newline_sets_dirty() {
    let mut d = make_doc("x");
    d.set_cursor_raw( 1);
    d.dirty = false;
    newline(&mut d);
    assert!(d.dirty);
}

#[test]
fn newline_resets_col_visual() {
    let mut d = make_doc("abc");
    d.cursor.set_col_visual(42.0);
    d.set_cursor_raw( 3);
    newline(&mut d);
    assert_eq!(d.cursor.col_visual(), 0.0);
}

// ── insert_at ────────────────────────────────────────────────────────

#[test]
fn insert_at_start() {
    let mut d = make_doc("bc");
    insert_at(&mut d, 0, "a");
    assert_eq!(d.content(), "abc");
}

#[test]
fn insert_at_mid() {
    let mut d = make_doc("ac");
    insert_at(&mut d, 1, "b");
    assert_eq!(d.content(), "abc");
}

#[test]
fn insert_at_end() {
    let mut d = make_doc("ab");
    insert_at(&mut d, 2, "c");
    assert_eq!(d.content(), "abc");
}

#[test]
fn insert_at_sets_dirty() {
    let mut d = make_doc("x");
    d.dirty = false;
    insert_at(&mut d, 0, "y");
    assert!(d.dirty);
}

#[test]
fn insert_at_cursor_unchanged() {
    let mut d = make_doc("abc");
    d.set_cursor_raw( 1);
    insert_at(&mut d, 2, "X");
    assert_eq!(cursor_raw(&d), 1); // курсор не тронут
}

// ── delete_range ─────────────────────────────────────────────────────

#[test]
fn delete_range_mid() {
    let mut d = make_doc("hello world");
    delete_range(&mut d, 5, 11);
    assert_eq!(d.content(), "hello");
}

#[test]
fn delete_range_start_only() {
    let mut d = make_doc("abc");
    delete_range(&mut d, 0, 0);
    assert_eq!(d.content(), "abc");
}

#[test]
fn delete_range_past_end() {
    let mut d = make_doc("abc");
    delete_range(&mut d, 1, 99);
    assert_eq!(d.content(), "a");
}

#[test]
fn delete_range_sets_dirty() {
    let mut d = make_doc("abc");
    d.dirty = false;
    delete_range(&mut d, 0, 1);
    assert!(d.dirty);
}

#[test]
fn delete_range_cursor_unchanged() {
    let mut d = make_doc("abc");
    d.set_cursor_raw( 2);
    delete_range(&mut d, 0, 1);
    assert_eq!(cursor_raw(&d), 2);
}

// ── unicode ──────────────────────────────────────────────────────────

#[test]
fn unicode_insert() {
    let mut d = make_doc("Приве");
    d.set_cursor_raw( 10);
    insert_at_cursor(&mut d, "т");
    assert_eq!(d.content(), "Привет");
    assert_eq!(d.cursor.raw(), 12);
}

#[test]
fn unicode_delete_before() {
    let mut d = make_doc("Привет");
    d.set_cursor_raw( 12);
    delete_before(&mut d);
    assert_eq!(d.content(), "Приве");
    assert_eq!(d.cursor.raw(), 10);
}

#[test]
fn unicode_insert_at() {
    let mut d = make_doc("Првет");
    insert_at(&mut d, 4, "и");
    assert_eq!(d.content(), "Привет");
}
