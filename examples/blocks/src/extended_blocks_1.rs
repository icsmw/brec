use std::fs::read;

use rand::random;

#[repr(C)]
struct PayloadA {
    pub str: String,
    pub num: u32,
    pub list: Vec<String>,
}
#[doc(hidden)]
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl<'de> _serde::Deserialize<'de> for PayloadA {
        fn deserialize<__D>(__deserializer: __D) -> _serde::__private::Result<Self, __D::Error>
        where
            __D: _serde::Deserializer<'de>,
        {
            #[allow(non_camel_case_types)]
            #[doc(hidden)]
            enum __Field {
                __field0,
                __field1,
                __field2,
                __ignore,
            }
            #[doc(hidden)]
            struct __FieldVisitor;
            #[automatically_derived]
            impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                type Value = __Field;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(__formatter, "field identifier")
                }
                fn visit_u64<__E>(self, __value: u64) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        0u64 => _serde::__private::Ok(__Field::__field0),
                        1u64 => _serde::__private::Ok(__Field::__field1),
                        2u64 => _serde::__private::Ok(__Field::__field2),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
                fn visit_str<__E>(
                    self,
                    __value: &str,
                ) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        "str" => _serde::__private::Ok(__Field::__field0),
                        "num" => _serde::__private::Ok(__Field::__field1),
                        "list" => _serde::__private::Ok(__Field::__field2),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
                fn visit_bytes<__E>(
                    self,
                    __value: &[u8],
                ) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        b"str" => _serde::__private::Ok(__Field::__field0),
                        b"num" => _serde::__private::Ok(__Field::__field1),
                        b"list" => _serde::__private::Ok(__Field::__field2),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
            }
            #[automatically_derived]
            impl<'de> _serde::Deserialize<'de> for __Field {
                #[inline]
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    _serde::Deserializer::deserialize_identifier(__deserializer, __FieldVisitor)
                }
            }
            #[doc(hidden)]
            struct __Visitor<'de> {
                marker: _serde::__private::PhantomData<PayloadA>,
                lifetime: _serde::__private::PhantomData<&'de ()>,
            }
            #[automatically_derived]
            impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                type Value = PayloadA;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(__formatter, "struct PayloadA")
                }
                #[inline]
                fn visit_seq<__A>(
                    self,
                    mut __seq: __A,
                ) -> _serde::__private::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::SeqAccess<'de>,
                {
                    let __field0 = match _serde::de::SeqAccess::next_element::<String>(&mut __seq)?
                    {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => {
                            return _serde::__private::Err(_serde::de::Error::invalid_length(
                                0usize,
                                &"struct PayloadA with 3 elements",
                            ));
                        }
                    };
                    let __field1 = match _serde::de::SeqAccess::next_element::<u32>(&mut __seq)? {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => {
                            return _serde::__private::Err(_serde::de::Error::invalid_length(
                                1usize,
                                &"struct PayloadA with 3 elements",
                            ));
                        }
                    };
                    let __field2 =
                        match _serde::de::SeqAccess::next_element::<Vec<String>>(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(_serde::de::Error::invalid_length(
                                    2usize,
                                    &"struct PayloadA with 3 elements",
                                ));
                            }
                        };
                    _serde::__private::Ok(PayloadA {
                        str: __field0,
                        num: __field1,
                        list: __field2,
                    })
                }
                #[inline]
                fn visit_map<__A>(
                    self,
                    mut __map: __A,
                ) -> _serde::__private::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::MapAccess<'de>,
                {
                    let mut __field0: _serde::__private::Option<String> = _serde::__private::None;
                    let mut __field1: _serde::__private::Option<u32> = _serde::__private::None;
                    let mut __field2: _serde::__private::Option<Vec<String>> =
                        _serde::__private::None;
                    while let _serde::__private::Some(__key) =
                        _serde::de::MapAccess::next_key::<__Field>(&mut __map)?
                    {
                        match __key {
                            __Field::__field0 => {
                                if _serde::__private::Option::is_some(&__field0) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field("str"),
                                    );
                                }
                                __field0 =
                                    _serde::__private::Some(_serde::de::MapAccess::next_value::<
                                        String,
                                    >(
                                        &mut __map
                                    )?);
                            }
                            __Field::__field1 => {
                                if _serde::__private::Option::is_some(&__field1) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field("num"),
                                    );
                                }
                                __field1 =
                                    _serde::__private::Some(_serde::de::MapAccess::next_value::<
                                        u32,
                                    >(
                                        &mut __map
                                    )?);
                            }
                            __Field::__field2 => {
                                if _serde::__private::Option::is_some(&__field2) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field("list"),
                                    );
                                }
                                __field2 =
                                    _serde::__private::Some(_serde::de::MapAccess::next_value::<
                                        Vec<String>,
                                    >(
                                        &mut __map
                                    )?);
                            }
                            _ => {
                                let _ = _serde::de::MapAccess::next_value::<_serde::de::IgnoredAny>(
                                    &mut __map,
                                )?;
                            }
                        }
                    }
                    let __field0 = match __field0 {
                        _serde::__private::Some(__field0) => __field0,
                        _serde::__private::None => _serde::__private::de::missing_field("str")?,
                    };
                    let __field1 = match __field1 {
                        _serde::__private::Some(__field1) => __field1,
                        _serde::__private::None => _serde::__private::de::missing_field("num")?,
                    };
                    let __field2 = match __field2 {
                        _serde::__private::Some(__field2) => __field2,
                        _serde::__private::None => _serde::__private::de::missing_field("list")?,
                    };
                    _serde::__private::Ok(PayloadA {
                        str: __field0,
                        num: __field1,
                        list: __field2,
                    })
                }
            }
            #[doc(hidden)]
            const FIELDS: &'static [&'static str] = &["str", "num", "list"];
            _serde::Deserializer::deserialize_struct(
                __deserializer,
                "PayloadA",
                FIELDS,
                __Visitor {
                    marker: _serde::__private::PhantomData::<PayloadA>,
                    lifetime: _serde::__private::PhantomData,
                },
            )
        }
    }
};
#[doc(hidden)]
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl _serde::Serialize for PayloadA {
        fn serialize<__S>(
            &self,
            __serializer: __S,
        ) -> _serde::__private::Result<__S::Ok, __S::Error>
        where
            __S: _serde::Serializer,
        {
            let mut __serde_state = _serde::Serializer::serialize_struct(
                __serializer,
                "PayloadA",
                false as usize + 1 + 1 + 1,
            )?;
            _serde::ser::SerializeStruct::serialize_field(&mut __serde_state, "str", &self.str)?;
            _serde::ser::SerializeStruct::serialize_field(&mut __serde_state, "num", &self.num)?;
            _serde::ser::SerializeStruct::serialize_field(&mut __serde_state, "list", &self.list)?;
            _serde::ser::SerializeStruct::end(__serde_state)
        }
    }
};
impl brec::Signature for PayloadA {
    fn sig() -> brec::ByteBlock {
        brec::ByteBlock::Len4([92u8, 52u8, 48u8, 71u8])
    }
}
impl brec::PayloadEncode for PayloadA {
    fn encode(&self) -> std::io::Result<Vec<u8>> {
        brec::bincode::serialize(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
    }
}
impl brec::PayloadEncodeReferred for PayloadA {
    fn encode(&self) -> std::io::Result<Option<&[u8]>> {
        Ok(None)
    }
}
impl brec::PayloadDecode<PayloadA> for PayloadA {
    fn decode(buf: &[u8]) -> std::io::Result<PayloadA> {
        brec::bincode::deserialize(buf)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
    }
}
impl brec::PayloadCrc for PayloadA {}
impl brec::PayloadSize for PayloadA {
    fn size(&self) -> std::io::Result<u64> {
        brec::bincode::serialized_size(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
    }
}
impl brec::ReadPayloadFrom<PayloadA> for PayloadA {}
impl brec::TryReadPayloadFrom<PayloadA> for PayloadA {}
impl brec::TryReadPayloadFromBuffered<PayloadA> for PayloadA {}
impl brec::WritePayloadTo for PayloadA {}
impl brec::WriteVectoredPayloadTo for PayloadA {}
#[repr(C)]
struct PayloadB {
    pub str: String,
    pub num: u32,
    pub list: Vec<String>,
}
#[doc(hidden)]
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl<'de> _serde::Deserialize<'de> for PayloadB {
        fn deserialize<__D>(__deserializer: __D) -> _serde::__private::Result<Self, __D::Error>
        where
            __D: _serde::Deserializer<'de>,
        {
            #[allow(non_camel_case_types)]
            #[doc(hidden)]
            enum __Field {
                __field0,
                __field1,
                __field2,
                __ignore,
            }
            #[doc(hidden)]
            struct __FieldVisitor;
            #[automatically_derived]
            impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                type Value = __Field;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(__formatter, "field identifier")
                }
                fn visit_u64<__E>(self, __value: u64) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        0u64 => _serde::__private::Ok(__Field::__field0),
                        1u64 => _serde::__private::Ok(__Field::__field1),
                        2u64 => _serde::__private::Ok(__Field::__field2),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
                fn visit_str<__E>(
                    self,
                    __value: &str,
                ) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        "str" => _serde::__private::Ok(__Field::__field0),
                        "num" => _serde::__private::Ok(__Field::__field1),
                        "list" => _serde::__private::Ok(__Field::__field2),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
                fn visit_bytes<__E>(
                    self,
                    __value: &[u8],
                ) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        b"str" => _serde::__private::Ok(__Field::__field0),
                        b"num" => _serde::__private::Ok(__Field::__field1),
                        b"list" => _serde::__private::Ok(__Field::__field2),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
            }
            #[automatically_derived]
            impl<'de> _serde::Deserialize<'de> for __Field {
                #[inline]
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    _serde::Deserializer::deserialize_identifier(__deserializer, __FieldVisitor)
                }
            }
            #[doc(hidden)]
            struct __Visitor<'de> {
                marker: _serde::__private::PhantomData<PayloadB>,
                lifetime: _serde::__private::PhantomData<&'de ()>,
            }
            #[automatically_derived]
            impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                type Value = PayloadB;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(__formatter, "struct PayloadB")
                }
                #[inline]
                fn visit_seq<__A>(
                    self,
                    mut __seq: __A,
                ) -> _serde::__private::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::SeqAccess<'de>,
                {
                    let __field0 = match _serde::de::SeqAccess::next_element::<String>(&mut __seq)?
                    {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => {
                            return _serde::__private::Err(_serde::de::Error::invalid_length(
                                0usize,
                                &"struct PayloadB with 3 elements",
                            ));
                        }
                    };
                    let __field1 = match _serde::de::SeqAccess::next_element::<u32>(&mut __seq)? {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => {
                            return _serde::__private::Err(_serde::de::Error::invalid_length(
                                1usize,
                                &"struct PayloadB with 3 elements",
                            ));
                        }
                    };
                    let __field2 =
                        match _serde::de::SeqAccess::next_element::<Vec<String>>(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(_serde::de::Error::invalid_length(
                                    2usize,
                                    &"struct PayloadB with 3 elements",
                                ));
                            }
                        };
                    _serde::__private::Ok(PayloadB {
                        str: __field0,
                        num: __field1,
                        list: __field2,
                    })
                }
                #[inline]
                fn visit_map<__A>(
                    self,
                    mut __map: __A,
                ) -> _serde::__private::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::MapAccess<'de>,
                {
                    let mut __field0: _serde::__private::Option<String> = _serde::__private::None;
                    let mut __field1: _serde::__private::Option<u32> = _serde::__private::None;
                    let mut __field2: _serde::__private::Option<Vec<String>> =
                        _serde::__private::None;
                    while let _serde::__private::Some(__key) =
                        _serde::de::MapAccess::next_key::<__Field>(&mut __map)?
                    {
                        match __key {
                            __Field::__field0 => {
                                if _serde::__private::Option::is_some(&__field0) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field("str"),
                                    );
                                }
                                __field0 =
                                    _serde::__private::Some(_serde::de::MapAccess::next_value::<
                                        String,
                                    >(
                                        &mut __map
                                    )?);
                            }
                            __Field::__field1 => {
                                if _serde::__private::Option::is_some(&__field1) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field("num"),
                                    );
                                }
                                __field1 =
                                    _serde::__private::Some(_serde::de::MapAccess::next_value::<
                                        u32,
                                    >(
                                        &mut __map
                                    )?);
                            }
                            __Field::__field2 => {
                                if _serde::__private::Option::is_some(&__field2) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field("list"),
                                    );
                                }
                                __field2 =
                                    _serde::__private::Some(_serde::de::MapAccess::next_value::<
                                        Vec<String>,
                                    >(
                                        &mut __map
                                    )?);
                            }
                            _ => {
                                let _ = _serde::de::MapAccess::next_value::<_serde::de::IgnoredAny>(
                                    &mut __map,
                                )?;
                            }
                        }
                    }
                    let __field0 = match __field0 {
                        _serde::__private::Some(__field0) => __field0,
                        _serde::__private::None => _serde::__private::de::missing_field("str")?,
                    };
                    let __field1 = match __field1 {
                        _serde::__private::Some(__field1) => __field1,
                        _serde::__private::None => _serde::__private::de::missing_field("num")?,
                    };
                    let __field2 = match __field2 {
                        _serde::__private::Some(__field2) => __field2,
                        _serde::__private::None => _serde::__private::de::missing_field("list")?,
                    };
                    _serde::__private::Ok(PayloadB {
                        str: __field0,
                        num: __field1,
                        list: __field2,
                    })
                }
            }
            #[doc(hidden)]
            const FIELDS: &'static [&'static str] = &["str", "num", "list"];
            _serde::Deserializer::deserialize_struct(
                __deserializer,
                "PayloadB",
                FIELDS,
                __Visitor {
                    marker: _serde::__private::PhantomData::<PayloadB>,
                    lifetime: _serde::__private::PhantomData,
                },
            )
        }
    }
};
#[doc(hidden)]
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl _serde::Serialize for PayloadB {
        fn serialize<__S>(
            &self,
            __serializer: __S,
        ) -> _serde::__private::Result<__S::Ok, __S::Error>
        where
            __S: _serde::Serializer,
        {
            let mut __serde_state = _serde::Serializer::serialize_struct(
                __serializer,
                "PayloadB",
                false as usize + 1 + 1 + 1,
            )?;
            _serde::ser::SerializeStruct::serialize_field(&mut __serde_state, "str", &self.str)?;
            _serde::ser::SerializeStruct::serialize_field(&mut __serde_state, "num", &self.num)?;
            _serde::ser::SerializeStruct::serialize_field(&mut __serde_state, "list", &self.list)?;
            _serde::ser::SerializeStruct::end(__serde_state)
        }
    }
};
impl brec::Signature for PayloadB {
    fn sig() -> brec::ByteBlock {
        brec::ByteBlock::Len4([230u8, 101u8, 57u8, 222u8])
    }
}
impl brec::PayloadEncode for PayloadB {
    fn encode(&self) -> std::io::Result<Vec<u8>> {
        brec::bincode::serialize(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
    }
}
impl brec::PayloadEncodeReferred for PayloadB {
    fn encode(&self) -> std::io::Result<Option<&[u8]>> {
        Ok(None)
    }
}
impl brec::PayloadDecode<PayloadB> for PayloadB {
    fn decode(buf: &[u8]) -> std::io::Result<PayloadB> {
        brec::bincode::deserialize(buf)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
    }
}
impl brec::PayloadCrc for PayloadB {}
impl brec::PayloadSize for PayloadB {
    fn size(&self) -> std::io::Result<u64> {
        brec::bincode::serialized_size(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
    }
}
impl brec::ReadPayloadFrom<PayloadB> for PayloadB {}
impl brec::TryReadPayloadFrom<PayloadB> for PayloadB {}
impl brec::TryReadPayloadFromBuffered<PayloadB> for PayloadB {}
impl brec::WritePayloadTo for PayloadB {}
impl brec::WriteVectoredPayloadTo for PayloadB {}
#[repr(C)]
struct BlockA {
    a: u32,
    b: u64,
    c: [u8; 100],
}
#[repr(C)]
struct BlockAReferred<'a>
where
    Self: Sized,
{
    __sig: &'a [u8; 4usize],
    a: u32,
    b: u64,
    c: &'a [u8; 100usize],
    __crc: &'a [u8; 4usize],
}

impl<'a> From<BlockAReferred<'a>> for BlockA {
    fn from(block: BlockAReferred<'a>) -> Self {
        BlockA {
            a: block.a,
            b: block.b,
            c: *block.c,
        }
    }
}
const BLOCKA: [u8; 4] = [110u8, 88u8, 23u8, 102u8];
impl brec::SignatureU32 for BlockAReferred<'_> {
    fn sig() -> &'static [u8; 4] {
        &BLOCKA
    }
}
impl brec::CrcU32 for BlockA {
    fn crc(&self) -> [u8; 4] {
        let mut hasher = brec::crc32fast::Hasher::new();
        hasher.update(&self.a.to_le_bytes());
        hasher.update(&self.b.to_le_bytes());
        hasher.update(&self.c);
        hasher.finalize().to_le_bytes()
    }
}
impl brec::CrcU32 for BlockAReferred<'_> {
    fn crc(&self) -> [u8; 4] {
        let mut hasher = brec::crc32fast::Hasher::new();
        hasher.update(&self.a.to_le_bytes());
        hasher.update(&self.b.to_le_bytes());
        hasher.update(self.c);
        hasher.finalize().to_le_bytes()
    }
}
impl brec::StaticSize for BlockA {
    fn ssize() -> u64 {
        120u64
    }
}
impl brec::ReadBlockFrom for BlockA {
    fn read<T: std::io::Read>(buf: &mut T, skip_sig: bool) -> Result<Self, brec::Error>
    where
        Self: Sized,
    {
        use brec::prelude::*;
        if !skip_sig {
            let mut sig = [0u8; 4];
            buf.read_exact(&mut sig)?;
            if sig != BLOCKA {
                return Err(brec::Error::SignatureDismatch);
            }
        }
        let mut a = [0u8; 4usize];
        buf.read_exact(&mut a)?;
        let a = u32::from_le_bytes(a);
        let mut b = [0u8; 8usize];
        buf.read_exact(&mut b)?;
        let b = u64::from_le_bytes(b);
        let mut c = [0u8; 100usize];
        buf.read_exact(&mut c)?;
        let mut crc = [0u8; 4];
        buf.read_exact(&mut crc)?;
        let block = BlockA { a, b, c };
        if block.crc() != crc {
            return Err(brec::Error::CrcDismatch);
        }
        Ok(block)
    }
}
impl<'a> brec::ReadBlockFromSlice<'a> for BlockAReferred<'a> {
    fn read_from_slice(buf: &'a [u8], skip_sig: bool) -> Result<Self, brec::Error>
    where
        Self: Sized,
    {
        use brec::prelude::*;
        if !skip_sig {
            if buf.len() < 4 {
                return Err(brec::Error::NotEnoughtSignatureData(buf.len(), 4));
            }
            if buf[..4] != BLOCKA {
                return Err(brec::Error::SignatureDismatch);
            }
        }
        let required = if skip_sig {
            BlockA::ssize() - 4
        } else {
            BlockA::ssize()
        } as usize;
        if buf.len() < required {
            return Err(brec::Error::NotEnoughData(buf.len(), required));
        }
        let __sig = if skip_sig {
            &BLOCKA
        } else {
            <&[u8; 4usize]>::try_from(&buf[0usize..4usize])?
        };
        let a = u32::from_le_bytes(buf[4usize..8usize].try_into()?);
        let b = u64::from_le_bytes(buf[8usize..16usize].try_into()?);
        let c = <&[u8; 100usize]>::try_from(&buf[16usize..116usize])?;
        let __crc = <&[u8; 4usize]>::try_from(&buf[116usize..116usize + 4usize])?;
        let crc = __crc;
        let block = BlockAReferred {
            __sig,
            a,
            b,
            c,
            __crc,
        };
        if block.crc() != *crc {
            return Err(brec::Error::CrcDismatch);
        }
        Ok(block)
    }
}
impl brec::TryReadFrom for BlockA {
    fn try_read<T: std::io::Read + std::io::Seek>(
        buf: &mut T,
    ) -> Result<brec::ReadStatus<Self>, brec::Error>
    where
        Self: Sized,
    {
        use brec::prelude::*;
        let mut sig_buf = [0u8; 4];
        let start_pos = buf.stream_position()?;
        let len = buf.seek(std::io::SeekFrom::End(0))? - start_pos;
        buf.seek(std::io::SeekFrom::Start(start_pos))?;
        if len < 4 {
            return Ok(brec::ReadStatus::NotEnoughData(4 - len));
        }
        buf.read_exact(&mut sig_buf)?;
        if sig_buf != BLOCKA {
            buf.seek(std::io::SeekFrom::Start(start_pos))?;
            return Err(brec::Error::SignatureDismatch);
        }
        if len < BlockA::ssize() {
            return Ok(brec::ReadStatus::NotEnoughData(BlockA::ssize() - len));
        }
        Ok(brec::ReadStatus::Success(BlockA::read(buf, true)?))
    }
}
impl brec::TryReadFromBuffered for BlockA {
    fn try_read<T: std::io::Read>(buf: &mut T) -> Result<brec::ReadStatus<Self>, brec::Error>
    where
        Self: Sized,
    {
        use brec::prelude::*;
        use std::io::BufRead;
        let mut reader = std::io::BufReader::new(buf);
        let bytes = reader.fill_buf()?;
        if bytes.len() < 4 {
            return Ok(brec::ReadStatus::NotEnoughData((4 - bytes.len()) as u64));
        }
        if !bytes.starts_with(&BLOCKA) {
            return Err(brec::Error::SignatureDismatch);
        }
        if (bytes.len() as u64) < BlockA::ssize() {
            return Ok(brec::ReadStatus::NotEnoughData(
                BlockA::ssize() - bytes.len() as u64,
            ));
        }
        reader.consume(4);
        let blk = BlockA::read(&mut reader, true);
        reader.consume(BlockA::ssize() as usize - 4);
        Ok(brec::ReadStatus::Success(blk?))
    }
}
impl brec::WriteTo for BlockA {
    fn write<T: std::io::Write>(&self, writer: &mut T) -> std::io::Result<usize> {
        use brec::prelude::*;
        let mut buffer = [0u8; 120usize];
        let mut offset = 0;
        let crc = self.crc();
        buffer[offset..offset + 4usize].copy_from_slice(&BLOCKA);
        offset += 4usize;
        buffer[offset..offset + 4usize].copy_from_slice(&self.a.to_le_bytes());
        offset += 4usize;
        buffer[offset..offset + 8usize].copy_from_slice(&self.b.to_le_bytes());
        offset += 8usize;
        unsafe {
            let dst = buffer.as_mut_ptr().add(offset);
            let src = self.c.as_ptr();
            std::ptr::copy_nonoverlapping(src, dst, 100usize);
        }
        offset += 100usize;
        unsafe {
            let dst = buffer.as_mut_ptr().add(offset);
            let src = crc.as_ptr();
            std::ptr::copy_nonoverlapping(src, dst, 4usize);
        }
        writer.write(&buffer)
    }
    fn write_all<T: std::io::Write>(&self, writer: &mut T) -> std::io::Result<()> {
        use brec::prelude::*;
        let mut buffer = [0u8; 120usize];
        let mut offset = 0;
        let crc = self.crc();
        buffer[offset..offset + 4usize].copy_from_slice(&BLOCKA);
        offset += 4usize;
        buffer[offset..offset + 4usize].copy_from_slice(&self.a.to_le_bytes());
        offset += 4usize;
        buffer[offset..offset + 8usize].copy_from_slice(&self.b.to_le_bytes());
        offset += 8usize;
        unsafe {
            let dst = buffer.as_mut_ptr().add(offset);
            let src = self.c.as_ptr();
            std::ptr::copy_nonoverlapping(src, dst, 100usize);
        }
        offset += 100usize;
        unsafe {
            let dst = buffer.as_mut_ptr().add(offset);
            let src = crc.as_ptr();
            std::ptr::copy_nonoverlapping(src, dst, 4usize);
        }
        writer.write_all(&buffer)
    }
}
impl brec::WriteVectoredTo for BlockA {
    fn slices(&self) -> std::io::Result<brec::IoSlices> {
        use brec::prelude::*;
        let mut slices = brec::IoSlices::default();
        slices.add_buffered(BLOCKA.to_vec());
        let mut buffer = [0u8; 12usize];
        let mut offset = 0;
        buffer[offset..offset + 4usize].copy_from_slice(&self.a.to_le_bytes());
        offset += 4usize;
        buffer[offset..offset + 8usize].copy_from_slice(&self.b.to_le_bytes());
        slices.add_buffered(buffer.to_vec());
        slices.add_slice(&self.c);
        slices.add_buffered(self.crc().to_vec());
        Ok(slices)
    }
}
pub enum Block {
    BlockA(BlockA),
}
pub enum BlockReferred<'a> {
    BlockA(BlockAReferred<'a>),
}
impl brec::BlockDef for Block {}
impl brec::Size for Block {
    fn size(&self) -> u64 {
        use brec::StaticSize;
        match self {
            Block::BlockA(..) => BlockA::ssize(),
        }
    }
}
impl brec::Size for BlockReferred<'_> {
    fn size(&self) -> u64 {
        use brec::StaticSize;
        match self {
            BlockReferred::BlockA(..) => BlockA::ssize(),
        }
    }
}
impl brec::ReadFrom for Block {
    fn read<T: std::io::Read>(buf: &mut T) -> Result<Self, brec::Error>
    where
        Self: Sized,
    {
        match <BlockA as brec::ReadBlockFrom>::read(buf, false) {
            Ok(blk) => return Ok(Block::BlockA(blk)),
            Err(err) => {
                if !match err {
                    brec::Error::SignatureDismatch => true,
                    _ => false,
                } {
                    return Err(err);
                }
            }
        }
        Err(brec::Error::SignatureDismatch)
    }
}
impl brec::ReadBlockFrom for Block {
    fn read<T: std::io::Read>(buf: &mut T, skip_sig: bool) -> Result<Self, brec::Error>
    where
        Self: Sized,
    {
        match <BlockA as brec::ReadBlockFrom>::read(buf, skip_sig) {
            Ok(blk) => return Ok(Block::BlockA(blk)),
            Err(err) => {
                if !match err {
                    brec::Error::SignatureDismatch => true,
                    _ => false,
                } {
                    return Err(err);
                }
            }
        }
        Err(brec::Error::SignatureDismatch)
    }
}
impl<'a> brec::ReadBlockFromSlice<'a> for BlockReferred<'a> {
    fn read_from_slice(buf: &'a [u8], skip_sig: bool) -> Result<Self, brec::Error>
    where
        Self: Sized,
    {
        match BlockAReferred::read_from_slice(buf, skip_sig) {
            Ok(blk) => return Ok(BlockReferred::BlockA(blk)),
            Err(err) => {
                if !match err {
                    brec::Error::SignatureDismatch => true,
                    _ => false,
                } {
                    return Err(err);
                }
            }
        }
        Err(brec::Error::SignatureDismatch)
    }
}
impl brec::TryReadFrom for Block {
    fn try_read<T: std::io::Read + std::io::Seek>(
        buf: &mut T,
    ) -> Result<brec::ReadStatus<Self>, brec::Error>
    where
        Self: Sized,
    {
        match <BlockA as brec::TryReadFrom>::try_read(buf) {
            Ok(brec::ReadStatus::Success(blk)) => {
                return Ok(brec::ReadStatus::Success(Block::BlockA(blk)));
            }
            Ok(brec::ReadStatus::NotEnoughData(needed)) => {
                return Ok(brec::ReadStatus::NotEnoughData(needed));
            }
            Err(err) => {
                if !match err {
                    brec::Error::SignatureDismatch => true,
                    _ => false,
                } {
                    return Err(err);
                }
            }
        }
        Err(brec::Error::SignatureDismatch)
    }
}
impl brec::TryReadFromBuffered for Block {
    fn try_read<T: std::io::Read>(buf: &mut T) -> Result<brec::ReadStatus<Self>, brec::Error>
    where
        Self: Sized,
    {
        match <BlockA as brec::TryReadFromBuffered>::try_read(buf) {
            Ok(brec::ReadStatus::Success(blk)) => {
                return Ok(brec::ReadStatus::Success(Block::BlockA(blk)));
            }
            Ok(brec::ReadStatus::NotEnoughData(needed)) => {
                return Ok(brec::ReadStatus::NotEnoughData(needed));
            }
            Err(err) => {
                if !match err {
                    brec::Error::SignatureDismatch => true,
                    _ => false,
                } {
                    return Err(err);
                }
            }
        }
        Err(brec::Error::SignatureDismatch)
    }
}
impl brec::WriteTo for Block {
    fn write<T: std::io::Write>(&self, buf: &mut T) -> std::io::Result<usize> {
        match self {
            Block::BlockA(blk) => blk.write(buf),
        }
    }
    fn write_all<T: std::io::Write>(&self, buf: &mut T) -> std::io::Result<()> {
        match self {
            Block::BlockA(blk) => blk.write_all(buf),
        }
    }
}
impl brec::WriteVectoredTo for Block {
    fn slices(&self) -> std::io::Result<brec::IoSlices> {
        match self {
            Block::BlockA(blk) => blk.slices(),
        }
    }
}
pub enum Payload {
    PayloadA(PayloadA),
    PayloadB(PayloadB),
}

