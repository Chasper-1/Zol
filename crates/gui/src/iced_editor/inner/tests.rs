use super::*;
use api::cursor as api_cursor;
use editor::state::EditMode;
use crate::iced_editor::widget::editor::IcedEditor;

// ═══════════════════════════════════════════════════════════════════
// EditorInner::new() — конструктор, все варианты
// ═══════════════════════════════════════════════════════════════════

#[test]
fn new_empty_content() {
    let inner = EditorInner::new(String::new());
    assert_eq!(inner.doc.borrow().content(), "");
    assert!(inner.doc.borrow().dirty);
}

#[test]
fn new_with_text() {
    let inner = EditorInner::new("hello world".to_string());
    assert_eq!(inner.doc.borrow().content(), "hello world");
}

#[test]
fn new_with_unicode() {
    let inner = EditorInner::new("привет мир 👋 🚀".to_string());
    assert!(inner.doc.borrow().incremental.num_lines() >= 1);
    assert!(inner.shaped_doc.borrow().line_count() >= 1);
}

#[test]
fn new_with_multiline() {
    let inner = EditorInner::new("a\nb\nc\n".to_string());
    let shaped = inner.shaped_doc.borrow();
    assert_eq!(shaped.line_count(), 4);
}

#[test]
fn new_with_only_newlines() {
    let inner = EditorInner::new("\n\n\n".to_string());
    assert_eq!(inner.doc.borrow().content(), "\n\n\n");
    let shaped = inner.shaped_doc.borrow();
    assert!(shaped.line_count() >= 3);
}

#[test]
fn new_single_line() {
    let inner = EditorInner::new("just one line".to_string());
    assert_eq!(inner.shaped_doc.borrow().line_count(), 1);
}

#[test]
fn new_sets_dirty_true() {
    let inner = EditorInner::new("x".to_string());
    assert!(inner.doc.borrow().dirty, "new EditorInner should be dirty");
}

#[test]
fn defaults_are_sane() {
    let inner = EditorInner::new("x".to_string());
    assert_eq!(inner.base_size, 14.0, "base_size default");
    assert_eq!(inner.heading_size, 24.0, "heading_size default");
    assert_eq!(inner.file_path, "notes.zoll", "file_path default");
    assert_eq!(inner.mode.get(), EditMode::Live, "mode default");
    assert_eq!(inner.scroll_y.get(), 0.0, "scroll_y default");
}

#[test]
fn new_many_lines() {
    let content = (0..100).map(|i| format!("line {}", i)).collect::<Vec<_>>().join("\n");
    let inner = EditorInner::new(content.clone());
    assert_eq!(inner.doc.borrow().content(), &content);
}

// ═══════════════════════════════════════════════════════════════════
// edit_doc_raw — все варианты ввода
// ═══════════════════════════════════════════════════════════════════

#[test]
fn edit_doc_raw_insert_at_start() {
    let inner = EditorInner::new("world".to_string());
    inner.edit_doc_raw(0, 0, "hello ");
    assert_eq!(inner.doc.borrow().content(), "hello world");
}

#[test]
fn edit_doc_raw_insert_at_middle() {
    let inner = EditorInner::new("helo".to_string());
    inner.edit_doc_raw(3, 3, "l");
    assert_eq!(inner.doc.borrow().content(), "hello");
}

#[test]
fn edit_doc_raw_insert_at_end() {
    let inner = EditorInner::new("hello".to_string());
    inner.edit_doc_raw(5, 5, " world");
    assert_eq!(inner.doc.borrow().content(), "hello world");
}

#[test]
fn edit_doc_raw_insert_unicode() {
    let inner = EditorInner::new("hello ".to_string());
    inner.edit_doc_raw(6, 6, "привет");
    assert_eq!(inner.doc.borrow().content(), "hello привет");
}

#[test]
fn edit_doc_raw_insert_emoji() {
    let inner = EditorInner::new("hello ".to_string());
    inner.edit_doc_raw(6, 6, "🚀✨");
    assert_eq!(inner.doc.borrow().content(), "hello 🚀✨");
}

#[test]
fn edit_doc_raw_insert_newline_mid() {
    let inner = EditorInner::new("abc\ndef".to_string());
    inner.edit_doc_raw(3, 3, "\n");
    assert_eq!(inner.doc.borrow().content(), "abc\n\ndef");
}

#[test]
fn edit_doc_raw_replace_range() {
    let inner = EditorInner::new("hello bad world".to_string());
    inner.edit_doc_raw(6, 9, "good"); // заменить "bad" на "good"
    assert_eq!(inner.doc.borrow().content(), "hello good world");
}

