use crate::editor::render::ShapedDocument;

/// Y-позиция (line_top) i-й строки.
pub fn layout_line_y(shaped: &ShapedDocument, line: usize) -> f32 {
    for run in shaped.buffer.layout_runs() {
        if run.line_i == line {
            return run.line_top;
        }
    }
    0.0
}
