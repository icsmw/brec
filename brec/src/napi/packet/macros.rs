macro_rules! impl_napi_simple {
    ($($ty:ty => $hint:expr),* $(,)?) => {
        $(
            impl NapiConvert for $ty {
                fn to_napi_value<'env>(&self, env: &'env Env) -> Result<Unknown<'env>, Error> {
                    self.to_owned()
                        .into_unknown(env)
                        .map_err(|err| NapiError::invalid_field($hint, err))
                }

                fn from_napi_value(_env: &Env, value: Unknown<'_>) -> Result<Self, Error> {
                    let v: $ty = napi_from_unknown($hint, value)?;
                    Ok(v)
                }
            }
        )*
    };
}

macro_rules! impl_napi_bigint_signed {
    ($( $ty:ty => ($hint:expr, $id:expr, $getter:ident) ),* $(,)?) => {
        $(
            impl NapiConvert for $ty {
                fn to_napi_value<'env>(&self, env: &'env Env) -> Result<Unknown<'env>, Error> {
                    let v = BigInt::from(*self);
                    v.into_unknown(env)
                        .map_err(|err| NapiError::invalid_field($hint, err))
                }

                fn from_napi_value(_env: &Env, value: Unknown<'_>) -> Result<Self, Error> {
                    match value
                        .get_type()
                        .map_err(|err| NapiError::invalid_field($hint, err))?
                    {
                        ValueType::BigInt => {
                            let raw: BigInt = napi_from_unknown($hint, value)?;
                            let (v, lossless) = raw.$getter();
                            if !lossless {
                                return Err(Error::Napi(NapiError::InvalidField(
                                    $id.to_string(),
                                    "BigInt is out of range".to_owned(),
                                )));
                            }
                            Ok(v)
                        }
                        other => Err(Error::Napi(NapiError::InvalidField(
                            $id.to_string(),
                            format!("expected BigInt, got {:?}", other),
                        ))),
                    }
                }
            }
        )*
    };
}

macro_rules! impl_napi_bigint_unsigned {
    ($( $ty:ty => ($hint:expr, $id:expr, $getter:ident) ),* $(,)?) => {
        $(
            impl NapiConvert for $ty {
                fn to_napi_value<'env>(&self, env: &'env Env) -> Result<Unknown<'env>, Error> {
                    let v = BigInt::from(*self);
                    v.into_unknown(env)
                        .map_err(|err| NapiError::invalid_field($hint, err))
                }

                fn from_napi_value(_env: &Env, value: Unknown<'_>) -> Result<Self, Error> {
                    match value
                        .get_type()
                        .map_err(|err| NapiError::invalid_field($hint, err))?
                    {
                        ValueType::BigInt => {
                            let raw: BigInt = napi_from_unknown($hint, value)?;
                            let (sign, v, lossless) = raw.$getter();
                            if sign || !lossless {
                                return Err(Error::Napi(NapiError::InvalidField(
                                    $id.to_string(),
                                    "BigInt is out of range".to_owned(),
                                )));
                            }
                            Ok(v)
                        }
                        other => Err(Error::Napi(NapiError::InvalidField(
                            $id.to_string(),
                            format!("expected BigInt, got {:?}", other),
                        ))),
                    }
                }
            }
        )*
    };
}
