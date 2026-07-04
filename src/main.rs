// src/main.rs
mod editor;
mod gui;

fn main() -> eframe::Result {
    // Просто делегируем запуск в изолированный модульgui
    gui::run_app()
}
