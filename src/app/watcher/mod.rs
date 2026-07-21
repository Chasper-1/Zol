use std::path::PathBuf;

use crossbeam_channel::{unbounded, Receiver, Sender};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher as _};

/// Событие файлового вотчера.
#[derive(Debug, Clone)]
pub enum FileEvent {
    ConfigChanged,
    PluginChanged,
    ThemeChanged,
    Other(PathBuf),
}

/// Наблюдатель за файлами конфигов/плагинов.
pub struct FileWatcher {
    _watcher: RecommendedWatcher,
    rx: Receiver<FileEvent>,
}

impl FileWatcher {
    pub fn new(watch_dir: PathBuf) -> Self {
        let (tx, rx) = unbounded();

        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    let fe = match event.kind {
                        EventKind::Modify(_) | EventKind::Create(_) => {
                            let path = event.paths.first().cloned().unwrap_or_default();
                            let name = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
                            if name.starts_with("config") {
                                FileEvent::ConfigChanged
                            } else if name.starts_with("plugin") {
                                FileEvent::PluginChanged
                            } else if name.starts_with("theme") {
                                FileEvent::ThemeChanged
                            } else {
                                FileEvent::Other(path)
                            }
                        }
                        _ => return,
                    };
                    let _ = tx.send(fe);
                }
            },
            Config::default(),
        )
        .expect("failed to create file watcher");

        let _ = watcher.watch(&watch_dir, RecursiveMode::NonRecursive);

        Self {
            _watcher: watcher,
            rx,
        }
    }

    pub fn events(&self) -> &Receiver<FileEvent> {
        &self.rx
    }
}

#[cfg(test)]
mod tests;
