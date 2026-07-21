use super::*;

#[test]
fn file_watcher_channel_works() {
    let dir = std::env::temp_dir().join("zol_watcher_test");
    let _ = std::fs::create_dir_all(&dir);

    let watcher = FileWatcher::new(dir.clone());

    // создаём файл — это должно породить событие
    let test_file = dir.join("config.ron");
    std::fs::write(&test_file, "()").ok();

    // даём время вотчеру заметить
    std::thread::sleep(std::time::Duration::from_millis(200));

    // проверяем что хотя бы одно событие пришло
    let rx = watcher.events();
    let events: Vec<_> = rx.try_iter().collect();
    assert!(!events.is_empty(), "должно быть хотя бы одно событие");

    let _ = std::fs::remove_dir_all(&dir);
}
