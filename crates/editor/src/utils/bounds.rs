/// Результат поиска границ строки.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LineBounds {
    pub start: usize,
    pub end: usize,
}
