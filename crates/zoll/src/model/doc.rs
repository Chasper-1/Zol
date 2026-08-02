//! Разобранный документ — плоский массив строк с типами и сегментами.
//!
//! Это **единственный кеш** документа. Никаких деревьев, никаких
//! промежуточных структур. Парсер пишет сразу сюда, редактор читает отсюда.

use super::line::{BlockContainer, BlockKind, BlockRole, ParsedLine};
use super::seg::{MarkStyle, Segment};

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
        let mut lines: Vec<ParsedLine> = text
            .lines()
            .map(|line| crate::parser::parse_line(line))
            .collect();
        assign_block_roles(&mut lines);
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
        assign_block_roles(&mut self.lines);
    }

    // Добавить строку в конец.
    pub fn push_line(&mut self, source: &str) {
        self.lines.push(crate::parser::parse_line(source));
        assign_block_roles(&mut self.lines);
    }

    // Вставить строку по индексу.
    pub fn insert_line(&mut self, idx: usize, source: &str) {
        self.lines.insert(idx, crate::parser::parse_line(source));
        assign_block_roles(&mut self.lines);
    }

    // Удалить строку по индексу.
    pub fn remove_line(&mut self, idx: usize) {
        self.lines.remove(idx);
        assign_block_roles(&mut self.lines);
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

// Назначить роли строкам блок-контейнеров по контексту документа.
//
// Парсер одной строки не знает контекста: он помечает любую блок-строку
// (`%%%`, `$$$`, `!!!`, ` ``` `) как `Open`. Этот проход исправляет роли:
// - первый маркер блока → `Open`
// - повторный маркер того же типа → `Close` (вложенность не поддерживается)
// - строки между ними → `Content`
// Для код-блоков содержимое не размечается (один PLAIN-сегмент на строку).
pub fn assign_block_roles(lines: &mut [ParsedLine]) {
    let mut active: Option<BlockContainer> = None;
    for line in lines.iter_mut() {
        if let BlockKind::Block { kind, role, .. } = &mut line.kind {
            if active == Some(*kind) {
                *role = BlockRole::Close;
                active = None;
            } else {
                *role = BlockRole::Open;
                active = Some(*kind);
            }
        } else if let Some(kind) = active {
            // Содержимое активного блока.
            if kind == BlockContainer::Code {
                // Код — сырой текст без inline-разметки.
                let len = line.source.len();
                line.kind = BlockKind::Block {
                    kind,
                    title: None,
                    role: BlockRole::Content,
                };
                line.segments = vec![Segment::new(0, len, MarkStyle::PLAIN)];
            } else {
                line.kind = BlockKind::Block {
                    kind,
                    title: None,
                    role: BlockRole::Content,
                };
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assign_roles_comment_block() {
        let doc = ParsedDoc::parse("%%%\nhidden\n%%%");
        assert_eq!(doc.lines.len(), 3);
        assert_eq!(
            doc.lines[0].kind,
            BlockKind::Block {
                kind: BlockContainer::Comment,
                title: None,
                role: BlockRole::Open
            }
        );
        assert_eq!(
            doc.lines[1].kind,
            BlockKind::Block {
                kind: BlockContainer::Comment,
                title: None,
                role: BlockRole::Content
            }
        );
        assert_eq!(
            doc.lines[2].kind,
            BlockKind::Block {
                kind: BlockContainer::Comment,
                title: None,
                role: BlockRole::Close
            }
        );
    }

    #[test]
    fn assign_roles_code_block_unparsed_content() {
        let doc = ParsedDoc::parse("```rust\nlet x = **5**;\n```");
        assert_eq!(
            doc.lines[0].kind,
            BlockKind::Block {
                kind: BlockContainer::Code,
                title: Some("rust".to_string()),
                role: BlockRole::Open
            }
        );
        // Содержимое код-блока — сырой PLAIN текст, без inline-разметки.
        assert_eq!(
            doc.lines[1].kind,
            BlockKind::Block {
                kind: BlockContainer::Code,
                title: None,
                role: BlockRole::Content
            }
        );
        assert_eq!(doc.lines[1].segments.len(), 1);
        assert_eq!(doc.lines[1].segments[0].style, MarkStyle::PLAIN);
        assert_eq!(
            doc.lines[2].kind,
            BlockKind::Block {
                kind: BlockContainer::Code,
                title: None,
                role: BlockRole::Close
            }
        );
    }

    #[test]
    fn roundtrip_block_comment() {
        // to_text всегда добавляет завершающий перенос.
        let text = "%%%\nsecret\n%%%\n";
        let doc = ParsedDoc::parse(text);
        assert_eq!(doc.to_text(), text);
    }

    #[test]
    fn roundtrip_code_block() {
        let text = "```rust\nfn main() {}\n```\n";
        let doc = ParsedDoc::parse(text);
        assert_eq!(doc.to_text(), text);
    }

    #[test]
    fn unclosed_block_roles() {
        // Незакрытый блок: маркер открыт, роль у остальных строк — Content.
        let doc = ParsedDoc::parse("%%%\ninside");
        assert_eq!(
            doc.lines[0].kind,
            BlockKind::Block {
                kind: BlockContainer::Comment,
                title: None,
                role: BlockRole::Open
            }
        );
        assert_eq!(
            doc.lines[1].kind,
            BlockKind::Block {
                kind: BlockContainer::Comment,
                title: None,
                role: BlockRole::Content
            }
        );
    }
}
