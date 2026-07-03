// src/main.rs
use gtk4::Application;
use gtk4::prelude::*;

// Подключаем изолированный модуль из папки gui
mod gui;

fn main() {
    let app = Application::builder()
        .application_id("com.yourusername.flint")
        .build();

    // Запускаем временный интерфейс
    app.connect_activate(gui::build_ui);

    app.run();
}
