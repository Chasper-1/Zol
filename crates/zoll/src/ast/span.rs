/// Диапазон индексов токенов, покрываемый узлом AST.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    /// Индекс первого токена (включительно).
    pub start: usize,
    /// Индекс после последнего токена (не включительно).
    pub end: usize,
}

impl Span {
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// Охватывает ли этот span индекс?
    pub fn contains(&self, idx: usize) -> bool {
        idx >= self.start && idx < self.end
    }
}
