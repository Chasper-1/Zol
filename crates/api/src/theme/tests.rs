use super::*;
use editor::theme::color::Rgba;

#[test]
fn theme_default_has_name() {
    let t = theme_default();
    assert_eq!(theme_get_name(&t), "default");
}

#[test]
fn theme_set_get_name() {
    let mut t = theme_default();
    theme_set_name(&mut t, "dark");
    assert_eq!(theme_get_name(&t), "dark");
}

#[test]
fn theme_set_bg_hex() {
    let mut t = theme_default();
    theme_set_bg(&mut t, "#ff0000").unwrap();
    assert!((t.background.r - 1.0).abs() < 0.01);
    assert!((t.background.g).abs() < 0.01);
    assert!((t.background.b).abs() < 0.01);
}

#[test]
fn theme_set_text_hex() {
    let mut t = theme_default();
    theme_set_text(&mut t, "#00ff00").unwrap();
    assert!((t.text.color.r).abs() < 0.01);
    assert!((t.text.color.g - 1.0).abs() < 0.01);
    assert!((t.text.color.b).abs() < 0.01);
}

#[test]
fn theme_set_bg_invalid_hex() {
    let mut t = theme_default();
    assert!(theme_set_bg(&mut t, "not a color").is_err());
}

#[test]
fn theme_set_text_invalid_hex() {
    let mut t = theme_default();
    assert!(theme_set_text(&mut t, "#xyz").is_err());
}

#[test]
fn theme_set_bg_short_hex() {
    let mut t = theme_default();
    theme_set_bg(&mut t, "#fff").unwrap();
    assert!((t.background.r - 1.0).abs() < 0.01);
    assert!((t.background.g - 1.0).abs() < 0.01);
    assert!((t.background.b - 1.0).abs() < 0.01);
}
