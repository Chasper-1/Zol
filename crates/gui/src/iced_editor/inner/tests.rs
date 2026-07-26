use super::*;
use api::cursor as api_cursor;
use editor::state::EditMode;

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
    assert!(inner.doc.borrow().dirty);
}

#[test]
fn new_shaped_doc_has_lines() {
    let inner = EditorInner::new("line1\nline2\nline3".to_string());
    let shaped = inner.shaped_doc.borrow();
    assert!(
        shaped.line_count() > 0,
        "shaped_doc should have lines after build"
    );
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
    inner.edit_doc(|doc| {
        doc.incremental.edit(0, 0, "**bold**");
    });
    let cache = inner.cache.borrow();
    assert!(cache.lines.len() >= 1);
}

#[test]
fn edit_doc_sets_dirty() {
    let inner = EditorInner::new("x".to_string());
    inner.doc.borrow_mut().dirty = false;
    inner.edit_doc(|doc| {
        doc.incremental.edit(1, 1, "y");
    });
    assert!(inner.doc.borrow().dirty, "edit_doc should set dirty=true");
}

#[test]
fn edit_doc_cache_updates_after_content_change() {
    let inner = EditorInner::new("hello".to_string());
    inner.edit_doc(|doc| {
        doc.incremental.edit(5, 5, " **world**");
    });
    let cache = inner.cache.borrow();
    assert!(
        !cache.lines.is_empty(),
        "cache should be rebuilt after content change"
    );
}

#[test]
fn edit_doc_multiple_calls() {
    let inner = EditorInner::new("".to_string());
    inner.edit_doc(|doc| {
        doc.incremental.edit(0, 0, "a");
    });
    inner.edit_doc(|doc| {
        let len = doc.incremental.source.len();
        doc.incremental.edit(len, len, "b");
    });
    assert_eq!(inner.doc.borrow().content(), "ab");
}

// ═══════════════════════════════════════════════════════════════════
// RefCell-specific тесты
//
// Эти тесты проверяют, что последовательность borrow/borrow_mut на
// RefCell<Document> не падает. Раньше edit_doc_raw содержала мёртвый
// код `let cursor = &self.doc.borrow().cursor;`, который продлевал
// жизнь Ref и взрывался на следующем borrow_mut().
// ═══════════════════════════════════════════════════════════════════

#[test]
fn edit_doc_raw_typing_does_not_panic() {
    // Многократный edit_doc_raw — симулирует печать символов
    let inner = EditorInner::new(String::new());
    for (i, ch) in "hello".chars().enumerate() {
        inner.edit_doc_raw(i, i, &ch.to_string());
    }
    assert_eq!(inner.doc.borrow().content(), "hello");
}

#[test]
fn edit_doc_raw_multiple_chars_at_once() {
    let inner = EditorInner::new(String::new());
    inner.edit_doc_raw(0, 0, "abc");
    inner.edit_doc_raw(3, 3, "def");
    assert_eq!(inner.doc.borrow().content(), "abcdef");
}

#[test]
fn edit_doc_raw_backspace_simulation() {
    // Симулирует то, что делает keyboard handler при Backspace:
    // 1. borrow() читает cursor.raw, content, prev_grapheme
    // 2. edit_doc_raw(from, to, "")
    // 3. borrow_mut() set_cursor_raw
    let inner = EditorInner::new("hello".to_string());
    // Ставим курсор в конец (по умолчанию он в 0)
    inner.doc.borrow_mut().set_cursor_raw(5);
    {
        let doc = inner.doc.borrow();
        let raw = doc.cursor.raw();
        assert_eq!(raw, 5);
        assert!(!doc.content().is_empty());
        let prev = editor::cursor::prev_grapheme_boundary(doc.content(), raw).unwrap_or(0);
        assert_eq!(prev, 4);
    }
    inner.edit_doc_raw(4, 5, "");
    {
        let mut doc = inner.doc.borrow_mut();
        doc.set_cursor_raw(4);
    }
    assert_eq!(inner.doc.borrow().content(), "hell");
}

#[test]
fn edit_doc_raw_backspace_empty_doc_does_not_panic() {
    let inner = EditorInner::new(String::new());
    let (from, to) = {
        let doc = inner.doc.borrow();
        let raw = doc.cursor.raw();
        if raw == 0 || doc.content().is_empty() {
            (0, 0)
        } else {
            let prev = editor::cursor::prev_grapheme_boundary(doc.content(), raw).unwrap_or(0);
            (prev, raw)
        }
    };
    assert_eq!(from, 0);
    assert_eq!(to, 0);
    // edit_doc_raw с (0,0) не должен паниковать
    inner.edit_doc_raw(from, to, "");
}

