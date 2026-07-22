//! Шейпинг текста через cosmic-text.
//!
//! Преобразует `Vec<Vec<TextRun>>` в сформованный `cosmic_text::Buffer`.
//! `Buffer` содержит все строки документа с вычисленными позициями глифов.
//!
//! От GUI не зависит. Для отрисовки используйте `painter.rs`.

use cosmic_text::{Align, Attrs, Buffer, Color as CosmicColor, Metrics, Scroll, Shaping, Style, UnderlineStyle, Weight};

use crate::editor::layout::TextRun;
use crate::editor::markup::segment::{STYLE_BOLD, STYLE_ITALIC, STYLE_STRIKETHROUGH, STYLE_UNDERLINE};
use crate::editor::theme::color::Rgba;

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

/// Сшейпить строки документа в `Buffer`.
///
/// `scroll_y` — вертикальный сдвиг прокрутки (в пикселях). cosmic-text
/// сформирует только строки, попадающие в видимое окно
/// `[scroll_y, scroll_y + viewport_height]`.
///
/// `viewport_height` — если Some, ограничивает шейпинг видимой областью
/// (cosmic-text не будет формировать строки за её пределами).
///
/// Каждый элемент `line_runs` — список `TextRun` для одной строки.
/// Склеивает строки через `\n` и отдаёт cosmic-text через `set_rich_text`.
pub fn shape_document(
    line_runs: &[Vec<TextRun>],
    font_system: &mut cosmic_text::FontSystem,
    base_size: f32,
    default_family: &str,
    scroll_y: f32,
    viewport_height: Option<f32>,
) -> ShapedDocument {
    let metrics = Metrics::new(base_size, base_size * 1.4);
    let mut buffer = Buffer::new_empty(metrics);
    buffer.set_size(None, viewport_height);

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
        // \n между строками — тоже часть текста, покрываем дефолт-спаном
        if i + 1 < line_count {
            let newline_text = &full_text[offset..offset + 1];
            spans.push((newline_text, default_attrs.clone()));
            offset += 1;
        }
    }

    buffer.set_rich_text(spans, &default_attrs, Shaping::Advanced, Some(Align::Left));
    buffer.set_scroll(Scroll::new(0, scroll_y, 0.0));
    buffer.shape_until_scroll(font_system, false);
    ShapedDocument::new(buffer)
}

/// Перевести [`Rgba`] редактора в цвет cosmic-text.
fn rgba_to_cosmic(c: &Rgba) -> CosmicColor {
    CosmicColor::rgba(
        (c.r * 255.0) as u8,
        (c.g * 255.0) as u8,
        (c.b * 255.0) as u8,
        (c.a * 255.0) as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::font;
    use crate::editor::theme::color::Rgba;

    fn make_runs(text: &str, size: f32) -> Vec<TextRun> {
        vec![TextRun::new(text, 0, Rgba::new(1.0, 1.0, 1.0), size)]
    }

    #[test]
    fn shape_single_line() {
        font::init();
        let doc = font::with_font_system(|fs| {
            shape_document(&[make_runs("hello", 14.0)], fs, 14.0, "sans-serif", 0.0, None)
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
                0.0,
                None,
            )
        });
        assert_eq!(doc.line_count(), 2);
    }

    #[test]
    fn shape_empty_line() {
        font::init();
        let doc = font::with_font_system(|fs| {
            shape_document(&[vec![]], fs, 14.0, "sans-serif", 0.0, None)
        });
        assert_eq!(doc.line_count(), 1);
        assert!(doc.total_height() > 0.0);
    }

    /// Диагностика: проверить glyph.start для строки с кириллицей и маркерами.
    /// Выявить, может ли курсор оказаться посередине glyph-кластера.
    #[test]
    fn glyph_starts_for_mixed_text() {
        font::init();
        let doc = font::with_font_system(|fs| {
            shape_document(&[make_runs("**текст**", 14.0)], fs, 14.0, "sans-serif", 0.0, None)
        });
        let run = doc.buffer.layout_runs().next().expect("должна быть одна строка");

        // Собираем все глифы с их start и x
        let glyphs: Vec<_> = run.glyphs.iter().map(|g| (g.start, g.x, g.w)).collect();
        // Для "**текст**" (14 байт: **текст**):
        //   '*' байт 0,1; 'т' байты 2-3; 'е' 4-5; 'к' 6-7; 'с' 8-9; 'т' 10-11; '*' 12,13
        // Каждый кластер = 1 глиф с уникальным start (байтовый offset кластера)
        // Кластеров должно быть 9: *, *, т, е, к, с, т, *, *
        assert_eq!(glyphs.len(), 9, "9 glyph-кластеров: glyphs={:?}", glyphs);

        // Проверим, что каждый glyph.start — байтовый offset начала кластера
        assert_eq!(glyphs[0].0, 0, "* (первый)");
        assert_eq!(glyphs[1].0, 1, "* (второй)");
        assert_eq!(glyphs[2].0, 2, "т (байт 2)");
        assert_eq!(glyphs[3].0, 4, "е (байт 4)");
        assert_eq!(glyphs[4].0, 6, "к (байт 6)");
        assert_eq!(glyphs[5].0, 8, "с (байт 8)");
        assert_eq!(glyphs[6].0, 10, "т (байт 10)");
        assert_eq!(glyphs[7].0, 12, "* (третий)");
        assert_eq!(glyphs[8].0, 13, "* (четвёртый)");

        // Проверим hit-testing: клик на x-координате каждого glyph
        // cosmic-text hit() возвращает ближайшую ГРАНИЦУ КЛАСТЕРА (cursor position),
        // а не начало glyph. Это может быть start или start+cluster_len.
        for (i, &(start, gx, gw)) in glyphs.iter().enumerate() {
            // Клик в начало glyph
            let hit = doc.buffer.hit(gx + 1.0, run.line_top + 1.0);
            assert!(hit.is_some(), "hit для glyph {} должен быть Some", i);
            let hit = hit.unwrap();
            // hit.index должен быть char boundary (не внутри multi-byte)
            let text = "**текст**";
            assert!(
                text.is_char_boundary(hit.index),
                "glyph {}: hit.index={} не является char boundary в {:?}",
                i, hit.index, text
            );
            // И должен быть корректным: start (перед) или start+cluster_len (после)
            let cluster_byte_len = if i + 1 < glyphs.len() {
                glyphs[i + 1].0 - start
            } else {
                text.len() - start
            };
            assert!(
                hit.index == start || hit.index == start + cluster_byte_len,
                "glyph {}: hit.index={} не в {{start={}, start+len={}}}",
                i, hit.index, start, start + cluster_byte_len,
            );
        }

        // Клик ПЕРЕД первым glyph'ом (слева от строки)
        let hit_before = doc.buffer.hit(-1.0, run.line_top + 1.0);
        assert!(hit_before.is_some(), "hit слева от строки должен быть Some");
        assert_eq!(hit_before.unwrap().index, 0, "hit слева от строки должен давать index=0");

        // Клик ПОСЛЕ последнего glyph'а (справа от строки)
        let last = glyphs.last().unwrap();
        let after_x = last.1 + last.2 + 5.0;
        let hit_after = doc.buffer.hit(after_x, run.line_top + 1.0);
        assert!(hit_after.is_some(), "hit справа от строки должен быть Some");
        assert_eq!(
            hit_after.unwrap().index, last.0 + 1,
            "hit справа от строки должен давать index = последний байт + 1 (len)"
        );
    }
}
