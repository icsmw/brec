macro_rules! impl_wasm_simple {
    ($($ty:ty => $hint:expr),* $(,)?) => {
        $(
            impl WasmConvert for $ty {
                fn to_wasm_value(&self) -> Result<wasm_bindgen::JsValue, Error> {
                    Ok(wasm_bindgen::JsValue::from(*self))
                }

                fn from_wasm_value(value: wasm_bindgen::JsValue) -> Result<Self, Error> {
                    let raw = value
                        .as_f64()
                        .ok_or_else(|| WasmError::invalid_field($hint, "expected Number"))?;
                    if !raw.is_finite() || raw.fract() != 0.0 {
                        return Err(WasmError::invalid_field($hint, "expected finite integer Number"));
                    }
                    let min = <$ty>::MIN as f64;
                    let max = <$ty>::MAX as f64;
                    if raw < min || raw > max {
                        return Err(WasmError::invalid_field($hint, "integer is out of range"));
                    }
                    Ok(raw as $ty)
                }
            }
        )*
    };
}

macro_rules! impl_wasm_bigint_signed {
    ($( $ty:ty => ($hint:expr, $id:expr) ),* $(,)?) => {
        $(
            impl WasmConvert for $ty {
                fn to_wasm_value(&self) -> Result<wasm_bindgen::JsValue, Error> {
                    Ok(js_sys::BigInt::from(*self).into())
                }

                fn from_wasm_value(value: wasm_bindgen::JsValue) -> Result<Self, Error> {
                    if !value.is_bigint() {
                        return Err(Error::Wasm(WasmError::InvalidField(
                            $id.to_string(),
                            "expected BigInt".to_owned(),
                        )));
                    }
                    let big: js_sys::BigInt = value
                        .dyn_into::<js_sys::BigInt>()
                        .map_err(|_| WasmError::invalid_field($hint, "failed to cast to BigInt"))?;
                    <$ty>::try_from(big).map_err(|_| {
                        Error::Wasm(WasmError::InvalidField(
                            $id.to_string(),
                            "BigInt is out of range".to_owned(),
                        ))
                    })
                }
            }
        )*
    };
}

macro_rules! impl_wasm_bigint_unsigned {
    ($( $ty:ty => ($hint:expr, $id:expr) ),* $(,)?) => {
        $(
            impl WasmConvert for $ty {
                fn to_wasm_value(&self) -> Result<wasm_bindgen::JsValue, Error> {
                    Ok(js_sys::BigInt::from(*self).into())
                }

                fn from_wasm_value(value: wasm_bindgen::JsValue) -> Result<Self, Error> {
                    if !value.is_bigint() {
                        return Err(Error::Wasm(WasmError::InvalidField(
                            $id.to_string(),
                            "expected BigInt".to_owned(),
                        )));
                    }
                    let big: js_sys::BigInt = value
                        .dyn_into::<js_sys::BigInt>()
                        .map_err(|_| WasmError::invalid_field($hint, "failed to cast to BigInt"))?;
                    <$ty>::try_from(big).map_err(|_| {
                        Error::Wasm(WasmError::InvalidField(
                            $id.to_string(),
                            "BigInt is out of range".to_owned(),
                        ))
                    })
                }
            }
        )*
    };
}
