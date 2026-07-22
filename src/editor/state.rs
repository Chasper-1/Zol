/// Режим редактирования.
///
/// Определяет, как отображается разметка и можно ли редактировать текст.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EditMode {
    /// Спрятать маркеры разметки, только чтение (редактирование отключено).
    Preview,
    /// Активная строка — Source, остальные — Preview.
    LivePreview,
    /// Показать сырую разметку с маркерами, полное редактирование.
    Source,
}

impl EditMode {
    /// Можно ли редактировать текст в этом режиме.
    pub fn is_editable(&self) -> bool {
        !matches!(self, EditMode::Preview)
    }

    /// Следующий режим по циклу: Preview → LivePreview → Source → Preview.
    pub fn next(&self) -> Self {
        match self {
            EditMode::Preview => EditMode::LivePreview,
            EditMode::LivePreview => EditMode::Source,
            EditMode::Source => EditMode::Preview,
        }
    }

    /// Название режима в нижнем регистре.
    pub fn name(&self) -> &'static str {
        match self {
            EditMode::Preview => "preview",
            EditMode::LivePreview => "live_preview",
            EditMode::Source => "source",
        }
    }
}
