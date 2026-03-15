/// Result type representing the outcome of a read attempt.
///
/// `ReadStatus` allows distinguishing between successful reads and cases
/// where more data is required to complete the read.
pub enum ReadStatus<T> {
    /// The read operation succeeded and produced a value of type `T`.
    Success(T),

    /// The read operation failed due to insufficient data.
    ///
    /// Contains the number of additional bytes required to complete the read.
    NotEnoughData(u64),
}

impl<T> ReadStatus<T> {
    /// Maps the inner `Success` value using the provided function.
    ///
    /// If the status is `Success`, applies `mapper` to the inner value and returns
    /// a new `ReadStatus::Success`. If the status is `NotEnoughData`, it is returned unchanged.
    ///
    /// # Arguments
    /// * `mapper` - A function to apply to the `Success` value.
    ///
    /// # Returns
    /// A new `ReadStatus` with the mapped type.
    pub fn map<K, F>(self, mapper: F) -> ReadStatus<K>
    where
        F: FnOnce(T) -> K,
    {
        match self {
            ReadStatus::Success(value) => ReadStatus::Success(mapper(value)),
            ReadStatus::NotEnoughData(n) => ReadStatus::NotEnoughData(n),
        }
    }
}

/// Result type representing the outcome of a packet read attempt.
///
/// At this stage it is intentionally equivalent to `ReadStatus` and is used to
/// separate packet-level API evolution from block/payload-level read statuses.
pub enum PacketReadStatus<T> {
    /// The packet read operation succeeded and produced a value of type `T`.
    Success(T),

    /// The packet read operation failed due to insufficient data.
    ///
    /// Contains the number of additional bytes required to complete the read.
    NotEnoughData(u64),
}

impl<T> PacketReadStatus<T> {
    /// Maps the inner `Success` value using the provided function.
    pub fn map<K, F>(self, mapper: F) -> PacketReadStatus<K>
    where
        F: FnOnce(T) -> K,
    {
        match self {
            PacketReadStatus::Success(value) => PacketReadStatus::Success(mapper(value)),
            PacketReadStatus::NotEnoughData(n) => PacketReadStatus::NotEnoughData(n),
        }
    }
}
