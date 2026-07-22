use editor::state::EditMode;

/// Установить режим редактирования.
pub fn mode_set(mode: &mut EditMode, new_mode: EditMode) {
    *mode = new_mode;
}

/// Текущий режим редактирования.
pub fn mode_get(mode: &EditMode) -> EditMode {
    *mode
}

/// Название режима в нижнем регистре.
pub fn mode_name(mode: EditMode) -> &'static str {
    mode.name()
}

/// Переключиться на следующий режим по циклу:
/// Preview → LivePreview → Source → Preview.
pub fn mode_cycle(mode: &mut EditMode) {
    *mode = mode.next();
}

/// Можно ли редактировать текст в этом режиме.
pub fn mode_is_editable(mode: EditMode) -> bool {
    mode.is_editable()
}
