// src/main.rs
use gtk::Application;
use gtk::prelude::*;

// Подключаем изолированный модуль из папки webkit
mod webkit;

fn main() {
    let app = Application::builder()
        .application_id("com.yourusername.flint")
        .build();

    // Запускаем временный интерфейс
    app.connect_activate(webkit::build_ui);

    app.run();
}
