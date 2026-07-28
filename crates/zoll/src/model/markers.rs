//! Определения маркеров zoll: inline, line, block.

use super::seg::MarkStyle;

/// Категория маркера.
///
/// Ровно три варианта — никаких "гибридных" типов в будущем.
/// Если кому-то понадобится новый маркер — он выберет одну из трёх.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkerCategory {
    /// Внутри строки: `**`, `//`, `~~`, `==`, `$`, ...
    Inline,
    /// Вся строка: `#1#`, `%%`, `-`, `1.`, `>`, `---`, `#:`
    Line,
    /// Несколько строк: `%%%..%%%`, `$$$..$$$`, `!!!..!!!`, ` ```..``` `
    Block,
}

/// Определение маркера zoll.
#[derive(Debug, Clone)]
pub struct MarkerDef {
    /// Открывающая последовательность (например, `**`).
    pub open: &'static str,
    /// Закрывающая последовательность (например, `**`).
    pub close: &'static str,
    /// Какой стиль применяет.
    pub style: MarkStyle,
    /// Категория маркера.
    pub category: MarkerCategory,
    /// Может ли быть многострочным (для Inline — нет).
    pub multiline: bool,
    /// Отслеживать вложенность одноимённых маркеров.
    pub track_depth: bool,
}

/// Все встроенные маркеры zoll.
///
/// Порядок: от длинных к коротким (чтобы `%%%` не перепутался с `%%`).
pub const MARKERS: &[MarkerDef] = &[
    // ── Block-маркеры ──────────────────────────────────────────
    MarkerDef {
        open: "%%%",
        close: "%%%",
        style: MarkStyle::COMMENT,
        category: MarkerCategory::Block,
        multiline: true,
        track_depth: false,
    },
    MarkerDef {
        open: "$$$",
        close: "$$$",
        style: MarkStyle::FORMULA,
        category: MarkerCategory::Block,
        multiline: true,
        track_depth: false,
    },
    MarkerDef {
        open: "!!!",
        close: "!!!",
        style: MarkStyle::SPOILER,
        category: MarkerCategory::Block,
        multiline: true,
        track_depth: true,
    },
    // ── Line-маркеры (inline по длине, но line по смыслу) ─────
    // %%, !!, $$ — line-level, но парсятся как inline-маркеры до конца строки.
    // Они определяются парсером parse_line() по первой колонке/позиции.
    // ── Inline-маркеры ─────────────────────────────────────────
    MarkerDef {
        open: "**",
        close: "**",
        style: MarkStyle::BOLD,
        category: MarkerCategory::Inline,
        multiline: false,
        track_depth: true,
    },
    MarkerDef {
        open: "//",
        close: "//",
        style: MarkStyle::ITALIC,
        category: MarkerCategory::Inline,
        multiline: false,
        track_depth: true,
    },
    MarkerDef {
        open: "__",
        close: "__",
        style: MarkStyle::UNDERLINE,
        category: MarkerCategory::Inline,
        multiline: false,
        track_depth: true,
    },
    MarkerDef {
        open: "~~",
        close: "~~",
        style: MarkStyle::STRIKETHROUGH,
        category: MarkerCategory::Inline,
        multiline: false,
        track_depth: true,
    },
    MarkerDef {
        open: "==",
        close: "==",
        style: MarkStyle::HIGHLIGHT,
        category: MarkerCategory::Inline,
        multiline: false,
        track_depth: true,
    },
    MarkerDef {
        open: "''",
        close: "''",
        style: MarkStyle::SUPERSCRIPT,
        category: MarkerCategory::Inline,
        multiline: false,
        track_depth: true,
    },
    MarkerDef {
        open: ",,",
        close: ",,",
        style: MarkStyle::SUBSCRIPT,
        category: MarkerCategory::Inline,
        multiline: false,
        track_depth: true,
    },
    MarkerDef {
        open: "++",
        close: "++",
        style: MarkStyle::INSERTION,
        category: MarkerCategory::Inline,
        multiline: false,
        track_depth: true,
    },
    MarkerDef {
        open: "--",
        close: "--",
        style: MarkStyle::DELETION,
        category: MarkerCategory::Inline,
        multiline: false,
        track_depth: true,
    },
    MarkerDef {
        open: "$",
        close: "$",
        style: MarkStyle::FORMULA,
        category: MarkerCategory::Inline,
        multiline: false,
        track_depth: true,
    },
];
