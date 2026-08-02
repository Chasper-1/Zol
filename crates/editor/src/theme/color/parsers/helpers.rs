// Вспомогательные функции для парсеров цветов.

// Разобрать строку вида `func(a, b, c)` или `func(a, b, c, alpha)`.
pub fn split_func_args<'a>(s: &'a str, name: &str) -> Result<Vec<&'a str>, String> {
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

// Парсит опциональный альфа-канал из 4-го аргумента.
pub fn parse_alpha(parts: &[&str], name: &str) -> Result<f32, String> {
    if parts.len() == 4 {
        let a: f32 = parts[3]
            .parse()
            .map_err(|e| format!("{}: альфа не число: {}", name, e))?;
        Ok(a.clamp(0.0, 1.0))
    } else {
        Ok(1.0)
    }
}

pub fn parse_0_255(s: &str) -> Result<u8, String> {
    let v: f32 = s
        .parse()
        .map_err(|e| format!("ожидалось число 0–255: {}", e))?;
    if v < 0.0 || v > 255.0 {
        return Err(format!("значение {} вне диапазона 0–255", v));
    }
    Ok(v as u8)
}

pub fn parse_0_100(s: &str) -> Result<f32, String> {
    let cleaned = s.trim_end_matches('%').trim();
    let v: f32 = cleaned
        .parse()
        .map_err(|e| format!("ожидалось число 0–100, получено «{}»: {}", s, e))?;
    if v < 0.0 || v > 100.0 {
        return Err(format!("значение {} вне диапазона 0–100", v));
    }
    Ok(v / 100.0)
}
