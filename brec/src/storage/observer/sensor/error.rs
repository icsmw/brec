use thiserror::Error;

#[derive(Error, Debug)]
pub enum SensorError {
    #[error("Expected file, but {0} isn't a file")]
    NotFile(String),
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Send Error")]
    SendError,
    #[error("Notify Error: {0}")]
    NotifyError(#[from] notify::Error),
    #[error("Disconnected")]
    Disconnected,
}

impl<T> From<crossbeam::channel::SendError<T>> for SensorError {
    fn from(_: crossbeam::channel::SendError<T>) -> Self {
        SensorError::SendError
    }
}
