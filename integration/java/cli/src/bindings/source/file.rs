/// Compile-time file name for a generated source artifact.
pub trait FileName {
    const FILE_NAME: &'static str;
}
