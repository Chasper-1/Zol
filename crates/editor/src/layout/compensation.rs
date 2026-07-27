//! Компенсация смещения между буферными и shaped-оффсетами.
//!
//! Когда маркеры скрыты (Live Preview), shaped-текст короче исходного
//! буфера. `LineCompensation` транслирует оффсеты между двумя
//! пространствами.
//!
//! # Семантика snapping'а
//!
//! - Позиции внутри скрытых **открывающих** маркеров → snap к началу контента
//! - Позиции внутри скрытых **закрывающих** маркеров → snap к концу контента

/// Предвычисленная компенсация для одной строки.
#[derive(Debug, Clone)]
pub struct LineCompensation {
    /// Байтовые диапазоны в исходной строке, которые скрыты (не рендерятся).
    /// Сортированы, не пересекаются.
    hidden_ranges: Vec<(usize, usize)>,
    /// Общая длина исходной строки в байтах.
    line_len: usize,
}

impl LineCompensation {
    /// Создать identity-компенсацию (ничего не скрыто).
    pub fn identity(line_len: usize) -> Self {
        Self {
            hidden_ranges: vec![],
            line_len,
        }
    }

    /// Создать из скрытых диапазонов.
    pub fn new(hidden_ranges: Vec<(usize, usize)>, line_len: usize) -> Self {
        debug_assert!(
            hidden_ranges.windows(2).all(|w| w[0].1 <= w[1].0),
            "hidden_ranges должны быть отсортированы и не пересекаться"
        );
        Self {
            hidden_ranges,
            line_len,
        }
    }

    /// Общее количество скрытых байт.
    pub fn hidden_len(&self) -> usize {
        self.hidden_ranges.iter().map(|&(s, e)| e - s).sum()
    }

    /// Компенсация identity (ничего не скрыто)?
    pub fn is_identity(&self) -> bool {
        self.hidden_ranges.is_empty()
    }

    /// Преобразовать буферный оффсет в shaped-оффсет.
    ///
    /// Для позиций внутри скрытых маркеров возвращает оффсет
    /// ближайшего видимого контента.
    pub fn buffer_to_shaped(&self, buffer_pos: usize) -> usize {
        let mut hidden_before = 0;
        for &(start, end) in &self.hidden_ranges {
            if buffer_pos <= start {
                break;
            }
            if buffer_pos < end {
                // Внутри скрытого диапазона — snap к границе контента
                return start - hidden_before;
            }
            hidden_before += end - start;
        }
        buffer_pos
            .saturating_sub(hidden_before)
            .min(self.line_len - self.hidden_len())
    }

    /// Преобразовать shaped-оффсет в буферный оффсет.
    ///
    /// Возвращает позицию первого байта контента (пропуская скрытые
    /// открывающие маркеры) или позицию после контента (перед скрытыми
    /// закрывающими маркерами).
    pub fn shaped_to_buffer(&self, shaped_pos: usize) -> usize {
        let visible_len = self.line_len - self.hidden_len();

        if shaped_pos >= visible_len {
            // За пределами видимого контента
            if self
                .hidden_ranges
                .last()
                .map_or(false, |&(_, end)| end == self.line_len)
            {
                // Строка заканчивается скрытыми маркерами → конец контента
                // (первый байт перед закрывающими маркерами)
                self.hidden_ranges.last().expect("guard above ensures .last() is Some").0
            } else {
                // Строка заканчивается видимым текстом → конец строки
                self.line_len
            }
        } else {
            let mut buf = shaped_pos;
            for &(start, end) in &self.hidden_ranges {
                if buf >= start {
                    buf += end - start;
                } else {
                    break;
                }
            }
            buf.min(self.line_len)
        }
    }
}

/// Результат `compute_line_runs_with_meta` — runs + скрытые диапазоны.
pub struct LineRunsResult {
    /// Стилизованные фрагменты строки.
    pub runs: Vec<crate::layout::TextRun>,
    /// Скрытые диапазоны маркеров (для компенсации).
    pub hidden_ranges: Vec<(usize, usize)>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::compute::compute_line_runs_with_meta;

    // ─── хелперы ─────────────────────────────────────────────────

    fn compensation(text: &str, show_markers: bool) -> LineCompensation {
        let result = compute_line_runs_with_meta(
            text,
            0,
            0,
            None,
            14.0,
            22.0,
            show_markers,
            None,
            &crate::theme::EditorTheme::default(),
        );
        LineCompensation::new(result.hidden_ranges, text.len())
    }

    // ─── buffer_to_shaped ─────────────────────────────────────────

    #[test]
    fn identity_plain_text() {
        let c = compensation("hello", true);
        for i in 0..=5 {
            assert_eq!(c.buffer_to_shaped(i), i.min(5));
        }
        assert!(c.is_identity());
    }

    #[test]
    fn identity_empty() {
        let c = compensation("", true);
        assert_eq!(c.buffer_to_shaped(0), 0);
        assert!(c.is_identity());
    }

