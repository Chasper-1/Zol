use std::fmt;

/// Единый центр управления: все move/delete вычисления.
///
/// Каждый метод получает только то состояние, которое нужно для расчёта,
/// и возвращает новую позицию / границы удаления.
/// Document применяет результат к реальному состоянию (cursor, incremental).
pub trait InputModel: Send + Sync + fmt::Debug {
    // ─── Move ──────────────────────────────────────────────

    /// На один grapheme-кластер влево.
    fn move_left(&self, content: &str, raw: usize) -> usize;

    /// На один grapheme-кластер вправо.
    fn move_right(&self, content: &str, raw: usize) -> usize;

    /// В начало строки.
    fn move_home(&self, line_starts: &[usize], line: usize) -> usize;

    /// В конец строки.
    fn move_end(&self, content: &str, line_starts: &[usize], line: usize) -> usize;

    /// На строку вверх (сохраняя col_visual).
    /// Возвращает (new_raw, new_col_visual).
    fn move_up(
        &self,
        content: &str,
        line_starts: &[usize],
        raw: usize,
        line: usize,
        col_visual: f64,
    ) -> (usize, f64);

    /// На строку вниз (сохраняя col_visual).
    /// Возвращает (new_raw, new_col_visual).
    fn move_down(
        &self,
        content: &str,
        line_starts: &[usize],
        raw: usize,
        line: usize,
        col_visual: f64,
    ) -> (usize, f64);

    /// На слово влево.
    fn word_left(&self, content: &str, raw: usize) -> usize;

    /// На слово вправо.
    fn word_right(&self, content: &str, raw: usize) -> usize;

    // ─── Delete (возвращают границу удаления) ────────────────

    /// Граница удаления символа перед курсором (Backspace).
    fn delete_char_before(&self, content: &str, raw: usize) -> usize;

    /// Граница удаления символа после курсора (Delete).
    fn delete_char_after(&self, content: &str, raw: usize) -> usize;

    /// Граница удаления слова перед курсором (Ctrl+Backspace).
    fn delete_word_before(&self, content: &str, raw: usize) -> usize;

    /// Граница удаления слова после курсора (Ctrl+Delete).
    fn delete_word_after(&self, content: &str, raw: usize) -> usize;

    /// Границы удаления строки (Ctrl+Shift+Backspace).
    /// Возвращает Option<(start, end)>.
    fn delete_line(
        &self,
        content: &str,
        line_starts: &[usize],
        line: usize,
    ) -> Option<(usize, usize)>;

    /// Граница удаления от курсора до конца строки (Ctrl+Shift+Delete).
    fn delete_to_line_end(
        &self,
        content: &str,
        line_starts: &[usize],
        raw: usize,
        line: usize,
    ) -> Option<usize>;
}
