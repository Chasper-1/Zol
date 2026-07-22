//! Данные виджета: [`IcedEditor`] — обёртка над `&EditorInner`.
//!
//! Сам виджет — почти пустая обёртка; вся логика — в `EditorInner`.
//! `last_bounds` нужен для автоскролла.

use std::cell::Cell;

use iced::Rectangle;

use crate::gui::iced_editor::inner::EditorInner;

/// Iced-виджет редактора.
///
/// Держит ссылку на состояние редактора (`EditorInner`) и кеширует
/// последние границы окна для автоскролла.
pub struct IcedEditor<'a> {
    /// Состояние редактора.
    pub inner: &'a EditorInner,
    /// Последние bounds окна (для автоскролла).
    pub last_bounds: Cell<Rectangle>,
}

impl<'a> IcedEditor<'a> {
    pub fn new(inner: &'a EditorInner) -> Self {
        Self {
            inner,
            last_bounds: Cell::new(Rectangle::default()),
        }
    }
}
