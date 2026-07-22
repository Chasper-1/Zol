//! Парсеры цветовых форматов.
//!
//! Все функции доступны только внутри модуля `color` (`pub(super)`).

use super::named::NAMED_COLORS;
use super::rgba::Rgba;

pub(super) fn parse_hex(s: &str) -> Result<Rgba, String> {
    let hex = &s[1..]; // убираем #
    let len = hex.len();

    // Расширяем короткую форму (#RGB → #RRGGBB, #RGBA → #RRGGBBAA)
    let expanded: String = match len {
        3 => hex.chars().flat_map(|c| [c, c]).collect(),
        4 => hex.chars().flat_map(|c| [c, c]).collect(),
        _ => hex.to_string(),
    };

    let expanded_len = expanded.len();
    if expanded_len != 6 && expanded_len != 8 {
        return Err(format!(
            "неверная длина hex-цвета: #{} ({} символов)",
            hex, len
        ));
    }

    let ch = |start: usize| -> Result<u8, String> {
        u8::from_str_radix(&expanded[start..start + 2], 16)
            .map_err(|_| format!("неверный hex: #{}", hex))
    };

    let r = ch(0)?;
    let g = ch(2)?;
    let b = ch(4)?;
    let a = if expanded_len == 8 { ch(6)? } else { 255 };

    Ok(Rgba::from_rgba8(r, g, b, a))
}

pub(super) fn parse_rgb(s: &str) -> Result<Rgba, String> {
    let parts = split_func_args(s, "rgb/rgba")?;
    let r = parse_0_255(parts[0])?;
    let g = parse_0_255(parts[1])?;
    let b = parse_0_255(parts[2])?;
    let a = parse_alpha(&parts, "rgb/rgba")?;
    Ok(Rgba::from_rgb8_a(r, g, b, a))
}

pub(super) fn parse_hsl(s: &str) -> Result<Rgba, String> {
    let parts = split_func_args(s, "hsl/hsla")?;
    let h = parts[0]
        .parse::<f32>()
        .map_err(|e| format!("hsl: оттенок не число: {}", e))?;
    let s = parse_0_100(parts[1])?;
    let l = parse_0_100(parts[2])?;
    let a = parse_alpha(&parts, "hsl/hsla")?;
    Ok(hsl_to_rgb(h, s, l).with_alpha(a))
}

pub(super) fn parse_oklch(s: &str) -> Result<Rgba, String> {
    let inner = s.trim_start_matches("oklch(").trim_end_matches(')').trim();

    let parts: Vec<&str> = inner
        .split(|c| c == ' ' || c == ',')
        .filter(|p| !p.is_empty())
        .collect();
    if parts.len() != 3 {
        return Err(format!(
            "oklch: нужно 3 аргумента (l c h), получено {}",
            parts.len()
        ));
    }

    let l = parse_0_100(parts[0])?; // уже нормировано в 0..1
    let c = parts[1]
        .parse::<f32>()
        .map_err(|e| format!("oklch: насыщенность не число: {}", e))?
        .max(0.0);
    let h = parts[2]
        .parse::<f32>()
        .map_err(|e| format!("oklch: оттенок не число: {}", e))?;

    Ok(oklch_to_rgb(l, c, h))
}

pub(super) fn parse_named(s: &str) -> Result<Rgba, String> {
    let s = s.trim().to_lowercase();

    if let Ok(idx) = NAMED_COLORS.binary_search_by_key(&s.as_str(), |(name, _)| name) {
        Ok(NAMED_COLORS[idx].1)
    } else {
        Err(format!("неизвестный цвет: «{}»", s))
    }
}

// —————— вспомогательные парсеры ——————

