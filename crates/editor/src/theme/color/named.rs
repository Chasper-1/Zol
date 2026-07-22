//! Именованные цвета.
//!
//! Алфавитный порядок для бинарного поиска (`binary_search_by_key`).

use super::rgba::Rgba;

pub(super) const NAMED_COLORS: &[(&str, Rgba)] = &[
    ("aqua", Rgba::new(0.0, 1.0, 1.0)),
    ("black", Rgba::new(0.0, 0.0, 0.0)),
    ("blue", Rgba::new(0.0, 0.0, 1.0)),
    ("cyan", Rgba::new(0.0, 1.0, 1.0)),
    ("fuchsia", Rgba::new(1.0, 0.0, 1.0)),
    ("gray", Rgba::new(0.5, 0.5, 0.5)),
    ("green", Rgba::new(0.0, 0.5, 0.0)),
    ("grey", Rgba::new(0.5, 0.5, 0.5)),
    ("lime", Rgba::new(0.0, 1.0, 0.0)),
    ("maroon", Rgba::new(0.5, 0.0, 0.0)),
    ("navy", Rgba::new(0.0, 0.0, 0.5)),
    ("olive", Rgba::new(0.5, 0.5, 0.0)),
    ("orange", Rgba::new(1.0, 0.647, 0.0)),
    ("pink", Rgba::new(1.0, 0.753, 0.796)),
    ("purple", Rgba::new(0.5, 0.0, 0.5)),
    ("red", Rgba::new(1.0, 0.0, 0.0)),
    ("silver", Rgba::new(0.753, 0.753, 0.753)),
    ("tan", Rgba::new(0.824, 0.706, 0.549)),
    ("teal", Rgba::new(0.0, 0.5, 0.5)),
    ("transparent", Rgba::new(0.0, 0.0, 0.0).with_alpha(0.0)),
    ("white", Rgba::new(1.0, 1.0, 1.0)),
    ("yellow", Rgba::new(1.0, 1.0, 0.0)),
];