#[test]
fn edit_doc_raw_enter_simulation() {
    // Симулирует Enter из keyboard handler:
    // 1. doc.borrow().cursor.raw() — временный Ref
    // 2. edit_doc_raw(raw, raw, "\n")
    // 3. doc.borrow_mut() set_cursor_raw, reset_col_visual
    let inner = EditorInner::new("abc".to_string());
    // Ставим курсор в конец
    inner.doc.borrow_mut().set_cursor_raw(3);

    let raw = inner.doc.borrow().cursor.raw();
    assert_eq!(raw, 3);

    inner.edit_doc_raw(raw, raw, "\n");

    {
        let mut doc = inner.doc.borrow_mut();
        doc.set_cursor_raw(raw + 1);
        doc.cursor.reset_col_visual();
    } // drop RefMut

    assert_eq!(inner.doc.borrow().content(), "abc\n");
}

#[test]
fn edit_doc_raw_delete_simulation() {
    // Симулирует Delete из keyboard handler
    let inner = EditorInner::new("hello".to_string());
    // Ставим курсор в начало
    inner.doc.borrow_mut().set_cursor_raw(0);

    let (from, to) = {
        let doc = inner.doc.borrow();
        let raw = doc.cursor.raw();
        if raw >= doc.content().len() || doc.content().is_empty() {
            (0, 0)
        } else {
            let next = editor::cursor::next_grapheme_boundary(doc.content(), raw)
                .unwrap_or(doc.content().len());
            (raw, next)
        }
    };
    assert_eq!(from, 0);
    assert_eq!(to, 1);

    inner.edit_doc_raw(from, to, "");
    assert_eq!(inner.doc.borrow().content(), "ello");
}

#[test]
fn edit_doc_raw_delete_at_end_does_not_panic() {
    let inner = EditorInner::new("hi".to_string());
    // Ставим курсор в конец
    inner.doc.borrow_mut().set_cursor_raw(2);

    let (from, to) = {
        let doc = inner.doc.borrow();
        let raw = doc.cursor.raw();
        if raw >= doc.content().len() || doc.content().is_empty() {
            (0, 0)
        } else {
            let next = editor::cursor::next_grapheme_boundary(doc.content(), raw)
                .unwrap_or(doc.content().len());
            (raw, next)
        }
    };
    assert_eq!(from, 0);
    assert_eq!(to, 0);
    inner.edit_doc_raw(from, to, ""); // не паникует
}

#[test]
fn edit_doc_raw_text_input_simulation() {
    // Симулирует текстовый ввод из keyboard handler
    let inner = EditorInner::new(String::new());

    let raw = inner.doc.borrow().cursor.raw();
    let text = "hello";
    inner.edit_doc_raw(raw, raw, text);

    {
        let mut doc = inner.doc.borrow_mut();
        doc.set_cursor_raw(raw + text.len());
    } // drop RefMut

    assert_eq!(inner.doc.borrow().content(), "hello");
    assert_eq!(inner.doc.borrow().cursor.raw(), 5);
}

#[test]
fn edit_doc_raw_refcell_no_leak() {
    // Проверяет, что внутри edit_doc_raw нет живых Ref после
    // возврата. Если был бы мёртвый код как раньше — тест упадёт.
    let inner = EditorInner::new(String::new());

    // Первый вызов — нормально
    inner.edit_doc_raw(0, 0, "a");

    // Сразу второй — если бы Ref не дропнулся, borrow_mut внутри
    // edit_doc_raw упал бы
    inner.edit_doc_raw(1, 1, "b");

    // Третий для верности
    inner.edit_doc_raw(2, 2, "c");

    assert_eq!(inner.doc.borrow().content(), "abc");
}

#[test]
fn edit_doc_with_closure_refcell() {
    // edit_doc — принимает FnOnce(&mut Document). Внутри он делает
    // borrow_mut, потом borrow, потом borrow_mut — проверяем что
    // не падает.
    let inner = EditorInner::new(String::new());

    inner.edit_doc(|doc| {
        doc.incremental.edit(0, 0, "**hello**");
    });

    // После edit_doc все Ref/RefMut должны быть дропнуты
    assert_eq!(inner.doc.borrow().content(), "**hello**");
}

