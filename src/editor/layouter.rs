use crate::editor::markup::{LineMarkup, parse_line};
use eframe::egui::text::{CCursorRange, CharIndex, LayoutJob};
use eframe::egui::{Color32, Context, FontFamily, FontId, Galley, Id, TextEdit, TextFormat, Stroke};

fn append_compensated(
    job: &mut LayoutJob,
    markup_left: &str,
    content: &str,
    markup_right: Option<&str>,
    format: TextFormat,
) {
    if !markup_left.is_empty() {
        job.append(
            &"\u{200B}".repeat(markup_left.chars().count()),
            0.0,
            format.clone(),
        );
    }

    job.append(content, 0.0, format.clone());

    if let Some(right) = markup_right {
        job.append(&"\u{200B}".repeat(right.chars().count()), 0.0, format);
    }
}

pub fn render_line(
    job: &mut LayoutJob,
    line: &str,
    is_active: bool,
    base_size: f32,
    heading_size: f32,
    font_family: FontFamily,
    show_markup: bool,
) {
    if show_markup || is_active {
        let is_heading = line.starts_with("# ");
        let format = TextFormat::simple(
            FontId::new(
                if is_heading { heading_size } else { base_size },
                font_family,
            ),
            Color32::WHITE,
        );
        job.append(line, 0.0, format);
    } else {
        match parse_line(line) {
            LineMarkup::Heading { content, marker } => {
                let format = TextFormat::simple(
                    FontId::new(heading_size, FontFamily::Proportional),
                    Color32::WHITE,
                );
                append_compensated(job, &marker, &content, None, format);
            }
            LineMarkup::Bold { content, marker } => {
                let format = TextFormat::simple(
                    FontId::new(base_size, font_family),
                    Color32::from_rgb(255, 100, 100),
                );
                append_compensated(job, &marker, &content, Some(&marker), format);
            }
            LineMarkup::Italic { content, marker } => {
                let mut format = TextFormat::simple(
                    FontId::new(base_size, font_family),
                    Color32::from_rgb(100, 200, 255),
                );
                format.italics = true;
                append_compensated(job, &marker, &content, Some(&marker), format);
            }
            LineMarkup::Strikethrough { content, marker } => {
                let mut format = TextFormat::simple(
                    FontId::new(base_size, font_family),
                    Color32::from_rgb(200, 150, 150),
                );
                format.strikethrough = Stroke::new(1.0, Color32::from_rgb(200, 150, 150));
                append_compensated(job, &marker, &content, Some(&marker), format);
            }
            LineMarkup::Superscript { content, marker } => {
                let format = TextFormat::simple(
                    FontId::new(base_size * 0.7, font_family),
                    Color32::from_rgb(150, 255, 150),
                );
                append_compensated(job, &marker, &content, Some(&marker), format);
            }
            LineMarkup::Subscript { content, marker } => {
                let format = TextFormat::simple(
                    FontId::new(base_size * 0.7, font_family),
                    Color32::from_rgb(255, 200, 100),
                );
                append_compensated(job, &marker, &content, Some(&marker), format);
            }
            LineMarkup::Code { content, marker } => {
                let format = TextFormat::simple(
                    FontId::new(base_size, FontFamily::Monospace),
                    Color32::from_rgb(200, 200, 200),
                );
                append_compensated(job, &marker, &content, Some(&marker), format);
            }
            LineMarkup::Plain(text) => {
                let format = TextFormat::simple(
                    FontId::new(base_size, font_family),
                    Color32::from_rgb(180, 180, 180),
                );
                job.append(&text, 0.0, format);
            }
        }
    }
}

pub fn adjust_cursor_for_markup(
    ctx: &Context,
    id: Id,
    line_text: &str,
    right: bool,
    galley: &Galley,
) {
    if let Some(mut state) = TextEdit::load_state(ctx, id) {
        if let LineMarkup::Heading { marker, .. } | LineMarkup::Bold { marker, .. } =
            parse_line(line_text)
        {
            let offset = marker.chars().count();

            if let Some(range) = state.cursor.char_range() {
                let mut c = range.primary;
                let current_index = c.index.0; // CharIndex -> usize
                let new_index = if right {
                    current_index + offset
                } else {
                    current_index.saturating_sub(offset)
                };
                let clamped = new_index.clamp(0, galley.text().chars().count());
                c.index = CharIndex(clamped);
                state.cursor.set_char_range(Some(CCursorRange::one(c)));
                state.store(ctx, id);
            }
        }
    }
}
