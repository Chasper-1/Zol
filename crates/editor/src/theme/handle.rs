//! Система ручек темы.
//!
//! Каждая настройка темы — это [`Handle<T>`] с категорией и именем.
//! Например, `Handle::<f32>::new("editor", "padding", 10.0)`.
//!
//! [`ThemeSystem`] хранит текущие значения всех ручек и может быть
//! прочитан/изменён через Rhai по строковому ключу `"category.name"`.

use std::collections::HashMap;

/// Описание одной ручки темы.
///
/// - `category` — неймспейс (`"editor"`, `"text"`, …)
/// - `name` — имя настройки (`"padding"`, `"color"`, …)
/// - `default` — значение по умолчанию
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
    Rgba(super::color::Rgba),
    String(String),
}

/// Система управления ручками темы.
///
/// Позволяет читать и менять настройки по строковому пути,
/// совместимому с Rhai: `"editor.padding"`, `"text.color"`, …
#[derive(Debug, Clone)]
pub struct ThemeSystem {
    values: HashMap<String, HandleValue>,
}

impl ThemeSystem {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    /// Установить значение ручки (типизированно).
    pub fn set<T: IntoHandleValue>(&mut self, handle: &Handle<T>, value: T) {
        self.values
            .insert(handle.key(), value.into_handle_value());
    }

    /// Получить значение ручки (типизированно).
    /// Если ручка ещё не задана, возвращает `None`.
    pub fn get<T: FromHandleValue>(&self, handle: &Handle<T>) -> Option<T> {
        self.values
            .get(&handle.key())
            .and_then(|v| T::from_handle_value(v))
    }

    /// Получить значение или дефолт.
    pub fn get_or_default<T: FromHandleValue + Clone + IntoHandleValue>(&self, handle: &Handle<T>) -> T {
        self.get(handle).unwrap_or_else(|| {
            T::from_handle_value(&handle.default.clone().into_handle_value())
                .expect("Handle::default должен конвертироваться")
        })
    }

    /// Установить значение по строковому пути (из Rhai).
    pub fn set_raw(&mut self, path: &str, value: HandleValue) {
        self.values.insert(path.to_string(), value);
    }

    /// Получить значение по строковому пути (для Rhai).
    pub fn get_raw(&self, path: &str) -> Option<HandleValue> {
        self.values.get(path).cloned()
    }

    /// Сбросить все значения к дефолтам.
    pub fn reset(&mut self) {
        self.values.clear();
    }
}

impl Default for ThemeSystem {
    fn default() -> Self {
        Self::new()
    }
}

// —————— трейты конвертации ——————

/// Преобразование значения в [`HandleValue`].
pub trait IntoHandleValue {
    fn into_handle_value(self) -> HandleValue;
}

/// Обратное преобразование из [`HandleValue`].
pub trait FromHandleValue: Sized {
    fn from_handle_value(v: &HandleValue) -> Option<Self>;
}

// float
impl IntoHandleValue for f32 {
    fn into_handle_value(self) -> HandleValue {
        HandleValue::Float(self)
    }
}
impl FromHandleValue for f32 {
    fn from_handle_value(v: &HandleValue) -> Option<Self> {
        match v {
            HandleValue::Float(f) => Some(*f),
            _ => None,
        }
    }
}

// Rgba
impl IntoHandleValue for super::color::Rgba {
    fn into_handle_value(self) -> HandleValue {
        HandleValue::Rgba(self)
    }
}
impl FromHandleValue for super::color::Rgba {
    fn from_handle_value(v: &HandleValue) -> Option<Self> {
        match v {
            HandleValue::Rgba(c) => Some(*c),
            _ => None,
        }
    }
}

// String
impl IntoHandleValue for String {
    fn into_handle_value(self) -> HandleValue {
        HandleValue::String(self)
    }
}
impl FromHandleValue for String {
    fn from_handle_value(v: &HandleValue) -> Option<Self> {
        match v {
            HandleValue::String(s) => Some(s.clone()),
            _ => None,
        }
    }
}