#[test]
fn edit_doc_raw_intensive_does_not_panic() {
    // Интенсивное использование — 100 вызовов edit_doc_raw подряд
    let inner = EditorInner::new(String::new());
    let text = "x".repeat(100);
    for (i, ch) in text.chars().enumerate() {
        inner.edit_doc_raw(i, i, &ch.to_string());
    }
    assert_eq!(inner.doc.borrow().content().len(), 100);
    assert_eq!(inner.doc.borrow().content(), text);
}

#[test]
fn edit_doc_raw_with_newlines_does_not_panic() {
    let inner = EditorInner::new("line1".to_string());
    inner.edit_doc_raw(5, 5, "\n");
    inner.edit_doc_raw(6, 6, "line2");
    assert_eq!(inner.doc.borrow().content(), "line1\nline2");

    // Проверяем что количество строк обновилось
    assert!(inner.doc.borrow().incremental.num_lines() >= 2);
}

#[test]
fn move_cursor_after_edit_doc_raw() {
    // После edit_doc_raw можно двигать курсор
    let inner = EditorInner::new("hello".to_string());
    inner.doc.borrow_mut().set_cursor_raw(0);

    // Пишем в начало
    inner.edit_doc_raw(0, 0, ">> ");

    // Двигаем курсор
    let mut doc = inner.doc.borrow_mut();
    doc.set_cursor_raw(0);
    assert_eq!(doc.content(), ">> hello");
}

#[test]
fn concurrent_borrow_and_edit_mixed() {
    // Смешанные borrow/borrow_mut в разных порядках
    let inner = EditorInner::new("hello world".to_string());

    // borrow → edit_doc → borrow
    let _line = inner.doc.borrow().cursor.line();
    inner.edit_doc_raw(5, 5, "!!");
    let _raw = inner.doc.borrow().cursor.raw();

    // borrow → borrow_mut → borrow
    let _content = inner.doc.borrow().content().len();
    inner.doc.borrow_mut().set_cursor_raw(0);
    let _line2 = inner.doc.borrow().cursor.line();

    assert_eq!(inner.doc.borrow().content(), "hello!! world");
}

#[test]
fn edit_doc_raw_at_line_boundaries() {
    // Правки на границах строк
    let inner = EditorInner::new("abc\ndef\nghi".to_string());

    // Вставляем в конец первой строки
    inner.edit_doc_raw(3, 3, "123");
    assert_eq!(inner.doc.borrow().content(), "abc123\ndef\nghi");

    // Удаляем конец первой строки и newline
    inner.edit_doc_raw(6, 7, "");
    assert_eq!(inner.doc.borrow().content(), "abc123def\nghi");

    // Вставляем много текста
    inner.edit_doc_raw(0, 0, "START\n");
    assert_eq!(inner.doc.borrow().content(), "START\nabc123def\nghi");
}

// ═══════════════════════════════════════════════════════════════════
// Хоткеи: режимы, word movement, scroll
// ═══════════════════════════════════════════════════════════════════

#[test]
fn esc_switches_to_preview() {
    let inner = EditorInner::new("test".to_string());
    assert_eq!(inner.get_mode(), EditMode::LivePreview);

    inner.set_mode(EditMode::Source);
    assert_eq!(inner.get_mode(), EditMode::Source);

    // Esc → Preview
    inner.set_mode(EditMode::Preview);
    assert_eq!(inner.get_mode(), EditMode::Preview);
}

#[test]
fn cycle_mode_loops() {
    let inner = EditorInner::new("test".to_string());
    assert_eq!(inner.get_mode(), EditMode::LivePreview);

    inner.cycle_mode(); // LivePreview → Source
    assert_eq!(inner.get_mode(), EditMode::Source);

    inner.cycle_mode(); // Source → Preview
    assert_eq!(inner.get_mode(), EditMode::Preview);

    inner.cycle_mode(); // Preview → LivePreview
    assert_eq!(inner.get_mode(), EditMode::LivePreview);
}

