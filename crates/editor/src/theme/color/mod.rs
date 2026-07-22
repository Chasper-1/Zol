pub(crate) mod named;
pub mod convert;
pub mod detect;
pub mod enforce;
pub mod parse;
pub mod parsers;
pub mod rgba;

pub use convert::rgba_to_u8;
pub use detect::detect_format;
pub use enforce::enforce_consistency;
pub use parse::{parse_color, ColorFormat};
pub use rgba::Rgba;

#[cfg(test)]
mod tests;
