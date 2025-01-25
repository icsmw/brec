use crate::*;
use std::convert::TryFrom;
use syn::{parse_str, Type};

#[test]
fn test() {
    let cases: Vec<(&str, Ty)> = vec![
        ("u8", Ty::new(TyDef::u8, false)),
        ("u16", Ty::new(TyDef::u16, false)),
        ("u32", Ty::new(TyDef::u32, false)),
        ("u64", Ty::new(TyDef::u64, false)),
        ("u128", Ty::new(TyDef::u128, false)),
        ("i8", Ty::new(TyDef::i8, false)),
        ("i16", Ty::new(TyDef::i16, false)),
        ("i32", Ty::new(TyDef::i32, false)),
        ("i64", Ty::new(TyDef::i64, false)),
        ("i128", Ty::new(TyDef::i128, false)),
        ("f16", Ty::new(TyDef::f16, false)),
        ("f32", Ty::new(TyDef::f32, false)),
        ("f64", Ty::new(TyDef::f64, false)),
        ("bool", Ty::new(TyDef::bool, false)),
        (
            "[u8; 10]",
            Ty::new(TyDef::Slice(10, Box::new(TyDef::u8)), false),
        ),
        (
            "[u16; 256]",
            Ty::new(TyDef::Slice(256, Box::new(TyDef::u16)), false),
        ),
        (
            "[i32; 1024]",
            Ty::new(TyDef::Slice(1024, Box::new(TyDef::i32)), false),
        ),
        (
            "[f64; 5]",
            Ty::new(TyDef::Slice(5, Box::new(TyDef::f64)), false),
        ),
        ("&u8", Ty::new(TyDef::u8, true)),
        ("&u16", Ty::new(TyDef::u16, true)),
        ("&i64", Ty::new(TyDef::i64, true)),
        ("&bool", Ty::new(TyDef::bool, true)),
        (
            "&[u8; 10]",
            Ty::new(TyDef::Slice(10, Box::new(TyDef::u8)), true),
        ),
        (
            "&[u32; 50]",
            Ty::new(TyDef::Slice(50, Box::new(TyDef::u32)), true),
        ),
        (
            "&[bool; 4]",
            Ty::new(TyDef::Slice(4, Box::new(TyDef::bool)), true),
        ),
        (
            "Option<u8>",
            Ty::new(TyDef::Option(Box::new(TyDef::u8)), false),
        ),
        (
            "Option<u16>",
            Ty::new(TyDef::Option(Box::new(TyDef::u16)), false),
        ),
        (
            "Option<bool>",
            Ty::new(TyDef::Option(Box::new(TyDef::bool)), false),
        ),
        (
            "Option<[u8; 10]>",
            Ty::new(
                TyDef::Option(Box::new(TyDef::Slice(10, Box::new(TyDef::u8)))),
                false,
            ),
        ),
        (
            "Option<[i32; 1024]>",
            Ty::new(
                TyDef::Option(Box::new(TyDef::Slice(1024, Box::new(TyDef::i32)))),
                false,
            ),
        ),
        (
            "Option<&u8>",
            Ty::new(TyDef::Option(Box::new(TyDef::u8)), false),
        ),
        (
            "Option<&[u16; 256]>",
            Ty::new(
                TyDef::Option(Box::new(TyDef::Slice(256, Box::new(TyDef::u16)))),
                false,
            ),
        ),
        (
            "std::option::Option<u8>",
            Ty::new(TyDef::Option(Box::new(TyDef::u8)), false),
        ),
        (
            "std::option::Option<[f32; 64]>",
            Ty::new(
                TyDef::Option(Box::new(TyDef::Slice(64, Box::new(TyDef::f32)))),
                false,
            ),
        ),
        (
            "std::option::Option<&[i16; 8]>",
            Ty::new(
                TyDef::Option(Box::new(TyDef::Slice(8, Box::new(TyDef::i16)))),
                false,
            ),
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
