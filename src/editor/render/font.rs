//! Глобальный доступ к `cosmic_text::FontSystem` и `cosmic_text::SwashCache`.
//!
//! Оба создаются один раз при старте. `FontSystem` — для шейпинга,
//! `SwashCache` — для растрирования глифов.
//!
//! # Poisoning
//! Все `lock()` вызовы обрабатывают отравление мьютекса через
//! `PoisonError::into_inner()` — это безопасно, т.к. единственное
//! состояние — `Option<FontSystem>`, и после паники мы просто
//! получаем `None` или `Some`.

use std::sync::{Mutex, PoisonError};

static FONT_SYSTEM: Mutex<Option<cosmic_text::FontSystem>> = Mutex::new(None);
static SWASH_CACHE: Mutex<Option<cosmic_text::SwashCache>> = Mutex::new(None);

/// Инициализировать глобальный `FontSystem` и `SwashCache`.
/// Вызывается один раз при старте приложения.
/// Безопасно вызывать многократно — повторные вызовы игнорируются.
pub fn init() {
    let mut fs_lock = FONT_SYSTEM.lock().unwrap_or_else(PoisonError::into_inner);
    if fs_lock.is_none() {
        *fs_lock = Some(cosmic_text::FontSystem::new());
    }
    drop(fs_lock);

    let mut sc_lock = SWASH_CACHE.lock().unwrap_or_else(PoisonError::into_inner);
    if sc_lock.is_none() {
        *sc_lock = Some(cosmic_text::SwashCache::new());
    }
}

/// Получить `&mut FontSystem` для шейпинга.
/// Автоматически вызывает `init()` при первом обращении.
pub fn with_font_system<F, T>(f: F) -> T
where
    F: FnOnce(&mut cosmic_text::FontSystem) -> T,
{
    // init() вызываем отдельно (не внутри lock(), чтобы избежать
    // reentrancy issues с Mutex, хотя в данном случае это не нужно,
    // т.к. init() захватывает и отпускает блокировку до нашего lock()).
    init();
    let mut lock = FONT_SYSTEM
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    f(lock.as_mut().expect("FontSystem not initialized after init()"))
}

/// Получить `&mut SwashCache` для растрирования.
/// Автоматически вызывает `init()` при первом обращении.
pub fn with_swash_cache<F, T>(f: F) -> T
where
    F: FnOnce(&mut cosmic_text::SwashCache) -> T,
{
    init();
    let mut lock = SWASH_CACHE
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    f(lock.as_mut().expect("SwashCache not initialized after init()"))
}
