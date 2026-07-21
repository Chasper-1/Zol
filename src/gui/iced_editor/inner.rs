//! Состояние редактора ([`EditorInner`]).
//!
//! ## Важно: синхронизация кеша
//!
//! Все мутации документа должны проходить через [`EditorInner::edit_doc()`],
//! который после изменения автоматически перестраивает `DocumentCache`
//! и помечает `shaped_doc` для перешейпа.
//!
//! Прямая работа с `doc.borrow_mut()` — только для чтения, мутации —
//! только через `edit_doc`.

use std::cell::{Cell, RefCell};

use crate::document::Document;
use crate::editor::cache::DocumentCache;
use crate::editor::render::{self, ShapedDocument};
use crate::editor::state::EditMode;
use crate::editor::theme::EditorTheme;

/// Имя файла по умолчанию для загрузки/сохранения.
const DEFAULT_FILE: &str = "notes.zoll";

/// Состояние редактора.
///
/// Поля-`RefCell` обеспечивают interior mutability — виджет держит
/// `&EditorInner`, а мутации происходят через `.borrow_mut()`.
///
/// `doc` — единый источник правды для контента и курсора.
/// `cache` — кеш разметки, синхронизируется через [`edit_doc()`](EditorInner::edit_doc).
/// `shaped_doc` — сшейпленный буфер cosmic-text для отрисовки.
pub struct EditorInner {
    /// Документ: контент + курсор + dirty-флаг.
    pub doc: RefCell<Document>,
    /// Сшейпленный документ (cosmic-text Buffer).
    pub shaped_doc: RefCell<ShapedDocument>,
    /// Кеш разметки — перестраивается при каждом изменении документа.
    pub cache: RefCell<DocumentCache>,
    /// Режим редактирования.
    pub mode: EditMode,
    /// Базовый размер шрифта.
    pub base_size: f32,
    /// Размер заголовков.
    pub heading_size: f32,
    /// Тема оформления.
    pub theme: EditorTheme,
    /// Вертикальный сдвиг прокрутки (пиксели).
    pub scroll_y: Cell<f32>,
    /// Путь к файлу (для загрузки/сохранения).
    pub file_path: String,
}

impl EditorInner {
    /// Создать редактор с заданным содержимым (сразу шейпим документ).
    pub fn new(content: String) -> Self {
        let base_size = 14.0;
        let heading_size = 24.0;
        let theme = EditorTheme::default();
        let doc = Document::new(&content);
        let cache = crate::editor::markup::parse_document(&content);
        let metrics = cosmic_text::Metrics::new(base_size, base_size * 1.4);
        let mut shaped_doc = ShapedDocument::new(cosmic_text::Buffer::new_empty(metrics));

        crate::editor::font::init();
        render::build(
            &mut shaped_doc,
            &content,
            &cache,
            EditMode::LivePreview,
            0,
            &theme,
            base_size,
            heading_size,
            0.0,
            None,
        );

        Self {
            doc: RefCell::new(doc),
            shaped_doc: RefCell::new(shaped_doc),
            cache: RefCell::new(cache),
            mode: EditMode::LivePreview,
            base_size,
            heading_size,
            theme,
            scroll_y: Cell::new(0.0),
            file_path: DEFAULT_FILE.to_string(),
        }
    }

    /// Применить замыкание к документу, перестроить кеш и пометить dirty.
    ///
    /// Единственный разрешённый способ мутации документа.
    /// После вызова `f`:
    /// 1. Перестраивается `DocumentCache` из нового содержимого.
    /// 2. `doc.dirty` выставляется в `true` (триггерит перешейп в `draw()`).
    ///
    /// # Пример
    /// ```ignore
    /// inner.edit_doc(|doc| {
    ///     api::text::insert_at_cursor(doc, "hello");
    /// });
    /// ```
    pub fn edit_doc<F>(&self, f: F)
    where
        F: FnOnce(&mut Document),
    {
        // 1. Мутируем документ (RefMut живёт пока вызвано f, потом падает)
        f(&mut self.doc.borrow_mut());

        // 2. Перестраиваем кеш разметки из нового содержимого
        let new_content = self.doc.borrow().content.clone();
        *self.cache.borrow_mut() = crate::editor::markup::parse_document(&new_content);

        // 3. Помечаем dirty для перешейпа в draw()
        self.doc.borrow_mut().dirty = true;
    }

    /// Пометить документ как dirty (для перешейпа).
    /// Вызывается после косвенных изменений (скролл, смена режима).
    pub fn mark_dirty(&self) {
        self.doc.borrow_mut().dirty = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------
    // EditorInner::new — проверяем, что не deadlock и shaped doc построен
    // ------------------------------------------------------------------

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
        assert_eq!(inner.mode, EditMode::LivePreview);
        assert_eq!(inner.scroll_y.get(), 0.0);
    }

    // ------------------------------------------------------------------
    // edit_doc — проверяем синхронизацию кеша и dirty
    // ------------------------------------------------------------------

    #[test]
    fn edit_doc_insert_syncs_cache() {
        let inner = EditorInner::new("".to_string());
        inner.edit_doc(|doc| {
            doc.content.insert_str(0, "**bold**");
        });
        // Кеш должен содержать сегменты для разметки
        let cache = inner.cache.borrow();
        assert!(cache.lines.len() >= 1);
    }

    #[test]
    fn edit_doc_sets_dirty() {
        let inner = EditorInner::new("x".to_string());
        inner.doc.borrow_mut().dirty = false; // сбросили dirty
        inner.edit_doc(|doc| {
            doc.content.push_str("y");
        });
        assert!(inner.doc.borrow().dirty, "edit_doc should set dirty=true");
    }

    #[test]
    fn edit_doc_cache_updates_after_content_change() {
        let inner = EditorInner::new("hello".to_string());
        inner.edit_doc(|doc| {
            doc.content.push_str(" **world**");
        });
        let cache = inner.cache.borrow();
        // Хотя бы одна линия должна быть в кеше
        assert!(
            !cache.lines.is_empty(),
            "cache should be rebuilt after content change"
        );
    }

    #[test]
    fn edit_doc_multiple_calls() {
        let inner = EditorInner::new("".to_string());
        inner.edit_doc(|doc| doc.content.push_str("a"));
        inner.edit_doc(|doc| doc.content.push_str("b"));
        assert_eq!(inner.doc.borrow().content, "ab");
    }
}
