use super::*;
use editor::state::EditMode;

#[test]
fn new_empty_content() {
    let inner = EditorInner::new(String::new());
    assert_eq!(inner.doc.borrow().content.as_str(), "");
    assert!(inner.doc.borrow().dirty);
}

#[test]
fn new_with_text() {
    let inner = EditorInner::new("hello world".to_string());
    assert_eq!(inner.doc.borrow().content.as_str(), "hello world");
    assert!(inner.doc.borrow().dirty);
}

#[test]
fn new_shaped_doc_has_lines() {
    let inner = EditorInner::new("line1\nline2\nline3".to_string());
    let shaped = inner.shaped_doc.borrow();
    assert!(shaped.line_count() > 0, "shaped_doc should have lines after build");
    assert!(shaped.total_height() > 0.0, "shaped_doc should have height");
}

#[test]
fn new_with_multiline() {
    let inner = EditorInner::new("a\nb\nc".to_string());
    let shaped = inner.shaped_doc.borrow();
    assert_eq!(shaped.line_count(), 3);
}

#[test]
fn new_with_unicode() {
    let inner = EditorInner::new("привет мир 👋".to_string());
    let shaped = inner.shaped_doc.borrow();
    assert!(shaped.line_count() > 0);
    assert!(shaped.total_height() > 0.0);
}

#[test]
fn new_single_line() {
    let inner = EditorInner::new("just one line".to_string());
    assert_eq!(inner.shaped_doc.borrow().line_count(), 1);
}

#[test]
fn defaults_are_sane() {
    let inner = EditorInner::new("x".to_string());
    assert_eq!(inner.base_size, 14.0);
    assert_eq!(inner.heading_size, 24.0);
    assert_eq!(inner.file_path, "notes.zoll");
    assert_eq!(inner.mode.get(), EditMode::LivePreview);
    assert_eq!(inner.scroll_y.get(), 0.0);
}

#[test]
fn edit_doc_insert_syncs_cache() {
    let inner = EditorInner::new("".to_string());
    inner.edit_doc(|doc| { doc.content.insert_str(0, "**bold**"); });
    let cache = inner.cache.borrow();
    assert!(cache.lines.len() >= 1);
}

#[test]
fn edit_doc_sets_dirty() {
    let inner = EditorInner::new("x".to_string());
    inner.doc.borrow_mut().dirty = false;
    inner.edit_doc(|doc| { doc.content.push_str("y"); });
    assert!(inner.doc.borrow().dirty, "edit_doc should set dirty=true");
}

#[test]
fn edit_doc_cache_updates_after_content_change() {
    let inner = EditorInner::new("hello".to_string());
    inner.edit_doc(|doc| { doc.content.push_str(" **world**"); });
    let cache = inner.cache.borrow();
    assert!(!cache.lines.is_empty(), "cache should be rebuilt after content change");
}

#[test]
fn edit_doc_multiple_calls() {
    let inner = EditorInner::new("".to_string());
    inner.edit_doc(|doc| doc.content.push_str("a"));
    inner.edit_doc(|doc| doc.content.push_str("b"));
    assert_eq!(inner.doc.borrow().content, "ab");
}
