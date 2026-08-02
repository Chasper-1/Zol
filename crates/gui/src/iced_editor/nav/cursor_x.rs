use editor::render::ShapedDocument;

// X-позиция курсора на строке `line` по глифам буфера.
//
// Учитывает компенсацию: если на строке есть скрытые маркеры,
// буферный `byte_in_line` транслируется в shaped-оффсет.
pub fn cursor_x_on_line(shaped: &ShapedDocument, line: usize, byte_in_line: usize) -> f32 {
    // Компенсация: буферный оффсет → shaped-оффсет
    let shaped_byte = shaped
        .compensation
        .get(line)
        .map(|c| c.buffer_to_shaped(byte_in_line))
        .unwrap_or(byte_in_line);

    for run in shaped.buffer.layout_runs() {
        if run.line_i != line {
            continue;
        }
        for glyph in run.glyphs.iter() {
            if glyph.start >= shaped_byte {
                return glyph.x;
            }
        }
        return run.glyphs.last().map(|g| g.x + g.w).unwrap_or(0.0);
    }
    0.0
}
