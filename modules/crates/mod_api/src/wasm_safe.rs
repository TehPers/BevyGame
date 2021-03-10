pub trait WasmSafe {}

macro_rules! impl_wasmsafe {
    [$($t:ty),*] => {
        $(impl WasmSafe for $t {})
    }
}

impl_wasmsafe![u8, i8, u16, i16, u32, i32, u64, i64, f32, f64];

impl<T> WasmSafe for *const T {}
impl<T> WasmSafe for *mut T {}
impl<T> WasmSafe for Option<*const T> {}
impl<T> WasmSafe for Option<*mut T> {}
