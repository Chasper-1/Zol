// ═══════════════════════════════════════════════════════════════════════
// ⚠  ВАЖНО: НИКОГДА НЕ ДЕЛАЙ СЛЕДУЮЩЕГО:
// ⚠
// ⚠  1. НЕ вызывай db.load_system_fonts() — это загружает ВСЕ шрифты
// ⚠     системы (300-2000+ файлов, 100MB-1GB+ памяти).
// ⚠     Вместо этого загружай только нужные семейства через
// ⚠     db.load_font_file() или db.load_fonts_dir().
// ⚠
// ⚠  2. НЕ создавай SwashCache без лимита — он растёт бесконтрольно.
// ⚠     Используй after_shape() для периодической очистки.
// ⚠
// ⚠  3. НЕ храни ссылку на FontGlobal вне lock() — deadlock + утечка.
// ⚠
// ⚠  Причина: OOM-краш на больших документах.
// ═══════════════════════════════════════════════════════════════════════

use std::sync::{Mutex, OnceLock};

use cosmic_text::fontdb;

pub(crate) struct FontGlobal {
    pub font_system: cosmic_text::FontSystem,
    pub swash_cache: cosmic_text::SwashCache,
    /// Счётчик вызовов shape для периодической очистки SwashCache.
    pub shape_count: u64,
}

/// Максимальное количество вызовов shape между очистками SwashCache.
const SWASH_CACHE_RESET_INTERVAL: u64 = 100;

/// Глобальный синглтон. Инициализируется один раз в [`init()`].
static GLOBAL: OnceLock<Mutex<FontGlobal>> = OnceLock::new();

/// Получить ссылку на глобальный мьютекс (паникует, если не инициализирован).
pub fn lock() -> &'static Mutex<FontGlobal> {
    GLOBAL.get().expect("font::init() must be called first")
}

/// Проинициализировать глобальный `FontSystem`.
pub fn init() {
    GLOBAL.get_or_init(|| {
        let mut db = fontdb::Database::new();

        // ⚠ НЕ ВЫЗЫВАЙ db.load_system_fonts() — см. предупреждение выше.
        //
        // Загружаем только базовые семейства шрифтов.
        // Если fc-match не сработал — пытаемся загрузить из стандартных
        // системных директорий, но не рекурсивно и только известные файлы.
        let families = ["sans-serif", "serif", "monospace"];
        for family in &families {
            if let Some(path) = resolve_font_file(family) {
                db.load_font_file(&path).ok();
            }
        }

        // Если после загрузки нет ни одного шрифта — fallback на известные
        // системные директории (только чтобы был хоть какой-то шрифт).
        if db.faces().count() == 0 {
            for dir in &["/usr/share/fonts", "/usr/local/share/fonts"] {
                db.load_fonts_dir(dir);
                if db.faces().count() > 0 {
                    break;
                }
            }
        }

        let font_system = cosmic_text::FontSystem::new_with_locale_and_db(
            "en".to_string(),
            db,
        );

        Mutex::new(FontGlobal {
            font_system,
            swash_cache: cosmic_text::SwashCache::new(),
            shape_count: 0,
        })
    });
}

/// Периодический сброс SwashCache для предотвращения бесконтрольного роста.
pub fn after_shape() {
    let mut guard = lock().lock().unwrap();
    guard.shape_count += 1;
    if guard.shape_count >= SWASH_CACHE_RESET_INTERVAL {
        guard.swash_cache = cosmic_text::SwashCache::new();
        guard.shape_count = 0;
    }
}

/// Найти файл шрифта для семейства через fc-match.
fn resolve_font_file(family: &str) -> Option<std::path::PathBuf> {
    let out = std::process::Command::new("fc-match")
        .args(["-f", "%{file}", family])
        .output()
        .ok()?;
    if out.status.success() {
        let s = std::str::from_utf8(&out.stdout).ok()?.trim();
        if !s.is_empty() {
            return Some(std::path::PathBuf::from(s));
        }
    }
    None
}