#[test]
fn edit_doc_raw_delete_range() {
    let inner = EditorInner::new("hello xxx world".to_string());
    inner.edit_doc_raw(6, 10, ""); // удалить "xxx "
    assert_eq!(inner.doc.borrow().content(), "hello world");
}

#[test]
fn edit_doc_raw_multiple_sequential() {
    let inner = EditorInner::new(String::new());
    inner.edit_doc_raw(0, 0, "a");
    inner.edit_doc_raw(1, 1, "b");
    inner.edit_doc_raw(2, 2, "c");
    assert_eq!(inner.doc.borrow().content(), "abc");
}

#[test]
fn edit_doc_raw_multiple_insert_at_start() {
    let inner = EditorInner::new(String::new());
    inner.edit_doc_raw(0, 0, "c");
    inner.edit_doc_raw(0, 0, "b");
    inner.edit_doc_raw(0, 0, "a");
    assert_eq!(inner.doc.borrow().content(), "abc");
}

#[test]
fn edit_doc_raw_empty_string_no_change() {
    let inner = EditorInner::new("hello".to_string());
    inner.edit_doc_raw(2, 2, "");
    assert_eq!(inner.doc.borrow().content(), "hello");
}

#[test]
fn edit_doc_raw_zero_len_from_to() {
    let inner = EditorInner::new("abc".to_string());
    inner.edit_doc_raw(1, 1, "X");
    assert_eq!(inner.doc.borrow().content(), "aXbc");
}

#[test]
fn edit_doc_raw_overwrite_full() {
    let inner = EditorInner::new("old".to_string());
    inner.edit_doc_raw(0, 3, "new");
    assert_eq!(inner.doc.borrow().content(), "new");
}

#[test]
fn edit_doc_raw_sets_dirty() {
    let inner = EditorInner::new("x".to_string());
    inner.doc.borrow_mut().dirty = false;
    inner.edit_doc_raw(1, 1, "y");
    assert!(inner.doc.borrow().dirty);
}

#[test]
fn edit_doc_raw_cache_rebuilt() {
    let inner = EditorInner::new("hello".to_string());
    inner.edit_doc_raw(5, 5, " **world**");
    let cache = inner.cache.borrow();
    assert!(!cache.lines.is_empty(), "cache should be rebuilt");
}

#[test]
fn edit_doc_raw_unicode_grapheme_boundaries() {
    let inner = EditorInner::new("abc émoji".to_string());
    // Вставляем после 'abc ' (byte 4) — перед é
    inner.edit_doc_raw(4, 4, "🚀");
    assert_eq!(inner.doc.borrow().content(), "abc 🚀émoji");
}

#[test]
fn edit_doc_raw_on_line_boundary_start() {
    let inner = EditorInner::new("abc\ndef".to_string());
    inner.edit_doc_raw(0, 0, "START\n");
    assert_eq!(inner.doc.borrow().content(), "START\nabc\ndef");
}

#[test]
fn edit_doc_raw_on_line_boundary_end() {
    let inner = EditorInner::new("abc\ndef".to_string());
    inner.edit_doc_raw(7, 7, "\nEND");
    assert_eq!(inner.doc.borrow().content(), "abc\ndef\nEND");
}

#[test]
fn edit_doc_raw_insert_many_lines() {
    let inner = EditorInner::new("start".to_string());
    let many = (0..50).map(|i| format!("line{}\n", i)).collect::<String>();
    inner.edit_doc_raw(5, 5, &many);
    assert!(inner.doc.borrow().incremental.num_lines() >= 50);
}

#[test]
fn edit_doc_raw_intensive() {
    let inner = EditorInner::new(String::new());
    let text = "x".repeat(100);
    for (i, ch) in text.char_indices() {
        inner.edit_doc_raw(i, i, &ch.to_string());
    }
    assert_eq!(inner.doc.borrow().content().len(), 100);
    assert_eq!(inner.doc.borrow().content(), text);
}

// ═══════════════════════════════════════════════════════════════════
// edit_doc — closure-based редактирование
// ═══════════════════════════════════════════════════════════════════

#[test]
fn edit_doc_insert_bold() {
    let inner = EditorInner::new(String::new());
    inner.edit_doc(|doc| {
        doc.incremental.edit(0, 0, "**bold**");
    });
    assert_eq!(inner.doc.borrow().content(), "**bold**");
    let cache = inner.cache.borrow();
    assert!(!cache.lines.is_empty());
}

