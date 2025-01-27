use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Not enought data; data len = {0}; required = {1}")]
    NotEnoughtData(usize, usize),
    #[error("Not enought data to read signature; data len = {0}; required = {1}")]
    NotEnoughtSignatureData(usize, usize),
    #[error("Invalid data align; data len = {0}; required = {1}; offset = {2} (expected 0)")]
    InvalidAlign(usize, usize, usize),
    #[error("TryFromSliceError: {0}")]
    TryFromSliceError(#[from] std::array::TryFromSliceError),
}
