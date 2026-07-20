use crate::editor::theme::EditorTheme;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EditMode {
    Preview,
    LivePreview,
    Source,
}

pub struct EditorState {
    pub theme: EditorTheme,
    pub content: String,
    pub mode: EditMode,
}

impl EditorState {
    pub fn new(theme: EditorTheme, text: String) -> Self {
        Self {
            theme,
            content: text,
            mode: EditMode::LivePreview,
        }
    }
}