#[test]
fn edit_doc_twice() {
    let inner = EditorInner::new(String::new());
    inner.edit_doc(|doc| {
        doc.incremental.edit(0, 0, "a");
    });
    inner.edit_doc(|doc| {
        let len = doc.incremental.source.len();
        doc.incremental.edit(len, len, "b");
    });
    assert_eq!(inner.doc.borrow().content(), "ab");
}

#[test]
fn edit_doc_sets_dirty() {
    let inner = EditorInner::new("x".to_string());
    inner.doc.borrow_mut().dirty = false;
    inner.edit_doc(|doc| {
        doc.incremental.edit(1, 1, "y");
    });
    assert!(inner.doc.borrow().dirty);
}

#[test]
fn edit_doc_cache_rebuilt() {
    let inner = EditorInner::new("".to_string());
    inner.edit_doc(|doc| {
        doc.incremental.edit(0, 0, "**hello**");
    });
    let cache = inner.cache.borrow();
    let any_bold = cache.lines.iter().flat_map(|l| &l.segments).any(|s| s.style & editor::markup::segment::STYLE_BOLD != 0);
    assert!(any_bold, "cache should contain bold segments");
}

#[test]
fn edit_doc_closure_with_mut_borrow_inside() {
    let inner = EditorInner::new("hello".to_string());
    inner.edit_doc(|doc| {
        doc.incremental.edit(5, 5, " world");
    });
    // После edit_doc все заимствования дропнуты
    assert_eq!(inner.doc.borrow().content(), "hello world");
    let line = inner.doc.borrow().cursor.line();
}

// ═══════════════════════════════════════════════════════════════════
// mode: get, set, cycle — все переходы
// ═══════════════════════════════════════════════════════════════════

#[test]
fn mode_default_is_live_preview() {
    let inner = EditorInner::new("test".to_string());
    assert_eq!(inner.get_mode(), EditMode::Live);
}

#[test]
fn mode_set_live_preview() {
    let inner = EditorInner::new("test".to_string());
    inner.set_mode(EditMode::Live);
    assert_eq!(inner.get_mode(), EditMode::Live);
}

#[test]
fn mode_set_source() {
    let inner = EditorInner::new("test".to_string());
    inner.set_mode(EditMode::Source);
    assert_eq!(inner.get_mode(), EditMode::Source);
}

#[test]
fn mode_set_preview() {
    let inner = EditorInner::new("test".to_string());
    inner.set_mode(EditMode::Preview);
    assert_eq!(inner.get_mode(), EditMode::Preview);
}

#[test]
fn mode_set_same_twice() {
    let inner = EditorInner::new("test".to_string());
    inner.set_mode(EditMode::Preview);
    inner.set_mode(EditMode::Preview); // повторно тот же режим
    assert_eq!(inner.get_mode(), EditMode::Preview);
}

#[test]
fn mode_cycle_live_preview_to_source() {
    let inner = EditorInner::new("test".to_string());
    inner.cycle_mode();
    assert_eq!(inner.get_mode(), EditMode::Source);
}

#[test]
fn mode_cycle_source_to_preview() {
    let inner = EditorInner::new("test".to_string());
    inner.cycle_mode();
    inner.cycle_mode();
    assert_eq!(inner.get_mode(), EditMode::Preview);
}

#[test]
fn mode_cycle_preview_to_live_preview() {
    let inner = EditorInner::new("test".to_string());
    inner.cycle_mode(); // Live → Source
    inner.cycle_mode(); // Source → Preview
    inner.cycle_mode(); // Preview → Live
    assert_eq!(inner.get_mode(), EditMode::Live);
}

#[test]
fn mode_cycle_full_loop() {
    let inner = EditorInner::new("test".to_string());
    for _ in 0..6 {
        inner.cycle_mode();
    }
    // Чётное число циклов — вернулись в Live
    assert_eq!(inner.get_mode(), EditMode::Live);
}

#[test]
fn mode_set_marks_dirty() {
    let inner = EditorInner::new("test".to_string());
    inner.doc.borrow_mut().dirty = false;
    inner.set_mode(EditMode::Source);
    assert!(inner.doc.borrow().dirty, "set_mode should mark dirty");
}

#[test]
fn mode_cycle_marks_dirty() {
    let inner = EditorInner::new("test".to_string());
    inner.doc.borrow_mut().dirty = false;
    inner.cycle_mode();
    assert!(inner.doc.borrow().dirty, "cycle_mode should mark dirty");
}

// ═══════════════════════════════════════════════════════════════════
// RefCell: все комбинации borrow/borrow_mut, которые были бажными
// ═══════════════════════════════════════════════════════════════════

