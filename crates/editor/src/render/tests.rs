use super::*;
use crate::cache::DocumentCache;
use crate::font;
use crate::state::EditMode;
use crate::theme::EditorTheme;

// ── build tests ──

#[test]
fn build_does_not_deadlock() {
    font::init();
    let metrics = cosmic_text::Metrics::new(14.0, 19.6);
    let mut doc = ShapedDocument::new(cosmic_text::Buffer::new_empty(metrics), vec![]);
    let cache = DocumentCache::default();
    let theme = EditorTheme::default();
    build(
        &mut doc,
        "hello",
        &cache,
        EditMode::Live,
        &theme,
        14.0,
        24.0,
        0.0,
        None,
        None,
        None,
    );
    assert!(doc.line_count() > 0, "doc should be shaped after build");
}

#[test]
fn build_multiline() {
    font::init();
    let metrics = cosmic_text::Metrics::new(14.0, 19.6);
    let mut doc = ShapedDocument::new(cosmic_text::Buffer::new_empty(metrics), vec![]);
    let cache = DocumentCache::default();
    let theme = EditorTheme::default();
    build(
        &mut doc,
        "line 1\nline 2\nline 3",
        &cache,
        EditMode::Source,
        &theme,
        14.0,
        24.0,
        0.0,
        None,
        None,
        None,
    );
    assert_eq!(doc.line_count(), 3);
}

#[test]
fn build_empty_content() {
    font::init();
    let metrics = cosmic_text::Metrics::new(14.0, 19.6);
    let mut doc = ShapedDocument::new(cosmic_text::Buffer::new_empty(metrics), vec![]);
    let cache = DocumentCache::default();
    let theme = EditorTheme::default();
    build(
        &mut doc,
        "",
        &cache,
        EditMode::Live,
        &theme,
        14.0,
        24.0,
        0.0,
        None,
        None,
        None,
    );
    assert_eq!(doc.line_count(), 1);
}

#[test]
fn build_with_scroll() {
    font::init();
    let metrics = cosmic_text::Metrics::new(14.0, 19.6);
    let mut doc = ShapedDocument::new(cosmic_text::Buffer::new_empty(metrics), vec![]);
    let cache = DocumentCache::default();
    let theme = EditorTheme::default();
    build(
        &mut doc,
        "hello\nworld",
        &cache,
        EditMode::Source,
        &theme,
        14.0,
        24.0,
        100.0,
        Some(200.0),
        None,
        None,
    );
    assert!(doc.total_height() >= 0.0);
}

// ─── render::build integration tests with reveal ────────────────────

