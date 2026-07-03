use gtk4::cairo::Context;

pub fn rounded_rect(cr: &Context, x: f64, y: f64, w: f64, h: f64, r: f64) {
    use std::f64::consts::{FRAC_PI_2, PI};
    cr.new_path();
    cr.arc(x + w - r, y + r, r, -FRAC_PI_2, 0.0);
    cr.arc(x + w - r, y + h - r, r, 0.0, FRAC_PI_2);
    cr.arc(x + r, y + h - r, r, FRAC_PI_2, PI);
    cr.arc(x + r, y + r, r, PI, PI * 1.5);
    cr.close_path();
}

pub fn hex_to_rgba(hex: &str) -> Option<(f64, f64, f64, f64)> {
    let h = hex.trim_start_matches('#');
    let parse = |s: &str| u8::from_str_radix(s, 16).ok().map(|n| n as f64 / 255.0);
    match h.len() {
        6 => Some((parse(&h[0..2])?, parse(&h[2..4])?, parse(&h[4..6])?, 1.0)),
        8 => Some((parse(&h[0..2])?, parse(&h[2..4])?, parse(&h[4..6])?, parse(&h[6..8])?)),
        _ => None,
    }
}