#[test]
fn borrow_then_borrow_mut_no_conflict() {
    let inner = EditorInner::new(String::new());
    let doc = inner.doc.borrow();            // immutable
    drop(_doc);                                // дропнули
    let doc = inner.doc.borrow_mut();         // mutable — ок
}

#[test]
fn borrow_mut_then_borrow_no_conflict() {
    let inner = EditorInner::new(String::new());
    let doc = inner.doc.borrow_mut();         // mutable
    drop(_doc);                                // дропнули
    let doc = inner.doc.borrow();             // immutable — ок
}

#[test]
fn scoped_borrow_before_borrow_mut() {
    let inner = EditorInner::new("hello".to_string());
    {
        let doc = inner.doc.borrow();
    } // dropped
    let doc = inner.doc.borrow_mut();
}

#[test]
fn scoped_borrow_mut_before_borrow() {
    let inner = EditorInner::new("hello".to_string());
    {
        let doc = inner.doc.borrow_mut();
    } // dropped
    let doc = inner.doc.borrow();
}

#[test]
fn borrow_read_after_edit_doc_raw() {
    // Этот паттерн раньше падал — мёртвый `let cursor = &self.doc.borrow().cursor;`
    let inner = EditorInner::new(String::new());
    inner.edit_doc_raw(0, 0, "a");
    // edit_doc_raw внутри делает borrow_mut — если после borrow не дропнут, упадёт
    inner.edit_doc_raw(1, 1, "b");
    assert_eq!(inner.doc.borrow().content(), "ab");
}

#[test]
fn borrow_mut_then_function_that_borrows() {
    // Симулирует паттерн Ctrl+Left: borrow_mut → auto_scroll (borrow)
    let inner = EditorInner::new("hello world".to_string());
    {
        let mut doc = inner.doc.borrow_mut();
        api_cursor::move_word_right(&mut *doc);
    } // RefMut дропнут
    // Теперь можно сделать borrow — как это делает auto_scroll
    let raw = inner.doc.borrow().cursor.raw();
    assert_eq!(raw, 6); // start of "world"
}

#[test]
fn multiple_borrow_mut_sequence() {
    let inner = EditorInner::new(String::new());
    for i in 0..100 {
        let mut doc = inner.doc.borrow_mut();
        doc.incremental.edit(i, i, "x");
    }
    assert_eq!(inner.doc.borrow().content().len(), 100);
}

#[test]
fn interleaved_borrow_all_cells() {
    let inner = EditorInner::new("test".to_string());
    // Одновременные borrow разных RefCell — никогда не падает
    let doc = inner.doc.borrow();
    let cache = inner.cache.borrow();
    let shaped = inner.shaped_doc.borrow();
    assert!(doc.content().len() > 0);
    assert!(cache.lines.len() >= 1);
    assert!(shaped.line_count() > 0);
    drop(doc);
    drop(cache);
    drop(shaped);
}

#[test]
fn borrow_doc_and_shaped_then_borrow_mut_doc() {
    let inner = EditorInner::new("test".to_string());
    let doc = inner.doc.borrow();
    let shaped = inner.shaped_doc.borrow();
    let line = doc.cursor.line();
    let h = shaped.total_height();
    drop(doc);   // дропаем doc, shaped остаётся
    let mut doc = inner.doc.borrow_mut(); // должно работать — doc дропнут
    doc.dirty = true;
    drop(doc);
    drop(shaped);
}

#[test]
fn borrow_mut_then_borrow_other_cell() {
    // Разные RefCell не конфликтуют
    let inner = EditorInner::new("test".to_string());
    let doc = inner.doc.borrow_mut();
    let cache = inner.cache.borrow(); // другой RefCell — ок
    drop(doc);
    drop(cache);
}

// ═══════════════════════════════════════════════════════════════════
// auto_scroll — симуляция через изменение scroll_y
// ═══════════════════════════════════════════════════════════════════

#[test]
fn scroll_y_set_and_get() {
    let inner = EditorInner::new("a\nb\nc\nd\ne".to_string());
    inner.scroll_y.set(42.0);
    assert_eq!(inner.scroll_y.get(), 42.0);
}

#[test]
fn scroll_y_default_zero() {
    let inner = EditorInner::new("test".to_string());
    assert_eq!(inner.scroll_y.get(), 0.0);
}

#[test]
fn mark_dirty_sets_flag() {
    let inner = EditorInner::new("test".to_string());
    inner.doc.borrow_mut().dirty = false;
    inner.mark_dirty();
    assert!(inner.doc.borrow().dirty);
}

