// ═══════════════════════════════════════════════════════════════════════
// ⚠  ВАЖНО: НИКОГОДА НЕ ДЕЛАЙ СЛЕДУЮЩЕГО:
// ⚠
// ⚠  1. НЕ вызывай line_runs.to_vec() — это клонирует ВСЕ TextRun'ы
// ⚠     со всеми строками. Вместо этого передавай line_runs по владению
// ⚠     (move) и сохраняй в ShapedDocument как есть.
// ⚠
// ⚠  2. НЕ создавай временную full_text String путём конкатенации всех
// ⚠     TextRun.text — это дублирует весь текст документа при каждом
// ⚠     reshape. (Текущий код пока это делает, TODO: исправить)
// ⚠
// ⚠  3. НЕ используй Shaping::Advanced если достаточно Basic —
// ⚠     Advanced шейпинг в 5-10x дороже по памяти.
// ⚠
// ⚠  Причина: line_runs.to_vec() при каждом редактировании создаёт
// ⚠  полную копию структуры документа, что на 100KB+ тексте →
// ⚠  десятки MB лишних аллокаций в секунду.
// ═══════════════════════════════════════════════════════════════════════

use cosmic_text::{
    Align, Attrs, Buffer, Color as CosmicColor, Metrics, Scroll, Shaping, Style, UnderlineStyle,
    Weight,
};

use super::shaped_doc::ShapedDocument;
use crate::layout::LineCompensation;
use crate::layout::TextRun;
use crate::segment::{STYLE_BOLD, STYLE_ITALIC, STYLE_STRIKETHROUGH, STYLE_UNDERLINE};

/// Сшейпить строки документа в Buffer.
pub fn shape_document(
    line_runs: Vec<Vec<TextRun>>,  // ⚠ Принимаем по владению, НЕ клонируем
    compensation: Vec<LineCompensation>,
    font_system: &mut cosmic_text::FontSystem,
    base_size: f32,
    default_family: &str,
    scroll_y: f32,
    viewport_height: Option<f32>,
) -> ShapedDocument {
    let metrics = Metrics::new(base_size, base_size * 1.4);
    let mut buffer = Buffer::new_empty(metrics);
    buffer.set_size(None, viewport_height);

    let mut full_text = String::new();
    for (i, runs) in line_runs.iter().enumerate() {
        if i > 0 {
            full_text.push('\n');
        }
        if runs.is_empty() {
            full_text.push('\u{200B}');
        } else {
            for run in runs {
                full_text.push_str(&run.text);
            }
        }
    }

    let default_attrs = Attrs::new()
        .metrics(metrics)
        .family(cosmic_text::Family::Name(default_family));

    let mut spans: Vec<(&str, Attrs<'_>)> = Vec::new();
    let mut offset: usize = 0;
    let line_count = line_runs.len();

    for (i, runs) in line_runs.iter().enumerate() {
        if runs.is_empty() {
            let ch = '\u{200B}';
            let ch_len = ch.len_utf8();
            let span_text = &full_text[offset..offset + ch_len];
            spans.push((span_text, default_attrs.clone()));
            offset += ch_len;
        } else {
            for run in runs {
                let family = run.font_family.as_deref().unwrap_or(default_family);
                let mut attrs = Attrs::new()
                    .metrics(Metrics::new(run.size, metrics.line_height))
                    .family(cosmic_text::Family::Name(family))
                    .color(rgba_to_cosmic(&run.color));

                if run.style_flags & STYLE_BOLD != 0 {
                    attrs = attrs.weight(Weight::BOLD);
                }
                if run.style_flags & STYLE_ITALIC != 0 {
                    attrs = attrs.style(Style::Italic);
                }
                if run.style_flags & STYLE_STRIKETHROUGH != 0 {
                    attrs = attrs.strikethrough();
                }
                if run.style_flags & STYLE_UNDERLINE != 0 {
                    attrs = attrs.underline(UnderlineStyle::Single);
                }

                let end = offset + run.text.len();
                let span_text = &full_text[offset..end];
                spans.push((span_text, attrs));
                offset = end;
            }
        }
        if i + 1 < line_count {
            let newline_text = &full_text[offset..offset + 1];
            spans.push((newline_text, default_attrs.clone()));
            offset += 1;
        }
    }

    buffer.set_rich_text(spans, &default_attrs, Shaping::Advanced, Some(Align::Left));
    buffer.set_scroll(Scroll::new(0, scroll_y, 0.0));
    buffer.shape_until_scroll(font_system, false);

    // ⚠ after_shape() НЕ вызываем здесь — это deadlock!
    // ⚠ Она вызывается в render::build() после выхода из with_font_system().
    // ⚠ (with_font_system держит Mutex, after_shape пытается взять его же)

    ShapedDocument {
        buffer,
        line_runs,        // ⚠ БЕЗ .to_vec() — передаём владение
        compensation,
    }
}

fn rgba_to_cosmic(c: &crate::theme::color::Rgba) -> CosmicColor {
    CosmicColor::rgba(
        (c.r * 255.0) as u8,
        (c.g * 255.0) as u8,
        (c.b * 255.0) as u8,
        (c.a * 255.0) as u8,
    )
}
