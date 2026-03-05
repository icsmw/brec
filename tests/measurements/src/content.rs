use crate::{AttachmentBincode, Metadata};

#[derive(Debug)]
pub struct TextualRow {
    pub msg: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq, PartialOrd, Clone)]
pub struct JSONRow<T> {
    pub meta: Metadata,
    pub payload: T,
}

pub trait MatchValue {
    fn contains_match(&self) -> bool;
}

impl MatchValue for String {
    fn contains_match(&self) -> bool {
        self.contains(crate::test::MATCH)
    }
}

impl MatchValue for AttachmentBincode {
    fn contains_match(&self) -> bool {
        self.name.contains(crate::test::MATCH)
    }
}
