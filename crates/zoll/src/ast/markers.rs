use super::style::MarkupStyle;

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

/// Все маркеры zoll. Упорядочены от длинных к коротким для правильного приоритета.
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
