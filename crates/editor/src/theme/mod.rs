pub mod color;
pub mod handle;
pub mod parser;
pub mod registry;
pub mod theme;

// Реэкспорты для обратной совместимости.
// Старые `use crate::theme::{EditorTheme, Rgba, ...}` продолжают работать.
pub use color::Rgba;
#[allow(dead_code)]
pub use handle::{HandleValue, ThemeSystem};
pub use parser::parse_theme;
pub use theme::EditorTheme;
#[allow(dead_code)]
pub use theme::TextTheme;
