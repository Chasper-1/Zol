// ═══════════════════════════════════════════════════════════════════════
// ⚠  ВАЖНО: НИКОГДА НЕ ДЕЛАЙ СЛЕДУЮЩЕГО:
// ⚠
// ⚠  - Не передавай all_runs по ссылке в shape_document — это вынудит
// ⚠    клонирование line_runs внутри.
// ⚠  - Не вызывай render::build() на каждый кадр — только когда
// ⚠    doc.dirty == true.
// ⚠  - Не создавай новый ShapedDocument каждый раз — переиспользуй
// ⚠    существующий (как сейчас).
// ⚠
// ⚠  Причина: render::build() — тяжёлая операция, вызывать её без
// ⚠  необходимости = убийство производительности и памяти.
// ═══════════════════════════════════════════════════════════════════════

use super::shape::shape_document;
use super::shaped_doc::ShapedDocument;
use crate::Viewport;
use crate::cache::DocumentCache;
use crate::layout;
use crate::layout::compensation::LineCompensation;
use crate::layout::reveal::RevealCtx;
use crate::state::EditMode;
use crate::theme::EditorTheme;

/// Собрать документ: вычислить TextRun'ы → сшейпить → готово к отрисовке.
#[allow(clippy::too_many_arguments)]
pub fn build(
    doc: &mut ShapedDocument,
    content: &str,
    cache: &DocumentCache,
    mode: EditMode,
    theme: &EditorTheme,
    base_size: f32,
    heading_size: f32,
    scroll_y: f32,
    viewport_height: Option<f32>,
    viewport: Option<&Viewport>,
    reveal: Option<&RevealCtx>,
) {
    crate::font::init();

    let font_family = theme.text.font_family.as_deref().unwrap_or("sans-serif");
    let default_color = theme.text.color;

    let lines: Vec<&str> = content.split('\n').collect();
    let mut all_runs: Vec<Vec<layout::TextRun>> = Vec::with_capacity(lines.len());
    let mut all_comp: Vec<LineCompensation> = Vec::with_capacity(lines.len());

    // Диапазон строк, которые нужно полноценно обрабатывать.
    let visible_range = viewport.map(|vp| vp.first_line..=vp.last_line);

    let mut line_start = 0usize;
    for (i, line) in lines.iter().enumerate() {
        let is_visible = visible_range.as_ref().is_none_or(|r| r.contains(&i));

        if line.is_empty() {
            all_runs.push(vec![layout::TextRun::new(
                "\u{200B}",
                0,
                crate::theme::color::Rgba::new(0.5, 0.5, 0.5),
                base_size,
            )]);
            all_comp.push(LineCompensation::identity(0));
        } else if is_visible {
            let show_markers = matches!(mode, EditMode::Source);
            let result = layout::compute::compute_line_runs_with_meta(
                line,
                line_start,
                i,
                cache.lines.get(i),
                base_size,
                heading_size,
                show_markers,
                reveal,
                theme,
            );
            all_comp.push(LineCompensation::new(result.hidden_ranges, line.len()));
            all_runs.push(result.runs);
        } else {
            // Строка вне viewport — только базовый цвет, без семантики
            all_runs.push(vec![layout::TextRun::new(
                line,
                0,
                default_color,
                base_size,
            )]);
            all_comp.push(LineCompensation::identity(line.len()));
        }

        line_start += line.len() + 1;
        if line_start > content.len() {
            line_start = content.len();
        }
    }

    // ⚠ with_font_system() держит Mutex на FontSystem.
    // shape_document НЕ вызывает after_shape() — это был бы deadlock!
    crate::font::with_font_system(|fs| {
        *doc = shape_document(
            all_runs, // было &all_runs, теперь move
            all_comp,
            fs,
            base_size,
            font_family,
            scroll_y,
            viewport_height,
        );
    });

    // ⚠ Очистка SwashCache ПОСЛЕ освобождения мьютекса.
    crate::font::after_shape();
}
