// Количество строк в тексте. Пустой текст = 1 строка.
pub fn count_lines(content: &str) -> usize {
    if content.is_empty() {
        return 1;
    }
    content.bytes().filter(|&b| b == b'\n').count() + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_lines_empty() {
        assert_eq!(count_lines(""), 1);
    }

    #[test]
    fn count_lines_single() {
        assert_eq!(count_lines("hello"), 1);
    }

    #[test]
    fn count_lines_two() {
        assert_eq!(count_lines("a\nb"), 2);
    }

    #[test]
    fn count_lines_three() {
        assert_eq!(count_lines("a\nb\nc"), 3);
    }

    #[test]
    fn count_lines_newline_only() {
        assert_eq!(count_lines("\n"), 2);
    }

    #[test]
    fn count_lines_trailing_newline() {
        assert_eq!(count_lines("a\n"), 2);
    }
}
