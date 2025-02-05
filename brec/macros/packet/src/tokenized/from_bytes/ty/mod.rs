use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use crate::*;

impl FromBytes for Ty {
    fn safe(&self, src: &Ident, from: usize, to: usize) -> TokenStream {
        match self {
            Ty::u8
            | Ty::u16
            | Ty::u32
            | Ty::u64
            | Ty::u128
            | Ty::i8
            | Ty::i16
            | Ty::i32
            | Ty::i64
            | Ty::i128
            | Ty::f32
            | Ty::f64 => {
                let ty = self.direct();
                quote! {
                   #ty::from_le_bytes(#src[#from..#to].try_into()?)
                }
            }
            Ty::bool => {
                quote! {
                   u8::from_le_bytes(#src[#from..#to].try_into()?) == 1
                }
            }
            Ty::Slice(len, ty) => match **ty {
                Ty::u8 => quote! {
                    <&[u8; #len]>::try_from(&#src[#from..#to])?
                },
                Ty::u16
                | Ty::u32
                | Ty::u64
                | Ty::u128
                | Ty::i8
                | Ty::i16
                | Ty::i32
                | Ty::i64
                | Ty::i128 => {
                    let ty_size = ty.size();
                    let default = ty.default();
                    let ty = ty.direct();
                    let expected = len * ty_size;
                    let chk = if ty_size == 1 {
                        quote! {}
                    } else {
                        quote! {
                            if bytes.as_ptr() as usize % #ty_size != 0 {
                                return Err(brec::Error::MisalignedPointer)
                            }
                        }
                    };
                    quote! {
                        {
                            let bytes = &#src[#from..#to];
                            #chk
                            if bytes.len() != #expected {
                                return Err(brec::Error::UnexpectedSliceLength)
                            }
                            let slice = unsafe { &*(bytes.as_ptr() as *const [#ty; #len]) };
                            if cfg!(target_endian = "big") {
                                let mut arr = [#default; #len];
                                for (i, &value) in slice.iter().enumerate() {
                                    arr[i] = #ty::from_le(value);
                                }
                                std::boxed::Box::leak(std::boxed::Box::new(arr))
                            } else {
                                slice
                            }
                        }
                    }
                }
                Ty::f32 | Ty::f64 => {
                    let ty_size = ty.size();
                    let default = ty.default();
                    let ty = ty.direct();
                    quote! {
                        {
                            let bytes = &#src[#from..#to];
                            if bytes.as_ptr() as usize % #ty_size != 0 {
                                return Err(brec::Error::MisalignedPointer)
                            }
                            if bytes.len() != #len * #ty_size {
                                return Err(brec::Error::UnexpectedSliceLength)
                            }
                            if cfg!(target_endian = "big") {
                                let mut arr = [#default; #len];
                                for (i, chunk) in bytes.chunks_exact(#ty_size).enumerate() {
                                    arr[i] = #ty::from_le_bytes(
                                        chunk.try_into().map_err(brec::Error::TryFromSliceError)?,
                                    );
                                }
                                std::boxed::Box::leak(std::boxed::Box::new(arr))
                            } else {
                                unsafe { &*(bytes.as_ptr() as *const [#ty; #len]) }
                            }
                        }
                    }
                }
                Ty::bool => {
                    let ty_size = ty.size();
                    let ty = ty.direct();
                    quote! {
                        {
                            let bytes = &#src[#from..#to];
                            if bytes.len() != #len {
                                return Err(brec::Error::UnexpectedSliceLength)
                            }
                            unsafe { &*(bytes.as_ptr() as *const [#ty; #len]) }
                        }
                    }
                }
                Ty::Slice(..) => quote! {},
            },
        }
    }

    fn r#unsafe(&self, src: &Ident, offset: usize) -> TokenStream {
        let ty = self.direct();
        if offset == 0 {
            if matches!(self, Ty::u8) {
                quote! {
                    unsafe { &*#src.as_ptr() }
                }
            } else {
                quote! {
                    unsafe { &*(#src.as_ptr() as *const #ty) }
                }
            }
        } else if matches!(self, Ty::u8) {
            quote! {
                unsafe { &*#src.as_ptr().add(#offset) }
            }
        } else {
            quote! {
               unsafe { &*(#src.as_ptr().add(#offset) as *const #ty) }
            }
        }
    }
}
