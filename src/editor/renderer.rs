use crate::editor::state::EditorState;
use crate::editor::layout::VisualLine;

use gtk4::cairo::Context;
use gtk4::pango::{FontDescription, Layout};

pub struct Renderer;

impl Renderer {
    pub fn draw(cr: &Context, width: i32, height: i32, state: &EditorState) {
        let theme = state.get_theme();

        // Фон
        cr.set_source_rgb(
            theme.background.r as f64,
            theme.background.g as f64,
            theme.background.b as f64,
        );
        cr.paint().unwrap();

        // Рамка
        cr.set_source_rgb(
            theme.border_color.r as f64,
            theme.border_color.g as f64,
            theme.border_color.b as f64,
        );
        cr.set_line_width(theme.border_width as f64);
        cr.rectangle(0.5, 0.5, width as f64 - 1.0, height as f64 - 1.0);
        cr.stroke().unwrap();

        // ---------- ТЕКСТ ----------

        let pango_ctx = pangocairo::functions::create_context(cr);

        let lines = vec![
            VisualLine {
                text: "Flint Notes".into(),
                font_size: theme.text.size,
                font_family: theme.text.font_family.clone(),
                x: theme.padding,
                y: theme.padding,
            },
        ];

        cr.set_source_rgb(1.0, 1.0, 1.0);

        for line in lines {
            let layout = Layout::new(&pango_ctx);

            layout.set_text(&line.text);

            let mut font = FontDescription::new();
            font.set_family(&line.font_family);
            font.set_size((line.font_size as i32) * gtk4::pango::SCALE);

            layout.set_font_description(Some(&font));

            cr.move_to(line.x as f64, line.y as f64);

            pangocairo::functions::show_layout(cr, &layout);
        }
    }
}
