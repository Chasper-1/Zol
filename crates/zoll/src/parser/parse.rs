use crate::ast::{MarkupDoc, MarkupNode, MarkupStyle, Span};
use crate::token::{SpannedToken, Token};
use crate::parser::marker_text::marker_text_for_close;

/// Парсит токены в AST-документ.
///
/// Принимает `&[SpannedToken]` — использует и логическую структуру, и позиции.
/// Каждый узел AST получает `Span` — диапазон индексов токенов, которые он покрывает.
pub fn parse(tokens: &[SpannedToken]) -> MarkupDoc {
    let mut doc = MarkupDoc {
        children: Vec::new(),
    };
    // Стек: каждый элемент — (накопленные children родителя, стиль, индекс Open-токена)
    let mut stack: Vec<(Vec<MarkupNode>, MarkupStyle, usize)> = Vec::new();
    let current_children = &mut doc.children;

    for (i, st) in tokens.iter().enumerate() {
        match &st.token {
            Token::Text(t) => {
                current_children.push(MarkupNode::Text(t.clone(), Span::new(i, i + 1)));
            }
            Token::Newline => {
                current_children.push(MarkupNode::Text("\n".to_string(), Span::new(i, i + 1)));
            }
            Token::Open(style) => {
                let saved_children = std::mem::take(current_children);
                stack.push((saved_children, *style, i));
                *current_children = Vec::new();
            }
            Token::Close(style) => {
                if let Some(idx) = stack.iter().rposition(|(_, s, _)| s == style) {
                    let open_idx = stack[idx].2;
                    let formatted = MarkupNode::Formatted {
                        style: *style,
                        children: std::mem::take(current_children),
                        span: Span::new(open_idx, i + 1),
                    };

                    let mut merged = vec![formatted];

                    for _ in (idx + 1..stack.len()).rev() {
                        let (mut orphan_children, orphan_style, orphan_open_idx) = stack.pop().unwrap();
                        orphan_children.push(MarkupNode::Formatted {
                            style: orphan_style,
                            children: std::mem::take(current_children),
                            span: Span::new(orphan_open_idx, i + 1),
                        });
                        *current_children = orphan_children;
                        merged = std::mem::take(current_children);
                    }

                    let (mut parent_children, _parent_style, _) = stack.pop().unwrap();
                    parent_children.extend(merged);
                    *current_children = parent_children;
                } else {
                    current_children.push(MarkupNode::Text(
                        marker_text_for_close(*style),
                        Span::new(i, i + 1),
                    ));
                }
            }
        }
    }

    // Всё, что осталось на стеке — незакрытые маркеры
    while let Some((mut parent_children, orphan_style, orphan_open_idx)) = stack.pop() {
        parent_children.push(MarkupNode::Formatted {
            style: orphan_style,
            children: std::mem::take(current_children),
            span: Span::new(orphan_open_idx, orphan_open_idx + 1), // только Open, Close нет
        });
        *current_children = parent_children;
    }

    doc
}
