use brec::prelude::*;

#[block]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct BlockU8 {
    field: u8,
}
