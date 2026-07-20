//! Iced-приложение Flint Notes (v0.14).
//!
//! Постепенно заменяет egui-версию в `app.rs` / `run.rs`.

use std::cell::RefCell;

use iced::widget::{container, scrollable};
use iced::{Element, Task, Theme};

use crate::editor::render::ShapedDocument;
use crate::gui::iced_editor::{editor_element, EditorInner};

/// Сообщения приложения.
#[derive(Debug, Clone)]
pub enum Message {
    Tick,
}

/// Состояние приложения.
struct AppState {
    inner: RefCell<EditorInner>,
}

fn boot() -> (AppState, Task<Message>) {
    let metrics = cosmic_text::Metrics::new(14.0, 19.6);
    let empty_buffer = cosmic_text::Buffer::new_empty(metrics);
    let shaped_doc = ShapedDocument::new(empty_buffer);

    let app = AppState {
        inner: RefCell::new(EditorInner::new(String::new(), shaped_doc)),
    };
    (app, Task::none())
}

fn update(_app_state: &mut AppState, _message: Message) {
    // TODO: команды редактора
}

fn view(app_state: &AppState) -> Element<'_, Message, Theme, iced::Renderer> {
    let editor = editor_element(&app_state.inner);

    container(scrollable(container(editor)))
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .into()
}

/// Запустить Iced-приложение.
pub fn run() -> iced::Result {
    iced::application(boot, update, view)
        .window_size(iced::Size::new(1200.0, 800.0))
        .run()
}
