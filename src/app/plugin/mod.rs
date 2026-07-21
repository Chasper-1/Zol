use std::path::Path;

/// Менеджер Rhai-плагинов.
#[derive(Debug)]
pub struct PluginManager {
    engine: rhai::Engine,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            engine: rhai::Engine::new(),
        }
    }

    pub fn engine(&self) -> &rhai::Engine {
        &self.engine
    }

    pub fn load(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let script = std::fs::read_to_string(path)?;
        self.engine.run(&script)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