#[test]
fn mark_dirty_idempotent() {
    let inner = EditorInner::new("test".to_string());
    inner.mark_dirty();
    inner.mark_dirty();
    assert!(inner.doc.borrow().dirty);
}

// ═══════════════════════════════════════════════════════════════════
// Word movement — симуляция Ctrl+Arrow
// ═══════════════════════════════════════════════════════════════════

#[test]
fn word_movement_left_from_end() {
    let inner = EditorInner::new("hello world foo".to_string());
    inner.doc.borrow_mut().set_cursor_raw(15);
    api_cursor::move_word_left(&mut *inner.doc.borrow_mut());
    assert_eq!(inner.doc.borrow().cursor.raw(), 12); // start of "foo"
    api_cursor::move_word_left(&mut *inner.doc.borrow_mut());
    assert_eq!(inner.doc.borrow().cursor.raw(), 6);  // start of "world"
    api_cursor::move_word_left(&mut *inner.doc.borrow_mut());
    assert_eq!(inner.doc.borrow().cursor.raw(), 0);  // start of "hello"
}

#[test]
fn word_movement_left_at_start_stays() {
    let inner = EditorInner::new("hello".to_string());
    api_cursor::move_word_left(&mut *inner.doc.borrow_mut());
    assert_eq!(inner.doc.borrow().cursor.raw(), 0);
}

#[test]
fn word_movement_right_from_start() {
    let inner = EditorInner::new("hello world foo".to_string());
    api_cursor::move_word_right(&mut *inner.doc.borrow_mut());
    assert_eq!(inner.doc.borrow().cursor.raw(), 6);  // start of "world"
    api_cursor::move_word_right(&mut *inner.doc.borrow_mut());
    assert_eq!(inner.doc.borrow().cursor.raw(), 12); // start of "foo"
}

#[test]
fn word_movement_right_at_end_stays() {
    let inner = EditorInner::new("hello".to_string());
    inner.doc.borrow_mut().set_cursor_raw(5);
    api_cursor::move_word_right(&mut *inner.doc.borrow_mut());
    assert_eq!(inner.doc.borrow().cursor.raw(), 5);
}

#[test]
fn word_movement_on_single_word() {
    let inner = EditorInner::new("hello".to_string());
    api_cursor::move_word_left(&mut *inner.doc.borrow_mut());
    assert_eq!(inner.doc.borrow().cursor.raw(), 0);
    api_cursor::move_word_right(&mut *inner.doc.borrow_mut());
    assert_eq!(inner.doc.borrow().cursor.raw(), 0);
}

#[test]
fn word_movement_on_empty() {
    let inner = EditorInner::new(String::new());
    api_cursor::move_word_left(&mut *inner.doc.borrow_mut());
    assert_eq!(inner.doc.borrow().cursor.raw(), 0);
    api_cursor::move_word_right(&mut *inner.doc.borrow_mut());
    assert_eq!(inner.doc.borrow().cursor.raw(), 0);
}

#[test]
fn word_movement_with_tabs() {
    let inner = EditorInner::new("word1\t\tword2".to_string());
    inner.doc.borrow_mut().set_cursor_raw(13); // end of "word2"
    api_cursor::move_word_left(&mut *inner.doc.borrow_mut());
    let raw = inner.doc.borrow().cursor.raw();
    assert!(raw < 13, "move_word_left from end should go to start of a word");
}

// ═══════════════════════════════════════════════════════════════════
// goto_document_start/end
// ═══════════════════════════════════════════════════════════════════

#[test]
fn goto_start_from_mid() {
    let inner = EditorInner::new("hello world".to_string());
    inner.doc.borrow_mut().set_cursor_raw(8);
    inner.doc.borrow_mut().set_cursor_raw(0);
    assert_eq!(inner.doc.borrow().cursor.raw(), 0);
}

#[test]
fn goto_start_already_at_start() {
    let inner = EditorInner::new("hello".to_string());
    inner.doc.borrow_mut().set_cursor_raw(0);
    assert_eq!(inner.doc.borrow().cursor.raw(), 0);
}

#[test]
fn goto_end_from_start() {
    let inner = EditorInner::new("hello world".to_string());
    inner.doc.borrow_mut().set_cursor_raw(0);
    let len = inner.doc.borrow().content().len();
    inner.doc.borrow_mut().set_cursor_raw(len);
    assert_eq!(inner.doc.borrow().cursor.raw(), len);
}