impl brec::PayloadInnerDef for Payload {}

impl brec::PayloadDef<Payload> for Payload {}

impl brec::PayloadEncode for Payload {
    fn encode(&self) -> std::io::Result<Vec<u8>> {
        match self {
            Payload::PayloadA(pl) => pl.encode(),
            Payload::PayloadB(pl) => pl.encode(),
        }
    }
}

impl brec::PayloadEncodeReferred for Payload {
    fn encode(&self) -> std::io::Result<Option<&[u8]>> {
        match self {
            Payload::PayloadA(pl) => pl.encode(),
            Payload::PayloadB(pl) => pl.encode(),
        }
    }
}

impl brec::PayloadCrc for Payload {
    fn crc(&self) -> std::io::Result<brec::ByteBlock> {
        match self {
            Payload::PayloadA(pl) => pl.crc(),
            Payload::PayloadB(pl) => pl.crc(),
        }
    }
}

impl brec::PayloadSize for Payload {
    fn size(&self) -> std::io::Result<u64> {
        match self {
            Payload::PayloadA(pl) => pl.size(),
            Payload::PayloadB(pl) => pl.size(),
        }
    }
}

impl brec::ExtractPayloadFrom<Payload> for Payload {
    fn read<B: std::io::Read>(
        buf: &mut B,
        header: &brec::PayloadHeader,
    ) -> Result<Payload, brec::Error>
    where
        Self: Sized,
    {
        match <PayloadA as brec::ReadPayloadFrom<PayloadA>>::read(buf, header) {
            Ok(pl) => return Ok(Payload::PayloadA(pl)),
            Err(err) => {
                if !matches!(err, brec::Error::SignatureDismatch) {
                    return Err(err);
                }
            }
        }
        match <PayloadB as brec::ReadPayloadFrom<PayloadB>>::read(buf, header) {
            Ok(pl) => return Ok(Payload::PayloadB(pl)),
            Err(err) => {
                if !matches!(err, brec::Error::SignatureDismatch) {
                    return Err(err);
                }
            }
        }
        Err(brec::Error::SignatureDismatch)
    }
}

