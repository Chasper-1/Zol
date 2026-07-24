use super::shared;
use crate::layout::types::TextRun;
use crate::markup::segment::{
    STYLE_BOLD, STYLE_CODE, STYLE_COMMENT, STYLE_DELETION, STYLE_DISPLAY_FORMULA, STYLE_FORMULA,
    STYLE_INSERTION, STYLE_ITALIC, STYLE_SUBSCRIPT, STYLE_SUPERSCRIPT,
};
use crate::theme::color::Rgba;

/// Создать `TextRun` по битовым флагам стиля.
pub fn text_run_for_style(
    text: &str,
    style: u32,
    base_size: f32,
) -> TextRun {
    let mut color = shared::TEXT_DEFAULT;
    let mut size = base_size;
    let mut family: Option<&str> = None;

    if style & STYLE_BOLD != 0 {
        color = Rgba::new(1.0, 0.392, 0.392);
    }
    if style & STYLE_ITALIC != 0 {
        color = Rgba::new(0.392, 0.784, 1.0);
    }
    if style & STYLE_CODE != 0 {
        color = Rgba::new(0.784, 0.784, 0.784);
        family = Some("monospace");
    }
    if style & STYLE_INSERTION != 0 {
        color = Rgba::new(0.392, 1.0, 0.392);
    }
    if style & STYLE_DELETION != 0 {
        color = Rgba::new(1.0, 0.314, 0.314);
    }
    if style & STYLE_COMMENT != 0 {
        color = Rgba::new(0.549, 0.549, 0.549);
    }
    if style & STYLE_SUPERSCRIPT != 0 {
        size = base_size * 0.7;
        color = Rgba::new(0.588, 1.0, 0.588);
    }
    if style & STYLE_SUBSCRIPT != 0 {
        size = base_size * 0.7;
        color = Rgba::new(1.0, 0.784, 0.392);
    }
    if style & STYLE_FORMULA != 0 {
        color = Rgba::new(0.314, 0.863, 0.471);
        family = Some("monospace");
    }
    if style & STYLE_DISPLAY_FORMULA != 0 {
        size = base_size * 1.3;
        color = Rgba::new(0.314, 0.863, 0.471);
        family = Some("monospace");
    }

    let mut run = TextRun::new(text, style, color, size);
    if let Some(f) = family {
        run = run.with_font(f);
    }
    run
}
