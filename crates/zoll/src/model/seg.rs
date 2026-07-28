//! Стили inline-разметки и сегменты строки.

/// Битовая маска стилей inline-разметки.
///
/// Каждый бит — один маркер. Можно комбинировать (BOLD | ITALIC).
/// u16 — хватает с запасом (сейчас 13 бит занято).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MarkStyle(pub u16);

impl MarkStyle {
    pub const PLAIN: Self = Self(0);
    pub const BOLD: Self = Self(1 << 0); // **text**
    pub const ITALIC: Self = Self(1 << 1); // //text//
    pub const UNDERLINE: Self = Self(1 << 2); // __text__
    pub const STRIKETHROUGH: Self = Self(1 << 3); // ~~text~~
    pub const CODE: Self = Self(1 << 4); // `text`
    pub const HIGHLIGHT: Self = Self(1 << 5); // ==text==
    pub const SUPERSCRIPT: Self = Self(1 << 6); // ''text''
    pub const SUBSCRIPT: Self = Self(1 << 7); // ,,text,,
    pub const INSERTION: Self = Self(1 << 8); // ++text++
    pub const DELETION: Self = Self(1 << 9); // --text--
    pub const COMMENT: Self = Self(1 << 10); // %%text%% / %%%...%%%
    pub const SPOILER: Self = Self(1 << 11); // !!text!! / !!!...!!!
    pub const FORMULA: Self = Self(1 << 12); // $text$ / $$$...$$$

    pub fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    pub fn bits(self) -> u16 {
        self.0
    }

    pub const fn from_bits(bits: u16) -> Self {
        Self(bits)
    }
}

impl std::ops::BitOr for MarkStyle {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

/// Стилизованный сегмент строки.
///
/// Не владеет текстом — ссылается на `ParsedLine.source` по байтовым границам.
/// При редактировании строки сегменты перестраиваются заново.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Segment {
    /// Байтовое начало в `source` (включая маркеры).
    pub start: usize,
    /// Байтовый конец в `source` (не включая).
    pub end: usize,
    /// Стиль сегмента (PLAIN для обычного текста).
    pub style: MarkStyle,
}

impl Segment {
    pub fn new(start: usize, end: usize, style: MarkStyle) -> Self {
        Self { start, end, style }
    }

    /// Длина сегмента в байтах.
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    /// Пустой ли сегмент.
    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }

    /// Извлечь текст из исходной строки.
    pub fn text<'a>(&self, source: &'a str) -> &'a str {
        &source[self.start..self.end]
    }
}
