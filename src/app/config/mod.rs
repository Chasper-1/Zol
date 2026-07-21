use std::path::PathBuf;

/// Основной конфиг приложения.
#[derive(Debug, Clone)]
pub struct Config {
    pub theme: String,
    pub font_size: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: String::from("default"),
            font_size: 14.0,
        }
    }
}

/// Путь к директории конфигов.
pub fn config_dir() -> PathBuf {
    directories::ProjectDirs::from("com", "zol", "Zol")
        .map(|d| d.config_dir().to_path_buf())
        .unwrap_or_else(|| {
            // fallback для тестов и экзотических платформ
            std::env::current_dir()
                .unwrap_or_default()
                .join(".zol")
        })
    // TODO: создать директорию если нет
    // TODO: читать config.ron
    // TODO: записывать config.ron
}

#[cfg(test)]
mod tests;
