use brec::prelude::*;

#[block]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct BlockF64 {
    field: f64,
}
