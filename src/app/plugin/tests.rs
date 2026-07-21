use super::*;

#[test]
fn plugin_manager_creates_engine() {
    let pm = PluginManager::new();
    // просто проверяем что создаётся без паники
    let _ = pm.engine();
}
