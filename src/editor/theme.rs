use rhai::Map;

#[derive(Debug, Clone)]
pub struct TextTheme {
    pub size: f32,
    pub line_height: f32,
    pub color: String,
    pub margin_x: f32,
    pub margin_y: f32,
    pub font_family: String,
}

#[derive(Debug, Clone)]
pub struct EditorTheme {
    pub padding: f32,
    pub radius: f32,
    pub background: String,
    pub border_color: String,
    pub border_width: f32,
    pub text: TextTheme,
}

pub fn parse_theme(map: Map) -> EditorTheme {
    let mut padding = 12.0;
    let mut radius = 12.0;
    let mut background = "#131316".to_string();
    let mut border_color = "#242428".to_string();
    let mut border_width = 1.0;
    
    let mut text_size = 16.0;
    let mut text_line_height = 24.0;
    let mut text_color = "#ffffff".to_string();
    let mut text_margin_x = 10.0;
    let mut text_margin_y = 15.0;
    let mut font_family = "SansSerif".to_string();

    if let Some(editor) = map.get("editor") {
        let m = editor.clone().cast::<Map>();
        if let Some(p) = m.get("padding") { padding = p.clone().cast::<f64>() as f32; }
        if let Some(r) = m.get("radius") { radius = r.clone().cast::<f64>() as f32; }
        if let Some(b) = m.get("background") { background = b.clone().cast::<String>(); }
        if let Some(bc) = m.get("border_color") { border_color = bc.clone().cast::<String>(); }
        if let Some(bw) = m.get("border_width") { border_width = bw.clone().cast::<f64>() as f32; }
    }

    if let Some(text) = map.get("text") {
        let m = text.clone().cast::<Map>();
        if let Some(s) = m.get("size") { text_size = s.clone().cast::<f64>() as f32; }
        if let Some(lh) = m.get("line_height") { text_line_height = lh.clone().cast::<f64>() as f32; }
        if let Some(c) = m.get("color") { text_color = c.clone().cast::<String>(); }
        if let Some(mx) = m.get("margin_x") { text_margin_x = mx.clone().cast::<f64>() as f32; }
        if let Some(my) = m.get("margin_y") { text_margin_y = my.clone().cast::<f64>() as f32; }
        if let Some(ff) = m.get("font_family") { font_family = ff.clone().cast::<String>(); }
    }

    EditorTheme {
        padding,
        radius,
        background,
        border_color,
        border_width,
        text: TextTheme {
            size: text_size,
            line_height: text_line_height,
            color: text_color,
            margin_x: text_margin_x,
            margin_y: text_margin_y,
            font_family,
        },
    }
}