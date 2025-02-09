pub trait WriteTo {
    fn write<T: std::io::Write>(&self, buf: &mut T) -> std::io::Result<usize>;
    fn write_all<T: std::io::Write>(&self, buf: &mut T) -> std::io::Result<()>;
}
