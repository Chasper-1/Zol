//! Helper-функции для генерации тестовых строк в бенчмарках.
//!
//! Каждый bench-файл использует только часть функций, поэтому
//! dead_code здесь ожидаемо.

#![allow(dead_code)]

use std::sync::LazyLock;

pub static PLAIN_LINE: LazyLock<String> = LazyLock::new(|| {
    "The quick brown fox jumps over the lazy dog.".repeat(5)
});

pub static MARKUP_LINE: LazyLock<String> = LazyLock::new(|| {
    "The **quick** brown //fox// jumps ~~over~~ the lazy dog. $$E=mc^2$$".to_string()
});

/// Создать строку из `n` линий plain-текста.
pub fn make_plain(n: usize) -> String {
    (0..n)
        .map(|i| format!("Line {}: {}\n", i, *PLAIN_LINE))
        .collect()
}

/// Создать строку из `n` линий с умеренной разметкой.
pub fn make_markup(n: usize) -> String {
    (0..n)
        .map(|i| format!("Line {}: {}\n", i % 1000, *MARKUP_LINE))
        .collect()
}

/// Создать строку из `n` линий, где каждая — заголовок #1#.
pub fn make_headers(n: usize) -> String {
    (0..n)
        .map(|i| format!("#1# Header {}\n", i))
        .collect()
}
