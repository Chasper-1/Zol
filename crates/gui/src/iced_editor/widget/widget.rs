//! Реализация трейта `Widget` для [`super::editor::IcedEditor`].
//!
//! Делегирует отрисовку в [`draw`], ввод в [`input`].
//! `size`, `layout`, `mouse_interaction` — минимальны.

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{self, Widget};
use iced::advanced::{mouse, Clipboard, Shell};
use iced::advanced::text::Renderer as TextRenderer;
use iced::{Element, Event, Length, Rectangle, Size};

use crate::iced_editor::inner::EditorInner;

use super::draw;
use super::editor::IcedEditor;
use super::input;

// ---------------------------------------------------------------------------
// Widget impl
// ---------------------------------------------------------------------------

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for IcedEditor<'a>
where
    Renderer: TextRenderer<Font = iced::Font>,
{
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fill)
    }

    fn layout(
        &mut self,
        tree: &mut widget::Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::Node::new(limits.max())
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let _ = (tree, theme, style, viewport);
        draw::draw(self, renderer, layout, cursor);
    }

    fn update(
        &mut self,
        tree: &mut widget::Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor_state: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        input::update(self, event, layout, cursor_state, shell);
    }

    fn mouse_interaction(
        &self,
        tree: &widget::Tree,
        _layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        mouse::Interaction::Text
    }
}

// ---------------------------------------------------------------------------
// Helper: Element из IcedEditor
// ---------------------------------------------------------------------------

/// Создать `Element` с редактором.
pub fn editor_element<'a, Message: 'a>(
    inner: &'a EditorInner,
) -> Element<'a, Message, iced::Theme, iced::Renderer> {
    Element::new(IcedEditor::new(inner))
}
