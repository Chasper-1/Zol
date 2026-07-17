mod api;
mod editor;
mod gui;
mod mdplus;

fn main() -> eframe::Result {
    // Просто делегируем запуск в изолированный модульgui
    gui::run::run_app()
}
