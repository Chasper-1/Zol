//! Инкрементальный документ с построчным хранением AST.
//!
//! `IncrementalDoc` хранит source + line_asts (по строке).
//! Любая правка перепарсивает только изменённые строки и пересобирает
//! общий AST через merge, начиная с первого затронутого блока.

use crate::ast::{LineAST, MarkupDoc};
use crate::parser::merge;

/// Инкрементальный документ.
///
/// # Пример
///
/// ```rust
/// use zoll::incremental::IncrementalDoc;
///
/// let mut doc = IncrementalDoc::new("**hello** world");
/// doc.edit(0, 0, "very ");
/// ```
pub struct IncrementalDoc {
    /// Исходный текст.
    pub source: String,
    /// Байтовые начала строк (line_starts[i] = байт начала строки i).
    pub line_starts: Vec<usize>,
    /// AST каждой строки (после parse_line).
    pub line_asts: Vec<LineAST>,
    /// Собранный общий AST (после merge).
    pub merged_ast: MarkupDoc,
}

impl IncrementalDoc {
    /// Создать новый документ из текста.
    pub fn new(text: &str) -> Self {
        let line_starts = build_line_starts(text);
        let line_asts: Vec<LineAST> = text.lines().map(|l| parse_line_or_empty(l)).collect();
        let merged_ast = merge(&line_asts);

        IncrementalDoc {
            source: text.to_string(),
            line_starts,
            line_asts,
            merged_ast,
        }
    }

    /// Применить правку: удалить `[from..to)` и вставить `text`.
    ///
    /// ## Как это работает
    ///
    /// 1. Определить номер изменённой строки по `from`.
    /// 2. Применить `source.replace_range(from..to, text)`.
    /// 3. Перестроить `line_starts` начиная с изменённой позиции.
    /// 4. Перепарсить изменённые строки (и только их).
    /// 5. Пересобрать общий AST (merge).
    pub fn edit(&mut self, from: usize, to: usize, text: &str) -> &MarkupDoc {
        let old_lines_before = self.line_at_byte(from);
        let old_lines_removed = if to > from {
            self.line_at_byte(to).saturating_sub(old_lines_before)
        } else {
            0
        };

        // Применяем правку
        self.source.replace_range(from..to, text);

        // Перестраиваем line_starts
        self.rebuild_line_starts(from);

        // Определяем, какие строки изменились
        let new_lines = self.source[self.line_starts[old_lines_before]..]
            .lines()
            .count()
            .max(1);
        let changed_line_count = new_lines + old_lines_removed;

        // Перепарсить изменённые строки
        let start_line = old_lines_before;
        let end_line = (start_line + changed_line_count).min(self.line_asts.len());

        // Если строк стало больше, расширяем line_asts
        while self.line_asts.len() < self.line_starts.len() {
            self.line_asts.push(LineAST::Empty);
        }

        for i in start_line..end_line {
            let line = self.get_line_text(i);
            self.line_asts[i] = parse_line_or_empty(&line);
        }

        // Если строк стало меньше (merged lines), сдвигаем
        let expected_lines = self.line_starts.len();
        self.line_asts.truncate(expected_lines);

        // Добавляем пустые, если нужно
        while self.line_asts.len() < expected_lines {
            self.line_asts.push(LineAST::Empty);
        }

        // Пересобрать AST от начала затронутого блока
        let merge_start = self.find_block_start(start_line);
        let partial: Vec<LineAST> = self.line_asts[merge_start..].to_vec();
        // Собираем полный AST: clean часть + новая merged часть
        if merge_start == 0 {
            self.merged_ast = merge(&self.line_asts);
        } else {
            let clean = merge(&self.line_asts[..merge_start]);
            let dirty = merge(&partial);
            // Склеиваем: берём clean, добавляем dirty
            let mut combined = clean;
            combined.children.extend(dirty.children);
            self.merged_ast = combined;
        }

        &self.merged_ast
    }

    /// Получить текст строки по индексу.
    fn get_line_text(&self, idx: usize) -> String {
        if idx >= self.line_starts.len() {
            return String::new();
        }
        let start = self.line_starts[idx];
        let end = if idx + 1 < self.line_starts.len() {
            self.line_starts[idx + 1]
        } else {
            self.source.len()
        };
        // Не включаем \n
        let mut line = self.source[start..end].to_string();
        if line.ends_with('\n') {
            line.pop();
            if line.ends_with('\r') {
                line.pop();
            }
        }
        line
    }

    /// Найти номер строки по байтовой позиции.
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

    /// Количество строк.
    pub fn num_lines(&self) -> usize {
        self.line_starts.len()
    }

    // ─── Приватные помощники ─────────────────────────────────

    /// Номер строки по байту (0-based).
    fn line_at_byte(&self, byte: usize) -> usize {
        self.line_number(byte)
    }