impl brec::TryExtractPayloadFrom<Payload> for Payload {
    fn try_read<B: std::io::Read + std::io::Seek>(
        buf: &mut B,
        header: &brec::PayloadHeader,
    ) -> Result<brec::ReadStatus<Payload>, brec::Error> {
        match <PayloadA as brec::TryReadPayloadFrom<PayloadA>>::try_read(buf, header) {
            Ok(brec::ReadStatus::Success(pl)) => {
                return Ok(brec::ReadStatus::Success(Payload::PayloadA(pl)))
            }
            Ok(brec::ReadStatus::NotEnoughData(needed)) => {
                return Ok(brec::ReadStatus::NotEnoughData(needed))
            }
            Err(err) => {
                if !matches!(err, brec::Error::SignatureDismatch) {
                    return Err(err);
                }
            }
        }
        match <PayloadB as brec::TryReadPayloadFrom<PayloadB>>::try_read(buf, header) {
            Ok(brec::ReadStatus::Success(pl)) => {
                return Ok(brec::ReadStatus::Success(Payload::PayloadB(pl)))
            }
            Ok(brec::ReadStatus::NotEnoughData(needed)) => {
                return Ok(brec::ReadStatus::NotEnoughData(needed))
            }
            Err(err) => {
                if !matches!(err, brec::Error::SignatureDismatch) {
                    return Err(err);
                }
            }
        }
        Err(brec::Error::SignatureDismatch)
    }
}

