pub(crate) mod data;
pub mod edit_doc;
pub mod mode;
pub mod reveal;

pub use data::EditorInner;

#[cfg(test)]
mod tests;