    #[test]
    fn identity_source_mode() {
        let c = compensation("**bold**", true);
        assert!(c.is_identity());
        assert_eq!(c.buffer_to_shaped(0), 0);
        assert_eq!(c.buffer_to_shaped(2), 2);
        assert_eq!(c.buffer_to_shaped(8), 8);
    }

    #[test]
    fn buf2shap_manual_hidden_ranges() {
        let c = LineCompensation::new(vec![(0, 2), (6, 8)], 8);
        // Внутри открывающих → snap к началу контента
        assert_eq!(c.buffer_to_shaped(0), 0);
        assert_eq!(c.buffer_to_shaped(1), 0);
        // Начало контента
        assert_eq!(c.buffer_to_shaped(2), 0);
        assert_eq!(c.buffer_to_shaped(3), 1);
        assert_eq!(c.buffer_to_shaped(4), 2);
        assert_eq!(c.buffer_to_shaped(5), 3);
        // Конец контента
        assert_eq!(c.buffer_to_shaped(6), 4);
        // Внутри закрывающих → snap к концу контента
        assert_eq!(c.buffer_to_shaped(7), 4);
        assert_eq!(c.buffer_to_shaped(8), 4);
    }

    #[test]
    fn buf2shap_mid_line_markers() {
        // "text **bold** more" — скрыты (6..8) и (14..16)
        let c = LineCompensation::new(vec![(5, 7), (12, 14)], 19);
        assert_eq!(c.buffer_to_shaped(0), 0);
        assert_eq!(c.buffer_to_shaped(4), 4);
        assert_eq!(c.buffer_to_shaped(5), 5); // начало открывающих
        assert_eq!(c.buffer_to_shaped(6), 5); // внутри
        assert_eq!(c.buffer_to_shaped(7), 5); // начало контента "bold"
        assert_eq!(c.buffer_to_shaped(8), 6);
        assert_eq!(c.buffer_to_shaped(11), 9);
        assert_eq!(c.buffer_to_shaped(12), 10); // начало закрывающих
        assert_eq!(c.buffer_to_shaped(13), 10); // внутри
        assert_eq!(c.buffer_to_shaped(14), 10); // " more"
        assert_eq!(c.buffer_to_shaped(18), 14);
    }

    // ─── shaped_to_buffer ─────────────────────────────────────────

    #[test]
    fn shap2buf_identity() {
        let c = compensation("hello", true);
        for i in 0..=5 {
            assert_eq!(c.shaped_to_buffer(i), i.min(5));
        }
    }

    #[test]
    fn shap2buf_manual_hidden_ranges() {
        let c = LineCompensation::new(vec![(0, 2), (6, 8)], 8);
        assert_eq!(c.shaped_to_buffer(0), 2); // 'b'
        assert_eq!(c.shaped_to_buffer(1), 3); // 'o'
        assert_eq!(c.shaped_to_buffer(2), 4); // 'l'
        assert_eq!(c.shaped_to_buffer(3), 5); // 'd'
        assert_eq!(c.shaped_to_buffer(4), 6); // после 'd', перед **
        assert_eq!(c.shaped_to_buffer(5), 6); // за пределами — clamp
    }

    #[test]
    fn shap2buf_mid_line_markers() {
        let c = LineCompensation::new(vec![(5, 7), (12, 14)], 19);
        assert_eq!(c.shaped_to_buffer(0), 0);
        assert_eq!(c.shaped_to_buffer(4), 4);
        assert_eq!(c.shaped_to_buffer(5), 7); // начало "bold"
        assert_eq!(c.shaped_to_buffer(6), 8);
        assert_eq!(c.shaped_to_buffer(9), 11);
        assert_eq!(c.shaped_to_buffer(10), 14); // начало " more"
        assert_eq!(c.shaped_to_buffer(14), 18);
        assert_eq!(c.shaped_to_buffer(15), 19); // за пределами
    }

    #[test]
    fn shap2buf_only_opening_markers() {
        // "**text" — открывающие скрыты, закрывающих нет
        let c = LineCompensation::new(vec![(0, 2)], 6);
        assert_eq!(c.shaped_to_buffer(0), 2);
        assert_eq!(c.shaped_to_buffer(1), 3);
        assert_eq!(c.shaped_to_buffer(2), 4);
        assert_eq!(c.shaped_to_buffer(3), 5);
        assert_eq!(c.shaped_to_buffer(4), 6); // конец строки (строка не заканчивается скрытыми маркерами)
    }

    #[test]
    fn roundtrip_buf_shap_buf() {
        let c = LineCompensation::new(vec![(0, 2), (6, 8)], 8);
        for buf in 0..=8 {
            let shaped = c.buffer_to_shaped(buf);
            let back = c.shaped_to_buffer(shaped);
            // Для позиций внутри контента должен быть точный roundtrip
            if buf >= 2 && buf <= 5 {
                assert_eq!(back, buf, "roundtrip failed for buf={}", buf);
            }
        }
    }
}