impl brec::TryExtractPayloadFromBuffered<Payload> for Payload {
    fn try_read<B: std::io::Read>(
        buf: &mut B,
        header: &brec::PayloadHeader,
    ) -> Result<brec::ReadStatus<Payload>, brec::Error> {
        match <PayloadA as brec::TryReadPayloadFromBuffered<PayloadA>>::try_read(buf, header) {
            Ok(brec::ReadStatus::Success(pl)) => {
                return Ok(brec::ReadStatus::Success(Payload::PayloadA(pl)))
            }
            Ok(brec::ReadStatus::NotEnoughData(needed)) => {
                return Ok(brec::ReadStatus::NotEnoughData(needed))
            }
            Err(err) => {
                if !matches!(err, brec::Error::SignatureDismatch) {
                    return Err(err);
                }
            }
        }
        match <PayloadB as brec::TryReadPayloadFromBuffered<PayloadB>>::try_read(buf, header) {
            Ok(brec::ReadStatus::Success(pl)) => {
                return Ok(brec::ReadStatus::Success(Payload::PayloadB(pl)))
            }
            Ok(brec::ReadStatus::NotEnoughData(needed)) => {
                return Ok(brec::ReadStatus::NotEnoughData(needed))
            }
            Err(err) => {
                if !matches!(err, brec::Error::SignatureDismatch) {
                    return Err(err);
                }
            }
        }
        Err(brec::Error::SignatureDismatch)
    }
}

impl brec::WritingPayloadTo for Payload {
    fn write<T: std::io::Write>(&mut self, buf: &mut T) -> std::io::Result<usize> {
        use brec::WritePayloadTo;
        match self {
            Payload::PayloadA(pl) => pl.write(buf),
            Payload::PayloadB(pl) => pl.write(buf),
        }
    }

    fn write_all<T: std::io::Write>(&mut self, buf: &mut T) -> std::io::Result<()> {
        use brec::WritePayloadTo;
        match self {
            Payload::PayloadA(pl) => pl.write_all(buf),
            Payload::PayloadB(pl) => pl.write_all(buf),
        }
    }
}

impl brec::WritingVectoredPayloadTo for Payload {
    fn slices(&mut self) -> std::io::Result<brec::IoSlices> {
        use brec::WriteVectoredPayloadTo;
        match self {
            Payload::PayloadA(pl) => pl.slices(),
            Payload::PayloadB(pl) => pl.slices(),
        }
    }
}
