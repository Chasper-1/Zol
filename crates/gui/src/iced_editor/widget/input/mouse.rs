//! Обработка событий мыши.

use iced::advanced::{Shell, mouse as iced_mouse};
use iced::{Rectangle, Point};
use iced::mouse::ScrollDelta;

use api::cursor as api_cursor;

use super::IcedEditor;
use super::auto_scroll;

pub fn handle_mouse<'a, Message>(
    this: &mut IcedEditor<'a>,
    mouse_event: &iced::mouse::Event,
    bounds: Rectangle,
    origin: Point,
    cursor_state: iced_mouse::Cursor,
    shell: &mut Shell<'_, Message>,
) {
    match mouse_event {
        iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left) => {
            if let Some(pos) = cursor_state.position_in(bounds) {
                let local_x = pos.x - origin.x;
                let local_y = pos.y - origin.y;

                let shaped = this.inner.shaped_doc.borrow();
                let scroll_y = this.inner.scroll_y.get();
                let cosmic_cursor = shaped.buffer.hit(local_x, local_y + scroll_y);

                if let Some(cosmic) = cosmic_cursor {
                    let doc = this.inner.doc.borrow();
                    let (line_start, line_end) = doc.line_bounds(cosmic.line).map(|b| (b.start, b.end)).unwrap_or((0, 0));
                    let line_len = line_end.saturating_sub(line_start);
                    let new_raw = (line_start + cosmic.index).min(line_start + line_len);
                    // doc borrow ends here (NLL)

                    let mut doc = this.inner.doc.borrow_mut();
                    api_cursor::cursor_set_raw(&mut *doc, new_raw);
                    api_cursor::cursor_set_line(&mut *doc, cosmic.line);
                    api_cursor::cursor_reset_col(&mut *doc);
                }
            }

            auto_scroll(this, bounds);
            shell.request_redraw();
        }
        iced::mouse::Event::WheelScrolled { delta } => {
            let amount = match delta {
                ScrollDelta::Lines { y, .. } => -y * 40.0,
                ScrollDelta::Pixels { y, .. } => -y,
            };
            if amount.abs() > 0.0 {
                let max_scroll =
                    (this.inner.shaped_doc.borrow().total_height() - bounds.height).max(0.0);
                let new_scroll =
                    (this.inner.scroll_y.get() + amount).clamp(0.0, max_scroll);
                this.inner.scroll_y.set(new_scroll);
                this.inner.mark_dirty();
                shell.request_redraw();
            }
        }
        _ => {}
    }
}
