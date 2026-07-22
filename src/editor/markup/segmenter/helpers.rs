use zoll::ast::MarkupStyle;

use crate::editor::markup::segment::StyleFlags;

/// Комбинирует стили (наследование).
pub fn combine_style(parent: MarkupStyle, child: MarkupStyle) -> MarkupStyle {
    MarkupStyle(parent.bits() | child.bits())
}

/// Преобразует AST-стиль в StyleFlags редактора.
pub fn to_style_flags(style: MarkupStyle) -> StyleFlags {
    style.bits()
}

/// Длина открывающего маркера по стилю.
pub fn marker_open_len(style: MarkupStyle) -> usize {
    for m in zoll::ast::MARKERS {
        if m.style == style {
            return m.open.len();
        }
    }
    0
}
