use editor::document::Document;
use std::path::Path;

pub fn file_save(doc: &Document, path: impl AsRef<Path>) -> std::io::Result<()> {
    std::fs::write(path, doc.content.as_bytes())
}

pub fn file_load(path: impl AsRef<Path>) -> std::io::Result<Document> {
    let content = std::fs::read_to_string(path)?;
    Ok(Document::new(&content))
}

pub fn file_save_str(text: &str, path: impl AsRef<Path>) -> std::io::Result<()> {
    std::fs::write(path, text.as_bytes())
}

pub fn file_load_str(path: impl AsRef<Path>) -> std::io::Result<String> {
    std::fs::read_to_string(path)
}
