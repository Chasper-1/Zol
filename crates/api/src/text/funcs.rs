use editor::document::Document;

pub fn insert_text(doc: &mut Document, text: &str) {
    doc.insert_text(text);
}

pub fn delete_before(doc: &mut Document) {
    doc.delete_before_cursor();
}

pub fn delete_after(doc: &mut Document) {
    doc.delete_after_cursor();
}

pub fn insert_newline(doc: &mut Document) {
    doc.insert_newline();
}

pub fn insert_at(doc: &mut Document, byte: usize, text: &str) {
    doc.incremental.edit(byte, byte, text);
    doc.dirty = true;
}

pub fn delete_range(doc: &mut Document, start: usize, end: usize) {
    let src: &str = &doc.incremental.source;
    if start >= end || start >= src.len() {
        return;
    }
    let end = end.min(src.len());
    // src borrow ends here (NLL)
    doc.incremental.edit(start, end, "");
    doc.dirty = true;
}

pub fn delete_word_before(doc: &mut Document) {
    doc.delete_word_before();
}

pub fn delete_word_after(doc: &mut Document) {
    doc.delete_word_after();
}

pub fn delete_line(doc: &mut Document) {
    doc.delete_line();
}

pub fn delete_to_line_end(doc: &mut Document) {
    doc.delete_to_line_end();
}
