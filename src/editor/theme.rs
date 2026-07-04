use eframe::egui::Color32;
use rhai::Map;

#[derive(Debug, Clone)]
pub struct TextTheme {
    pub size: f32,
    pub color: Rgba,
    pub font_family: Option<String>, // Используем Option, чтобы не хардкодить
}

#[derive(Debug, Clone)]
pub struct EditorTheme {
    pub padding: f32,
    pub radius: f32,
    pub background: Rgba,
    pub text: TextTheme,
}

#[derive(Clone, Debug)]
pub struct Rgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Rgba {
    pub fn to_color32(&self) -> Color32 {
        Color32::from_rgba_unmultiplied(
            (self.r * 255.0) as u8,
            (self.g * 255.0) as u8,
            (self.b * 255.0) as u8,
            (self.a * 255.0) as u8,
        )
    }
}

fn parse_rgba_string(s: &str) -> Rgba {
    let cleaned = s.replace("rgba(", "").replace(")", "");
    let parts: Vec<&str> = cleaned.split(',').map(|p| p.trim()).collect();
    if parts.len() == 4 {
        Rgba {
            r: parts[0].parse::<f32>().unwrap_or(0.0) / 255.0,
            g: parts[1].parse::<f32>().unwrap_or(0.0) / 255.0,
            b: parts[2].parse::<f32>().unwrap_or(0.0) / 255.0,
            a: parts[3].parse::<f32>().unwrap_or(1.0),
        }
    } else {
        Rgba {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    }
}

pub fn parse_theme(map: Map) -> EditorTheme {
    let mut padding = 10.0;
    let mut radius = 16.0;
    let mut background = Rgba {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.9,
    };
    let mut text_size = 14.0;
    let mut text_color = Rgba {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    let mut font_family = None; // Инициализируем пустотой

    if let Some(editor) = map.get("editor") {
        let m = editor.clone().cast::<Map>();
        if let Some(p) = m.get("padding") {
            padding = p.clone().cast::<f64>() as f32;
        }
        if let Some(r) = m.get("radius") {
            radius = r.clone().cast::<f64>() as f32;
        }
        if let Some(b) = m.get("background") {
            background = parse_rgba_string(&b.clone().cast::<String>());
        }
    }

    if let Some(text) = map.get("text") {
        let m = text.clone().cast::<Map>();
        if let Some(s) = m.get("size") {
            text_size = s.clone().cast::<f64>() as f32;
        }
        if let Some(c) = m.get("color") {
            text_color = parse_rgba_string(&c.clone().cast::<String>());
        }
        // Теперь мы просто берем то, что есть в конфиге, без хардкода
        if let Some(ff) = m.get("font_family") {
            font_family = Some(ff.clone().cast::<String>());
        }
    }

    EditorTheme {
        padding,
        radius,
        background,
        text: TextTheme {
            size: text_size,
            color: text_color,
            font_family,
        },
    }
}
