use rhai::Map;
use crate::theme::{Theme, EditorTheme, TextTheme};

pub fn apply(map: Map) -> Theme {
    let mut theme = Theme::default();

    if let Some(editor) = map.get("editor") {
        let m = editor.clone().cast::<Map>();

        theme.editor.padding =
            m.get("padding").unwrap().clone().cast::<f32>();

        theme.editor.radius =
            m.get("radius").unwrap().clone().cast::<f32>();

        theme.editor.background =
            m.get("background").unwrap().clone().cast::<String>();
    }

    if let Some(text) = map.get("text") {
        let m = text.clone().cast::<Map>();

        theme.text.size =
            m.get("size").unwrap().clone().cast::<f32>();

        theme.text.line_height =
            m.get("line_height").unwrap().clone().cast::<f32>();

        theme.text.color =
            m.get("color").unwrap().clone().cast::<String>();
    }

    theme
}