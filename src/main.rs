mod api;
mod editor;
mod gui;
mod zml;

fn main() {
    // Если передан аргумент --iced — запускаем Iced-версию
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--iced") {
        match gui::app_iced::run() {
            Ok(_) => {}
            Err(e) => eprintln!("[Zol] Iced завершился с ошибкой: {:?}", e),
        }
    } else {
        // egui-версия (основной режим до полного переезда)
        gui::run::run_app().expect("egui run failed");
    }
}
