use super::*;
use crate::editor::cache::DocumentCache;
use crate::editor::font;
use crate::editor::state::EditMode;
use crate::editor::theme::EditorTheme;

// ── build tests ──

#[test]
fn build_does_not_deadlock() {
    font::init();
    let metrics = cosmic_text::Metrics::new(14.0, 19.6);
    let mut doc = ShapedDocument::new(cosmic_text::Buffer::new_empty(metrics), vec![]);
    let cache = DocumentCache::default();
    let theme = EditorTheme::default();
    build(&mut doc, "hello", &cache, EditMode::LivePreview, 0, &theme, 14.0, 24.0, 0.0, None);
    assert!(doc.line_count() > 0, "doc should be shaped after build");
}

#[test]
fn build_multiline() {
    font::init();
    let metrics = cosmic_text::Metrics::new(14.0, 19.6);
    let mut doc = ShapedDocument::new(cosmic_text::Buffer::new_empty(metrics), vec![]);
    let cache = DocumentCache::default();
    let theme = EditorTheme::default();
    build(&mut doc, "line 1\nline 2\nline 3", &cache, EditMode::Source, 0, &theme, 14.0, 24.0, 0.0, None);
    assert_eq!(doc.line_count(), 3);
}

#[test]
fn build_empty_content() {
    font::init();
    let metrics = cosmic_text::Metrics::new(14.0, 19.6);
    let mut doc = ShapedDocument::new(cosmic_text::Buffer::new_empty(metrics), vec![]);
    let cache = DocumentCache::default();
    let theme = EditorTheme::default();
    build(&mut doc, "", &cache, EditMode::LivePreview, 0, &theme, 14.0, 24.0, 0.0, None);
    assert_eq!(doc.line_count(), 1);
}

#[test]
fn build_with_scroll() {
    font::init();
    let metrics = cosmic_text::Metrics::new(14.0, 19.6);
    let mut doc = ShapedDocument::new(cosmic_text::Buffer::new_empty(metrics), vec![]);
    let cache = DocumentCache::default();
    let theme = EditorTheme::default();
    build(&mut doc, "hello\nworld", &cache, EditMode::Source, 0, &theme, 14.0, 24.0, 100.0, Some(200.0));
    assert!(doc.total_height() >= 0.0);
}

// ── shape tests ──

use crate::editor::layout::TextRun;
use crate::editor::theme::color::Rgba;

fn make_runs(text: &str, size: f32) -> Vec<TextRun> {
    vec![TextRun::new(text, 0, Rgba::new(1.0, 1.0, 1.0), size)]
}

#[test]
fn shape_single_line() {
    font::init();
    let doc = font::with_font_system(|fs| {
        shape::shape_document(&[make_runs("hello", 14.0)], fs, 14.0, "sans-serif", 0.0, None)
    });
    assert!(doc.total_height() > 0.0);
    assert_eq!(doc.line_count(), 1);
}

#[test]
fn shape_multiple_lines() {
    font::init();
    let doc = font::with_font_system(|fs| {
        shape::shape_document(
            &[make_runs("line1", 14.0), make_runs("line2", 14.0)],
            fs, 14.0, "sans-serif", 0.0, None,
        )
    });
    assert_eq!(doc.line_count(), 2);
}

#[test]
fn shape_empty_line() {
    font::init();
    let doc = font::with_font_system(|fs| {
        shape::shape_document(&[vec![]], fs, 14.0, "sans-serif", 0.0, None)
    });
    assert_eq!(doc.line_count(), 1);
    assert!(doc.total_height() > 0.0);
}

#[test]
fn glyph_starts_for_mixed_text() {
    font::init();
    let doc = font::with_font_system(|fs| {
        shape::shape_document(&[make_runs("**текст**", 14.0)], fs, 14.0, "sans-serif", 0.0, None)
    });
    let run = doc.buffer.layout_runs().next().expect("должна быть одна строка");
    let glyphs: Vec<_> = run.glyphs.iter().map(|g| (g.start, g.x, g.w)).collect();
    assert_eq!(glyphs.len(), 9, "9 glyph-кластеров: glyphs={:?}", glyphs);
    assert_eq!(glyphs[0].0, 0, "* (первый)");
    assert_eq!(glyphs[1].0, 1, "* (второй)");
    assert_eq!(glyphs[2].0, 2, "т (байт 2)");
    assert_eq!(glyphs[3].0, 4, "е (байт 4)");
    assert_eq!(glyphs[4].0, 6, "к (байт 6)");
    assert_eq!(glyphs[5].0, 8, "с (байт 8)");
    assert_eq!(glyphs[6].0, 10, "т (байт 10)");
    assert_eq!(glyphs[7].0, 12, "* (третий)");
    assert_eq!(glyphs[8].0, 13, "* (четвёртый)");
}
