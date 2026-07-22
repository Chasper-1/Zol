use super::rgba::Rgba;

/// Конвертирует Rgba в кортеж u8.
pub fn rgba_to_u8(c: &Rgba) -> (u8, u8, u8, u8) {
    (
        (c.r * 255.0) as u8,
        (c.g * 255.0) as u8,
        (c.b * 255.0) as u8,
        (c.a * 255.0) as u8,
    )
}
