use crate::editor::theme::EditorTheme;
use crate::editor::markup::{DocumentCache, parse_document};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EditMode {
    Preview,     // Чистый просмотр, ссылки кликабельны
    LivePreview, // Гибрид: активная строка — код, остальные — красивые
    Source,      // Чистый исходный код
}

pub struct EditorState {
    pub theme: EditorTheme,
    pub content: String,
    pub document_cache: DocumentCache,
    pub mode: EditMode,
}

impl EditorState {
    pub fn new(theme: EditorTheme, text: String) -> Self {
        let document_cache = parse_document(&text);
    
        Self {
            theme,
            content: text,
            document_cache,
            mode: EditMode::LivePreview,
        }
    }
}