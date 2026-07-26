//! Управление раскрытием маркеров в Live-режиме.

use super::data::EditorInner;
use editor::cursor::marker_skip;
use editor::state::EditMode;

impl EditorInner {
    /// Переключить раскрытие маркеров для сегмента под курсором.
    ///
    /// Находит сегмент в кеше, соответствующий текущей позиции курсора,
    /// и переключает его состояние раскрытия.
    /// Если сегмент не найден — ничего не делает.
    pub fn toggle_reveal_at_cursor(&self) {
        let (cursor_raw, cursor_line) = {
            let doc = self.doc.borrow();
            (doc.cursor.raw(), doc.cursor.line())
        };

        // Ищем сегмент, содержащий курсор
        let cache = self.cache.borrow();
        let line_cache = cache.lines.get(cursor_line);

        let found = line_cache.map(|lc| {
            for seg in &lc.segments {
                if cursor_raw >= seg.raw_start && cursor_raw < seg.raw_end {
                    return Some(seg.raw_start);
                }
            }
            None
        });

        if let Some(Some(raw_start)) = found {
            let mut revealed = self.revealed.borrow_mut();
            revealed.toggle(cursor_line, raw_start);
            self.mark_dirty();
        }
    }

    /// Если режим не Source — сдвинуть курсор с байта маркера на контент.
    pub fn snap_cursor_from_markers(&self) {
        if self.mode.get() == EditMode::Source {
            return;
        }

        let mut doc = self.doc.borrow_mut();
        let cursor_raw = doc.cursor.raw();
        let cursor_line = doc.cursor.line();
        let cache = self.cache.borrow();

        let Some(line_cache) = cache.lines.get(cursor_line) else {
            return;
        };

        // Вычисляем line_start для текущей строки
        let line_starts = &doc.incremental.line_starts;
        let line_start = line_starts.get(cursor_line).copied().unwrap_or(0);

        // Копируем line_starts и source, чтобы избежать borrow конфликта
        let source = doc.incremental.source.clone();
        let ls = line_starts.clone();

        let line_text = ls
            .get(cursor_line)
            .and_then(|&start| {
                let end = ls.get(cursor_line + 1).copied().unwrap_or(source.len());
                if start < source.len() {
                    Some(&source[start..end])
                } else {
                    None
                }
            })
            .unwrap_or("");

        let snapped = marker_skip::snap_forward_line(cursor_raw, line_text, line_cache, line_start);
        if snapped != cursor_raw {
            doc.cursor.set_raw(&source, &ls, snapped);
        }
    }
}
