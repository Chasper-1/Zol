//! Трейты конвертации значений в/из [`HandleValue`].

use super::HandleValue;

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
impl IntoHandleValue for crate::theme::color::Rgba {
    fn into_handle_value(self) -> HandleValue {
        HandleValue::Rgba(self)
    }
}
impl FromHandleValue for crate::theme::color::Rgba {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::color::Rgba;

    #[test]
    fn float_to_from_handle() {
        let v = 42.0f32.into_handle_value();
        assert_eq!(f32::from_handle_value(&v), Some(42.0));
    }

    #[test]
    fn rgba_to_from_handle() {
        let c = Rgba::new(0.5, 0.3, 0.2);
        let v = c.into_handle_value();
        assert_eq!(Rgba::from_handle_value(&v), Some(c));
    }

    #[test]
    fn string_to_from_handle() {
        let s = "hello".to_string();
        let v = s.into_handle_value();
        assert_eq!(String::from_handle_value(&v), Some("hello".to_string()));
    }

    #[test]
    fn wrong_type_returns_none() {
        let v = HandleValue::Float(1.0);
        assert_eq!(Rgba::from_handle_value(&v), None);
        assert_eq!(String::from_handle_value(&v), None);
    }
}
