use crate::ast::{MarkupDoc, MarkupNode, MarkupStyle};
use crate::token::{SpannedToken, Token};
use crate::parser::marker_text::marker_text_for_close;

/// Парсит токены в AST-документ.
///
/// Принимает `&[SpannedToken]` — позиции игнорируются, парсер работает
/// только с логической структурой токенов.
pub fn parse(tokens: &[SpannedToken]) -> MarkupDoc {
    let mut doc = MarkupDoc {
        children: Vec::new(),
    };
    // Стек: каждый элемент — (накопленные children родителя, стиль открытого маркера)
    let mut stack: Vec<(Vec<MarkupNode>, MarkupStyle)> = Vec::new();
    // Текущий список children (пишем в doc.children, пока не войдём в formatted)
    let current_children = &mut doc.children;

    for st in tokens {
        match &st.token {
            Token::Text(t) => {
                current_children.push(MarkupNode::Text(t.clone()));
            }
            Token::Newline => {
                current_children.push(MarkupNode::Text("\n".to_string()));
            }
            Token::Open(style) => {
                let saved_children = std::mem::take(current_children);
                stack.push((saved_children, *style));
                *current_children = Vec::new();
            }
            Token::Close(style) => {
                if let Some(idx) = stack.iter().rposition(|(_, s)| s == style) {
                    let formatted = MarkupNode::Formatted {
                        style: *style,
                        children: std::mem::take(current_children),
                    };

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

                    let (mut parent_children, _parent_style) = stack.pop().unwrap();
                    parent_children.extend(merged);
                    *current_children = parent_children;
                } else {
                    current_children.push(MarkupNode::Text(marker_text_for_close(*style)));
                }
            }
        }
    }

    // Всё, что осталось на стеке — незакрытые маркеры
    while let Some((mut parent_children, orphan_style)) = stack.pop() {
        parent_children.push(MarkupNode::Formatted {
            style: orphan_style,
            children: std::mem::take(current_children),
        });
        *current_children = parent_children;
    }

    doc
}
