//! Парсинг темы из Rhai Map.
//!
//! Читает `theme.rhai`, скомпилированный в `rhai::Map`, и преобразует в [`EditorTheme`].
//! Использует новый парсер цветов [`parse_color`]
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

#[cfg(test)]
mod tests {
    use super::*;
    use rhai::Map;

    #[test]
    fn parse_theme_default() {
        let map = Map::new();
        let theme = parse_theme(map);
        assert_eq!(theme.name, "custom");
        assert!((theme.padding - 10.0).abs() < 0.001);
        assert!((theme.text.size - 14.0).abs() < 0.001);
    }

    #[test]
    fn parse_theme_with_editor() {
        let mut map = Map::new();
        let mut editor = Map::new();
        editor.insert("padding".into(), rhai::Dynamic::from(20.0_f64));
        editor.insert("radius".into(), rhai::Dynamic::from(8.0_f64));
        map.insert("editor".into(), rhai::Dynamic::from(editor));
        let theme = parse_theme(map);
        assert!((theme.padding - 20.0).abs() < 0.001);
        assert!((theme.radius - 8.0).abs() < 0.001);
    }

    #[test]
    fn parse_theme_with_text() {
        let mut map = Map::new();
        let mut text = Map::new();
        text.insert("size".into(), rhai::Dynamic::from(16.0_f64));
        text.insert("color".into(), rhai::Dynamic::from("#ff0000".to_string()));
        map.insert("text".into(), rhai::Dynamic::from(text));
        let theme = parse_theme(map);
        assert!((theme.text.size - 16.0).abs() < 0.001);
        assert!((theme.text.color.r - 1.0).abs() < 0.01);
    }

    #[test]
    fn parse_theme_invalid_color() {
        let mut map = Map::new();
        let mut editor = Map::new();
        editor.insert("background".into(), rhai::Dynamic::from("not-a-color".to_string()));
        map.insert("editor".into(), rhai::Dynamic::from(editor));
        let theme = parse_theme(map);
        assert!((theme.background.a - 0.9).abs() < 0.001);
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
