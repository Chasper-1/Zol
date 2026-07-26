pub mod convert;

/// Описание одной ручки темы.
#[derive(Debug, Clone)]
pub struct Handle<T> {
    pub category: &'static str,
    pub name: &'static str,
    pub default: T,
}

impl<T> Handle<T> {
    /// Строковый ключ для использования в `ThemeSystem`.
    pub fn key(&self) -> String {
        format!("{}.{}", self.category, self.name)
    }
}

/// Типизированное значение ручки внутри [`ThemeSystem`].
#[derive(Debug, Clone)]
pub enum HandleValue {
    Float(f32),
    Rgba(crate::theme::color::Rgba),
    String(String),
}

/// Система управления ручками темы.
#[derive(Debug, Clone)]
pub struct ThemeSystem {
    values: std::collections::HashMap<String, HandleValue>,
}

impl ThemeSystem {
    pub fn new() -> Self {
        Self {
            values: std::collections::HashMap::new(),
        }
    }

    pub fn set<T: convert::IntoHandleValue>(&mut self, handle: &Handle<T>, value: T) {
        self.values
            .insert(handle.key(), value.into_handle_value());
    }

    pub fn get<T: convert::FromHandleValue>(&self, handle: &Handle<T>) -> Option<T> {
        self.values
            .get(&handle.key())
            .and_then(|v| T::from_handle_value(v))
    }

    pub fn get_or_default<T: convert::FromHandleValue + Clone + convert::IntoHandleValue>(&self, handle: &Handle<T>) -> T {
        self.get(handle).unwrap_or_else(|| {
            T::from_handle_value(&handle.default.clone().into_handle_value())
                .expect("Handle::default должен конвертироваться")
        })
    }

    pub fn set_raw(&mut self, path: &str, value: HandleValue) {
        self.values.insert(path.to_string(), value);
    }

    pub fn get_raw(&self, path: &str) -> Option<HandleValue> {
        self.values.get(path).cloned()
    }

    pub fn reset(&mut self) {
        self.values.clear();
    }
}

impl Default for ThemeSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::color::Rgba;
    use crate::theme::registry::handles::{PADDING, BACKGROUND, TEXT_FONT};

    #[test]
    fn theme_system_set_get_float() {
        let mut ts = ThemeSystem::new();
        ts.set(&PADDING, 15.0f32);
        assert_eq!(ts.get(&PADDING), Some(15.0f32));
    }

    #[test]
    fn theme_system_get_default() {
        let ts = ThemeSystem::new();
        assert_eq!(ts.get_or_default(&PADDING), 10.0f32);
    }

    #[test]
    fn theme_system_set_get_rgba() {
        let mut ts = ThemeSystem::new();
        let color = Rgba::new(1.0, 0.0, 0.0);
        ts.set(&BACKGROUND, color);
        assert_eq!(ts.get(&BACKGROUND), Some(color));
    }

    #[test]
    fn theme_system_set_get_string() {
        let mut ts = ThemeSystem::new();
        ts.set(&TEXT_FONT, "monospace".to_string());
        assert_eq!(ts.get(&TEXT_FONT), Some("monospace".to_string()));
    }

    #[test]
    fn theme_system_get_missing() {
        let ts = ThemeSystem::new();
        assert_eq!(ts.get::<f32>(&PADDING), None);
    }

    #[test]
    fn theme_system_overwrite() {
        let mut ts = ThemeSystem::new();
        ts.set(&PADDING, 5.0f32);
        ts.set(&PADDING, 20.0f32);
        assert_eq!(ts.get(&PADDING), Some(20.0f32));
    }

    #[test]
    fn theme_system_reset() {
        let mut ts = ThemeSystem::new();
        ts.set(&PADDING, 15.0f32);
        ts.reset();
        assert_eq!(ts.get::<f32>(&PADDING), None);
    }

    #[test]
    fn theme_system_set_raw() {
        let mut ts = ThemeSystem::new();
        ts.set_raw("editor.padding", HandleValue::Float(12.0));
        let v = ts.get_raw("editor.padding");
        assert!(matches!(v, Some(HandleValue::Float(f)) if (f - 12.0).abs() < 0.001));
    }
}
