/// Безопасное извлечение подстроки с корректировкой UTF-8 границ.
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
