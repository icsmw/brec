use crate::*;
use std::convert::TryFrom;
use syn::{parse_str, Type};

#[test]
fn test() {
    let cases: Vec<(&str, Ty)> = vec![
        ("u8", Ty::u8),
        ("u16", Ty::u16),
        ("u32", Ty::u32),
        ("u64", Ty::u64),
        ("u128", Ty::u128),
        ("i8", Ty::i8),
        ("i16", Ty::i16),
        ("i32", Ty::i32),
        ("i64", Ty::i64),
        ("i128", Ty::i128),
        ("f32", Ty::f32),
        ("f64", Ty::f64),
        ("bool", Ty::bool),
        ("[u8; 10]", Ty::Slice(10, Box::new(Ty::u8))),
        ("[u16; 256]", Ty::Slice(256, Box::new(Ty::u16))),
        ("[i32; 1024]", Ty::Slice(1024, Box::new(Ty::i32))),
        ("[f64; 5]", Ty::Slice(5, Box::new(Ty::f64))),
        ("Option<u8>", Ty::Option(Box::new(Ty::u8))),
        ("Option<u16>", Ty::Option(Box::new(Ty::u16))),
        ("Option<bool>", Ty::Option(Box::new(Ty::bool))),
        (
            "Option<[u8; 10]>",
            Ty::Option(Box::new(Ty::Slice(10, Box::new(Ty::u8)))),
        ),
        (
            "Option<[i32; 1024]>",
            Ty::Option(Box::new(Ty::Slice(1024, Box::new(Ty::i32)))),
        ),
        ("std::option::Option<u8>", Ty::Option(Box::new(Ty::u8))),
        (
            "std::option::Option<[f32; 64]>",
            Ty::Option(Box::new(Ty::Slice(64, Box::new(Ty::f32)))),
        ),
    ];
    for (type_str, exp_ty) in cases {
        let ty: Type = parse_str(type_str).expect("Type is parsed");
        let ty = Ty::try_from(&ty).expect("Ty is parsed from Type");
        if ty != exp_ty {
            eprintln!("Origin: {type_str};\nExpectation:{exp_ty:?};\nExtracted:{ty:?};")
        }
        assert_eq!(ty, exp_ty);
    }
}
