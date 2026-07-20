//! Шейпинг текста через cosmic-text.
//!
//! Преобразует `Vec<Vec<TextRun>>` в сформованный `cosmic_text::Buffer`.
//! `Buffer` содержит все строки документа с вычисленными позициями глифов.
//!
//! От GUI не зависит. Для отрисовки используйте `painter.rs`.

use cosmic_text::{Align, Attrs, Buffer, Metrics, Shaping};

use crate::editor::layout::TextRun;

/// Сформованный документ — обёртка над cosmic-text `Buffer`.
///
/// Содержит все строки с глифами, позициями и метриками.
/// Передаётся в `paint()` для отрисовки.
#[derive(Debug)]
pub struct ShapedDocument {
    pub buffer: Buffer,
}

impl ShapedDocument {
    pub fn new(buffer: Buffer) -> Self {
        Self { buffer }
    }

    /// Общая высота документа в пикселях.
    pub fn total_height(&self) -> f32 {
        self.buffer
            .layout_runs()
            .last()
            .map(|run| run.line_y + run.line_height)
            .unwrap_or(0.0)
    }

    /// Количество строк.
    pub fn line_count(&self) -> usize {
        self.buffer.lines.len()
    }

    /// Высота i-й строки.
    pub fn line_height(&self, i: usize) -> f32 {
        self.buffer
            .layout_runs()
            .nth(i)
            .map(|run| run.line_height)
            .unwrap_or(0.0)
    }
}

/// Сшейпить все строки документа в один `Buffer`.
///
/// Каждый элемент `line_runs` — список `TextRun` для одной строки.
/// Склеивает строки через `\n` и отдаёт cosmic-text через `set_rich_text`,
/// который сам создаёт `BufferLine` и выставляет dirty-флаги.
pub fn shape_document(
    line_runs: &[Vec<TextRun>],
    font_system: &mut cosmic_text::FontSystem,
    base_size: f32,
    default_family: &str,
) -> ShapedDocument {
    let metrics = Metrics::new(base_size, base_size * 1.4);
    let mut buffer = Buffer::new_empty(metrics);
    buffer.set_size(None, None);

    // 1. Собираем полный текст (склейка строк через \n)
    let mut full_text = String::new();
    for (i, runs) in line_runs.iter().enumerate() {
        if i > 0 {
            full_text.push('\n');
        }
        if runs.is_empty() {
            full_text.push('\u{200B}'); // zero-width space для пустой строки
        } else {
            for run in runs {
                full_text.push_str(&run.text);
            }
        }
    }

    // 2. Собираем spans (ссылаемся на full_text — больше его не меняем).
    //    \n между строками тоже покрываем span'ом с дефолтными аттрибутами.
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
                let attrs = Attrs::new()
                    .metrics(Metrics::new(run.size, metrics.line_height))
                    .family(cosmic_text::Family::Name(family));
                let end = offset + run.text.len();
                let span_text = &full_text[offset..end];
                spans.push((span_text, attrs));
                offset = end;
            }
        }
        // \n между строками — тоже часть текста, покрываем дефолт-спаном
        if i + 1 < line_count {
            let newline_text = &full_text[offset..offset + 1];
            spans.push((newline_text, default_attrs.clone()));
            offset += 1;
        }
    }

    buffer.set_rich_text(spans, &default_attrs, Shaping::Advanced, Some(Align::Left));
    buffer.shape_until_scroll(font_system, true);
    ShapedDocument::new(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::render::font;
    use crate::editor::theme::color::Rgba;

    fn make_runs(text: &str, size: f32) -> Vec<TextRun> {
        vec![TextRun::new(text, 0, Rgba::new(1.0, 1.0, 1.0), size)]
    }

    #[test]
    fn shape_single_line() {
        font::init();
        let doc = font::with_font_system(|fs| {
            shape_document(&[make_runs("hello", 14.0)], fs, 14.0, "sans-serif")
        });
        assert!(doc.total_height() > 0.0);
        assert_eq!(doc.line_count(), 1);
    }

    #[test]
    fn shape_multiple_lines() {
        font::init();
        let doc = font::with_font_system(|fs| {
            shape_document(
                &[make_runs("line1", 14.0), make_runs("line2", 14.0)],
                fs,
                14.0,
                "sans-serif",
            )
        });
        assert_eq!(doc.line_count(), 2);
    }

    #[test]
    fn shape_empty_line() {
        font::init();
        let doc = font::with_font_system(|fs| {
            shape_document(&[vec![]], fs, 14.0, "sans-serif")
        });
        assert_eq!(doc.line_count(), 1);
        assert!(doc.total_height() > 0.0);
    }
}
