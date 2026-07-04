pub enum LineMarkup {
    Heading { content: String, marker: String },
    Bold { content: String, marker: String },
    Plain(String),
}

/// Анализирует строку и возвращает тип разметки[cite: 2]
pub fn parse_line(line: &str) -> LineMarkup {
    if line.starts_with("# ") {
        LineMarkup::Heading {
            content: line[2..].to_string(),
            marker: "# ".to_string(),
        }
    } else if line.starts_with("**") && line.ends_with("**") && line.len() > 4 {
        LineMarkup::Bold {
            content: line[2..line.len() - 2].to_string(),
            marker: "**".to_string(),
        }
    } else {
        LineMarkup::Plain(line.to_string())
    }
}