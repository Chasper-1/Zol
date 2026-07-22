//! Реестр всех ручек темы.
//!
//! Каждая константа — это [`Handle<T>`], описывающий одну настройку.
//! Категория задаёт неймспейс, имя — конкретную настройку.
//!
//! # Пример
//!
//! ```ignore
//! use theme::handles::PADDING;
//! system.set(&PADDING, 12.0);
//! let v: f32 = system.get(&PADDING).unwrap_or(10.0);
//! ```

use super::color::Rgba;
use super::handle::Handle;

/// Реестр ручек темы.
pub mod handles {
    use super::*;

    // — editor —
    pub const PADDING: Handle<f32> = Handle {
        category: "editor",
        name: "padding",
        default: 10.0,
    };

    pub const RADIUS: Handle<f32> = Handle {
        category: "editor",
        name: "radius",
        default: 16.0,
    };

    pub const BACKGROUND: Handle<Rgba> = Handle {
        category: "editor",
        name: "background",
        default: Rgba::new(0.153, 0.18, 0.2).with_alpha(0.9),
    };

    // — text —
    pub const TEXT_SIZE: Handle<f32> = Handle {
        category: "text",
        name: "size",
        default: 14.0,
    };

    pub const TEXT_COLOR: Handle<Rgba> = Handle {
        category: "text",
        name: "color",
        default: Rgba::new(0.804, 0.839, 0.957),
    };

    pub const TEXT_FONT: Handle<String> = Handle {
        category: "text",
        name: "font_family",
        default: String::new(),
    };
}
