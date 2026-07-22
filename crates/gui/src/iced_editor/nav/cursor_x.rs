use editor::render::ShapedDocument;

/// X-позиция курсора на строке `line` по глифам буфера.
pub fn cursor_x_on_line(shaped: &ShapedDocument, line: usize, byte_in_line: usize) -> f32 {
    for run in shaped.buffer.layout_runs() {
        if run.line_i != line {
            continue;
        }
        for glyph in run.glyphs.iter() {
            if glyph.start >= byte_in_line {
                return glyph.x;
            }
        }
        return run
            .glyphs
            .last()
            .map(|g| g.x + g.w)
            .unwrap_or(0.0);
    }
    0.0
}
