// Режим отображения редактора.
//
// Определяет, как отображается разметка и можно ли редактировать текст.
// Все режимы строятся из Source (прямой текст с форматированием):
//
// - **Source** — маркеры видны, полное редактирование, цвета/размеры.
// - **Preview** — маркеры скрыты, редактирование отключено, курсора нет.
// - **Live** — как Preview (маркеры скрыты), но редактирование активно.
//            Ctrl+Space точечно раскрывает маркеры под курсором.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EditMode {
    // Маркеры скрыты, только чтение, курсора нет.
    Preview,
    // Маркеры скрыты по умолчанию, редактирование есть,
    // Ctrl+Space раскрывает маркеры точечно.
    Live,
    // Полное отображение: маркеры видны, редактирование, цвета/размеры.
    Source,
}

impl EditMode {
    // Можно ли редактировать текст в этом режиме.
    pub fn is_editable(&self) -> bool {
        matches!(self, EditMode::Source | EditMode::Live)
    }

    // Следующий режим по циклу: Preview → Live → Source → Preview.
    pub fn next(&self) -> Self {
        match self {
            EditMode::Preview => EditMode::Live,
            EditMode::Live => EditMode::Source,
            EditMode::Source => EditMode::Preview,
        }
    }

    // Название режима в нижнем регистре.
    pub fn name(&self) -> &'static str {
        match self {
            EditMode::Preview => "preview",
            EditMode::Live => "live",
            EditMode::Source => "source",
        }
    }
}
