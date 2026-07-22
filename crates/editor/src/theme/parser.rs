//! Парсинг темы из Rhai Map.
//!
//! Читает `theme.rhai`, скомпилированный в `rhai::Map`, и преобразует в [`EditorTheme`].
//! Использует новый парсер цветов [`parse_color`](super::color::parse_color)
//! вместо старого примитивного `parse_rgba_string`.

use rhai::Map;

use super::color::{parse_color, Rgba};
use super::theme::{EditorTheme, TextTheme};

/// Парсит тему из Rhai-отображения.
pub fn parse_theme(rhai: Map) -> EditorTheme {
    let mut padding = 10.0f32;
    let mut radius = 16.0f32;
    let mut background = Rgba::new(0.153, 0.18, 0.2).with_alpha(0.9);
    let mut text_size = 14.0f32;
    let mut text_color = Rgba::new(0.804, 0.839, 0.957);
    let mut font_family = None;

    // Читаем блок "editor"
    if let Some(editor) = rhai.get("editor") {
        let m = editor.clone().cast::<Map>();
        if let Some(p) = m.get("padding") {
            padding = p.clone().cast::<f64>() as f32;
        }
        if let Some(r) = m.get("radius") {
            radius = r.clone().cast::<f64>() as f32;
        }
        if let Some(b) = m.get("background") {
            let s = b.clone().cast::<String>();
            match parse_color(&s) {
                Ok(c) => background = c,
                Err(e) => {
                    eprintln!("[Zol] Ошибка парсинга цвета «editor.background»: {}", e);
                }
            }
        }
    }

    // Читаем блок "text"
    if let Some(text) = rhai.get("text") {
        let m = text.clone().cast::<Map>();
        if let Some(s) = m.get("size") {
            text_size = s.clone().cast::<f64>() as f32;
        }
        if let Some(c) = m.get("color") {
            let s = c.clone().cast::<String>();
            match parse_color(&s) {
                Ok(c) => text_color = c,
                Err(e) => {
                    eprintln!("[Zol] Ошибка парсинга цвета «text.color»: {}", e);
                }
            }
        }
        if let Some(ff) = m.get("font_family") {
            font_family = Some(ff.clone().cast::<String>());
        }
    }

    EditorTheme {
        name: String::from("custom"),
        padding,
        radius,
        background,
        text: TextTheme {
            size: text_size,
            color: text_color,
            font_family,
        },
    }
}