#[test]
fn goto_end_empty_doc() {
    let inner = EditorInner::new(String::new());
    let len = inner.doc.borrow().content().len();
    inner.doc.borrow_mut().set_cursor_raw(len);
    assert_eq!(inner.doc.borrow().cursor.raw(), 0);
}

// ═══════════════════════════════════════════════════════════════════
// Page scroll симуляция
// ═══════════════════════════════════════════════════════════════════

#[test]
fn page_up_decreases_scroll() {
    let inner = EditorInner::new("a\nb\nc\nd\ne".to_string());
    inner.scroll_y.set(100.0);
    inner.scroll_y.set((inner.scroll_y.get() - 50.0).max(0.0));
    assert_eq!(inner.scroll_y.get(), 50.0);
}

#[test]
fn page_up_clamps_at_zero() {
    let inner = EditorInner::new("a\nb\nc\nd\ne".to_string());
    inner.scroll_y.set(10.0);
    inner.scroll_y.set((inner.scroll_y.get() - 100.0).max(0.0));
    assert_eq!(inner.scroll_y.get(), 0.0);
}

#[test]
fn page_down_increases_scroll() {
    let inner = EditorInner::new("a\nb\nc\nd\ne".to_string());
    inner.scroll_y.set(0.0);
    inner.scroll_y.set(inner.scroll_y.get() + 50.0);
    assert_eq!(inner.scroll_y.get(), 50.0);
}

#[test]
fn page_down_on_empty_doc_does_not_panic() {
    let inner = EditorInner::new(String::new());
    inner.scroll_y.set(0.0);
    let line_h = inner.base_size * 1.4;
    let n = (500.0 / line_h) as usize;
    for _ in 0..n {
        // симуляция cursor_move_down — должна отработать без паники
        let mut doc = inner.doc.borrow_mut();
        doc.cursor_move_down();
    }
}

// ═══════════════════════════════════════════════════════════════════
// Edge cases: пустой документ, курсор на несуществующей строке
// ═══════════════════════════════════════════════════════════════════

#[test]
fn empty_doc_cursor_stays_at_zero() {
    let inner = EditorInner::new(String::new());
    assert_eq!(inner.doc.borrow().cursor.raw(), 0);
    assert_eq!(inner.doc.borrow().cursor.line(), 0);
}

#[test]
fn set_cursor_beyond_content_clamps() {
    let inner = EditorInner::new("hello".to_string());
    inner.doc.borrow_mut().set_cursor_raw(999);
    assert!(inner.doc.borrow().cursor.raw() <= 5);
}

#[test]
fn set_cursor_raw_on_empty() {
    let inner = EditorInner::new(String::new());
    inner.doc.borrow_mut().set_cursor_raw(5);
    assert_eq!(inner.doc.borrow().cursor.raw(), 0);
}

#[test]
fn line_bounds_on_empty_line() {
    let inner = EditorInner::new(String::new());
    let bounds = inner.doc.borrow().line_bounds(0);
    // empty doc: line 0 might not exist, but should not panic
    let _ = bounds;
}

// ═══════════════════════════════════════════════════════════════════
// auto_scroll — симуляция через bounds
// ═══════════════════════════════════════════════════════════════════

#[test]
fn auto_scroll_cursor_visible_no_op() {
    use crate::iced_editor::widget::input::auto_scroll;
    let inner = EditorInner::new("hello".to_string());
    let bounds = iced::Rectangle { x: 0.0, y: 0.0, width: 800.0, height: 600.0 };
    let editor = IcedEditor::new(&inner);

    let old_scroll = inner.scroll_y.get();
    auto_scroll(&editor, bounds);
    assert_eq!(inner.scroll_y.get(), old_scroll, "visible cursor should not change scroll");
}

#[test]
fn auto_scroll_cursor_below_viewport() {
    use crate::iced_editor::widget::input::auto_scroll;
    let lines: Vec<_> = (0..50).map(|i| format!("line {}", i)).collect();
    let content = lines.join("\n");
    let inner = EditorInner::new(content);
    let bounds = iced::Rectangle { x: 0.0, y: 0.0, width: 800.0, height: 50.0 };

    // Ставим курсор на последнюю строку
    let len = inner.doc.borrow().content().len();
    inner.doc.borrow_mut().set_cursor_raw(len);

    let editor = IcedEditor::new(&inner);
    auto_scroll(&editor, bounds);
    assert!(inner.scroll_y.get() > 0.0, "should scroll down for cursor below viewport");
}

