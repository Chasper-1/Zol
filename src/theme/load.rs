use rhai::{Engine, Map, Dynamic};

pub fn load_theme(path: &str) -> Map {
    let engine = Engine::new();
    let result: Map = engine.eval_file(path).unwrap();
    result
}