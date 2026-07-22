//! Типы темы редактора.
//!
//! [`EditorTheme`] — финальный «снимок» темы для быстрого чтения рендером.
//! Наполняется из [`ThemeSystem`](super::handle::ThemeSystem) или напрямую.

use super::color::Rgba;

/// Тема текста (шрифт, размер, цвет).
#[derive(Debug, Clone)]
pub struct TextTheme {
    pub size: f32,
    pub color: Rgba,
    /// Если `None` — используется системный шрифт по умолчанию.
    pub font_family: Option<String>,
}

/// Тема редактора — снимок всех настроек для быстрого доступа.
#[derive(Debug, Clone)]
pub struct EditorTheme {
    /// Название темы (для идентификации в конфигах).
    pub name: String,
    pub padding: f32,
    pub radius: f32,
    pub background: Rgba,
    pub text: TextTheme,
}

impl Default for EditorTheme {
    fn default() -> Self {
        Self {
            name: String::from("default"),
            padding: 10.0,
            radius: 16.0,
            background: Rgba::new(0.153, 0.18, 0.2).with_alpha(0.9),
            text: TextTheme {
                size: 14.0,
                color: Rgba::new(0.804, 0.839, 0.957),
                font_family: None,
            },
        }
    }
}
