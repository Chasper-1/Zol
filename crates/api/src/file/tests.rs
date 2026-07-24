use std::io::Read;
use editor::document::Document;
use super::*;

#[test]
fn file_save_load_roundtrip() {
    let dir = std::env::temp_dir();
    let path = dir.join("zol_test_roundtrip.zoll");

    let d = Document::new("hello world");
    file_save(&d, &path).unwrap();

    let loaded = file_load(&path).unwrap();
    assert_eq!(loaded.content(), "hello world");

    let _ = std::fs::remove_file(&path);
}

#[test]
fn file_load_missing_returns_error() {
    let dir = std::env::temp_dir();
    let path = dir.join("zol_test_nonexistent.zoll");
    // файла нет — должна быть ошибка
    assert!(file_load(&path).is_err());
}

#[test]
fn file_save_str_load_str() {
    let dir = std::env::temp_dir();
    let path = dir.join("zol_test_str.zoll");

    file_save_str("test content", &path).unwrap();

    let s = file_load_str(&path).unwrap();
    assert_eq!(s, "test content");

    let _ = std::fs::remove_file(&path);
}

#[test]
fn file_load_empty_file() {
    let dir = std::env::temp_dir();
    let path = dir.join("zol_test_empty.zoll");

    file_save_str("", &path).unwrap();
    let d = file_load(&path).unwrap();
    assert!(d.content().is_empty());

    let _ = std::fs::remove_file(&path);
}

#[test]
fn file_save_dirty_flag_preserved() {
    let dir = std::env::temp_dir();
    let path = dir.join("zol_test_dirty.zoll");

    let mut d = Document::new("text");
    d.dirty = false;
    file_save(&d, &path).unwrap();

    let loaded = file_load(&path).unwrap();
    // новый документ всегда dirty на старте
    assert!(loaded.dirty);

    let _ = std::fs::remove_file(&path);
}
