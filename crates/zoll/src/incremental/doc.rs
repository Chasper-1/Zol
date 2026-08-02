//! Инкрементальный документ — обёртка над `ParsedDoc`.
//!
//! Хранит единственный кеш: `ParsedDoc { lines: Vec<ParsedLine> }`.
//! При правке перепарсивает только затронутые строки.
//! merge-фазы нет — каждая строка уже знает свой тип.

use crate::ParsedDoc;
use crate::viewport::Viewport;

// Инкрементальный документ.
//
// Единственное поле — `doc: ParsedDoc`. Никаких дополнительных структур.
pub struct IncrementalDoc {
    // Единственный кеш документа.
    pub doc: ParsedDoc,
    // Исходный текст (для обратной совместимости и быстрого доступа).
    pub source: String,
    // Байтовые начала строк.
    pub line_starts: Vec<usize>,
}

impl IncrementalDoc {
    // Создать новый документ из текста.
    pub fn new(text: &str) -> Self {
        let line_starts = build_line_starts(text);
        let doc = ParsedDoc::parse(text);
        IncrementalDoc {
            source: text.to_string(),
            line_starts,
            doc,
        }
    }

    // Применить правку: удалить `[from..to)` и вставить `text`.
    //
    // Перепарсивает только затронутые строки.
    // Возвращает ссылку на обновлённый `ParsedDoc`.
    pub fn edit(&mut self, from: usize, to: usize, text: &str) -> &ParsedDoc {
        let start_line = self.line_at_byte(from);
        let _end_line_old = if to > from {
            self.line_at_byte(to.min(self.source.len()))
        } else {
            start_line
        };

        let removed_ends_with_newline = if to > from {
            matches!(
                self.source.as_bytes().get(to.saturating_sub(1)),
                Some(b'\n')
            )
        } else {
            false
        };

        let old_line_count = self.line_starts.len();

        // Применяем правку к source
        self.source.replace_range(from..to, text);
        self.rebuild_line_starts(from, to, text, removed_ends_with_newline);

        let new_line_count = self.line_starts.len();

        // Выравниваем количество строк в doc.lines
        while self.doc.lines.len() < new_line_count {
            self.doc.lines.push(crate::ParsedLine::empty());
        }
        self.doc.lines.truncate(new_line_count);

        // Перепарсиваем изменившиеся строки
        // (от start_line до конца, так как индексы строк могли сдвинуться)
        for i in start_line..new_line_count.min(old_line_count.max(new_line_count)) {
            let line_text = self.get_line_text(i);
            self.doc.lines[i] = crate::parser::parse_line(line_text);
        }

        &self.doc
    }

    // Применить правку и перепарсить только видимый диапазон.
    pub fn edit_visible(
        &mut self,
        from: usize,
        to: usize,
        text: &str,
        _viewport: &Viewport,
    ) -> &ParsedDoc {
        // Пока работает как обычный edit, потом можно оптимизировать
        self.edit(from, to, text)
    }

    // Получить текст строки по индексу.
    fn get_line_text(&self, idx: usize) -> &str {
        if idx >= self.line_starts.len() {
            return "";
        }
        let start = self.line_starts[idx];
        let end = if idx + 1 < self.line_starts.len() {
            self.line_starts[idx + 1]
        } else {
            self.source.len()
        };
        let line = &self.source[start..end];
        if let Some(stripped) = line.strip_suffix('\n') {
            stripped.strip_suffix('\r').unwrap_or(stripped)
        } else {
            line
        }
    }

    // Номер строки по байтовой позиции.
    pub fn line_number(&self, byte_pos: usize) -> usize {
        let byte_pos = byte_pos.min(self.source.len());
        match self.line_starts.binary_search(&byte_pos) {
            Ok(i) => i,
            Err(i) => {
                if i == 0 {
                    0
                } else {
                    i - 1
                }
            }
        }
    }

    // Количество строк.
    pub fn num_lines(&self) -> usize {
        self.line_starts.len()
    }

    fn line_at_byte(&self, byte: usize) -> usize {
        self.line_number(byte)
    }

    fn rebuild_line_starts(
        &mut self,
        from: usize,
        to_old: usize,
        text: &str,
        removed_ends_with_newline: bool,
    ) {
        let start_idx = self.line_at_byte(from);
        let mut result: Vec<usize> = self.line_starts[..=start_idx].to_vec();
        result.truncate(start_idx + 1);

        for (i, c) in text.char_indices() {
            if c == '\n' {
                result.push(from + i + 1);
            }
        }

        let delta = text.len() as isize - (to_old - from) as isize;
        for i in (start_idx + 1)..self.line_starts.len() {
            let old_pos = self.line_starts[i];
            if old_pos < to_old {
                continue;
            }
            if old_pos == to_old && removed_ends_with_newline {
                continue;
            }
            let new_pos = (old_pos as isize + delta) as usize;
            if result.last().copied().map_or(true, |last| new_pos > last) {
                result.push(new_pos);
            }
        }

        self.line_starts = result;
    }
}

