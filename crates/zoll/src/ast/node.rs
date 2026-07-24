use super::span::Span;
use super::style::MarkupStyle;

/// Узел AST.
#[derive(Debug, Clone, PartialEq)]
pub enum MarkupNode {
    /// Простой текст. span охватывает ровно один токен (Text или Newline).
    Text(String, Span),
    /// Форматированный блок с вложенными узлами.
    Formatted {
        style: MarkupStyle,
        children: Vec<MarkupNode>,
        span: Span,
    },
}

impl MarkupNode {
    /// Вернуть span узла.
    pub fn span(&self) -> Span {
        match self {
            MarkupNode::Text(_, span) => *span,
            MarkupNode::Formatted { span, .. } => *span,
        }
    }
}