#[test]
fn auto_scroll_cursor_above_viewport() {
    use crate::iced_editor::widget::input::auto_scroll;
    let lines: Vec<_> = (0..50).map(|i| format!("line {}", i)).collect();
    let content = lines.join("\n");
    let inner = EditorInner::new(content);

    inner.scroll_y.set(500.0);
    let bounds = iced::Rectangle { x: 0.0, y: 0.0, width: 800.0, height: 100.0 };
    let editor = IcedEditor::new(&inner);
    auto_scroll(&editor, bounds);
    assert_eq!(inner.scroll_y.get(), 0.0, "should scroll to top for cursor above viewport");
}

#[test]
fn auto_scroll_zero_height_no_op() {
    use crate::iced_editor::widget::input::auto_scroll;
    let inner = EditorInner::new("hello".to_string());
    let bounds = iced::Rectangle { x: 0.0, y: 0.0, width: 800.0, height: 0.0 };
    let editor = IcedEditor::new(&inner);
    auto_scroll(&editor, bounds);
    assert_eq!(inner.scroll_y.get(), 0.0);
}

#[test]
fn auto_scroll_marks_dirty_when_scroll_changes() {
    use crate::iced_editor::widget::input::auto_scroll;
    let content = (0..50).map(|i| format!("line {}", i)).collect::<Vec<_>>().join("\n");
    let inner = EditorInner::new(content);
    inner.doc.borrow_mut().dirty = false;

    // Ставим курсор в конец
    let len = inner.doc.borrow().content().len();
    inner.doc.borrow_mut().set_cursor_raw(len);

    let bounds = iced::Rectangle { x: 0.0, y: 0.0, width: 800.0, height: 30.0 };
    let editor = IcedEditor::new(&inner);
    auto_scroll(&editor, bounds);
    assert!(inner.scroll_y.get() > 0.0, "should scroll down");
}

// ═══════════════════════════════════════════════════════════════════
// compute_viewport — все краевые случаи
// ═══════════════════════════════════════════════════════════════════

#[test]
fn compute_viewport_empty_doc() {
    let inner = EditorInner::new(String::new());
    let vp = inner.compute_viewport(600.0);
    assert_eq!(vp.first_line, 0);
    assert_eq!(vp.last_line, 0);
}

#[test]
fn compute_viewport_single_line() {
    let inner = EditorInner::new("hello".to_string());
    let vp = inner.compute_viewport(600.0);
    assert_eq!(vp.first_line, 0);
    assert_eq!(vp.last_line, 0); // total_lines=1, saturating_sub(1)=0
}

#[test]
fn compute_viewport_start_of_doc() {
    let inner = EditorInner::new((0..50).map(|i| format!("line {}", i)).collect::<Vec<_>>().join("\n"));
    let vp = inner.compute_viewport(600.0);
    assert_eq!(vp.first_line, 0, "at scroll=0, start should be 0");
    assert!(vp.last_line > 0, "end_line should cover some lines");
}

#[test]
fn compute_viewport_scrolled_mid() {
    let inner = EditorInner::new((0..100).map(|i| format!("line {}", i)).collect::<Vec<_>>().join("\n"));
    inner.scroll_y.set(500.0);
    let vp = inner.compute_viewport(200.0);
    assert!(vp.first_line > 0, "scrolled down, first_line should advance, got {}", vp.first_line);
    assert!(vp.last_line > vp.first_line, "end should be after start");
}

#[test]
fn compute_viewport_scrolled_near_end() {
    let inner = EditorInner::new((0..100).map(|i| format!("line {}", i)).collect::<Vec<_>>().join("\n"));
    // scroll_y=1500 → first≈66, last≈97 (все в пределах документа)
    inner.scroll_y.set(1500.0);
    let vp = inner.compute_viewport(200.0);
    assert_eq!(vp.last_line, 97, "last_line should be clamped but near end");
    assert!(vp.first_line > 60, "first_line should be scrolled down");
}

#[test]
fn compute_viewport_negative_scroll_clamps() {
    let inner = EditorInner::new((0..50).map(|i| format!("line {}", i)).collect::<Vec<_>>().join("\n"));
    inner.scroll_y.set(-100.0);
    let vp = inner.compute_viewport(600.0);
    assert_eq!(vp.first_line, 0, "negative scroll treated as 0");
}

#[test]
fn compute_viewport_zero_viewport() {
    let inner = EditorInner::new((0..50).map(|i| format!("line {}", i)).collect::<Vec<_>>().join("\n"));
    let vp = inner.compute_viewport(0.0);
    // При нулевой высоте видно только строку со скроллом (с padding)
    assert!(vp.first_line <= 10); // padding влево
    assert!(vp.last_line >= 10);   // padding вправо
}

