/// Начало предыдущего слова (char-safe, is_whitespace).
pub fn prev_word_start(content: &str, from: usize) -> usize {
    let from = from.min(content.len());
    if from == 0 || content.is_empty() {
        return 0;
    }

    let mut pos = from;

    // 1. Пропустить пробелы назад
    for (i, ch) in content[..pos].char_indices().rev() {
        if ch.is_whitespace() {
            pos = i;
        } else {
            break;
        }
    }
    if pos == 0 {
        return 0;
    }

    // 2. Пропустить непробелы назад (текущее слово)
    let mut start = pos;
    for (i, ch) in content[..pos].char_indices().rev() {
        if !ch.is_whitespace() {
            start = i;
        } else {
            break;
        }
    }

    // Если не сдвинулись — ищем предыдущее слово
    if start == from || start == pos {
        let mut p = from;
        for (i, ch) in content[..p].char_indices().rev() {
            if !ch.is_whitespace() {
                p = i;
            } else {
                break;
            }
        }
        let mut after_space = p;
        for (i, ch) in content[..p].char_indices().rev() {
            if ch.is_whitespace() {
                after_space = i;
            } else {
                break;
            }
        }
        let mut word_start = after_space;
        for (i, ch) in content[..after_space].char_indices().rev() {
            if !ch.is_whitespace() {
                word_start = i;
            } else {
                break;
            }
        }
        return word_start;
    }

    start
}

/// Начало следующего слова (char-safe, is_whitespace).
pub fn next_word_start(content: &str, from: usize) -> usize {
    let len = content.len();
    let mut pos = from.min(len);
    if pos >= len {
        return len;
    }

    // 1. Если на непробельном — пропускаем слово
    if let Some(ch) = content[pos..].chars().next() {
        if !ch.is_whitespace() {
            for (i, c) in content[pos..].char_indices() {
                if c.is_whitespace() {
                    pos += i;
                    break;
                }
            }
        }
    }

    // 2. Пропускаем пробелы к началу следующего слова
    for (i, c) in content[pos..].char_indices() {
        if !c.is_whitespace() {
            pos += i;
            return pos;
        }
    }

    len
}
