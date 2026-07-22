use editor::document::Document;
use editor::cursor;

pub fn insert_at_cursor(doc: &mut Document, text: &str) {
    let raw = doc.cursor.raw();
    doc.content.insert_str(raw, text);
    doc.cursor.set_raw(&doc.content, raw + text.len());
    doc.dirty = true;
}

pub fn delete_before(doc: &mut Document) {
    let raw = doc.cursor.raw();
    if raw == 0 || doc.content.is_empty() {
        return;
    }
    let prev = cursor::prev_grapheme_boundary(&doc.content, raw).unwrap_or(0);
    doc.content.drain(prev..raw);
    doc.cursor.set_raw(&doc.content, prev);
    doc.dirty = true;
}

pub fn delete_after(doc: &mut Document) {
    let raw = doc.cursor.raw();
    if raw >= doc.content.len() || doc.content.is_empty() {
        return;
    }
    let next = cursor::next_grapheme_boundary(&doc.content, raw).unwrap_or(doc.content.len());
    doc.content.drain(raw..next);
    doc.cursor.set_raw(&doc.content, raw);
    doc.dirty = true;
}

pub fn newline(doc: &mut Document) {
    let raw = doc.cursor.raw();
    doc.content.insert(raw, '\n');
    doc.cursor.set_raw(&doc.content, raw + 1);
    doc.cursor.reset_col_visual();
    doc.dirty = true;
}

pub fn insert_at(doc: &mut Document, byte: usize, text: &str) {
    doc.content.insert_str(byte, text);
    doc.dirty = true;
}

pub fn delete_range(doc: &mut Document, start: usize, end: usize) {
    if start >= end || start >= doc.content.len() {
        return;
    }
    let end = end.min(doc.content.len());
    doc.content.drain(start..end);
    doc.dirty = true;
}
