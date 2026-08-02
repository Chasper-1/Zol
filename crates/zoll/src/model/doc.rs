//! Разобранный документ — плоский массив строк с типами и сегментами.
//!
//! Это **единственный кеш** документа. Никаких деревьев, никаких
//! промежуточных структур. Парсер пишет сразу сюда, редактор читает отсюда.

use super::line::{BlockKind, BlockRole, ParsedLine};

// Разобранный документ zoll.
//
// # Кеш
//
// Всё состояние документа — `lines`. При редактировании строки
// перепарсивается только она, `lines[i] = parse_line(new_text)`.
// merge-фаза не нужна — каждая строка уже знает свой `BlockKind`.
//
// # Сериализация
//
// Для получения текста из документа: проход по `lines`,
// группировка блок-контейнеров (%%%, $$$, !!!, \`\`\`) по
// соседним `BlockKind::Block`.
#[derive(Debug, Clone)]
pub struct ParsedDoc {
    // Строки документа, каждая со своим типом и сегментами.
    pub lines: Vec<ParsedLine>,
}

impl ParsedDoc {
    // Создать пустой документ.
    pub fn new() -> Self {
        Self { lines: Vec::new() }
    }

    // Создать документ из текста (парсит всё сразу).
    pub fn parse(text: &str) -> Self {
        let lines: Vec<ParsedLine> = text
            .lines()
            .map(|line| crate::parser::parse_line(line))
            .collect();
        Self { lines }
    }

    // Количество строк.
    pub fn num_lines(&self) -> usize {
        self.lines.len()
    }

    // Получить строку по индексу.
    pub fn line(&self, idx: usize) -> Option<&ParsedLine> {
        self.lines.get(idx)
    }

    // Заменить строку по индексу (перепарсить новую).
    pub fn set_line(&mut self, idx: usize, source: &str) {
        if idx < self.lines.len() {
            self.lines[idx] = crate::parser::parse_line(source);
        }
    }

    // Добавить строку в конец.
    pub fn push_line(&mut self, source: &str) {
        self.lines.push(crate::parser::parse_line(source));
    }

    // Вставить строку по индексу.
    pub fn insert_line(&mut self, idx: usize, source: &str) {
        self.lines.insert(idx, crate::parser::parse_line(source));
    }

    // Удалить строку по индексу.
    pub fn remove_line(&mut self, idx: usize) {
        self.lines.remove(idx);
    }

    // Полный текст документа (сериализация).
    //
    // Проходит по строкам, группируя блок-контейнеры,
    // и возвращает текст в формате zoll.
    pub fn to_text(&self) -> String {
        let mut out = String::new();
        let mut i = 0;
        while i < self.lines.len() {
            let line = &self.lines[i];
            // Блок-контейнеры: группируем Open → Content → Close
            if let BlockKind::Block {
                kind,
                title: _,
                role,
            } = &line.kind
            {
                if *role == BlockRole::Open {
                    // Открывающий маркер
                    out.push_str(&line.source);
                    out.push('\n');
                    i += 1;
                    // Содержимое до Close
                    while i < self.lines.len() {
                        match &self.lines[i].kind {
                            BlockKind::Block {
                                kind: bk, role: br, ..
                            } if *bk == *kind && *br == BlockRole::Close => {
                                out.push_str(&self.lines[i].source);
                                out.push('\n');
                                i += 1;
                                break;
                            }
                            _ => {
                                out.push_str(&self.lines[i].source);
                                out.push('\n');
                                i += 1;
                            }
                        }
                    }
                    continue;
                }
                // Если Open не нашли, но блок есть — пишем как есть
                out.push_str(&line.source);
                out.push('\n');
                i += 1;
            } else {
                out.push_str(&line.source);
                out.push('\n');
                i += 1;
            }
        }
        out
    }
}

impl Default for ParsedDoc {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ParsedDoc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_text())
    }
}
