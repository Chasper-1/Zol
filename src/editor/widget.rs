use crate::editor::renderer::draw_text;
use crate::editor::state::EditorState;
use crate::editor::utils::{hex_to_rgba, rounded_rect};
use gtk4::DrawingArea;
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
            let mut st = state_rc.borrow_mut();
            
            // Разыменовываем RefMut один раз, чтобы получить &mut EditorState
            let st_inner = &mut *st;
        
            let scale = widget.scale_factor() as f64;
            let theme = st_inner.theme.clone();
        
            let padding = theme.padding as f64;
            let w = width as f64 - padding * 2.0;
            let h = height as f64 - padding * 2.0;
            if w <= 0.0 || h <= 0.0 {
                return;
            }
        
            // Рисуем фон/рамку
            if let Some((r, g, b, a)) = hex_to_rgba(&theme.background) {
                cr.set_source_rgba(r, g, b, a);
                rounded_rect(cr, padding, padding, w, h, theme.radius as f64);
                cr.fill().unwrap();
            }
        
            // Логика текста (теперь через st_inner)
            st_inner.buffer.set_size(
                Some((w - theme.text.margin_x as f64 * 2.0) as f32),
                Some((h - theme.text.margin_y as f64 * 2.0) as f32),
            );
            st_inner.buffer.shape_until_scroll(&mut st_inner.font_system, false);
        
            let runs: Vec<_> = st_inner
                .buffer
                .layout_runs()
                .map(|r| (r.line_y as f64, r.glyphs.to_vec()))
                .collect();
            
            let color = hex_to_rgba(&theme.text.color).unwrap_or((1.0, 1.0, 1.0, 1.0));
        
            // Вызываем рендерер, передавая st_inner
            draw_text(
                cr,
                &runs,
                st_inner,
                padding,
                theme.text.margin_x as f64,
                theme.text.margin_y as f64,
                scale,
                color,
            );
        });
        area
    }
}
