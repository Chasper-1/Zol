use crate::editor::markup::segment::StyleFlags;
use crate::editor::markup::segment::{
    STYLE_BOLD, STYLE_COMMENT, STYLE_DELETION, STYLE_DISPLAY_FORMULA, STYLE_FORMULA,
    STYLE_HIGHLIGHT, STYLE_INSERTION, STYLE_ITALIC, STYLE_STRIKETHROUGH, STYLE_SUBSCRIPT,
    STYLE_SUPERSCRIPT, STYLE_UNDERLINE,
};

pub struct MarkerDef {
    pub open: &'static str,
    pub close: &'static str,
    pub style: StyleFlags,
    pub multiline: bool,
}

pub const MARKERS: &[MarkerDef] = &[
    MarkerDef {
        open: "/*",
        close: "*\\",
        style: STYLE_COMMENT,
        multiline: true,
    },
    MarkerDef {
        open: "$$",
        close: "$$",
        style: STYLE_DISPLAY_FORMULA,
        multiline: true,
    },
    MarkerDef {
        open: "//",
        close: "//",
        style: STYLE_ITALIC,
        multiline: false,
    },
    MarkerDef {
        open: "**",
        close: "**",
        style: STYLE_BOLD,
        multiline: false,
    },
    MarkerDef {
        open: "__",
        close: "__",
        style: STYLE_UNDERLINE,
        multiline: false,
    },
    MarkerDef {
        open: "''",
        close: "''",
        style: STYLE_SUPERSCRIPT,
        multiline: false,
    },
    MarkerDef {
        open: ",,",
        close: ",,",
        style: STYLE_SUBSCRIPT,
        multiline: false,
    },
    MarkerDef {
        open: "~~",
        close: "~~",
        style: STYLE_STRIKETHROUGH,
        multiline: false,
    },
    MarkerDef {
        open: "==",
        close: "==",
        style: STYLE_HIGHLIGHT,
        multiline: false,
    },
    MarkerDef {
        open: "++",
        close: "++",
        style: STYLE_INSERTION,
        multiline: false,
    },
    MarkerDef {
        open: "--",
        close: "--",
        style: STYLE_DELETION,
        multiline: false,
    },
    MarkerDef {
        open: "$",
        close: "$",
        style: STYLE_FORMULA,
        multiline: false,
    },
];
