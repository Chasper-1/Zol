use cosmic_text::{LayoutGlyph, SwashContent};
use gtk4::cairo::{Context, Format, ImageSurface};
use crate::editor::EditorState;

pub fn draw_text(
    cr: &Context,
    runs: &[(f64, Vec<LayoutGlyph>)],
    st: &mut EditorState, // Принимаем весь стейт
    padding: f64,
    mx: f64,
    my: f64,
    scale: f64,
    color: (f64, f64, f64, f64),
) {
    cr.set_source_rgba(color.0, color.1, color.2, color.3);

    for (line_y, glyphs) in runs {
        for glyph in glyphs {
            let physical = glyph.physical(
                (
                    (padding + mx) as f32 * scale as f32,
                    (padding + my + line_y) as f32 * scale as f32,
                ),
                scale as f32,
            );

            // Используем st.font_system и st.swash_cache из принятого стейта
            if let Some(image) = st
                .swash_cache
                .get_image(&mut st.font_system, physical.cache_key)
            {
                if image.placement.width == 0
                    || image.placement.height == 0
                    || image.content != SwashContent::Mask
                {
                    continue;
                }

                let width = image.placement.width as usize;
                let height = image.placement.height as usize;
                let mut surface =
                    ImageSurface::create(Format::A8, width as i32, height as i32).unwrap();

                {
                    // ВАЖНО: Сначала stride, потом data
                    let stride = surface.stride() as usize;
                    let mut data = surface.data().unwrap();

                    for y in 0..height {
                        let src_row = &image.data[y * width..(y + 1) * width];
                        let dst_row = &mut data[y * stride..y * stride + width];
                        dst_row.copy_from_slice(src_row);
                    }
                }

                cr.mask_surface(
                    &surface,
                    (physical.x as f64 / scale) + image.placement.left as f64,
                    (physical.y as f64 / scale) + image.placement.top as f64,
                )
                .unwrap();
            }
        }
    }
}
