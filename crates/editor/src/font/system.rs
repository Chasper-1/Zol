// ═══════════════════════════════════════════════════════════════════════
// ⚠  ВАЖНО: НИКОГДА НЕ ДЕЛАЙ СЛЕДУЮЩЕГО:
// ⚠
// ⚠  - Не клонируй FontSystem — он содержит базу всех шрифтов.
// ⚠  - Не создавай новый FontSystem в каждом вызове — init() один раз.
// ⚠  - Не храни MutexGuard (результат lock()) дольше одного вызова —
// ⚠    это заблокирует все остальные потоки.
// ⚠
// ⚠  Причина: каждая копия FontSystem = дублирование всей базы шрифтов.
// ═══════════════════════════════════════════════════════════════════════

use std::sync::PoisonError;

use super::global::{FontGlobal, lock};

// Доступ к `FontSystem` для шейпинга.
pub fn with_font_system<F, T>(f: F) -> T
where
    F: FnOnce(&mut cosmic_text::FontSystem) -> T,
{
    let mut guard = lock().lock().unwrap_or_else(PoisonError::into_inner);
    f(&mut guard.font_system)
}

// Доступ к `SwashCache` для растрирования.
pub fn with_swash_cache<F, T>(f: F) -> T
where
    F: FnOnce(&mut cosmic_text::SwashCache) -> T,
{
    let mut guard = lock().lock().unwrap_or_else(PoisonError::into_inner);
    f(&mut guard.swash_cache)
}

// Доступ к `FontSystem` и `SwashCache` одновременно.
pub fn with_font_and_cache<F, T>(f: F) -> T
where
    F: FnOnce(&mut cosmic_text::FontSystem, &mut cosmic_text::SwashCache) -> T,
{
    let mut guard = lock().lock().unwrap_or_else(PoisonError::into_inner);
    let FontGlobal {
        font_system,
        swash_cache,
        ..
    } = &mut *guard;
    let result = f(font_system, swash_cache);
    drop(guard);
    crate::font::after_shape();
    result
}
