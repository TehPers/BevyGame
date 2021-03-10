/// A WASM-FFI safe Option<T> enum, using a 32-bit "signed" tag (i32).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[repr(C, i32)]
pub enum WasmOption<T> {
    None,
    Some(T),
}

impl<T> Default for WasmOption<T> {
    fn default() -> Self {
        WasmOption::None
    }
}

impl<T> From<Option<T>> for WasmOption<T> {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(inner) => WasmOption::Some(inner),
            None => WasmOption::None,
        }
    }
}

impl<T> From<WasmOption<T>> for Option<T> {
    fn from(value: WasmOption<T>) -> Self {
        match value {
            WasmOption::Some(inner) => Some(inner),
            WasmOption::None => None,
        }
    }
}
