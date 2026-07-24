use std::cell::{Cell, RefCell};

use editor::document::Document;
use editor::cache::DocumentCache;
use editor::render::{self, ShapedDocument};
use editor::state::EditMode;
use editor::theme::EditorTheme;

/// Имя файла по умолчанию.
const DEFAULT_FILE: &str = "notes.zoll";

/// Состояние редактора.
pub struct EditorInner {
    pub doc: RefCell<Document>,
    pub shaped_doc: RefCell<ShapedDocument>,
    pub cache: RefCell<DocumentCache>,
    pub mode: Cell<EditMode>,
    pub base_size: f32,
    pub heading_size: f32,
    pub theme: EditorTheme,
    pub scroll_y: Cell<f32>,
    pub file_path: String,
}

impl EditorInner {
    pub fn new(content: String) -> Self {
        let base_size = 14.0;
        let heading_size = 24.0;
        let theme = EditorTheme::default();
        let doc = Document::new(&content);

        // Кеш из IncrementalDoc (без полного parse_full)
        let cache = editor::markup::segmenter::incremental_to_cache(&doc.incremental);

        let metrics = cosmic_text::Metrics::new(base_size, base_size * 1.4);
        let mut shaped_doc = ShapedDocument::new(cosmic_text::Buffer::new_empty(metrics), vec![]);

        editor::font::init();
        render::build(
            &mut shaped_doc,
            doc.content(),
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
            mode: Cell::new(EditMode::LivePreview),
            base_size,
            heading_size,
            theme,
            scroll_y: Cell::new(0.0),
            file_path: DEFAULT_FILE.to_string(),
        }
    }

    pub fn mark_dirty(&self) {
        self.doc.borrow_mut().dirty = true;
    }
}