/// Разобрать строку вида `func(a, b, c)` или `func(a, b, c, alpha)`.
fn split_func_args<'a>(s: &'a str, name: &str) -> Result<Vec<&'a str>, String> {
    let inner = s
        .trim_start_matches(|c: char| c.is_alphabetic() || c == '_')
        .trim_start_matches('(')
        .trim_end_matches(')')
        .trim();

    let parts: Vec<&str> = inner.split(',').map(|p| p.trim()).filter(|p| !p.is_empty()).collect();
    if parts.len() < 3 || parts.len() > 4 {
        return Err(format!(
            "{}: нужно 3 или 4 аргумента, получено {}",
            name,
            parts.len()
        ));
    }
    Ok(parts)
}

/// Парсит опциональный альфа-канал из 4-го аргумента.
fn parse_alpha(parts: &[&str], name: &str) -> Result<f32, String> {
    if parts.len() == 4 {
        let a: f32 = parts[3]
            .parse()
            .map_err(|e| format!("{}: альфа не число: {}", name, e))?;
        Ok(a.clamp(0.0, 1.0))
    } else {
        Ok(1.0)
    }
}

fn parse_0_255(s: &str) -> Result<u8, String> {
    let v: f32 = s
        .parse()
        .map_err(|e| format!("ожидалось число 0–255: {}", e))?;
    if v < 0.0 || v > 255.0 {
        return Err(format!("значение {} вне диапазона 0–255", v));
    }
    Ok(v as u8)
}

fn parse_0_100(s: &str) -> Result<f32, String> {
    let cleaned = s.trim_end_matches('%').trim();
    let v: f32 = cleaned
        .parse()
        .map_err(|e| format!("ожидалось число 0–100, получено «{}»: {}", s, e))?;
    if v < 0.0 || v > 100.0 {
        return Err(format!("значение {} вне диапазона 0–100", v));
    }
    Ok(v / 100.0) // нормируем в 0..1
}

// —————— конвертеры цветовых пространств ——————

/// HSL → sRGB по стандартной формуле.
fn hsl_to_rgb(h: f32, s: f32, l: f32) -> Rgba {
    // h — градусы 0..360, s и l — нормированные 0..1
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;

    let (r, g, b) = match (h as i32).rem_euclid(360) {
        0..=59 => (c, x, 0.0),
        60..=119 => (x, c, 0.0),
        120..=179 => (0.0, c, x),
        180..=239 => (0.0, x, c),
        240..=299 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    Rgba::new(r + m, g + m, b + m)
}

/// oklch → sRGB.
///
/// Аппроксимация по OKLCH → OKLab → линейный sRGB → гамма-коррекция.
fn oklch_to_rgb(l: f32, c: f32, h: f32) -> Rgba {
    // oklch → oklab
    let h_rad = h.to_radians();
    let a = c * h_rad.cos();
    let b_val = c * h_rad.sin();

    // oklab → линейный sRGB (матрица 3×3)
    let l_ = l + 0.3963377774 * a + 0.2158037573 * b_val;
    let m_ = l - 0.1055613458 * a - 0.0638541728 * b_val;
    let s_ = l - 0.0894841775 * a - 1.2914855480 * b_val;

    let l3 = l_ * l_ * l_;
    let m3 = m_ * m_ * m_;
    let s3 = s_ * s_ * s_;

    // Преобразование LMS → sRGB
    let r_lin = 4.0767416621 * l3 - 3.3077115913 * m3 + 0.2309699292 * s3;
    let g_lin = -1.2684380046 * l3 + 2.6097574011 * m3 - 0.3413193965 * s3;
    let b_lin = -0.0041960863 * l3 - 0.7034186147 * m3 + 1.7076147010 * s3;

    // Гамма-коррекция (sRGB transfer function)
    fn srgb_gamma(c: f32) -> f32 {
        let c = c.max(0.0).min(1.0);
        if c <= 0.0031308 {
            12.92 * c
        } else {
            1.055 * c.powf(1.0 / 2.4) - 0.055
        }
    }

    Rgba::new(srgb_gamma(r_lin), srgb_gamma(g_lin), srgb_gamma(b_lin))
}
