pub mod edit_doc;
pub mod mode;
pub(crate) mod data;

pub use data::EditorInner;

#[cfg(test)]
mod tests;
