/// Номер строки, содержащей указанный байтовый индекс.
pub fn line_of_byte(content: &str, byte: usize) -> usize {
    if content.is_empty() || byte == 0 {
        return 0;
    }
    let byte = byte.min(content.len());
    let safe_byte = if content.is_char_boundary(byte) {
        byte
    } else {
        let mut adjusted = byte;
        while adjusted > 0 && !content.is_char_boundary(adjusted) {
            adjusted -= 1;
        }
        adjusted
    };
    content[..safe_byte].bytes().filter(|&b| b == b'\n').count()
}
