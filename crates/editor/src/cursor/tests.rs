use std::time::Instant;

use super::grapheme::{clamp_to_char_boundary, next_grapheme_boundary, prev_grapheme_boundary};
use super::types::Cursor;

fn ls(s: &str) -> Vec<usize> {
    let mut v = vec![0usize];
    for (i, c) in s.char_indices() {
        if c == '\n' {
            v.push(i + 1);
        }
    }
    v
}

fn cursor_at(raw: usize, line: usize, col: f64) -> Cursor {
    let mut c = Cursor::new();
    c.raw = raw;
    c.line = line;
    c.col_visual = col as f32;
    c
}

#[test]
fn new_cursor_has_zero_position() {
    let c = Cursor::new();
    assert_eq!(c.raw(), 0);
    assert_eq!(c.line(), 0);
}

#[test]
fn raw_returns_current_position() {
    let c = cursor_at(5, 1, 0.0);
    assert_eq!(c.raw(), 5);
}

#[test]
fn line_returns_cached_line() {
    let c = cursor_at(10, 2, 0.0);
    assert_eq!(c.line(), 2);
}

#[test]
fn col_visual_returns_stored_value() {
    let c = cursor_at(0, 0, 42.0);
    assert_eq!(c.col_visual(), 42.0);
}

#[test]
fn set_raw_clamps_to_char_boundary() {
    let mut c = Cursor::new();
    let text = "héllo";
    // 'é' is 2 bytes at positions 1-2; byte 1 is valid start, byte 2 is continuation
    c.set_raw(text, &ls(text), 2);
    assert_eq!(c.raw(), 1); // clamped to 1 (start of 'é')
    c.set_raw(text, &ls(text), 3);
    assert_eq!(c.raw(), 3); // after 'é'
}

#[test]
fn set_raw_clamps_past_end() {
    let mut c = Cursor::new();
    let text = "abc";
    c.set_raw(text, &ls(text), 10);
    assert_eq!(c.raw(), 3);
}

#[test]
fn set_raw_on_empty_string() {
    let mut c = Cursor::new();
    c.set_raw("", &[], 5);
    assert_eq!(c.raw(), 0);
}

#[test]
fn set_raw_updates_line() {
    let mut c = Cursor::new();
    let text = "hello\nworld\nfoo";
    c.set_raw(text, &ls(text), 7);
    assert_eq!(c.raw(), 7);
    assert_eq!(c.line(), 1);
    c.set_raw(text, &ls(text), 14);
    assert_eq!(c.line(), 2);
}

#[test]
fn set_raw_on_newline_returns_next_line() {
    let mut c = Cursor::new();
    let text = "hello\nworld";
    c.set_raw(text, &ls(text), 6); // '\n'
    assert_eq!(c.raw(), 6);
    assert_eq!(c.line(), 1);
}

#[test]
fn reset_col_visual_sets_to_zero() {
    let mut c = cursor_at(0, 0, 50.0);
    c.reset_col_visual();
    assert_eq!(c.col_visual(), 0.0);
}

// ─── Выделение ────────────────────────────────────

#[test]
fn no_selection_by_default() {
    let c = Cursor::new();
    assert!(!c.has_selection());
    assert_eq!(c.selection_range(), None);
}

#[test]
fn begin_selection_sets_anchor() {
    let mut c = cursor_at(5, 0, 0.0);
    c.begin_selection();
    assert_eq!(c.selection_range(), Some((5, 5)));
}

#[test]
fn clear_selection_removes_anchor() {
    let mut c = cursor_at(5, 0, 0.0);
    c.begin_selection();
    c.clear_selection();
    assert!(!c.has_selection());
}

#[test]
fn selection_range_orders_correctly() {
    let mut c = cursor_at(10, 0, 0.0);
    c.anchor = Some(3);
    assert_eq!(c.selection_range(), Some((3, 10)));
}

#[test]
fn begin_selection_does_not_overwrite_existing_anchor() {
    let mut c = cursor_at(10, 0, 0.0);
    c.anchor = Some(3);
    c.begin_selection();
    assert_eq!(c.selection_range(), Some((3, 10)));
}

// ─── Мигание ──────────────────────────────────────

#[test]
fn blink_starts_visible() {
    let mut c = Cursor::new();
    let now = Instant::now();
    c.force_blink_at(now);
    assert!(c.should_blink_at(now));
}

#[test]
fn blink_becomes_invisible_after_visible_period() {
    let mut c = Cursor::new();
    let start = Instant::now();
    c.force_blink_at(start);
    let past_visible = start + std::time::Duration::from_millis(531);
    assert!(!c.should_blink_at(past_visible));
}

#[test]
fn blink_becomes_visible_again_after_full_period() {
    let mut c = Cursor::new();
    let start = Instant::now();
    c.force_blink_at(start);
    let next_cycle = start + std::time::Duration::from_millis(1061);
    assert!(c.should_blink_at(next_cycle));
}

// ─── Grapheme boundaries (basic smoke) ────────

#[test]
fn clamp_to_char_boundary_at_valid_boundary() {
    assert_eq!(clamp_to_char_boundary("hello", 2), 2);
}

#[test]
fn clamp_to_char_boundary_at_grapheme_reduces() {
    // 'é' is 2 bytes; byte 2 is a continuation byte → clamp to 1 (start of é)
    assert_eq!(clamp_to_char_boundary("héllo", 2), 1);
}

#[test]
fn clamp_to_char_boundary_past_end_returns_len() {
    assert_eq!(clamp_to_char_boundary("hi", 10), 2);
}

#[test]
fn clamp_to_char_boundary_empty() {
    assert_eq!(clamp_to_char_boundary("", 0), 0);
}

#[test]
fn prev_grapheme_boundary_from_mid() {
    let text = "abcd";
    assert_eq!(prev_grapheme_boundary(text, 3), Some(2));
}

#[test]
fn next_grapheme_boundary_from_start() {
    let text = "abcd";
    assert_eq!(next_grapheme_boundary(text, 0), Some(1));
}

#[test]
fn next_grapheme_boundary_at_end_returns_none() {
    let text = "abc";
    assert_eq!(next_grapheme_boundary(text, 3), None);
}
