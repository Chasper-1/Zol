/// Номер строки, содержащей указанный байтовый индекс.
///
/// Использует `line_starts` для O(log n) бинарного поиска (вместо O(n) сканирования).
pub fn line_of_byte(content: &str, line_starts: &[usize], byte: usize) -> usize {
    if content.is_empty() || line_starts.is_empty() {
        return 0;
    }
    if byte == 0 {
        return 0;
    }
    let byte_pos = byte.min(content.len());
    match line_starts.binary_search(&byte_pos) {
        Ok(i) => i,
        Err(0) => 0,
        Err(i) => i - 1,
    }
}
