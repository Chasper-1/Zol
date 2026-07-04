use crate::editor::markup::{MarkupCache, SegmentStyle};
use eframe::egui::text::{CCursorRange, CharIndex, LayoutJob};
use eframe::egui::{
    Align, Color32, Context, FontFamily, FontId, Galley, Id, Stroke, TextEdit, TextFormat,
};

fn append_compensated(
    job: &mut LayoutJob,
    left: usize,
    content: &str,
    right: usize,
    format: TextFormat,
) {
    if left > 0 {
        job.append(&"\u{200B}".repeat(left), 0.0, format.clone());
    }

    job.append(content, 0.0, format.clone());

    if right > 0 {
        job.append(&"\u{200B}".repeat(right), 0.0, format);
    }
}

pub fn render_line(
    job: &mut LayoutJob,
    line: &str,
    cache: &MarkupCache,
    base_size: f32,
    heading_size: f32,
    font_family: FontFamily,
    show_markup: bool,
) {
    let default_format = TextFormat::simple(
        FontId::new(base_size, font_family.clone()),
        Color32::from_rgb(180, 180, 180),
    );

    // SOURCE РЕЖИМ → показываем сырой текст
    if show_markup {
        job.append(line, 0.0, default_format);
        return;
    }

    // Заголовок
    if line.starts_with("# ") {
        let content = &line[2..];

        let format = TextFormat::simple(FontId::new(heading_size, font_family), Color32::WHITE);

        job.append(content, 0.0, format);
        return;
    }

    // Парсим разметку
    for seg in &cache.segments {
        let format = match seg.style {
            SegmentStyle::Plain => default_format.clone(),

            SegmentStyle::Bold => TextFormat::simple(
                FontId::new(base_size, font_family.clone()),
                Color32::from_rgb(255, 100, 100),
            ),

            SegmentStyle::Italic => {
                let mut f = TextFormat::simple(
                    FontId::new(base_size, font_family.clone()),
                    Color32::from_rgb(100, 200, 255),
                );
                f.italics = true;
                f
            }

            SegmentStyle::Strikethrough => {
                let mut f = TextFormat::simple(
                    FontId::new(base_size, font_family.clone()),
                    Color32::from_rgb(200, 150, 150),
                );
                f.strikethrough = Stroke::new(1.0, Color32::from_rgb(200, 150, 150));
                f
            }

            SegmentStyle::Superscript => {
                let mut f = TextFormat::simple(
                    FontId::new(base_size * 0.7, font_family.clone()),
                    Color32::from_rgb(150, 255, 150),
                );
                f.valign = Align::TOP;
                f
            }

            SegmentStyle::Subscript => {
                let mut f = TextFormat::simple(
                    FontId::new(base_size * 0.7, font_family.clone()),
                    Color32::from_rgb(255, 200, 100),
                );
                f.valign = Align::BOTTOM;
                f
            }

            SegmentStyle::Code => TextFormat::simple(
                FontId::new(base_size, FontFamily::Monospace),
                Color32::from_rgb(200, 200, 200),
            ),
        };

        append_compensated(
            job,
            seg.left_marker_len,
            &seg.text,
            seg.right_marker_len,
            format,
        );
    }
}

/*pub fn adjust_cursor_for_markup(
    ctx: &Context,
    id: Id,
    line_text: &str,
    right: bool,
    galley: &Galley,
) {
    if let Some(mut state) = TextEdit::load_state(ctx, id) {
        let offset = if line_text.starts_with("# ") {
            2
        } else {
            let cache = parse_line(line_text);

            cache
                .segments
                .iter()
                .find(|s| !matches!(s.style, SegmentStyle::Plain))
                .map(|s| s.left_marker_len)
                .unwrap_or(0)
        };

        if offset == 0 {
            return;
        }

        if let Some(range) = state.cursor.char_range() {
            let mut c = range.primary;

            let current = c.index.0;
            let new_index = if right {
                current + offset
            } else {
                current.saturating_sub(offset)
            };

            c.index = CharIndex(new_index.min(galley.text().chars().count()));

            state.cursor.set_char_range(Some(CCursorRange::one(c)));
            state.store(ctx, id);
        }
    }
}*/
