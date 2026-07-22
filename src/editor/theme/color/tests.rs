use super::*;

macro_rules! assert_rgba {
    ($left:expr, $right:expr $(,)?) => {
        let l = $left;
        let r = $right;
        let eps = 0.005;
        assert!(
            (l.r - r.r).abs() < eps
                && (l.g - r.g).abs() < eps
                && (l.b - r.b).abs() < eps
                && (l.a - r.a).abs() < eps,
            "left: {:?}, right: {:?}",
            l,
            r,
        );
    };
}

// —————— хелперы ——————

fn check_parse(input: &str, expected: Rgba) {
    let c = parse_color(input).unwrap();
    assert_rgba!(c, expected);
}

fn check_err(input: &str) {
    assert!(parse_color(input).is_err());
}

fn check_detect(input: &str, expected: Option<ColorFormat>) {
    assert_eq!(detect_format(input), expected);
}

// —————— hex ——————

#[test]
fn parse_hex() {
    check_parse("#F00", Rgba::new(1.0, 0.0, 0.0));
    check_parse("#F00F", Rgba::new(1.0, 0.0, 0.0));
    check_parse("#FF0000", Rgba::new(1.0, 0.0, 0.0));
    check_parse("#FF000080", Rgba::new(1.0, 0.0, 0.0).with_alpha(0.502));
}

#[test]
fn parse_hex_invalid_length() {
    check_err("#FF");
}

// —————— rgb / rgba ——————

#[test]
fn parse_rgb() {
    check_parse("rgb(255, 0, 0)", Rgba::new(1.0, 0.0, 0.0));
}

#[test]
fn parse_rgba() {
    check_parse("rgba(255, 0, 0, 0.5)", Rgba::new(1.0, 0.0, 0.0).with_alpha(0.5));
}

// —————— hsl / hsla ——————

#[test]
fn parse_hsl() {
    check_parse("hsl(0, 100%, 50%)", Rgba::new(1.0, 0.0, 0.0));
}

#[test]
fn parse_hsla() {
    check_parse("hsla(0, 100%, 50%, 0.5)", Rgba::new(1.0, 0.0, 0.0).with_alpha(0.5));
}

// —————— именованные ——————

#[test]
fn parse_named() {
    check_parse("red", Rgba::new(1.0, 0.0, 0.0));
    check_parse("white", Rgba::new(1.0, 1.0, 1.0));
    check_parse("transparent", Rgba::new(0.0, 0.0, 0.0).with_alpha(0.0));
}

#[test]
fn parse_named_case_insensitive() {
    check_parse("RED", Rgba::new(1.0, 0.0, 0.0));
    check_parse("Transparent", Rgba::new(0.0, 0.0, 0.0).with_alpha(0.0));
}

#[test]
fn parse_named_unknown() {
    check_err("ultramarine");
}

// —————— detect ——————

#[test]
fn detect_formats() {
    check_detect("#FF0000", Some(ColorFormat::Hex));
    check_detect("rgb(255,0,0)", Some(ColorFormat::Rgb));
    check_detect("hsl(0,100%,50%)", Some(ColorFormat::Hsl));
    check_detect("oklch(50% 0.1 30)", Some(ColorFormat::Oklch));
    check_detect("red", Some(ColorFormat::Named));
    check_detect("not-a-color", None);
}

#[test]
fn oklch() {
    // OKLCH(0.5 0.1 30) примерно соответствует sRGB (0.5, 0.3, 0.2)
    let c = parse_color("oklch(50% 0.1 30)").unwrap();
    // Проверка, что не паника и результат в 0..1
    assert!(c.r >= 0.0 && c.r <= 1.0);
    assert!(c.g >= 0.0 && c.g <= 1.0);
    assert!(c.b >= 0.0 && c.b <= 1.0);
}

#[test]
fn consistency_pass() {
    let colors = ["#FF0000", "#00FF00", "#0000FF"];
    assert_eq!(enforce_consistency(&colors).unwrap(), ColorFormat::Hex);
}

#[test]
fn consistency_fail() {
    let colors = ["#FF0000", "rgb(0,255,0)"];
    assert!(enforce_consistency(&colors).is_err());
}

#[test]
fn hsl_known_colors() {
    // Красный: hsl(0, 100%, 50%) → rgb(255, 0, 0)
    check_parse("hsl(0, 100%, 50%)", Rgba::new(1.0, 0.0, 0.0));
    // Зелёный: hsl(120, 100%, 50%) → rgb(0, 255, 0)
    check_parse("hsl(120, 100%, 50%)", Rgba::new(0.0, 1.0, 0.0));
    // Синий: hsl(240, 100%, 50%) → rgb(0, 0, 255)
    check_parse("hsl(240, 100%, 50%)", Rgba::new(0.0, 0.0, 1.0));
    // Белый: hsl(0, 0%, 100%)
    check_parse("hsl(0, 0%, 100%)", Rgba::new(1.0, 1.0, 1.0));
    // Чёрный: hsl(0, 0%, 0%)
    check_parse("hsl(0, 0%, 0%)", Rgba::new(0.0, 0.0, 0.0));
}
