use super::*;

#[test]
fn config_default_theme() {
    let cfg = Config::default();
    assert_eq!(cfg.theme, "default");
}

#[test]
fn config_default_font_size() {
    let cfg = Config::default();
    assert!(cfg.font_size > 0.0);
}

#[test]
fn config_dir_returns_some_path() {
    let dir = config_dir();
    assert!(!dir.as_os_str().is_empty());
}
