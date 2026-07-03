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

        area.set_draw_func(move |widget, cr, width, height| {
            cr.set_antialias(Antialias::Best);
            let scale = widget.scale_factor() as f64;
            cr.scale(1.0 / scale, 1.0 / scale);

            let (theme, mx, my, scale) = {
                let st = state_rc.borrow();
                (
                    st.theme.clone(),
                    st.theme.text.margin_x as f64,
                    st.theme.text.margin_y as f64,
                    widget.scale_factor() as f64,
                )
            };

            let padding = theme.padding as f64;
            let w = width as f64 - padding * 2.0;
            let h = height as f64 - padding * 2.0;

            if w <= 0.0 || h <= 0.0 {
                return;
            }

            // Отрисовка фона
            if let Some((r, g, b, a)) = hex_to_rgba(&theme.background) {
                cr.set_source_rgba(r, g, b, a);
                rounded_rect(cr, padding, padding, w, h, theme.radius as f64);
                cr.fill().unwrap();
            }

            // Отрисовка рамки
            if let Some((r, g, b, a)) = hex_to_rgba(&theme.border_color) {
                cr.set_source_rgba(r, g, b, a);
                cr.set_line_width(theme.border_width as f64);
                rounded_rect(cr, padding, padding, w, h, theme.radius as f64);
                cr.stroke().unwrap();
            }

            // Обновление размера буфера и shape_until_scroll
            {
                let mut st = state_rc.borrow_mut();
                st.buffer
                    .set_size(Some((w - mx * 2.0) as f32), Some((h - my * 2.0) as f32));
                let EditorState {
                    buffer,
                    font_system,
                    ..
                } = &mut *st;
                buffer.shape_until_scroll(font_system, false);
            }

            // Сбор layout_runs
            let layout_runs: Vec<(f64, Vec<cosmic_text::LayoutGlyph>)> = {
                let st = state_rc.borrow();
                st.buffer
                    .layout_runs()
                    .map(|run| (run.line_y as f64, run.glyphs.to_vec()))
                    .collect()
            };

            let (tr, tg, tb, ta) = hex_to_rgba(&theme.text.color).unwrap_or((1.0, 1.0, 1.0, 1.0));

            for (line_y, glyphs) in layout_runs {
                for glyph in glyphs {
                    let physical = glyph.physical(
                        ((padding + mx) as f32 * scale as f32, (padding + my + line_y) as f32 * scale as f32),
                        scale as f32
                    );

                    {
                        let mut st = state_rc.borrow_mut();
                        let EditorState {
                            swash_cache,
                            font_system,
                            ..
                        } = &mut *st;
                        if let Some(image) = swash_cache.get_image(font_system, physical.cache_key)
                        {
                            if image.placement.width == 0
                                || image.placement.height == 0
                                || image.content != cosmic_text::SwashContent::Mask
                            {
                                continue;
                            }

                            let data = &image.data;
                            let data_len = data.len();
                            let width_orig = image.placement.width as usize;
                            let height_orig = image.placement.height as usize;
                            let expected_len = width_orig * height_orig;

                            // Определяем реальные размеры поверхности на основе длины данных
                            let (surface_width, surface_height) = if data_len == expected_len {
                                (width_orig, height_orig)
                            } else if data_len % height_orig == 0 {
                                (data_len / height_orig, height_orig)
                            } else if data_len % width_orig == 0 {
                                (width_orig, data_len / width_orig)
                            } else {
                                // Некорректные данные — используем оригинальные размеры,
                                // но скопируем только доступную часть
                                (width_orig, height_orig)
                            };

                            let mut surface = ImageSurface::create(
                                Format::A8,
                                surface_width as i32,
                                surface_height as i32,
                            )
                            .unwrap();

                            if let Ok(mut surf_data) = surface.data() {
                                let surf_len = surf_data.len();
                                if surf_len == data_len {
                                    surf_data.copy_from_slice(data);
                                } else {
                                    let copy_len = surf_len.min(data_len);
                                    // Побайтовое копирование, чтобы избежать паники при несовпадении длин
                                    for i in 0..copy_len {
                                        surf_data[i] = data[i];
                                    }
                                    // Остальные байты уже нулевые (ImageSurface::create обнуляет)
                                }
                            }

                            cr.save().unwrap();
                            cr.set_source_rgba(tr, tg, tb, ta);
                            cr.mask_surface(
                                &surface,
                                physical.x as f64 + image.placement.left as f64,
                                physical.y as f64 + image.placement.top as f64
                            ).unwrap();
                            cr.restore().unwrap();
                        }
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

fn hex_to_rgba(hex: &str) -> Option<(f64, f64, f64, f64)> {
    let h = hex.trim_start_matches('#');
    if h.len() == 6 {
        Some((
            u8::from_str_radix(&h[0..2], 16).ok()? as f64 / 255.0,
            u8::from_str_radix(&h[2..4], 16).ok()? as f64 / 255.0,
            u8::from_str_radix(&h[4..6], 16).ok()? as f64 / 255.0,
            1.0,
        ))
    } else if h.len() == 8 {
        Some((
            u8::from_str_radix(&h[0..2], 16).ok()? as f64 / 255.0,
            u8::from_str_radix(&h[2..4], 16).ok()? as f64 / 255.0,
            u8::from_str_radix(&h[4..6], 16).ok()? as f64 / 255.0,
            u8::from_str_radix(&h[6..8], 16).ok()? as f64 / 255.0,
        ))
    } else {
        None
    }
}