    /// Перестраивает line_starts после правки в `from`.
    fn rebuild_line_starts(&mut self, from: usize) {
        let start_idx = self.line_at_byte(from);

        // Сохраняем начала строк ДО from (включая строку с from)
        let mut result: Vec<usize> = self.line_starts[..=start_idx].to_vec();
        result.truncate(start_idx + 1); // отрезаем хвост (устаревшие позиции)

        // Находим все начала строк ПОСЛЕ from в новом source
        let suffix: Vec<usize> = self.source[from..]
            .char_indices()
            .filter(|(_, c)| *c == '\n')
            .map(|(i, _)| from + i + 1)
            .collect();

        result.extend(suffix);
        self.line_starts = result;
    }

    /// Найти начало блок-левел блока, содержащего `line`.
    /// Если строка не внутри блока, возвращает `line`.
    fn find_block_start(&self, line: usize) -> usize {
        let mut i = line;
        // Идём назад, ищем незакрытый BlockMarker или SpoilerBlockOpen
        let mut depth = 0i32;
        while i > 0 {
            i -= 1;
            match &self.line_asts[i] {
                LineAST::BlockMarker(bt) => {
                    if depth == 0 {
                        return i; // Начало блока
                    }
                    depth -= 1;
                }
                LineAST::SpoilerBlockOpen(_) => {
                    if depth == 0 {
                        return i;
                    }
                    depth -= 1;
                }
                LineAST::Empty | LineAST::Header(_, _) | LineAST::ThematicBreak => {
                    if depth == 0 {
                        return line; // Вне блока, начинаем с line
                    }
                }
                _ => {}
            }
        }
        0
    }
}

// ─── Помощники ───────────────────────────────────────────────

/// Парсит строку или возвращает Empty для пустой.
fn parse_line_or_empty(line: &str) -> LineAST {
    if line.trim().is_empty() && line.is_empty() {
        // Реальная пустая строка (не просто пробелы)
        if line.is_empty() {
            return LineAST::Empty;
        }
    }
    crate::parser::parse_line(line)
}

/// Построить массив начал строк из текста.
pub fn build_line_starts(text: &str) -> Vec<usize> {
    let mut starts = vec![0usize];
    for (i, c) in text.char_indices() {
        if c == '\n' {
            starts.push(i + 1);
        }
    }
    starts
}

// ─── Тесты ───────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::MarkupNode;
    use crate::ast::MarkupStyle;

    #[test]
    fn new_doc_creates_lines() {
        let doc = IncrementalDoc::new("hello\nworld");
        assert_eq!(doc.line_starts.len(), 2);
        assert_eq!(doc.line_asts.len(), 2);
    }

    #[test]
    fn edit_single_line() {
        let mut doc = IncrementalDoc::new("hello world");
        doc.edit(0, 5, "hi");
        assert_eq!(doc.source, "hi world");
    }

    #[test]
    fn edit_preserves_ast() {
        let mut doc = IncrementalDoc::new("**bold** text");
        doc.edit(9, 13, "content"); // меняем "text" (позиции 9..13) на "content"
        // Проверяем: bold должен сохраниться
        let has_bold = doc.merged_ast.children.iter().any(|n| {
            matches!(n, MarkupNode::Formatted { style, .. } if *style == MarkupStyle::BOLD)
        });
        assert!(has_bold, "Bold formatting should be preserved after edit");
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
        assert!(doc.num_lines() >= 3, "should have at least 3 lines, got {}", doc.num_lines());
        assert_eq!(doc.source, "hello \nnew\nlines\nworld");
    }

    #[test]
    fn edit_removes_lines() {
        let mut doc = IncrementalDoc::new("a\nb\nc\nd");
        doc.edit(2, 5, ""); // удаляем "b\nc" → остаётся "a\n\nd" (два \n подряд)
        assert_eq!(doc.source, "a\n\nd");
    }

    #[test]
    fn simple_text_parse() {
        let doc = IncrementalDoc::new("hello world");
        assert_eq!(doc.merged_ast.children.len(), 1);
    }

    #[test]
    fn empty_source() {
        let doc = IncrementalDoc::new("");
        assert_eq!(doc.line_starts.len(), 1);
        assert_eq!(doc.merged_ast.children.len(), 0);
    }

    #[test]
    fn header_in_doc() {
        let doc = IncrementalDoc::new("#1# Title\ncontent");
        assert_eq!(doc.merged_ast.children.len(), 2);
        assert!(matches!(&doc.merged_ast.children[0], MarkupNode::Header { level: 1, .. }));
    }

    #[test]
    fn multiline_paragraph() {
        let doc = IncrementalDoc::new("line1\nline2\n\nline3");
        // line1 + \n + line2 — один параграф
        // пустая строка — разделитель
        // line3 — второй параграф
        assert!(doc.merged_ast.children.len() >= 2);
    }
}
