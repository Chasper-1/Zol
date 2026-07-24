use crate::ast::{MarkupDoc, MarkupStyle};
use crate::token::{self, SpannedToken, Token};
use crate::ast::MARKERS;

/// Инкрементальный документ zoll.
///
/// Держит исходный текст, токены с позициями и AST.
 /// При правке через [`edit`](Self::edit) перетокенизирует только
/// изменившийся диапазон, а не весь текст.
pub struct IncrementalDoc {
    /// Полный исходный текст.
    pub source: String,
    /// Все токены с байтовыми позициями.
    pub tokens: Vec<SpannedToken>,
    /// AST (перестраивается из токенов при каждой правке).
    pub ast: MarkupDoc,
}

impl IncrementalDoc {
    /// Создать новый документ с полным парсингом текста.
    pub fn new(source: &str) -> Self {
        let tokens = token::tokenize(source);
        let ast = crate::parser::parse(&tokens);
        Self {
            source: source.to_string(),
            tokens,
            ast,
        }
    }

    /// Применить правку: удалить `[from..to)` и вставить `text`.
    ///
    /// Возвращает ссылку на обновлённый AST.
    ///
    /// ## Алгоритм
    ///
    /// 1. Применить правку к `source`
    /// 2. Найти `dirty_start` — байт, с которого нужно перетокенизировать:
    ///    - начало строки, содержащей `from`
    ///    - или раньше, если строка внутри multiline-маркера (`%%`, `$$`, `!!!`)
    /// 3. Удалить все старые токены с позицией ≥ `dirty_start`
    /// 4. Перетокенизировать `source[dirty_start..]`
    /// 5. Добавить новые токены
    /// 6. Перепарсить полный список токенов в AST
    pub fn edit(&mut self, from: usize, to: usize, text: &str) -> &MarkupDoc {
        // 1. Применить правку к source
        self.source.replace_range(from..to, text);

        // 2. Найти dirty_start
        let dirty_start = self.find_dirty_start(from);

        // 3. Удалить старые токены от dirty_start и дальше
        let keep = self
            .tokens
            .iter()
            .position(|t| t.start >= dirty_start)
            .unwrap_or(self.tokens.len());
        self.tokens.truncate(keep);

        // 4. Перетокенизировать от dirty_start до конца
        let new_tokens = token::tokenize_range(&self.source, dirty_start..self.source.len());
        self.tokens.extend(new_tokens);

        // 5. Перепарсить
        self.ast = crate::parser::parse(&self.tokens);

        &self.ast
    }

    /// Определяет байт, с которого нужно начинать перетокенизацию.
    ///
    /// В простом случае — начало строки, содержащей `from`.
    /// Если строка находится внутри multiline-маркера (`%%`, `$$`, `!!!`),
    /// откатывается до открытия этого маркера.
    fn find_dirty_start(&self, from: usize) -> usize {
        // Начало строки, содержащей `from` (в новом source)
        let line_start = self.source[..from]
            .rfind('\n')
            .map(|p| p + 1)
            .unwrap_or(0);

        // Сканируем токены от начала до line_start,
        // отслеживая открытые multiline-маркеры
        let mut open_multiline: Vec<MarkupStyle> = Vec::new();
        let mut earliest_open = line_start;

        for st in &self.tokens {
            if st.start >= line_start {
                break;
            }
            match &st.token {
                Token::Open(style) if is_multiline(*style) => {
                    if open_multiline.is_empty() {
                        earliest_open = st.start;
                    }
                    open_multiline.push(*style);
                }
                Token::Close(style) if is_multiline(*style) => {
                    // Ищем и удаляем последнее вхождение (для вложенных однотипных)
                    if let Some(idx) = open_multiline.iter().rposition(|s| s == style) {
                        open_multiline.remove(idx);
                        if open_multiline.is_empty() {
                            earliest_open = line_start; // сброс: всё закрыто
                        }
                    }
                }
                _ => {}
            }
        }

        // Если есть незакрытые multiline-маркеры — начинаем с самого раннего
        if open_multiline.is_empty() {
            line_start
        } else {
            earliest_open
        }
    }
}

