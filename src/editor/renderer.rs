use crate::editor::state::EditorState;
use gtk4::cairo::Context;

pub struct Renderer;

impl Renderer {
    pub fn draw(
        cr: &Context,
        width: i32,
        height: i32,
        state: &EditorState,
    ) {
        let theme = state.get_theme();

        // Фон
        cr.set_source_rgb(
            theme.background.r as f64,
            theme.background.g as f64,
            theme.background.b as f64,
        );

        cr.rectangle(
            0.0,
            0.0,
            width as f64,
            height as f64,
        );

        cr.fill().unwrap();

        // Рамка
        cr.set_source_rgb(
            theme.border_color.r as f64,
            theme.border_color.g as f64,
            theme.border_color.b as f64,
        );

        cr.set_line_width(theme.border_width as f64);

        cr.rectangle(
            0.5,
            0.5,
            width as f64 - 1.0,
            height as f64 - 1.0,
        );

        cr.stroke().unwrap();
    }
}