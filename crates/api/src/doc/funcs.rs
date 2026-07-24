use editor::document::Document;
use editor::utils;

pub fn doc_create(text: &str) -> Document {
    Document::new(text)
}

pub fn doc_text(doc: &Document) -> &str {
    &doc.incremental.source
}

pub fn doc_line(doc: &Document, idx: usize) -> Option<&str> {
    utils::line_text(&doc.incremental.source, idx)
}

pub fn doc_line_count(doc: &Document) -> usize {
    utils::count_lines(&doc.incremental.source)
}

pub fn doc_len(doc: &Document) -> usize {
    doc.incremental.source.len()
}

pub fn doc_is_empty(doc: &Document) -> bool {
    doc.incremental.source.is_empty()
}

pub fn doc_set_text(doc: &mut Document, text: &str) {
    doc.incremental = zoll::incremental::IncrementalDoc::new(text);
    doc.cursor = editor::cursor::Cursor::new();
    doc.dirty = true;
}

pub fn doc_is_dirty(doc: &Document) -> bool {
    doc.dirty
}

pub fn doc_set_dirty(doc: &mut Document, dirty: bool) {
    doc.dirty = dirty;
}

pub fn doc_make_dirty(doc: &mut Document) {
    doc.dirty = true;
}