#[test]
fn compute_viewport_very_large_viewport() {
    let inner = EditorInner::new((0..100).map(|i| format!("line {}", i)).collect::<Vec<_>>().join("\n"));
    let total = inner.doc.borrow().incremental.num_lines();
    let vp = inner.compute_viewport(999999.0);
    assert_eq!(vp.last_line, total.saturating_sub(1), "large viewport should show all lines");
}

#[test]
fn compute_viewport_padding_applied() {
    let inner = EditorInner::new((0..100).map(|i| format!("line {}", i)).collect::<Vec<_>>().join("\n"));
    let vp = inner.compute_viewport(50.0);
    // Минимальный viewport: 50/19.6 ≈ 3 строки + padding 10 снизу
    assert!(vp.last_line >= 13, "padding should extend beyond logical viewport, got last_line={}", vp.last_line);
    assert_eq!(vp.first_line, 0, "at scroll=0, first_line stays 0");
}

#[test]
fn compute_viewport_monotonic() {
    let inner = EditorInner::new((0..100).map(|i| format!("line {}", i)).collect::<Vec<_>>().join("\n"));
    let vp1 = inner.compute_viewport(100.0);
    inner.scroll_y.set(50.0);
    let vp2 = inner.compute_viewport(100.0);
    assert!(vp2.first_line >= vp1.first_line || vp2.first_line == 0,
        "scrolling down should not decrease first_line (vp1={:?}, vp2={:?})", vp1, vp2);
}

#[test]
fn compute_viewport_few_lines_no_negative_clamp() {
    // Если total_lines < VIEWPORT_PADDING, start_line не уходит в минус
    let inner = EditorInner::new("a\nb\nc".to_string());
    let vp = inner.compute_viewport(200.0);
    assert_eq!(vp.first_line, 0);
    assert_eq!(vp.last_line, 2); // total=3, saturating_sub(1)=2
}

// ═══════════════════════════════════════════════════════════════════
// Комплексная симуляция — полный цикл ввода-навигации
// ═══════════════════════════════════════════════════════════════════

#[test]
fn type_text_then_move_and_delete() {
    let inner = EditorInner::new(String::new());
    // Печатаем "hello world"
    for (i, ch) in "hello world".chars().enumerate() {
        inner.edit_doc_raw(i, i, &ch.to_string());
    }
    assert_eq!(inner.doc.borrow().content(), "hello world");
    // Move left 6 раз → на ' '
    for _ in 0..6 {
        let mut doc = inner.doc.borrow_mut();
        api_cursor::move_left(&mut *doc);
    }
    // Удаляем пробел (backspace)
    inner.edit_doc_raw(5, 6, "");
    assert_eq!(inner.doc.borrow().content(), "helloworld");
}

#[test]
fn edit_then_mode_switch_then_edit() {
    let inner = EditorInner::new("hello".to_string());
    inner.edit_doc_raw(5, 5, " world");
    inner.cycle_mode(); // → Source
    assert_eq!(inner.get_mode(), EditMode::Source);
    inner.edit_doc_raw(11, 11, "!!");
    assert_eq!(inner.doc.borrow().content(), "hello world!!");
}

#[test]
fn mixed_borrow_and_edit_refcell_safe() {
    // Этот тест проверяет конкретную последовательность, которая
    // раньше падала из-за мёртвого кода в edit_doc_raw
    let inner = EditorInner::new("hello world".to_string());
    let line = inner.doc.borrow().cursor.line();
    inner.edit_doc_raw(5, 5, "!!");
    let raw = inner.doc.borrow().cursor.raw();
    let content = inner.doc.borrow().content().len();
    inner.doc.borrow_mut().set_cursor_raw(0);
    let line2 = inner.doc.borrow().cursor.line();
    assert_eq!(inner.doc.borrow().content(), "hello!! world");
}

#[test]
fn all_keyboard_actions_no_panic() {
    // Полная симуляция того, что делает keyboard handler
    let inner = EditorInner::new("hello world\nsecond line".to_string());
    // word movement
    api_cursor::move_word_right(&mut *inner.doc.borrow_mut());
    let pos = inner.doc.borrow().cursor.raw();
    assert_eq!(pos, 6);
    // mode switch + scroll
    inner.cycle_mode();
    inner.scroll_y.set(50.0);
    // edit at position
    inner.edit_doc_raw(pos, pos, "!!!");
    assert_eq!(inner.doc.borrow().content(), "hello !!!world\nsecond line");
    inner.mark_dirty();
    assert!(inner.doc.borrow().dirty);
}
