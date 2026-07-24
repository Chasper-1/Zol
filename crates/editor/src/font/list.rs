use std::sync::PoisonError;

use super::global::lock;

/// Список всех доступных семейств шрифтов.
pub fn list_families() -> Vec<String> {
    let guard = lock().lock().unwrap_or_else(PoisonError::into_inner);
    let mut families: Vec<String> = guard
        .font_system
        .db()
        .faces()
        .flat_map(|f| f.families.iter().map(|(name, _)| name.clone()))
        .collect();
    families.sort();
    families.dedup();
    families
}

/// Пересканировать системные шрифты.
pub fn reload_system_fonts() {
    let mut guard = lock().lock().unwrap_or_else(PoisonError::into_inner);
    guard.font_system.db_mut().load_system_fonts();
}
