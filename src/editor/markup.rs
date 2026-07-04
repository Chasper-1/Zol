pub enum LineMarkup {
    Heading { content: String, marker: String },
    Bold { content: String, marker: String },
    Italic { content: String, marker: String },
    Strikethrough { content: String, marker: String },
    Superscript { content: String, marker: String },
    Subscript { content: String, marker: String },
    Code { content: String, marker: String },
    Plain(String),
}

/// Анализирует строку и возвращает тип разметки
pub fn parse_line(line: &str) -> LineMarkup {
    // Проверяем заголовок
    if line.starts_with("# ") {
        return LineMarkup::Heading {
            content: line[2..].to_string(),
            marker: "# ".to_string(),
        };
    }

    // Проверяем жирный (самый длинный маркер)
    if line.starts_with("**") && line.ends_with("**") && line.len() > 4 {
        return LineMarkup::Bold {
            content: line[2..line.len() - 2].to_string(),
            marker: "**".to_string(),
        };
    }

    // Зачёркнутый
    if line.starts_with("~~") && line.ends_with("~~") && line.len() > 4 {
        return LineMarkup::Strikethrough {
            content: line[2..line.len() - 2].to_string(),
            marker: "~~".to_string(),
        };
    }

    // Курсив (одинарные звёздочки или подчёркивания)
    if (line.starts_with('*') && line.ends_with('*') && line.len() > 2)
        || (line.starts_with('_') && line.ends_with('_') && line.len() > 2)
    {
        let marker = if line.starts_with('*') { "*" } else { "_" };
        return LineMarkup::Italic {
            content: line[1..line.len() - 1].to_string(),
            marker: marker.to_string(),
        };
    }

    // Верхний индекс (маркер ^)
    if line.starts_with('^') && line.ends_with('^') && line.len() > 2 {
        return LineMarkup::Superscript {
            content: line[1..line.len() - 1].to_string(),
            marker: "^".to_string(),
        };
    }

    // Нижний индекс (маркер ~)
    if line.starts_with('~') && line.ends_with('~') && line.len() > 2 {
        return LineMarkup::Subscript {
            content: line[1..line.len() - 1].to_string(),
            marker: "~".to_string(),
        };
    }

    // Моноширинный код (обратные кавычки)
    if line.starts_with('`') && line.ends_with('`') && line.len() > 2 {
        return LineMarkup::Code {
            content: line[1..line.len() - 1].to_string(),
            marker: "`".to_string(),
        };
    }

    // Если ничего не подошло – обычный текст
    LineMarkup::Plain(line.to_string())
}