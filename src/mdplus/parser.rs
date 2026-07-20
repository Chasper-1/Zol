//! Стековый парсер mdplus.
//!
//! Преобразует поток токенов в AST (MarkupDoc).
//! Использует стек вместо рекурсии — нет переполнения стека на глубокой вложенности.

use super::ast::{MarkupDoc, MarkupNode, MarkupStyle};
use super::token::Token;

/// Парсит токены в AST-документ.
pub fn parse(tokens: &[Token]) -> MarkupDoc {
    let mut doc = MarkupDoc {
        children: Vec::new(),
    };
    // Стек: каждый элемент — (накопленные_children_родителя, стиль_открытого_маркера_на_этом_уровне)
    // Стиль открытого маркера используется для поиска парного Close.
    let mut stack: Vec<(Vec<MarkupNode>, MarkupStyle)> = Vec::new();
    // Текущий список children (пишем в doc.children, пока не войдём в formatted)
    let current_children = &mut doc.children;

    for token in tokens {
        match token {
            Token::Text(t) => {
                current_children.push(MarkupNode::Text(t.clone()));
            }
            Token::Newline => {
                current_children.push(MarkupNode::Text("\n".to_string()));
            }
            Token::Open(style) => {
                // Сохраняем текущий контекст в стек
                let saved_children = std::mem::take(current_children);
                stack.push((saved_children, *style));
                // Создаём новый контейнер для вложенных узлов
                *current_children = Vec::new();
            }
            Token::Close(style) => {
                // Ищем соответствующий Open в стеке (идём сверху вниз)
                if let Some(idx) = stack.iter().rposition(|(_, s)| s == style) {
                    // Собираем текущие children в Formatted-узел
                    let formatted = MarkupNode::Formatted {
                        style: *style,
                        children: std::mem::take(current_children),
                    };

                    // Узлы между idx и вершиной — orphans (непарные открытия).
                    // Оборачиваем каждый orphan-level как Formatted и вкладываем друг в друга.
                    let mut merged = vec![formatted];

                    for _ in (idx + 1..stack.len()).rev() {
                        let (mut orphan_children, orphan_style) = stack.pop().unwrap();
                        orphan_children.push(MarkupNode::Formatted {
                            style: orphan_style,
                            children: std::mem::take(current_children),
                        });
                        *current_children = orphan_children;
                        merged = std::mem::take(current_children);
                    }

                    // Восстанавливаем родительский контекст с idx
                    let (mut parent_children, _parent_style) = stack.pop().unwrap();
                    parent_children.extend(merged);
                    *current_children = parent_children;
                } else {
                    // Нет парного Open — закрывающий маркер считается текстом
                    current_children.push(MarkupNode::Text(marker_text_for_close(*style)));
                }
            }
        }
    }

    // Всё, что осталось на стеке — незакрытые маркеры, оформляем как Formatted с текстом внутри
    while let Some((mut parent_children, orphan_style)) = stack.pop() {
        parent_children.push(MarkupNode::Formatted {
            style: orphan_style,
            children: std::mem::take(current_children),
        });
        *current_children = parent_children;
    }

    doc
}

/// Возвращает текст маркера для стиля (используется при непарных закрытиях).
fn marker_text_for_close(style: MarkupStyle) -> String {
    use super::ast::MARKERS;
    for m in MARKERS {
        if m.style == style {
            return m.close.to_string();
        }
    }
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mdplus::token::tokenize;

    fn flatten_text(node: &MarkupNode) -> String {
        match node {
            MarkupNode::Text(t) => t.clone(),
            MarkupNode::Formatted { children, .. } => children.iter().map(flatten_text).collect(),
        }
    }

    fn collect_text(doc: &MarkupDoc) -> String {
        doc.children.iter().map(flatten_text).collect()
    }

    #[test]
    fn plain_text_roundtrip() {
        let tokens = tokenize("hello world");
        let doc = parse(&tokens);
        assert_eq!(collect_text(&doc), "hello world");
    }

    #[test]
    fn bold_roundtrip() {
        let tokens = tokenize("**bold**");
        let doc = parse(&tokens);
        assert_eq!(collect_text(&doc), "bold");
    }

    #[test]
    fn bold_with_text() {
        let tokens = tokenize("a **bold** b");
        let doc = parse(&tokens);
        assert_eq!(collect_text(&doc), "a bold b");
        assert_eq!(doc.children.len(), 3);
        assert!(matches!(doc.children[0], MarkupNode::Text(_)));
        assert!(matches!(doc.children[1], MarkupNode::Formatted { .. }));
        assert!(matches!(doc.children[2], MarkupNode::Text(_)));
    }

    #[test]
    fn nested_bold_italic() {
        let tokens = tokenize("**a //b// c**");
        let doc = parse(&tokens);
        assert_eq!(collect_text(&doc), "a b c");
        if let MarkupNode::Formatted { style, children } = &doc.children[0] {
            assert_eq!(*style, MarkupStyle::BOLD);
            assert_eq!(children.len(), 3);
            if let MarkupNode::Formatted { style, .. } = &children[1] {
                assert_eq!(*style, MarkupStyle::ITALIC);
            } else {
                panic!("expected italic");
            }
        } else {
            panic!("expected bold");
        }
    }

    #[test]
    fn no_close_returns_plain() {
        let tokens = tokenize("**bold");
        let doc = parse(&tokens);
        assert_eq!(collect_text(&doc), "**bold");
    }

    #[test]
    fn same_type_nesting() {
        let tokens = tokenize("**a **b** c**");
        let doc = parse(&tokens);
        assert_eq!(collect_text(&doc), "a b c");
        if let MarkupNode::Formatted { style, children } = &doc.children[0] {
            assert_eq!(*style, MarkupStyle::BOLD);
            assert_eq!(children.len(), 3);
            if let MarkupNode::Formatted { style, .. } = &children[1] {
                assert_eq!(*style, MarkupStyle::BOLD);
            } else {
                panic!("expected nested bold");
            }
        } else {
            panic!("expected bold");
        }
    }

    #[test]
    fn spoiler_inline() {
        let tokens = tokenize("!!secret!!");
        let doc = parse(&tokens);
        assert_eq!(collect_text(&doc), "secret");
        if let MarkupNode::Formatted { style, .. } = &doc.children[0] {
            assert_eq!(*style, MarkupStyle::SPOILER);
        } else {
            panic!("expected spoiler");
        }
    }

    #[test]
    fn comment_is_marked() {
        let tokens = tokenize("before %%comment%% after");
        let doc = parse(&tokens);
        assert_eq!(collect_text(&doc), "before comment after");
    }
}
