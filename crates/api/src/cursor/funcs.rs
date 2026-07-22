use editor::document::Document;

pub fn move_left(doc: &mut Document) {
    doc.cursor.move_left(&doc.content);
}

pub fn move_right(doc: &mut Document) {
    doc.cursor.move_right(&doc.content);
}

pub fn move_home(doc: &mut Document) {
    doc.cursor.move_home(&doc.content);
}

pub fn move_end(doc: &mut Document) {
    doc.cursor.move_end(&doc.content);
}

pub fn move_up(doc: &mut Document) {
    doc.cursor.move_up(&doc.content);
}

pub fn move_down(doc: &mut Document) {
    doc.cursor.move_down(&doc.content);
}

pub fn move_word_left(doc: &mut Document) {
    doc.cursor.move_word_left(&doc.content);
}

pub fn move_word_right(doc: &mut Document) {
    doc.cursor.move_word_right(&doc.content);
}

pub fn cursor_raw(doc: &Document) -> usize {
    doc.cursor.raw()
}

pub fn cursor_set_raw(doc: &mut Document, byte: usize) {
    doc.cursor.set_raw(&doc.content, byte);
}

pub fn cursor_line(doc: &Document) -> usize {
    doc.cursor.line()
}

pub fn cursor_set_line(doc: &mut Document, line: usize) {
    doc.cursor.set_line(line);
}

pub fn cursor_col(doc: &Document) -> f32 {
    doc.cursor.col_visual()
}

pub fn cursor_set_col(doc: &mut Document, col: f32) {
    doc.cursor.set_col_visual(col);
}

pub fn cursor_reset_col(doc: &mut Document) {
    doc.cursor.reset_col_visual();
}
