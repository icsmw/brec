use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Storage slot is damaged. Reason: {0}")]
    DamagedSlot(crate::Error),

    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    RootError(#[from] crate::Error),
}
