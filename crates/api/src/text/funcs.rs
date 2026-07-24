use editor::document::Document;

pub fn insert_at_cursor(doc: &mut Document, text: &str) {
    doc.insert_at_cursor(text);
}

pub fn delete_before(doc: &mut Document) {
    doc.delete_before_cursor();
}

pub fn delete_after(doc: &mut Document) {
    doc.delete_after_cursor();
}

pub fn newline(doc: &mut Document) {
    doc.newline_at_cursor();
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
