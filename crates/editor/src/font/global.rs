use std::sync::{Mutex, OnceLock};

use cosmic_text::fontdb;

pub(crate) struct FontGlobal {
    pub font_system: cosmic_text::FontSystem,
    pub swash_cache: cosmic_text::SwashCache,
}

/// Глобальный синглтон. Инициализируется один раз в [`init()`].
static GLOBAL: OnceLock<Mutex<FontGlobal>> = OnceLock::new();

/// Получить ссылку на глобальный мьютекс (паникует, если не инициализирован).
pub fn lock() -> &'static Mutex<FontGlobal> {
    GLOBAL.get().expect("font::init() must be called first")
}

/// Проинициализировать глобальный `FontSystem`.
pub fn init() {
    let _ = GLOBAL.get_or_init(|| {
        let mut db = fontdb::Database::new();
        db.load_system_fonts();

        let font_system = cosmic_text::FontSystem::new_with_locale_and_db(
            "en".to_string(),
            db,
        );

        Mutex::new(FontGlobal {
            font_system,
            swash_cache: cosmic_text::SwashCache::new(),
        })
    });
}
