use crate::iced_editor::scroll;
use crate::iced_editor::widget::editor::IcedEditor;
use iced::advanced::Layout;
use iced::advanced::{Shell, mouse as iced_mouse};
use iced::{Event, Point, Rectangle};

/// Точка входа для `Widget::update()`.
pub fn update<'a, Message>(
    this: &mut IcedEditor<'a>,
    event: &Event,
    layout: Layout<'_>,
    cursor_state: iced_mouse::Cursor,
    shell: &mut Shell<'_, Message>,
) {
    let bounds = layout.bounds();
    let origin = Point::new(bounds.x, bounds.y);

    match event {
        Event::Keyboard(kb_event) => {
            super::keyboard::handle_keyboard(this, kb_event, bounds, origin, shell);
        }
        Event::Mouse(mouse_event) => {
            super::mouse::handle_mouse(this, mouse_event, bounds, origin, cursor_state, shell);
        }
        _ => {}
    }
}

/// Проверить, виден ли курсор, и если нет — скорректировать `scroll_y`.
pub(crate) fn auto_scroll(this: &IcedEditor<'_>, bounds: Rectangle) {
    let cursor_line = this.inner.doc.borrow().cursor.line();
    let new_scroll_y = scroll::ensure_cursor_visible(
        this.inner.scroll_y.get(),
        bounds.height,
        &this.inner.shaped_doc.borrow(),
        cursor_line,
    );
    if (new_scroll_y - this.inner.scroll_y.get()).abs() > 0.5 {
        this.inner.scroll_y.set(new_scroll_y);
        this.inner.mark_dirty();
    }
}
