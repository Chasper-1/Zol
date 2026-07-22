//! Управление шрифтами: системные + встроенные.

mod global;
mod list;
mod system;

pub use global::init;
pub use list::{list_families, reload_system_fonts};
pub use system::{with_font_and_cache, with_font_system, with_swash_cache};

#[cfg(test)]
mod tests;
