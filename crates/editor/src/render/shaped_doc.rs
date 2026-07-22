use cosmic_text::Buffer;

use crate::layout::TextRun;

/// Сформованный документ — обёртка над cosmic-text `Buffer`.
#[derive(Debug)]
pub struct ShapedDocument {
    pub buffer: Buffer,
    pub line_runs: Vec<Vec<TextRun>>,
}

impl ShapedDocument {
    pub fn new(buffer: Buffer, line_runs: Vec<Vec<TextRun>>) -> Self {
        Self { buffer, line_runs }
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
