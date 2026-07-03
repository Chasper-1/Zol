use crate::gui::theme_parser::EditorTheme;
use gtk4::DrawingArea;
use gtk4::prelude::*; // Импортируем тему из gui или твоего модуля theme

pub struct EditorWidget;

impl EditorWidget {
    // Теперь функция принимает тему, полученную из Rhai
    pub fn new(theme: EditorTheme) -> DrawingArea {
        let area = DrawingArea::builder()
            .hexpand(true)
            .vexpand(true)
            .focusable(true)
            .build();

        // Передаем тему внутрь замыкания отрисовки
        area.set_draw_func(move |_, cr, width, height| {
            let x = 0.0;
            let y = 0.0;
            let w = width as f64;
            let h = height as f64;

            // Используем радиус из Rhai скрипта
            let r = theme.radius as f64;

            rounded_rect(cr, x, y, w, h, r);

            // Парсим фоновый цвет из Rhai (например, "#131316")
            if let Some((r_col, g_col, b_col)) = hex_to_rgb(&theme.background) {
                cr.set_source_rgb(r_col, g_col, b_col);
            } else {
                cr.set_source_rgb(0.13, 0.13, 0.16); // Дефолтный фолбек
            }
            cr.fill_preserve().unwrap();

            // Цвет рамки сделаем чуть светлее фонового для объема
            cr.set_source_rgb(0.24, 0.24, 0.28);
            cr.set_line_width(1.0);
            cr.stroke().unwrap();
        });

        area
    }
}

fn rounded_rect(cr: &gtk4::cairo::Context, x: f64, y: f64, w: f64, h: f64, r: f64) {
    use std::f64::consts::{FRAC_PI_2, PI};
    cr.new_path();
    cr.arc(x + w - r, y + r, r, -FRAC_PI_2, 0.0);
    cr.arc(x + w - r, y + h - r, r, 0.0, FRAC_PI_2);
    cr.arc(x + r, y + h - r, r, FRAC_PI_2, PI);
    cr.arc(x + r, y + r, r, PI, PI * 1.5);
    cr.close_path();
}

// Утилита для конвертации HEX строки в RGB для Cairo
fn hex_to_rgb(hex: &str) -> Option<(f64, f64, f64)> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f64 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f64 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f64 / 255.0;
    Some((r, g, b))
}
