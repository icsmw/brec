use brec::prelude::*;

#[block]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct BlockU16 {
    field: u16,
}
