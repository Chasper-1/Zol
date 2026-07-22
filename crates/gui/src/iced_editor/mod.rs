//! Iced-виджет редактора.
//!
//! Текст рисуется реальной растеризацией глифов: `cosmic_text::Buffer::draw`
//! отдаёт пер-пиксельные цвета (с учётом `glyph.color_opt`), которые
//! композитятся в offscreen-RGBA-буфер, а затем выводятся одним
//! `renderer.draw_image`. Это корректно отображает буквы и цвета разметки
//! (в отличие от `fill_quad` по глифам, который рисует прямоугольники).
//!
//! Модули:
//! - [`inner`] — состояние редактора ([`EditorInner`]);
//! - [`widget`] — виджет ([`widget::editor::IcedEditor`]) и его `Widget`-импл;
//! - [`nav`] — вертикальная навигация (сохранение пиксельной X);
//! - [`scroll`] — автоскролл курсора в видимую зону.

pub mod inner;
pub mod nav;
pub mod scroll;
pub mod widget;

pub use inner::EditorInner;
pub use widget::editor_element;
