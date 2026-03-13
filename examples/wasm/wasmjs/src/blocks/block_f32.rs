use brec::prelude::*;

#[block]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct BlockF32 {
    field: f32,
}
