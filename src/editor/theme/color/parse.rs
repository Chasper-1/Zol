use super::parsers;
use super::rgba::Rgba;

/// Формат записи цвета.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorFormat {
    Hex,
    Rgb,
    Hsl,
    Oklch,
    Named,
}

/// Парсит строку в цвет.
pub fn parse_color(s: &str) -> Result<Rgba, String> {
    let s = s.trim();
    if s.starts_with('#') {
        parsers::parse_hex(s)
    } else if s.starts_with("rgba(") || s.starts_with("rgb(") {
        parsers::parse_rgb(s)
    } else if s.starts_with("hsla(") || s.starts_with("hsl(") {
        parsers::parse_hsl(s)
    } else if s.starts_with("oklch(") {
        parsers::parse_oklch(s)
    } else {
        parsers::parse_named(s)
    }
}
