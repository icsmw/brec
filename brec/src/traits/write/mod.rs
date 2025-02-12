pub mod slices;

pub use slices::*;

pub trait WriteTo {
    fn write<T: std::io::Write>(&self, buf: &mut T) -> std::io::Result<usize>;
    fn write_all<T: std::io::Write>(&self, buf: &mut T) -> std::io::Result<()>;
}

pub trait WriteVectoredTo {
    fn write_vectored<T: std::io::Write>(&self, buf: &mut T) -> std::io::Result<usize> {
        buf.write_vectored(&self.slices()?.get())
    }

    fn slices(&self) -> std::io::Result<IoSlices>;

    fn write_vectored_all<T: std::io::Write>(&self, buf: &mut T) -> std::io::Result<()> {
        self.slices()?.write_vectored_all(buf)
    }
}

pub trait MutWriteTo {
    fn write<T: std::io::Write>(&mut self, buf: &mut T) -> std::io::Result<usize>;
    fn write_all<T: std::io::Write>(&mut self, buf: &mut T) -> std::io::Result<()>;
}

pub trait MutWriteVectoredTo {
    fn write_vectored<T: std::io::Write>(&mut self, buf: &mut T) -> std::io::Result<usize> {
        buf.write_vectored(&self.slices()?.get())
    }

    fn slices(&mut self) -> std::io::Result<IoSlices>;

    fn write_vectored_all<T: std::io::Write>(&mut self, buf: &mut T) -> std::io::Result<()> {
        self.slices()?.write_vectored_all(buf)
    }
}