#[test]
fn build_reveal_inline_markers_appear_disappear() {
    use crate::layout::reveal::RevealCtx;
    use crate::markup::segmenter::to_document_cache;
    use zoll::ast::{MarkupDoc, MarkupNode, MarkupStyle};

    font::init();

    let doc = MarkupDoc {
        children: vec![MarkupNode::Formatted {
            style: MarkupStyle::BOLD,
            children: vec![MarkupNode::Text("текст".to_string())],
        }],
    };
    let cache = to_document_cache(&doc);
    assert_eq!(cache.lines[0].segments[0].left_marker_len, 2);
    assert_eq!(cache.lines[0].segments[0].raw_start, 2);
    assert_eq!(cache.lines[0].segments[0].raw_end, 12);

    let theme = EditorTheme::default();
    let content = "**текст**";
    let metrics = cosmic_text::Metrics::new(14.0, 19.6);

    // 1. Cursor на строке → маркеры видны (3 TextRun'а)
    let mut shaped = ShapedDocument::new(cosmic_text::Buffer::new_empty(metrics), vec![]);
    let reveal_on = RevealCtx {
        cursor_raw: Some(2),
        cursor_line: Some(0),
        block_of_line: &[],
    };
    build(
        &mut shaped,
        content,
        &cache,
        EditMode::Live,
        &theme,
        14.0,
        24.0,
        0.0,
        None,
        None,
        Some(&reveal_on),
    );
    assert_eq!(shaped.line_runs[0].len(), 3, "1: cursor на строке → 3 runs");
    assert_eq!(shaped.line_runs[0][2].text, "**", "1: closing marker");

    // 2. Cursor ушёл → маркеры скрыты (1 run)
    let reveal_off = RevealCtx {
        cursor_raw: Some(0),
        cursor_line: Some(1),
        block_of_line: &[],
    };
    build(
        &mut shaped,
        content,
        &cache,
        EditMode::Live,
        &theme,
        14.0,
        24.0,
        0.0,
        None,
        None,
        Some(&reveal_off),
    );
    assert_eq!(shaped.line_runs[0].len(), 1, "2: cursor ушёл → 1 run");

    // 3. Cursor вернулся → снова 3 runs (ОБЯЗАТЕЛЬНО с закрывающими)
    build(
        &mut shaped,
        content,
        &cache,
        EditMode::Live,
        &theme,
        14.0,
        24.0,
        0.0,
        None,
        None,
        Some(&reveal_on),
    );
    assert_eq!(
        shaped.line_runs[0].len(),
        3,
        "3: cursor вернулся → снова 3 runs"
    );
    assert_eq!(shaped.line_runs[0][0].text, "**", "3: open marker");
    assert_eq!(shaped.line_runs[0][1].text, "текст", "3: content");
    assert_eq!(shaped.line_runs[0][2].text, "**", "3: close marker");

    // 4. Повторный уход и возврат (имитация многократного переключения)
    build(
        &mut shaped,
        content,
        &cache,
        EditMode::Live,
        &theme,
        14.0,
        24.0,
        0.0,
        None,
        None,
        Some(&reveal_off),
    );
    assert_eq!(shaped.line_runs[0].len(), 1, "4: снова ушёл → 1 run");

    build(
        &mut shaped,
        content,
        &cache,
        EditMode::Live,
        &theme,
        14.0,
        24.0,
        0.0,
        None,
        None,
        Some(&reveal_on),
    );
    assert_eq!(shaped.line_runs[0].len(), 3, "5: снова вернулся → 3 runs");
    assert_eq!(
        shaped.line_runs[0][2].text, "**",
        "5: closing marker после повторного возврата"
    );
}

// ── shape tests ──

use crate::layout::LineCompensation;
use crate::layout::TextRun;
use crate::theme::color::Rgba;

fn make_runs(text: &str, size: f32) -> Vec<TextRun> {
    vec![TextRun::new(text, 0, Rgba::new(1.0, 1.0, 1.0), size)]
}

#[test]
fn shape_single_line() {
    font::init();
    let doc = font::with_font_system(|fs| {
        shape::shape_document(
            vec![make_runs("hello", 14.0)],
            vec![LineCompensation::identity(5)],
            fs,
            14.0,
            "sans-serif",
            0.0,
            None,
        )
    });
    assert!(doc.total_height() > 0.0);
    assert_eq!(doc.line_count(), 1);
}

#[test]
fn shape_multiple_lines() {
    font::init();
    let doc = font::with_font_system(|fs| {
        shape::shape_document(
            vec![make_runs("line1", 14.0), make_runs("line2", 14.0)],
            vec![LineCompensation::identity(5), LineCompensation::identity(5)],
            fs,
            14.0,
            "sans-serif",
            0.0,
            None,
        )
    });
    assert_eq!(doc.line_count(), 2);
}

#[test]
fn shape_empty_line() {
    font::init();
    let doc = font::with_font_system(|fs| {
        shape::shape_document(
            vec![vec![]],
            vec![LineCompensation::identity(0)],
            fs,
            14.0,
            "sans-serif",
            0.0,
            None,
        )
    });
    assert_eq!(doc.line_count(), 1);
    assert!(doc.total_height() > 0.0);
}

#[test]
fn glyph_starts_for_mixed_text() {
    font::init();
    let doc = font::with_font_system(|fs| {
        shape::shape_document(
            vec![make_runs("**текст**", 14.0)],
            vec![LineCompensation::identity(14)],
            fs,
            14.0,
            "sans-serif",
            0.0,
            None,
        )
    });
    let run = doc
        .buffer
        .layout_runs()
        .next()
        .expect("должна быть одна строка");
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
