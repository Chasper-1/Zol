mod api;
mod document;
mod editor;
mod gui;
mod zml;

fn main() {
    match gui::app_iced::run() {
        Ok(_) => {}
        Err(e) => eprintln!("[Zol] Iced завершился с ошибкой: {:?}", e),
    }
}
