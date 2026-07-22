//! Чистая раскладка строки: сегменты → [`crate::layout::TextRun`]ы.

mod bounds;
mod line_runs;
mod shared;
mod style;

pub use bounds::cursor_line_bounds;
pub use line_runs::compute_line_runs;

#[cfg(test)]
mod tests;
