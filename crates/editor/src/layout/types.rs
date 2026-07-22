//! Типы раскладки текста, независимые от GUI-фреймворка.
//!
//! Используется компоновщиком (`render/layout.rs`) для описания результата
//! разбора строки, а адаптер (`render/mod.rs`) превращает эти типы
//! в конкретные глифы/галеи выбранного фреймворка (egui / Iced).

use crate::theme::color::Rgba;

/// Один стилизованный фрагмент строки текста.
///
/// Не содержит никаких зависимостей от GUI — только данные.
/// Адаптер преобразует `TextRun` в формат конкретного фреймворка.
#[derive(Debug, Clone)]
pub struct TextRun {
    /// Текст фрагмента.
    pub text: String,
    /// Битовые флаги стиля из `editor::markup::segment::STYLE_*`.
    pub style_flags: u32,
    /// Цвет текста.
    pub color: Rgba,
    /// Размер шрифта в пунктах/единицах фреймворка.
    pub size: f32,
    /// Имя семейства шрифта (если переопределён).
    pub font_family: Option<String>,
}

impl TextRun {
    pub fn new(text: &str, style_flags: u32, color: Rgba, size: f32) -> Self {
        Self {
            text: text.to_string(),
            style_flags,
            color,
            size,
            font_family: None,
        }
    }

    pub fn with_font(mut self, family: &str) -> Self {
        self.font_family = Some(family.to_string());
        self
    }

    /// Является ли этот ран невидимым (комментарий или zero-width).
    pub fn is_invisible(&self) -> bool {
        self.text.is_empty()
    }
}

/// Результат раскладки одной строки.
#[derive(Debug, Clone)]
pub struct LineLayout {
    /// Стилизованные фрагменты строки.
    pub runs: Vec<TextRun>,
}

impl LineLayout {
    pub fn new(runs: Vec<TextRun>) -> Self {
        Self { runs }
    }

    /// Текст строки без маркеров (склеивает все runs).
    pub fn plain_text(&self) -> String {
        self.runs.iter().map(|r| r.text.as_str()).collect()
    }

    /// Пустая ли строка.
    pub fn is_empty(&self) -> bool {
        self.runs.is_empty() || self.runs.iter().all(|r| r.text.is_empty())
    }
}
