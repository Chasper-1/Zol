//! AST (Abstract Syntax Tree) для mdplus-разметки.
//!
//! Этот модуль не зависит от крейтов, только от `std`.
//! Может быть вынесен в отдельный проект.

/// Документ — корень AST.
#[derive(Debug, Clone, PartialEq)]
pub struct MarkupDoc {
    pub children: Vec<MarkupNode>,
}

/// Узел AST.
#[derive(Debug, Clone, PartialEq)]
pub enum MarkupNode {
    /// Простой текст.
    Text(String),
    /// Форматированный блок с вложенными узлами.
    Formatted {
        style: MarkupStyle,
        children: Vec<MarkupNode>,
    },
}

/// Битовая маска стилей разметки.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MarkupStyle(pub u32);

#[allow(dead_code)]
impl MarkupStyle {
    pub const PLAIN: Self = Self(0);
    pub const BOLD: Self = Self(1 << 0);
    pub const ITALIC: Self = Self(1 << 1);
    pub const UNDERLINE: Self = Self(1 << 2);
    pub const STRIKETHROUGH: Self = Self(1 << 3);
    pub const SUPERSCRIPT: Self = Self(1 << 4);
    pub const SUBSCRIPT: Self = Self(1 << 5);
    pub const CODE: Self = Self(1 << 6);
    pub const HIGHLIGHT: Self = Self(1 << 7);
    pub const SPOILER: Self = Self(1 << 8);
    pub const SPOILER_BLOCK: Self = Self(1 << 9);
    pub const INSERTION: Self = Self(1 << 10);
    pub const DELETION: Self = Self(1 << 11);
    pub const COMMENT: Self = Self(1 << 12);
    pub const FORMULA: Self = Self(1 << 13);
    pub const DISPLAY_FORMULA: Self = Self(1 << 14);

    pub fn contains(&self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    pub fn bits(&self) -> u32 {
        self.0
    }
}

#[allow(dead_code)]
impl MarkupStyle {
    pub const fn from_bits(bits: u32) -> Self {
        Self(bits)
    }
}

impl std::ops::BitOr for MarkupStyle {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

/// Определение маркера разметки.
#[derive(Debug, Clone)]
pub struct MarkerDef {
    pub open: &'static str,
    pub close: &'static str,
    pub style: MarkupStyle,
    pub multiline: bool,
    /// Отслеживать вложенность одноимённых маркеров.
    /// `false` для комментариев — первый же `%%` закрывает.
    pub track_depth: bool,
}

/// Все маркеры mdplus. Упорядочены от длинных к коротким для правильного приоритета.
pub const MARKERS: &[MarkerDef] = &[
    MarkerDef {
        open: "%%",
        close: "%%",
        style: MarkupStyle::COMMENT,
        multiline: true,
        track_depth: false,
    },
    MarkerDef {
        open: "$$",
        close: "$$",
        style: MarkupStyle::DISPLAY_FORMULA,
        multiline: true,
        track_depth: true,
    },
    MarkerDef {
        open: "!!!",
        close: "!!!",
        style: MarkupStyle::SPOILER_BLOCK,
        multiline: true,
        track_depth: true,
    },
    MarkerDef {
        open: "!!",
        close: "!!",
        style: MarkupStyle::SPOILER,
        multiline: false,
        track_depth: true,
    },
    MarkerDef {
        open: "//",
        close: "//",
        style: MarkupStyle::ITALIC,
        multiline: false,
        track_depth: true,
    },
    MarkerDef {
        open: "**",
        close: "**",
        style: MarkupStyle::BOLD,
        multiline: false,
        track_depth: true,
    },
    MarkerDef {
        open: "__",
        close: "__",
        style: MarkupStyle::UNDERLINE,
        multiline: false,
        track_depth: true,
    },
    MarkerDef {
        open: "''",
        close: "''",
        style: MarkupStyle::SUPERSCRIPT,
        multiline: false,
        track_depth: true,
    },
    MarkerDef {
        open: ",,",
        close: ",,",
        style: MarkupStyle::SUBSCRIPT,
        multiline: false,
        track_depth: true,
    },
    MarkerDef {
        open: "~~",
        close: "~~",
        style: MarkupStyle::STRIKETHROUGH,
        multiline: false,
        track_depth: true,
    },
    MarkerDef {
        open: "==",
        close: "==",
        style: MarkupStyle::HIGHLIGHT,
        multiline: false,
        track_depth: true,
    },
    MarkerDef {
        open: "++",
        close: "++",
        style: MarkupStyle::INSERTION,
        multiline: false,
        track_depth: true,
    },
    MarkerDef {
        open: "--",
        close: "--",
        style: MarkupStyle::DELETION,
        multiline: false,
        track_depth: true,
    },
    MarkerDef {
        open: "$",
        close: "$",
        style: MarkupStyle::FORMULA,
        multiline: false,
        track_depth: true,
    },
];