// Построить массив начал строк из текста.
pub fn build_line_starts(text: &str) -> Vec<usize> {
    let mut starts = vec![0usize];
    for (i, c) in text.char_indices() {
        if c == '\n' {
            starts.push(i + 1);
        }
    }
    starts
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BlockKind;

    #[test]
    fn new_doc_creates_lines() {
        let doc = IncrementalDoc::new("hello\nworld");
        assert_eq!(doc.num_lines(), 2);
        assert_eq!(doc.doc.lines.len(), 2);
    }

    #[test]
    fn edit_single_line() {
        let mut doc = IncrementalDoc::new("hello world");
        doc.edit(0, 5, "hi");
        assert_eq!(doc.source, "hi world");
    }

    #[test]
    fn edit_preserves_line_count() {
        let mut doc = IncrementalDoc::new("line1\nline2\nline3");
        assert_eq!(doc.num_lines(), 3);
        doc.edit(0, 0, "X");
        assert_eq!(doc.num_lines(), 3);
    }

    #[test]
    fn edit_adds_newlines() {
        let mut doc = IncrementalDoc::new("hello world");
        doc.edit(6, 6, "\nnew\nlines\n");
        assert!(doc.num_lines() >= 3);
        assert_eq!(doc.source, "hello \nnew\nlines\nworld");
    }

    #[test]
    fn edit_removes_lines() {
        let mut doc = IncrementalDoc::new("a\nb\nc\nd");
        doc.edit(2, 5, "");
        assert_eq!(doc.source, "a\n\nd");
    }

    #[test]
    fn simple_text_parse() {
        let doc = IncrementalDoc::new("hello world");
        assert_eq!(doc.doc.lines.len(), 1);
        assert_eq!(doc.doc.lines[0].kind, BlockKind::Paragraph);
    }

    #[test]
    fn empty_source() {
        let doc = IncrementalDoc::new("");
        assert_eq!(doc.line_starts.len(), 1);
        assert!(doc.doc.lines.is_empty());
    }

    #[test]
    fn header_in_doc() {
        let doc = IncrementalDoc::new("#1# Title\ncontent");
        assert_eq!(doc.doc.lines.len(), 2);
        assert_eq!(doc.doc.lines[0].kind, BlockKind::Header(1));
        assert_eq!(doc.doc.lines[1].kind, BlockKind::Paragraph);
    }

    #[test]
    fn build_line_starts_empty() {
        assert_eq!(build_line_starts(""), vec![0]);
    }

    #[test]
    fn build_line_starts_single_line() {
        assert_eq!(build_line_starts("hello"), vec![0]);
    }

    #[test]
    fn build_line_starts_two_lines() {
        assert_eq!(build_line_starts("ab\ncd"), vec![0, 3]);
    }

    #[test]
    fn line_number_byte_zero() {
        let doc = IncrementalDoc::new("hello\nworld");
        assert_eq!(doc.line_number(0), 0);
    }

    #[test]
    fn line_number_byte_in_first_line() {
        let doc = IncrementalDoc::new("hello\nworld");
        assert_eq!(doc.line_number(3), 0);
    }

    #[test]
    fn line_number_byte_on_newline() {
        let doc = IncrementalDoc::new("hello\nworld");
        assert_eq!(doc.line_number(5), 0);
        assert_eq!(doc.line_number(6), 1);
    }

    #[test]
    fn edit_preserves_bold() {
        let mut doc = IncrementalDoc::new("**bold** text");
        doc.edit(9, 13, "content");
        assert_eq!(doc.source, "**bold** content");
        // После правки bold-сегмент должен сохраниться
        let has_bold = doc.doc.lines[0]
            .segments
            .iter()
            .any(|s| s.style.contains(crate::MarkStyle::BOLD));
        assert!(has_bold, "Bold should be preserved after edit");
    }

    #[test]
    fn parsedoc_to_text_roundtrip() {
        let text = "#1# Title\n\nHello **world**\n- item\n";
        let doc = IncrementalDoc::new(text);
        let out = doc.doc.to_text();
        assert_eq!(text, out, "roundtrip must preserve text");
    }
}
