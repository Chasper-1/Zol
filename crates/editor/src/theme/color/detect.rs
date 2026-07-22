use super::named::NAMED_COLORS;
use super::parse::ColorFormat;
use super::parsers;

/// Определяет формат цвета по строке.
pub fn detect_format(s: &str) -> Option<ColorFormat> {
    let s = s.trim();
    if s.starts_with('#') {
        Some(ColorFormat::Hex)
    } else if s.starts_with("rgba(") || s.starts_with("rgb(") {
        Some(ColorFormat::Rgb)
    } else if s.starts_with("hsla(") || s.starts_with("hsl(") {
        Some(ColorFormat::Hsl)
    } else if s.starts_with("oklch(") {
        Some(ColorFormat::Oklch)
    } else if NAMED_COLORS
        .binary_search_by_key(&s, |(name, _)| name)
        .is_ok()
    {
        Some(ColorFormat::Named)
    } else {
        None
    }
}
