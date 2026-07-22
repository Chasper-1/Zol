use super::detect::detect_format;
use super::parse::ColorFormat;

/// Проверяет, что все цвета в списке имеют один и тот же формат.
pub fn enforce_consistency(colors: &[&str]) -> Result<ColorFormat, String> {
    let first = colors.first().ok_or("список цветов пуст")?;
    let fmt = detect_format(first)
        .ok_or_else(|| format!("неизвестный формат цвета: «{}»", first))?;

    for (i, c) in colors.iter().enumerate().skip(1) {
        let other = detect_format(c)
            .ok_or_else(|| format!("строка {}: неизвестный формат цвета «{}»", i + 1, c))?;
        if other != fmt {
            let fmt_name = |f: ColorFormat| -> &'static str {
                match f {
                    ColorFormat::Hex => "hex",
                    ColorFormat::Rgb => "rgb/rgba",
                    ColorFormat::Hsl => "hsl/hsla",
                    ColorFormat::Oklch => "oklch",
                    ColorFormat::Named => "named",
                }
            };
            return Err(format!(
                "строка {}: формат «{}» не совпадает с «{}» (строка 1)",
                i + 1,
                fmt_name(other),
                fmt_name(fmt),
            ));
        }
    }
    Ok(fmt)
}
