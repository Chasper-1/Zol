// ═══════════════════════════════════════════════════════════════════════
// ⚠  ВАЖНО: НИКОГДА НЕ ДЕЛАЙ СЛЕДУЮЩЕГО:
// ⚠
// ⚠  - НЕ клонируй line_runs (не вызывай .clone() или .to_vec()).
// ⚠    Это создаёт полную копию ВСЕХ TextRun'ов со строками.
// ⚠
// ⚠  - НЕ создавай несколько ShapedDocument для одного документа —
// ⚠    каждый содержит buffer + line_runs, что удваивает память.
// ⚠
// ⚠  - НЕ храни ShapedDocument в Arc/Rc если не требуется разделение —
// ⚠    это маскирует реальное потребление памяти.
// ⚠
// ⚠  Причина: line_runs.to_vec() на 100KB+ документе = 1MB+ копий
// ⚠  строк при каждом reshape, что убивает кэш CPU и память.
// ═══════════════════════════════════════════════════════════════════════

use cosmic_text::Buffer;

use crate::layout::LineCompensation;
use crate::layout::TextRun;

// Сформованный документ — обёртка над cosmic-text `Buffer`.
#[derive(Debug)]
pub struct ShapedDocument {
    pub buffer: Buffer,
    pub line_runs: Vec<Vec<TextRun>>,
    // Компенсация смещения для каждой строки (буфер → shaped).
    pub compensation: Vec<LineCompensation>,
}

impl ShapedDocument {
    pub fn new(buffer: Buffer, line_runs: Vec<Vec<TextRun>>) -> Self {
        Self {
            buffer,
            line_runs,
            compensation: vec![],
        }
    }

    pub fn total_height(&self) -> f32 {
        self.buffer
            .layout_runs()
            .last()
            .map(|run| run.line_y + run.line_height)
            .unwrap_or(0.0)
    }

    pub fn line_count(&self) -> usize {
        self.buffer.lines.len()
    }

    pub fn line_height(&self, i: usize) -> f32 {
        self.buffer
            .layout_runs()
            .nth(i)
            .map(|run| run.line_height)
            .unwrap_or(0.0)
    }
}