#[test]
fn word_movement_left() {
    let inner = EditorInner::new("hello world foo".to_string());
    // cursor starts at 0
    inner.doc.borrow_mut().set_cursor_raw(15); // end of "foo"

    api_cursor::move_word_left(&mut *inner.doc.borrow_mut());
    assert_eq!(inner.doc.borrow().cursor.raw(), 12); // start of "foo"

    api_cursor::move_word_left(&mut *inner.doc.borrow_mut());
    assert_eq!(inner.doc.borrow().cursor.raw(), 6); // start of "world"

    api_cursor::move_word_left(&mut *inner.doc.borrow_mut());
    assert_eq!(inner.doc.borrow().cursor.raw(), 0); // start of "hello"
}

#[test]
fn word_movement_right() {
    let inner = EditorInner::new("hello world foo".to_string());

    api_cursor::move_word_right(&mut *inner.doc.borrow_mut());
    assert_eq!(inner.doc.borrow().cursor.raw(), 6); // start of "world"

    api_cursor::move_word_right(&mut *inner.doc.borrow_mut());
    assert_eq!(inner.doc.borrow().cursor.raw(), 12); // start of "foo"

    // последнее слово — не двигается (нет следующего слова)
    api_cursor::move_word_right(&mut *inner.doc.borrow_mut());
    assert_eq!(inner.doc.borrow().cursor.raw(), 12);
}

#[test]
fn word_movement_does_not_panic_on_single_word() {
    let inner = EditorInner::new("hello".to_string());
    api_cursor::move_word_left(&mut *inner.doc.borrow_mut());
    assert_eq!(inner.doc.borrow().cursor.raw(), 0);

    // одно слово — move_word_right не двигает
    api_cursor::move_word_right(&mut *inner.doc.borrow_mut());
    assert_eq!(inner.doc.borrow().cursor.raw(), 0);
}

#[test]
fn goto_document_start() {
    let inner = EditorInner::new("hello world".to_string());
    inner.doc.borrow_mut().set_cursor_raw(8);

    // Ctrl+Home
    inner.doc.borrow_mut().set_cursor_raw(0);
    assert_eq!(inner.doc.borrow().cursor.raw(), 0);
}

#[test]
fn goto_document_end() {
    let inner = EditorInner::new("hello world".to_string());
    inner.doc.borrow_mut().set_cursor_raw(0);

    // Ctrl+End
    let len = inner.doc.borrow().content().len();
    inner.doc.borrow_mut().set_cursor_raw(len);
    assert_eq!(inner.doc.borrow().cursor.raw(), 11);
}

#[test]
fn scroll_page_down_does_not_panic() {
    let inner = EditorInner::new("a\nb\nc\nd\ne".to_string());
    // scroll_y is f32, just change it
    inner.scroll_y.set(100.0);
    assert_eq!(inner.scroll_y.get(), 100.0);
    inner.mark_dirty();
}

#[test]
fn scroll_page_up_does_not_panic() {
    let inner = EditorInner::new("a\nb\nc\nd\ne".to_string());
    inner.scroll_y.set(50.0);
    assert_eq!(inner.scroll_y.get(), 50.0);
    // scroll up — decrease
    let new = (inner.scroll_y.get() - 30.0).max(0.0);
    inner.scroll_y.set(new);
    assert_eq!(inner.scroll_y.get(), 20.0);
    inner.mark_dirty();
}

#[test]
fn scroll_page_up_clamps_at_zero() {
    let inner = EditorInner::new("a\nb\nc\nd\ne".to_string());
    inner.scroll_y.set(10.0);
    let new = (inner.scroll_y.get() - 100.0).max(0.0);
    inner.scroll_y.set(new);
    assert_eq!(inner.scroll_y.get(), 0.0);
}

#[test]
fn all_keyboard_actions_refcell_safe() {
    // Симулирует последовательность: movement → edit → mode switch → scroll
    // Проверяет что нет RefCell конфликтов
    let inner = EditorInner::new("hello world\nsecond line".to_string());

    // word movement: от 0 → start of "world" (pos 6)
    api_cursor::move_word_right(&mut *inner.doc.borrow_mut());
    let pos = inner.doc.borrow().cursor.raw();
    assert_eq!(pos, 6);

    // mode switch
    inner.cycle_mode();
    assert_eq!(inner.get_mode(), EditMode::Source);

    // scroll
    inner.scroll_y.set(50.0);
    assert_eq!(inner.scroll_y.get(), 50.0);

    // edit: вставляем перед "world" → "hello !!!world"
    inner.edit_doc_raw(pos, pos, "!!!");
    assert_eq!(inner.doc.borrow().content(), "hello !!!world\nsecond line");
    inner.mark_dirty();
    assert!(inner.doc.borrow().dirty);
}
