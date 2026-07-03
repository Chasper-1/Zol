use rhai::Map;

#[derive(Debug, Clone)]
pub struct TextTheme {
    pub size: f32,
    pub line_height: f32,
    pub color: Rgba,
    pub margin_x: f32,
    pub margin_y: f32,
    pub font_family: String,
}

#[derive(Debug, Clone)]
pub struct EditorTheme {
    pub padding: f32,
    pub radius: f32,
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
    pub a: f32,
}

// Вспомогательная функция для получения цвета из Map
fn map_to_color(map: &Map) -> Rgba {
    Rgba {
        // Мы берем число, приводим к f32 и делим на 255.0
        r: map
            .get("r")
            .map(|v| v.clone().cast::<f64>() as f32 / 255.0)
            .unwrap_or(0.0),
        g: map
            .get("g")
            .map(|v| v.clone().cast::<f64>() as f32 / 255.0)
            .unwrap_or(0.0),
        b: map
            .get("b")
            .map(|v| v.clone().cast::<f64>() as f32 / 255.0)
            .unwrap_or(0.0),
        // Альфа обычно остается в диапазоне 0.0 - 1.0, так что ее не трогаем
        a: map
            .get("a")
            .map(|v| v.clone().cast::<f64>() as f32)
            .unwrap_or(1.0),
    }
}

pub fn parse_theme(map: Map) -> EditorTheme {
    let mut padding = 12.0;
    let mut radius = 12.0;
    // Дефолтные значения — теперь только Rgba
    let mut background = Rgba {
        r: 0.078,
        g: 0.078,
        b: 0.086,
        a: 1.0,
    };
    let mut border_color = Rgba {
        r: 0.141,
        g: 0.141,
        b: 0.157,
        a: 1.0,
    };
    let mut border_width = 1.0;

    let mut text_size = 16.0;
    let mut text_line_height = 24.0;
    let mut text_color = Rgba {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    let mut text_margin_x = 10.0;
    let mut text_margin_y = 15.0;
    let mut font_family = "SansSerif".to_string();

    if let Some(editor) = map.get("editor") {
        let m = editor.clone().cast::<Map>();
        if let Some(p) = m.get("padding") {
            padding = p.clone().cast::<f64>() as f32;
        }
        if let Some(r) = m.get("radius") {
            radius = r.clone().cast::<f64>() as f32;
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
        if let Some(lh) = m.get("line_height") {
            text_line_height = lh.clone().cast::<f64>() as f32;
        }
        if let Some(c) = m.get("color") {
            text_color = map_to_color(&c.clone().cast::<Map>());
        }
        if let Some(mx) = m.get("margin_x") {
            text_margin_x = mx.clone().cast::<f64>() as f32;
        }
        if let Some(my) = m.get("margin_y") {
            text_margin_y = my.clone().cast::<f64>() as f32;
        }
        if let Some(ff) = m.get("font_family") {
            font_family = ff.clone().cast::<String>();
        }
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
