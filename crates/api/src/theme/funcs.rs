use editor::theme::EditorTheme;

pub fn theme_default() -> EditorTheme {
    EditorTheme::default()
}

pub fn theme_set_name(theme: &mut EditorTheme, name: &str) {
    theme.name = name.to_string();
}

pub fn theme_get_name(theme: &EditorTheme) -> &str {
    &theme.name
}

pub fn theme_set_bg(theme: &mut EditorTheme, hex: &str) -> Result<(), String> {
    let rgba = editor::theme::color::parse_color(hex)?;
    theme.background = rgba;
    Ok(())
}

pub fn theme_set_text(theme: &mut EditorTheme, hex: &str) -> Result<(), String> {
    let rgba = editor::theme::color::parse_color(hex)?;
    theme.text.color = rgba;
    Ok(())
}
