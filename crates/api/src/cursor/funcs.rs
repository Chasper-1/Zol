use editor::document::Document;

pub fn move_left(doc: &mut Document) {
    doc.cursor_move_left();
}

pub fn move_right(doc: &mut Document) {
    doc.cursor_move_right();
}

pub fn move_home(doc: &mut Document) {
    doc.cursor_move_home();
}

pub fn move_end(doc: &mut Document) {
    doc.cursor_move_end();
}

pub fn move_up(doc: &mut Document) {
    doc.cursor_move_up();
}

pub fn move_down(doc: &mut Document) {
    doc.cursor_move_down();
}

pub fn move_word_left(doc: &mut Document) {
    doc.cursor_move_word_left();
}

pub fn move_word_right(doc: &mut Document) {
    doc.cursor_move_word_right();
}

pub fn cursor_raw(doc: &Document) -> usize {
    doc.cursor.raw()
}

pub fn cursor_set_raw(doc: &mut Document, byte: usize) {
    doc.set_cursor_raw(byte);
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
