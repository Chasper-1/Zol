fn main() {
    match gui::app_iced::run() {
        Ok(_) => {}
        Err(e) => eprintln!("[Zol] Iced завершился с ошибкой: {:?}", e),
    }
}
