use std::cell::{Cell, RefCell};

use editor::document::Document;
use editor::cache::DocumentCache;
use editor::render::{self, ShapedDocument};
use editor::state::EditMode;
use editor::theme::EditorTheme;
use editor::Viewport;

/// Имя файла по умолчанию.
const DEFAULT_FILE: &str = "notes.zoll";

/// Количество строк буфера над и под viewport (чтобы не мерцало при скролле).
const VIEWPORT_PADDING: usize = 10;

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
    /// Последний вычисленный viewport (строки, которые нужно парсить/рендерить).
    pub viewport: Cell<Viewport>,
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

        let total_lines = doc.incremental.num_lines();
        let initial_vp = Viewport::new(0, total_lines.saturating_sub(1).min(99));

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
            viewport: Cell::new(initial_vp),
        }
    }

    /// Вычислить viewport из scroll_y и высоты виджета.
    ///
    /// Использует `base_size * 1.4` как приблизительную высоту строки.
    /// Точное вычисление требует shaped buffer (chicken-and-egg),
    /// поэтому на первом проходе — приближение.
    pub fn compute_viewport(&self, viewport_px: f32) -> Viewport {
        let line_h = self.base_size * 1.4;
        let total = self.doc.borrow().incremental.num_lines();
        if total == 0 {
            return Viewport::new(0, 0);
        }
        let scroll = self.scroll_y.get().max(0.0);
        let first = (scroll / line_h).floor() as usize;
        let last = ((scroll + viewport_px) / line_h).ceil() as usize;

        // Добавляем буфер
        let first = first.saturating_sub(VIEWPORT_PADDING);
        let last = (last + VIEWPORT_PADDING).min(total.saturating_sub(1));
        Viewport::new(first, last)
    }

    pub fn mark_dirty(&self) {
        self.doc.borrow_mut().dirty = true;
    }
}
