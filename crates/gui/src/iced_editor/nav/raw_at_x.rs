use editor::render::ShapedDocument;

/// Ближайший к `x` content-offset на строке `line`.
///
/// Учитывает компенсацию: shaped-оффсет глифа транслируется
/// в буферный оффсет для корректной позиции при скрытых маркерах.
pub fn raw_at_x_on_line(
    shaped: &ShapedDocument,
    line: usize,
    x: f32,
    line_start: usize,
    line_end: usize,
) -> usize {
    if line_end <= line_start {
        return line_start;
    }

    let comp = shaped.compensation.get(line);

    let mut best: Option<(f32, usize)> = None;
    for run in shaped.buffer.layout_runs() {
        if run.line_i != line {
            continue;
        }
        for glyph in run.glyphs.iter() {
            let dist = (glyph.x - x).abs();
            // shaped-оффсет глифа → буферный оффсет
            let buffer_offset = comp
                .map(|c| c.shaped_to_buffer(glyph.start))
                .unwrap_or(glyph.start);
            let cand = line_start + buffer_offset;
            if best.map_or(true, |(bd, _)| dist < bd) {
                best = Some((dist, cand));
            }
        }
        if let Some(last) = run.glyphs.last() {
            let end_x = last.x + last.w;
            let dist = (end_x - x).abs();
            // Конец строки — всегда line_end (буферная позиция)
            if best.map_or(true, |(bd, _)| dist < bd) {
                best = Some((dist, line_end));
            }
        }
        break;
    }
    best.map_or(line_start, |(_, c)| c)
}
