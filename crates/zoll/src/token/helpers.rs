/// Первые байты всех маркеров + `\n` и `\\`.
/// Для каждого байта: true, если с него может начинаться маркер,
/// `\n` или `\\`. Используется для batch-skip plain text.
const SPECIAL_BYTE: [bool; 256] = {
    let mut arr = [false; 256];
    arr[b'\n' as usize] = true;
    arr[b'\\' as usize] = true;
    arr[b'%' as usize] = true;
    arr[b'$' as usize] = true;
    arr[b'!' as usize] = true;
    arr[b'/' as usize] = true;
    arr[b'*' as usize] = true;
    arr[b'_' as usize] = true;
    arr[b'\'' as usize] = true;
    arr[b',' as usize] = true;
    arr[b'~' as usize] = true;
    arr[b'=' as usize] = true;
    arr[b'+' as usize] = true;
    arr[b'-' as usize] = true;
    arr
};

/// Длина UTF-8 символа по первому байту (ASCII fast-path).
#[inline]
pub fn utf8_char_len(first_byte: u8) -> usize {
    if first_byte < 128 {
        1
    } else if first_byte & 0xE0 == 0xC0 {
        2
    } else if first_byte & 0xF0 == 0xE0 {
        3
    } else if first_byte & 0xF8 == 0xF0 {
        4
    } else {
        1 // continuation byte — невалидный старт, но не крешимся
    }
}

/// Проверяет, может ли байт начинать маркер, `\n` или `\\`.
#[inline]
pub fn is_special_byte(b: u8) -> bool {
    SPECIAL_BYTE[b as usize]
}

/// Нативный поиск следующего символа `'\n'` начиная с байта `from`.
/// Использует `str::find('\n')` — под капотом `memchr` в оптимизируемых сборках.
pub fn next_newline(text: &str, from: usize) -> Option<usize> {
    text[from..].find('\n').map(|p| from + p)
}

/// Проверка, что в позиции стоит пробельный символ (или конец текста).
#[inline]
pub fn is_whitespace_at(text: &str, pos: usize) -> bool {
    text.as_bytes()
        .get(pos)
        .map_or(true, |&b| b.is_ascii_whitespace())
}

/// Проверка, что перед позицией стоит пробельный символ.
#[inline]
pub fn is_whitespace_before(text: &str, pos: usize) -> bool {
    if pos == 0 {
        return true;
    }
    text.as_bytes()
        .get(pos - 1)
        .map_or(true, |&b| b.is_ascii_whitespace())
}
