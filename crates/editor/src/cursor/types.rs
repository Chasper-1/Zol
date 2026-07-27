use std::time::Instant;

/// Позиция курсора в тексте.
///
/// `raw` всегда указывает на валидную **grapheme**-границу.
/// `line` — кешированный номер строки.
/// `anchor` — фиксированный конец выделения (None = нет выделения).
#[derive(Debug)]
pub struct Cursor {
    /// Байтовый оффсет от начала текста (активный конец выделения).
    pub(crate) raw: usize,
    /// Строка, в которой находится `raw`.
    pub(crate) line: usize,
    /// Горизонтальная позиция для move_up/down (в пикселях).
    pub(crate) col_visual: f32,
    /// Время последнего изменения видимости курсора.
    pub(crate) last_blink: Instant,
    /// Фиксированный конец выделения; None = нет выделения.
    pub(crate) anchor: Option<usize>,
}

impl Cursor {
    /// Создать курсор в начале текста.
    pub fn new() -> Self {
        Self {
            raw: 0,
            line: 0,
            col_visual: 0.0,
            last_blink: Instant::now(),
            anchor: None,
        }
    }

    // ── Геттеры ──

    pub fn raw(&self) -> usize {
        self.raw
    }
    pub fn line(&self) -> usize {
        self.line
    }
    pub fn col_visual(&self) -> f32 {
        self.col_visual
    }

    /// Установить `line` напрямую (для move_up/down).
    pub fn set_line(&mut self, line: usize) {
        self.line = line;
        self.force_blink();
    }

    pub fn set_col_visual(&mut self, x: f32) {
        self.col_visual = x;
    }
    pub fn reset_col_visual(&mut self) {
        self.col_visual = 0.0;
    }

    // ── Выделение ──

    /// Диапазон выделения `(start, end)`, если выделение активно.
    pub fn selection_range(&self) -> Option<(usize, usize)> {
        self.anchor.map(|a| (a.min(self.raw), a.max(self.raw)))
    }

    /// Есть ли активное выделение.
    pub fn has_selection(&self) -> bool {
        self.anchor.is_some()
    }

    /// Сбросить выделение.
    pub fn clear_selection(&mut self) {
        self.anchor = None;
    }

    /// Установить anchor в текущую позицию (начало shift-расширения).
    pub fn begin_selection(&mut self) {
        if self.anchor.is_none() {
            self.anchor = Some(self.raw);
        }
    }

    // ── Мигание ──

    const BLINK_PERIOD_MS: u128 = 1060;
    const VISIBLE_MS: u128 = 530;

    pub fn should_blink(&self) -> bool {
        self.should_blink_at(Instant::now())
    }

    pub(crate) fn should_blink_at(&self, now: Instant) -> bool {
        let elapsed = now.duration_since(self.last_blink);
        let phase = elapsed.as_millis() % Self::BLINK_PERIOD_MS;
        phase < Self::VISIBLE_MS
    }

    pub fn force_blink(&mut self) {
        self.last_blink = Instant::now();
    }

    #[cfg(test)]
    pub(crate) fn force_blink_at(&mut self, now: Instant) {
        self.last_blink = now;
    }
}
