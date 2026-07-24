fn current_rss_kb() -> Option<usize> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    let line = status.lines().find(|l| l.starts_with("VmRSS:"))?;
    let val = line.split(':').nth(1)?.trim();
    let kb = val.split_whitespace().next()?;
    kb.parse::<usize>().ok()
}

fn main() {
    let memory = std::env::args().any(|a| a == "--memory");

    if memory {
        let rss = current_rss_kb().map(|k| format!("{:>8} kB", k)).unwrap_or_else(|| "  неизвестно".into());
        eprintln!("[mem] старт: {rss}");

        // Опросник каждые 200 мс
        let start = std::time::Instant::now();
        std::thread::spawn(move || loop {
            std::thread::sleep(std::time::Duration::from_millis(200));
            let t = start.elapsed();
            let rss = current_rss_kb().map(|k| format!("{:>8} kB", k)).unwrap_or_else(|| "  неизвестно".into());
            eprintln!("[mem] {:>4}.{:03}s: {rss}", t.as_secs(), t.subsec_millis());
        });
    }

    match gui::app_iced::run() {
        Ok(_) => {}
        Err(e) => eprintln!("[Zol] Iced завершился с ошибкой: {:?}", e),
    }

    if memory {
        let rss = current_rss_kb().map(|k| format!("{:>8} kB", k)).unwrap_or_else(|| "  неизвестно".into());
        eprintln!("[mem] выход: {rss}");
    }
}
