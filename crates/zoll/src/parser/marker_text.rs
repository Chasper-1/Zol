use crate::ast::MARKERS;
use crate::ast::MarkupStyle;

/// Возвращает текст маркера для стиля (используется при непарных закрытиях).
pub fn marker_text_for_close(style: MarkupStyle) -> String {
    for m in MARKERS {
        if m.style == style {
            return m.close.to_string();
        }
    }
    String::new()
}
