// Безопасное извлечение подстроки с корректировкой UTF-8 границ.
pub fn safe_slice(content: &str, start: usize, end: usize) -> &str {
    let len = content.len();
    let start = start.min(len);
    let end = end.min(len);
    let start = if content.is_char_boundary(start) {
        start
    } else {
        safe_prev_boundary(content, start)
    };
    let end = if content.is_char_boundary(end) {
        end
    } else {
        safe_prev_boundary(content, end)
    };
    if start >= end {
        return &content[start..start];
    }
    &content[start..end]
}

#[inline]
fn safe_prev_boundary(content: &str, byte: usize) -> usize {
    let mut b = byte.min(content.len());
    while b > 0 && !content.is_char_boundary(b) {
        b -= 1;
    }
    b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_slice_full() {
        assert_eq!(safe_slice("hello", 0, 5), "hello");
    }

    #[test]
    fn safe_slice_prefix() {
        assert_eq!(safe_slice("hello", 0, 3), "hel");
    }

    #[test]
    fn safe_slice_unicode_char_boundary() {
        assert_eq!(safe_slice("привет", 0, 6), "при");
    }

    #[test]
    fn safe_slice_unicode_mid_char() {
        let s = safe_slice("привет", 1, 5);
        assert!(s.len() <= 5);
    }

    #[test]
    fn safe_slice_zero_range() {
        assert_eq!(safe_slice("hello", 0, 0), "");
    }

    #[test]
    fn safe_slice_same_start_end() {
        assert_eq!(safe_slice("hello", 3, 3), "");
    }

    #[test]
    fn safe_slice_past_end() {
        assert_eq!(safe_slice("hello", 10, 20), "");
    }
}
