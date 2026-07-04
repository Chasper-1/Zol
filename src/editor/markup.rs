pub enum LineMarkup {
    Heading {
        content: String,
        marker: String,
    },
    Bold {
        before: String,
        content: String,
        after: String,
        marker: String,
    },
    Italic {
        before: String,
        content: String,
        after: String,
        marker: String,
    },
    Strikethrough {
        before: String,
        content: String,
        after: String,
        marker: String,
    },
    Superscript {
        before: String,
        content: String,
        after: String,
        marker: String,
    },
    Subscript {
        before: String,
        content: String,
        after: String,
        marker: String,
    },
    Code {
        before: String,
        content: String,
        after: String,
        marker: String,
    },
    Plain(String),
}

/// 1. Функция ищет конкретный маркер СТРОГО в начале строки
fn match_start<'a>(line: &'a str, marker: &str) -> Option<&'a str> {
    if line.starts_with(marker) {
        Some(&line[marker.len()..])
    } else {
        None
    }
}

/// 2. Функция проверяет, обернута ли строка в парный маркер с двух сторон в любом месте
fn match_paired<'a>(line: &'a str, marker: &str) -> Option<(&'a str, &'a str, &'a str)> {
    let m_len = marker.len();
    if let Some(start_idx) = line.find(marker) {
        let before = &line[..start_idx];
        let after_first = &line[start_idx + m_len..];
        if let Some(end_idx) = after_first.find(marker) {
            let content = &after_first[..end_idx];
            let after = &after_first[end_idx + m_len..];
            return Some((before, content, after));
        }
    }
    None
}

/// Анализирует строку и возвращает тип разметки (без дублирования кода)
pub fn parse_line(line: &str) -> LineMarkup {
    if let Some(content) = match_start(line, "# ") {
        return LineMarkup::Heading {
            content: content.to_string(),
            marker: "# ".to_string(),
        };
    }

    if let Some((b, c, a)) = match_paired(line, "**") {
        return LineMarkup::Bold {
            before: b.to_string(),
            content: c.to_string(),
            after: a.to_string(),
            marker: "**".to_string(),
        };
    }
    if let Some((b, c, a)) = match_paired(line, "~~") {
        return LineMarkup::Strikethrough {
            before: b.to_string(),
            content: c.to_string(),
            after: a.to_string(),
            marker: "~~".to_string(),
        };
    }
    if let Some((b, c, a)) = match_paired(line, "*") {
        return LineMarkup::Italic {
            before: b.to_string(),
            content: c.to_string(),
            after: a.to_string(),
            marker: "*".to_string(),
        };
    }
    if let Some((b, c, a)) = match_paired(line, "_") {
        return LineMarkup::Italic {
            before: b.to_string(),
            content: c.to_string(),
            after: a.to_string(),
            marker: "_".to_string(),
        };
    }
    if let Some((b, c, a)) = match_paired(line, "^") {
        return LineMarkup::Superscript {
            before: b.to_string(),
            content: c.to_string(),
            after: a.to_string(),
            marker: "^".to_string(),
        };
    }
    if let Some((b, c, a)) = match_paired(line, "~") {
        return LineMarkup::Subscript {
            before: b.to_string(),
            content: c.to_string(),
            after: a.to_string(),
            marker: "~".to_string(),
        };
    }
    if let Some((b, c, a)) = match_paired(line, "`") {
        return LineMarkup::Code {
            before: b.to_string(),
            content: c.to_string(),
            after: a.to_string(),
            marker: "`".to_string(),
        };
    }

    LineMarkup::Plain(line.to_string())
}
