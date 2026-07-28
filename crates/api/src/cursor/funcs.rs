use editor::document::Document;

pub fn move_left(doc: &mut Document) {
    doc.move_left();
}

pub fn move_right(doc: &mut Document) {
    doc.move_right();
}

pub fn move_home(doc: &mut Document) {
    doc.move_home();
}

pub fn move_end(doc: &mut Document) {
    doc.move_end();
}

pub fn move_up(doc: &mut Document) {
    doc.move_up();
}

pub fn move_down(doc: &mut Document) {
    doc.move_down();
}

pub fn move_word_left(doc: &mut Document) {
    doc.move_word_left();
}

pub fn move_word_right(doc: &mut Document) {
    doc.move_word_right();
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

// ─── Selection-extended movement ──────────────────────

pub fn move_left_select(doc: &mut Document) {
    doc.move_left_select();
}

pub fn move_right_select(doc: &mut Document) {
    doc.move_right_select();
}

pub fn move_home_select(doc: &mut Document) {
    doc.move_home_select();
}

pub fn move_end_select(doc: &mut Document) {
    doc.move_end_select();
}

pub fn move_up_select(doc: &mut Document) {
    doc.move_up_select();
}

pub fn move_down_select(doc: &mut Document) {
    doc.move_down_select();
}

pub fn move_word_left_select(doc: &mut Document) {
    doc.move_word_left_select();
}

pub fn move_word_right_select(doc: &mut Document) {
    doc.move_word_right_select();
}

// ─── Document-level movement ─────────────────────────

pub fn move_to_document_start(doc: &mut Document) {
    doc.move_to_document_start();
}

pub fn move_to_document_end(doc: &mut Document) {
    doc.move_to_document_end();
}

pub fn page_up(doc: &mut Document, lines: usize) {
    doc.page_up(lines);
}

pub fn page_down(doc: &mut Document, lines: usize) {
    doc.page_down(lines);
}
