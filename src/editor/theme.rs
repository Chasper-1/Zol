use rhai::Map;

#[derive(Debug, Clone)]
pub struct TextTheme {
    pub size: f32,
    pub font_family: String,
}

#[derive(Debug, Clone)]
pub struct EditorTheme {
    pub padding: f32,
    pub background: Rgba,
    pub border_color: Rgba,
    pub border_width: f32,
    pub text: TextTheme,
}

#[derive(Clone, Debug)]
pub struct Rgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

fn map_to_color(map: &Map) -> Rgba {
    let get_val = |key: &str| -> f32 {
        map.get(key)
            .map(|v| {
                v.as_float()
                    .unwrap_or_else(|_| v.as_int().unwrap_or(0) as f64) as f32
            })
            .unwrap_or(0.0)
    };

    Rgba {
        r: get_val("r") / 255.0,
        g: get_val("g") / 255.0,
        b: get_val("b") / 255.0,
    }
}

pub fn parse_theme(map: Map) -> EditorTheme {
    let mut padding = 12.0;
    let mut background = Rgba {
        r: 0.078,
        g: 0.078,
        b: 0.086,
    };
    let mut border_color = Rgba {
        r: 0.141,
        g: 0.141,
        b: 0.157,
    };
    let mut border_width = 1.0;
    let mut text_size = 16.0;
    let mut font_family = "SansSerif".to_string();

    if let Some(editor) = map.get("editor") {
        let m = editor.clone().cast::<Map>();
        if let Some(p) = m.get("padding") {
            padding = p.clone().cast::<f64>() as f32;
        }
        if let Some(b) = m.get("background") {
            background = map_to_color(&b.clone().cast::<Map>());
        }
        if let Some(bc) = m.get("border_color") {
            border_color = map_to_color(&bc.clone().cast::<Map>());
        }
        if let Some(bw) = m.get("border_width") {
            border_width = bw.clone().cast::<f64>() as f32;
        }
    }

    if let Some(text) = map.get("text") {
        let m = text.clone().cast::<Map>();
        if let Some(s) = m.get("size") {
            text_size = s.clone().cast::<f64>() as f32;
        }
        if let Some(ff) = m.get("font_family") {
            font_family = ff.clone().cast::<String>();
        }
    }

    EditorTheme {
        padding,
        background,
        border_color,
        border_width,
        text: TextTheme {
            size: text_size,
            font_family,
        },
    }
}
