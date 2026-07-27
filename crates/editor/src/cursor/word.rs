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

    // Курсор на '\n' (конец строки) — переходим на следующую,
    // к концу первого слова.
    if content.as_bytes()[pos] == b'\n' {
        let rest = &content[pos..];
        match rest.char_indices().find(|(_, c)| !c.is_whitespace()) {
            Some((skip, _)) => {
                let word_start = pos + skip;
                match content[word_start..]
                    .char_indices()
                    .find(|(_, c)| c.is_whitespace())
                {
                    Some((end, _)) => return word_start + end,
                    None => return len,
                }
            }
            None => return len,
        }
    }

    // 1. Если на непробельном — пропускаем слово
    if let Some(ch) = content[pos..].chars().next() {
        if !ch.is_whitespace() {
            match content[pos..]
                .char_indices()
                .find(|(_, c)| c.is_whitespace())
            {
                Some((i, _)) => pos += i,
                None => pos = len,
            }
        }
    }

    // 2. Пропускаем пробелы к началу следующего слова (не переходим строку)
    for (i, c) in content[pos..].char_indices() {
        if c == '\n' {
            break;
        }
        if !c.is_whitespace() {
            pos += i;
            return pos;
        }
    }

    pos
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prev_word_start_mid_second_word() {
        assert_eq!(prev_word_start("hello world", 6), 0);
    }

    #[test]
    fn prev_word_start_after_first_word() {
        assert_eq!(prev_word_start("hello world", 5), 0);
    }

    #[test]
    fn prev_word_start_mid_first_word() {
        assert_eq!(prev_word_start("hello", 3), 0);
    }

    #[test]
    fn prev_word_start_empty() {
        assert_eq!(prev_word_start("", 0), 0);
    }

    #[test]
    fn prev_word_start_spaced() {
        assert_eq!(prev_word_start("  spaced  ", 10), 2);
    }

    #[test]
    fn prev_word_start_three_words() {
        assert_eq!(prev_word_start("hello world foo", 15), 12);
    }

    #[test]
    fn next_word_start_from_start() {
        assert_eq!(next_word_start("hello world", 0), 6);
    }

    #[test]
    fn next_word_start_single_word() {
        // Одно слово — прыгаем к его концу
        assert_eq!(next_word_start("hello", 0), 5);
    }

    #[test]
    fn next_word_start_from_second_word() {
        // Начало последнего слова — прыгаем к его концу
        assert_eq!(next_word_start("hello world", 6), 11);
    }

    #[test]
    fn next_word_start_empty() {
        assert_eq!(next_word_start("", 0), 0);
    }

    #[test]
    fn next_word_start_multi_spaces() {
        assert_eq!(next_word_start("a   b", 0), 4);
    }

    #[test]
    fn next_word_start_at_end() {
        assert_eq!(next_word_start("hello", 5), 5);
    }
}
