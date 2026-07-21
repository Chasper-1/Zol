//! Iced-приложение Zol (v0.14).
//!
//! Постепенно заменяет egui-версию в `app.rs` / `run.rs`.

use iced::widget::container;
use iced::{Element, Task, Theme};

use crate::gui::iced_editor::{editor_element, EditorInner};

/// Сообщения приложения.
#[derive(Debug, Clone)]
pub enum Message {
    Tick,
}

/// Состояние приложения.
struct AppState {
    inner: EditorInner,
}

fn boot() -> (AppState, Task<Message>) {
    // Загружаем содержимое заметки (или пусто, если файла нет).
    let content = std::fs::read_to_string("notes.zoll").unwrap_or_default();

    let app = AppState {
        inner: EditorInner::new(content),
    };
    (app, Task::none())
}

fn update(_app_state: &mut AppState, _message: Message) {
    // TODO: команды редактора
}

fn view(app_state: &AppState) -> Element<'_, Message, Theme, iced::Renderer> {
    let editor = editor_element(&app_state.inner);

    container(editor)
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
