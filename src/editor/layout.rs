use crate::editor::state::EditorState;

#[derive(Clone, Debug)]
pub struct VisualLine {
    pub text: String,
    pub font_size: f32,
    pub font_family: String,
    pub x: f32,
    pub y: f32,
}

pub fn build(state: &EditorState) -> Vec<VisualLine> {
    let theme = state.get_theme();

    state
        .text
        .lines()
        .enumerate()
        .map(|(i, line)| VisualLine {
            text: line.to_string(),
            font_size: theme.text.size,
            font_family: theme.text.font_family.clone(),
            x: theme.padding,
            y: theme.padding + i as f32 * (theme.text.size + 8.0),
        })
        .collect()
}
