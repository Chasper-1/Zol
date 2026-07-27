pub(crate) mod data;
pub mod edit_doc;
pub mod mode;

pub use data::EditorInner;

#[cfg(test)]
mod tests;
