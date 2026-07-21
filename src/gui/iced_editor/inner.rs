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
