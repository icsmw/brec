pub const FIELD_SIG: &str = "__sig";
pub const FIELD_CRC: &str = "__crc";
pub const FIELD_NEXT: &str = "__next";

pub fn is_reserved_field_name<S: AsRef<str>>(name: S) -> bool {
    [FIELD_SIG, FIELD_CRC, FIELD_NEXT].contains(&name.as_ref())
}
