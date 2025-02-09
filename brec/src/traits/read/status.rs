pub enum ReadStatus<T> {
    Success(T),
    NotEnoughData(u64),
}

impl<T> ReadStatus<T> {
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
