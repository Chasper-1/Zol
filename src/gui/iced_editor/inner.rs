//! Состояние редактора ([`EditorInner`]).

use std::cell::{Cell, RefCell};

use crate::editor::cache::DocumentCache;
use crate::editor::cursor::Cursor;
use crate::editor::render::{self, ShapedDocument};
use crate::editor::state::EditMode;
use crate::editor::theme::EditorTheme;

/// Имя файла по умолчанию для загрузки/сохранения.
const DEFAULT_FILE: &str = "notes.zml";

/// Состояние редактора.
///
/// Поля-`RefCell` обеспечивают interior mutability — виджет держит
/// `&EditorInner`, а мутации происходят через `.borrow_mut()` отдельных
/// полей. `dirty` сигнализирует `draw()`, что `shaped_doc` нужно
/// перестроить.
pub struct EditorInner {
    pub content: RefCell<String>,
    pub cursor: RefCell<Cursor>,
    pub shaped_doc: RefCell<ShapedDocument>,
    pub cache: DocumentCache,
    pub mode: EditMode,
    pub dirty: Cell<bool>,
    pub base_size: f32,
    pub heading_size: f32,
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
        let cache = DocumentCache::default();
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
            content: RefCell::new(content),
            cursor: RefCell::new(Cursor::new()),
            shaped_doc: RefCell::new(shaped_doc),
            cache,
            mode: EditMode::LivePreview,
            dirty: Cell::new(false),
            base_size,
            heading_size,
            theme,
            scroll_y: Cell::new(0.0),
            file_path: DEFAULT_FILE.to_string(),
        }
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
        assert_eq!(inner.content.borrow().as_str(), "");
        assert!(!inner.dirty.get());
    }

    #[test]
    fn new_with_text() {
        let inner = EditorInner::new("hello world".to_string());
        assert_eq!(inner.content.borrow().as_str(), "hello world");
        assert!(!inner.dirty.get());
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
        assert_eq!(inner.file_path, "notes.zml");
        assert_eq!(inner.mode, EditMode::LivePreview);
        assert_eq!(inner.scroll_y.get(), 0.0);
    }
}
