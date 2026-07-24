//! AST для zoll-разметки.

mod doc;
mod markers;
mod node;
mod span;
mod style;

pub use doc::MarkupDoc;
pub use markers::MarkerDef;
pub use node::MarkupNode;
pub use span::Span;
pub use style::MarkupStyle;

/// Все маркеры zoll.
pub use markers::MARKERS;
