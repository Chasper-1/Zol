//! Iced-виджет редактора.
//!
//! - [`editor`] — данные виджета (`IcedEditor`)
//! - [`draw`] — отрисовка текста и курсора
//! - [`input`] — обработка ввода
//! - [`widget`] — реализация трейта `Widget`

pub mod draw;
pub mod editor;
pub mod input;
pub mod widget;

pub use editor::IcedEditor;
pub use widget::editor_element;
