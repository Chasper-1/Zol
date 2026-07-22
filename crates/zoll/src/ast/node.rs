use super::style::MarkupStyle;

/// Узел AST.
#[derive(Debug, Clone, PartialEq)]
pub enum MarkupNode {
    /// Простой текст.
    Text(String),
    /// Форматированный блок с вложенными узлами.
    Formatted {
        style: MarkupStyle,
        children: Vec<MarkupNode>,
    },
}
