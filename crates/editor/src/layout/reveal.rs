//! Состояние раскрытия маркеров разметки в Live-режиме.
//!
//! Хранит пары `(строка, raw_start_сегмента)` для сегментов,
//! чьи маркеры принудительно показаны (через Ctrl+Space).

use std::collections::HashSet;

/// Трекер раскрытых маркеров.
#[derive(Debug, Clone, Default)]
pub struct RevealState {
    /// Множество `(line, segment_raw_start)` — какие сегменты раскрыты.
    revealed: HashSet<(usize, usize)>,
}

impl RevealState {
    pub fn is_revealed(&self, line: usize, segment_raw_start: usize) -> bool {
        self.revealed.contains(&(line, segment_raw_start))
    }

    /// Переключить состояние раскрытия для сегмента.
    /// Возвращает `true`, если теперь раскрыт.
    pub fn toggle(&mut self, line: usize, segment_raw_start: usize) -> bool {
        if !self.revealed.remove(&(line, segment_raw_start)) {
            self.revealed.insert((line, segment_raw_start));
            true
        } else {
            false
        }
    }

    /// Сбросить все раскрытия (при смене режима).
    pub fn clear(&mut self) {
        self.revealed.clear();
    }
}
