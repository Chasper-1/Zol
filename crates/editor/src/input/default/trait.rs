use std::fmt;

// Единый центр управления: все move/delete вычисления.
//
// Каждый метод получает только то состояние, которое нужно для расчёта,
// и возвращает новую позицию / границы удаления.
// Document применяет результат к реальному состоянию (cursor, incremental).
pub trait InputModel: Send + Sync + fmt::Debug {
    // ─── Move ──────────────────────────────────────────────

    // На один grapheme-кластер влево.
    fn move_left(&self, content: &str, raw: usize) -> usize;

    // На один grapheme-кластер вправо.
    fn move_right(&self, content: &str, raw: usize) -> usize;

    // В начало строки.
    fn move_home(&self, line_starts: &[usize], line: usize) -> usize;

    // В конец строки.
    fn move_end(&self, content: &str, line_starts: &[usize], line: usize) -> usize;

    // На строку вверх (сохраняя col_visual).
    // Возвращает (new_raw, new_col_visual).
    fn move_up(
        &self,
        content: &str,
        line_starts: &[usize],
        raw: usize,
        line: usize,
        col_visual: f64,
    ) -> (usize, f64);

    // На строку вниз (сохраняя col_visual).
    // Возвращает (new_raw, new_col_visual).
    fn move_down(
        &self,
        content: &str,
        line_starts: &[usize],
        raw: usize,
        line: usize,
        col_visual: f64,
    ) -> (usize, f64);

    // На слово влево.
    fn word_left(&self, content: &str, raw: usize) -> usize;

    // На слово вправо.
    fn word_right(&self, content: &str, raw: usize) -> usize;

    // ─── Delete (возвращают границу удаления) ────────────────

    // Граница удаления символа перед курсором (Backspace).
    fn delete_char_before(&self, content: &str, raw: usize) -> usize;

    // Граница удаления символа после курсора (Delete).
    fn delete_char_after(&self, content: &str, raw: usize) -> usize;

    // Граница удаления слова перед курсором (Ctrl+Backspace).
    fn delete_word_before(&self, content: &str, raw: usize) -> usize;

    // Граница удаления слова после курсора (Ctrl+Delete).
    fn delete_word_after(&self, content: &str, raw: usize) -> usize;

    // Границы удаления строки (Ctrl+Shift+Backspace).
    // Возвращает Option<(start, end)>.
    fn delete_line(
        &self,
        content: &str,
        line_starts: &[usize],
        line: usize,
    ) -> Option<(usize, usize)>;

    // Граница удаления от курсора до конца строки (Ctrl+Shift+Delete).
    fn delete_to_line_end(
        &self,
        content: &str,
        line_starts: &[usize],
        raw: usize,
        line: usize,
    ) -> Option<usize>;

    // ─── Document-level movement ────────────────────────

    fn move_to_document_start(&self) -> usize {
        0
    }

    fn move_to_document_end(&self, content: &str) -> usize {
        content.len()
    }

    fn move_page_up(
        &self,
        content: &str,
        line_starts: &[usize],
        raw: usize,
        line: usize,
        col_visual: f64,
        lines: usize,
    ) -> (usize, f64) {
        let mut cur_raw = raw;
        let mut cur_col = col_visual;
        let mut current_line = line;
        for _ in 0..lines {
            if current_line == 0 {
                break;
            }
            let result = self.move_up(content, line_starts, cur_raw, current_line, cur_col);
            cur_raw = result.0;
            cur_col = result.1;
            current_line = current_line.saturating_sub(1);
        }
        (cur_raw, cur_col)
    }

    fn move_page_down(
        &self,
        content: &str,
        line_starts: &[usize],
        raw: usize,
        line: usize,
        col_visual: f64,
        lines: usize,
    ) -> (usize, f64) {
        let mut cur_raw = raw;
        let mut cur_col = col_visual;
        let total_lines = line_starts.len();
        let mut current_line = line;
        for _ in 0..lines {
            if current_line + 1 >= total_lines {
                break;
            }
            let result = self.move_down(content, line_starts, cur_raw, current_line, cur_col);
            cur_raw = result.0;
            cur_col = result.1;
            current_line = current_line
                .saturating_add(1)
                .min(total_lines.saturating_sub(1));
        }
        (cur_raw, cur_col)
    }
}
