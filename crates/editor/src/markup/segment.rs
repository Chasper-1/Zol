pub type StyleFlags = u32;

#[allow(dead_code)]
pub const STYLE_PLAIN: StyleFlags = 0;
pub const STYLE_BOLD: StyleFlags = 1 << 0;
pub const STYLE_ITALIC: StyleFlags = 1 << 1;
pub const STYLE_UNDERLINE: StyleFlags = 1 << 2;
pub const STYLE_STRIKETHROUGH: StyleFlags = 1 << 3;
pub const STYLE_SUPERSCRIPT: StyleFlags = 1 << 4;
pub const STYLE_SUBSCRIPT: StyleFlags = 1 << 5;
pub const STYLE_CODE: StyleFlags = 1 << 6;
pub const STYLE_HIGHLIGHT: StyleFlags = 1 << 7;
#[allow(dead_code)]
pub const STYLE_SPOILER: StyleFlags = 1 << 8;
pub const STYLE_INSERTION: StyleFlags = 1 << 9;
pub const STYLE_DELETION: StyleFlags = 1 << 10;
pub const STYLE_COMMENT: StyleFlags = 1 << 11;
pub const STYLE_FORMULA: StyleFlags = 1 << 12;
pub const STYLE_DISPLAY_FORMULA: StyleFlags = 1 << 13;

#[derive(Clone, Debug)]
pub struct Segment {
    pub text: String,

    pub style: StyleFlags,

    pub left_marker_len: usize,
    pub right_marker_len: usize,

    pub raw_start: usize,
    pub raw_end: usize,
}
