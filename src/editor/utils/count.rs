/// Количество строк в тексте. Пустой текст = 1 строка.
pub fn count_lines(content: &str) -> usize {
    if content.is_empty() {
        return 1;
    }
    content.bytes().filter(|&b| b == b'\n').count() + 1
}
