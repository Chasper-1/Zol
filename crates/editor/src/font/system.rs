use std::sync::PoisonError;

use super::global::{lock, FontGlobal};

/// Доступ к `FontSystem` для шейпинга.
pub fn with_font_system<F, T>(f: F) -> T
where
    F: FnOnce(&mut cosmic_text::FontSystem) -> T,
{
    let mut guard = lock().lock().unwrap_or_else(PoisonError::into_inner);
    f(&mut guard.font_system)
}

/// Доступ к `SwashCache` для растрирования.
pub fn with_swash_cache<F, T>(f: F) -> T
where
    F: FnOnce(&mut cosmic_text::SwashCache) -> T,
{
    let mut guard = lock().lock().unwrap_or_else(PoisonError::into_inner);
    f(&mut guard.swash_cache)
}

/// Доступ к `FontSystem` и `SwashCache` одновременно.
pub fn with_font_and_cache<F, T>(f: F) -> T
where
    F: FnOnce(&mut cosmic_text::FontSystem, &mut cosmic_text::SwashCache) -> T,
{
    let mut guard = lock().lock().unwrap_or_else(PoisonError::into_inner);
    let FontGlobal { font_system, swash_cache } = &mut *guard;
    f(font_system, swash_cache)
}