/// Проверяет, является ли стиль multiline-маркером.
fn is_multiline(style: MarkupStyle) -> bool {
    MARKERS.iter().any(|m| m.style == style && m.multiline)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::MarkupNode;

    /// Извлекает текст из AST (без стилей).
    fn collect_text(doc: &MarkupDoc) -> String {
        fn flatten(node: &MarkupNode) -> String {
            match node {
                MarkupNode::Text(t) => t.clone(),
                MarkupNode::Formatted { children, .. } => {
                    children.iter().map(flatten).collect()
                }
            }
        }
        doc.children.iter().map(flatten).collect()
    }

    #[test]
    fn initial_parse() {
        let doc = IncrementalDoc::new("hello world");
        assert_eq!(collect_text(&doc.ast), "hello world");
    }

    #[test]
    fn insert_text() {
        let mut doc = IncrementalDoc::new("hello world");
        doc.edit(5, 5, " beautiful"); // "hello" + " beautiful" + " world"
        assert_eq!(collect_text(&doc.ast), "hello beautiful world");
    }

    #[test]
    fn delete_text() {
        let mut doc = IncrementalDoc::new("hello beautiful world");
        doc.edit(6, 16, ""); // удалить "beautiful "
        assert_eq!(collect_text(&doc.ast), "hello world");
    }

    #[test]
    fn replace_text() {
        let mut doc = IncrementalDoc::new("hello world");
        doc.edit(6, 11, "there"); // "world" → "there"
        assert_eq!(collect_text(&doc.ast), "hello there");
    }

    #[test]
    fn insert_inside_bold() {
        let mut doc = IncrementalDoc::new("**bold**");
        // "**bold**": 0-2 = **, 2-6 = "bold", 6-8 = **
        doc.edit(4, 4, "X"); // "**boXld**"
        assert_eq!(collect_text(&doc.ast), "boXld");
        // Структура должна сохраниться: один Formatted(BOLD)
        assert_eq!(doc.ast.children.len(), 1);
        assert!(matches!(
            &doc.ast.children[0],
            MarkupNode::Formatted { style, .. } if style.contains(MarkupStyle::BOLD)
        ));
    }

    #[test]
    fn replace_with_marker() {
        let mut doc = IncrementalDoc::new("hello world");
        doc.edit(0, 5, "**hello**"); // заменить "hello" на "**hello**"
        let text = collect_text(&doc.ast);
        assert_eq!(text, "hello world");
        // Должен появиться Formatted(BOLD)
        assert!(doc.ast.children.iter().any(|node| matches!(
            node,
            MarkupNode::Formatted { style, .. } if style.contains(MarkupStyle::BOLD)
        )));
    }

    #[test]
    fn edit_inside_formula() {
        let mut doc = IncrementalDoc::new("$$sum = 5$$\nplain text");
        // Вставляем 'X' перед 'plain' — правка вне формулы, формула не должна сломаться
        let pos = doc.source.find("plain").unwrap();
        doc.edit(pos, pos, "X");
        let text = collect_text(&doc.ast);
        assert_eq!(text, "sum = 5\nXplain text");
        // DISPLAY_FORMULA должен сохраниться
        assert!(doc.ast.children.iter().any(|node| matches!(
            node,
            MarkupNode::Formatted { style, .. } if style.contains(MarkupStyle::DISPLAY_FORMULA)
        )));
    }

    #[test]
    fn edit_outside_formula() {
        let mut doc = IncrementalDoc::new("$$sum = 5$$\nplain text");
        let pos = doc.source.find("plain").unwrap();
        doc.edit(pos, pos, "X");
        let text = collect_text(&doc.ast);
        assert!(text.contains("Xplain"));
    }

    #[test]
    fn tokens_are_positioned_correctly_after_edit() {
        let mut doc = IncrementalDoc::new("a **b** c");
        doc.edit(4, 5, "xyz"); // b → xyz
        assert_eq!(doc.tokens.len(), 5);
        // Text("a ")
        assert_eq!(doc.tokens[0].token, Token::Text("a ".to_string()));
        assert_eq!(doc.tokens[0].start, 0);
        // Open(BOLD)
        assert_eq!(doc.tokens[1].start, 2);
        // Text("xyz")
        assert_eq!(doc.tokens[2].token, Token::Text("xyz".to_string()));
        assert_eq!(doc.tokens[2].start, 4);
        assert_eq!(doc.tokens[2].end, 7);
        // Close(BOLD)
        assert_eq!(doc.tokens[3].start, 7);
        assert_eq!(doc.tokens[3].end, 9);
        // Text(" c")
        assert_eq!(doc.tokens[4].start, 9);
        assert_eq!(doc.tokens[4].end, 11);
    }

    #[test]
    fn multiple_edits() {
        let mut doc = IncrementalDoc::new("");
        doc.edit(0, 0, "a");
        doc.edit(1, 1, "b"); // ab
        doc.edit(2, 2, "c"); // abc
        assert_eq!(collect_text(&doc.ast), "abc");
    }

    #[test]
    fn edit_after_comment() {
        let mut doc = IncrementalDoc::new("%%comment%%\nline");
        let pos = doc.source.find("line").unwrap() + "line".len();
        doc.edit(pos, pos, "!"); // после "line" → "line!"
        let text = collect_text(&doc.ast);
        assert!(text.contains("line!"));
    }
}
