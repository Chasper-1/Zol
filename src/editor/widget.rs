use crate::editor::state::EditorState;
use gtk4::DrawingArea;
use gtk4::cairo::{Antialias, Format, ImageSurface};
use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

pub struct EditorWidget;

impl EditorWidget {
    pub fn new(state: EditorState) -> DrawingArea {
        let area = DrawingArea::builder()
            .hexpand(true)
            .vexpand(true)
            .focusable(true)
            .build();

        let state_rc = Rc::new(RefCell::new(state));

        area.set_draw_func(move |_, cr, width, height| {
            // 1. НАСТРОЙКА АНТИАЛИАСИНГА
            cr.set_antialias(Antialias::Best);

            let mut st_borrow = state_rc.borrow_mut();

            // Разделяем заимствования
            let st = &mut *st_borrow;
            let theme = &st.theme;
            let buffer = &mut st.buffer;
            let font_system = &mut st.font_system;
            let swash_cache = &mut st.swash_cache;

            let padding = theme.padding as f64;

            let x = padding;
            let y = padding;
            let w = width as f64 - (padding * 2.0);
            let h = height as f64 - (padding * 2.0);

            if w <= 0.0 || h <= 0.0 {
                return;
            }

            // 2. ОТРИСОВКА КАРТОЧКИ РЕДАКТОРА (ФОН)
            let r = theme.radius as f64;
            rounded_rect(cr, x, y, w, h, r);

            if let Some((r_col, g_col, b_col)) = hex_to_rgb(&theme.background) {
                cr.set_source_rgb(r_col, g_col, b_col);
            } else {
                cr.set_source_rgb(0.12, 0.12, 0.18);
            }
            cr.fill_preserve().unwrap();

            // Рамка карточки
            if let Some((br, bg, bb)) = hex_to_rgb(&theme.border_color) {
                cr.set_source_rgb(br, bg, bb);
            } else {
                cr.set_source_rgb(0.24, 0.24, 0.28);
            }
            cr.set_line_width(theme.border_width as f64);
            cr.stroke().unwrap();

            // 3. ПОДГОТОВКА ТЕКСТА К РЕНДЕРУ (ФОРМИРОВАНИЕ ШЕЙПОВ)
            let margin_x = theme.text.margin_x as f64;
            let margin_y = theme.text.margin_y as f64;

            buffer.set_size(
                Some(w as f32 - (margin_x as f32 * 2.0)),
                Some(h as f32 - (margin_y as f32 * 2.0)),
            );
            buffer.shape_until_scroll(font_system, false);

            // 4. ЗАРЯЖАЕМ ЦВЕТ ТЕКСТА ЗАРАНЕЕ В CAIRO PIPELINE
            let text_color = hex_to_rgba(&theme.text.color).unwrap_or((0.8, 0.84, 0.96, 1.0));
            cr.set_source_rgba(text_color.0, text_color.1, text_color.2, text_color.3);

            // Итерируемся по строкам и символам
            for run in buffer.layout_runs() {
                for glyph in run.glyphs {
                    let physical_glyph = glyph.physical(
                        (
                            x as f32 + margin_x as f32,
                            y as f32 + margin_y as f32 + run.line_y,
                        ),
                        1.0,
                    );

                    if let Some(image) =
                        swash_cache.get_image(font_system, physical_glyph.cache_key)
                    {
                        if image.placement.width == 0 || image.placement.height == 0 {
                            continue;
                        }

                        // Создаем A8 маску под размер глифа
                        let mut surface = ImageSurface::create(
                            Format::A8,
                            image.placement.width as i32,
                            image.placement.height as i32,
                        )
                        .unwrap();

                        if let Ok(mut data) = surface.data() {
                            let len = image.data.len().min(data.len());
                            data[..len].copy_from_slice(&image.data[..len]);
                        }

                        let x_pos = physical_glyph.x as f64 + image.placement.left as f64;
                        let y_pos = physical_glyph.y as f64 - image.placement.top as f64;

                        // НАКЛАДЫВАЕМ МАСКУ НА УЖЕ ВЫБРАННЫЙ ЦВЕТ
                        cr.mask_surface(&surface, x_pos as f64, y_pos as f64)
                            .unwrap();
                    }
                }
            }
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

fn hex_to_rgba(hex: &str) -> Option<(f64, f64, f64, f64)> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 && hex.len() != 8 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f64 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f64 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f64 / 255.0;
    let a = if hex.len() == 8 {
        u8::from_str_radix(&hex[6..8], 16).ok()? as f64 / 255.0
    } else {
        1.0
    };
    Some((r, g, b, a))
}
