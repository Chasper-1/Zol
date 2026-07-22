pub mod bounds;
pub mod count;
pub mod line_bounds;
pub mod line_of_byte;
pub mod line_text;
pub mod safe_slice;

pub use bounds::LineBounds;
pub use count::count_lines;
pub use line_bounds::line_bounds;
pub use line_of_byte::line_of_byte;
pub use line_text::{line_end_byte, line_start_byte, line_text};
pub use safe_slice::safe_slice;

#[cfg(test)]
mod tests;
