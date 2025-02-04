use brec::*;
mod extended {}
pub struct CustomBlock {
    field_u8: u8,
    field_u16: u16,
    field_u32: u32,
    field_u64: u64,
    field_u128: u128,
    field_i8: i8,
    field_i16: i16,
    field_i32: i32,
    field_i64: i64,
    field_i128: i128,
    field_f32: f32,
    field_f64: f64,
    field_bool: bool,
    field_u8_slice: [u8; 100],
    field_u16_slice: [u16; 100],
    field_u32_slice: [u32; 100],
    field_u64_slice: [u64; 100],
    field_u128_slice: [u128; 100],
    field_i8_slice: [i8; 100],
    field_i16_slice: [i16; 100],
    field_i32_slice: [i32; 100],
    field_i64_slice: [i64; 100],
    field_i128_slice: [i128; 100],
    field_f32_slice: [f32; 100],
    field_f64_slice: [f64; 100],
    field_bool_slice: [bool; 100],
}
#[repr(C)]
struct CustomBlockReferred<'a>
where
    Self: Sized,
{
    __sig: &'a [u8; 4usize],
    field_u8: u8,
    field_u16: u16,
    field_u32: u32,
    field_u64: u64,
    field_u128: u128,
    field_i8: i8,
    field_i16: i16,
    field_i32: i32,
    field_i64: i64,
    field_i128: i128,
    field_f32: f32,
    field_f64: f64,
    field_bool: bool,
    field_u8_slice: &'a [u8; 100usize],
    field_u16_slice: &'a [u16; 100usize],
    field_u32_slice: &'a [u32; 100usize],
    field_u64_slice: &'a [u64; 100usize],
    field_u128_slice: &'a [u128; 100usize],
    field_i8_slice: &'a [i8; 100usize],
    field_i16_slice: &'a [i16; 100usize],
    field_i32_slice: &'a [i32; 100usize],
    field_i64_slice: &'a [i64; 100usize],
    field_i128_slice: &'a [i128; 100usize],
    field_f32_slice: &'a [f32; 100usize],
    field_f64_slice: &'a [f64; 100usize],
    field_bool_slice: &'a [bool; 100usize],
    __crc: u32,
}
impl<'a> From<CustomBlockReferred<'a>> for CustomBlock {
    fn from(block: CustomBlockReferred<'a>) -> Self {
        CustomBlock {
            field_u8: block.field_u8,
            field_u16: block.field_u16,
            field_u32: block.field_u32,
            field_u64: block.field_u64,
            field_u128: block.field_u128,
            field_i8: block.field_i8,
            field_i16: block.field_i16,
            field_i32: block.field_i32,
            field_i64: block.field_i64,
            field_i128: block.field_i128,
            field_f32: block.field_f32,
            field_f64: block.field_f64,
            field_bool: block.field_bool,
            field_u8_slice: *block.field_u8_slice,
            field_u16_slice: *block.field_u16_slice,
            field_u32_slice: *block.field_u32_slice,
            field_u64_slice: *block.field_u64_slice,
            field_u128_slice: *block.field_u128_slice,
            field_i8_slice: *block.field_i8_slice,
            field_i16_slice: *block.field_i16_slice,
            field_i32_slice: *block.field_i32_slice,
            field_i64_slice: *block.field_i64_slice,
            field_i128_slice: *block.field_i128_slice,
            field_f32_slice: *block.field_f32_slice,
            field_f64_slice: *block.field_f64_slice,
            field_bool_slice: *block.field_bool_slice,
        }
    }
}
const CUSTOMBLOCK: [u8; 4] = [95u8, 120u8, 118u8, 13u8];
impl Signature for CustomBlockReferred<'_> {
    fn sig() -> &'static [u8; 4] {
        &CUSTOMBLOCK
    }
}
impl brec::Crc for CustomBlock {
    fn crc(&self) -> [u8; 4] {
        let mut hasher = brec::crc32fast::Hasher::new();
        hasher.update(&[self.field_u8]);
        hasher.update(&self.field_u16.to_le_bytes());
        hasher.update(&self.field_u32.to_le_bytes());
        hasher.update(&self.field_u64.to_le_bytes());
        hasher.update(&self.field_u128.to_le_bytes());
        hasher.update(&self.field_i8.to_le_bytes());
        hasher.update(&self.field_i16.to_le_bytes());
        hasher.update(&self.field_i32.to_le_bytes());
        hasher.update(&self.field_i64.to_le_bytes());
        hasher.update(&self.field_i128.to_le_bytes());
        hasher.update(&self.field_f32.to_le_bytes());
        hasher.update(&self.field_f64.to_le_bytes());
        hasher.update(&[self.field_bool as u8]);
        hasher.update(&self.field_u8_slice);
        let bytes = {
            let mut bytes = [0u8; 200usize];
            for (n, &p) in self.field_u16_slice.iter().enumerate() {
                bytes[n * 2usize..n * 2usize + 2usize].copy_from_slice(&p.to_le_bytes());
            }
            bytes
        };
        hasher.update(&bytes);
        let bytes = {
            let mut bytes = [0u8; 400usize];
            for (n, &p) in self.field_u32_slice.iter().enumerate() {
                bytes[n * 4usize..n * 4usize + 4usize].copy_from_slice(&p.to_le_bytes());
            }
            bytes
        };
        hasher.update(&bytes);
        let bytes = {
            let mut bytes = [0u8; 800usize];
            for (n, &p) in self.field_u64_slice.iter().enumerate() {
                bytes[n * 8usize..n * 8usize + 8usize].copy_from_slice(&p.to_le_bytes());
            }
            bytes
        };
        hasher.update(&bytes);
        let bytes = {
            let mut bytes = [0u8; 1600usize];
            for (n, &p) in self.field_u128_slice.iter().enumerate() {
                bytes[n * 16usize..n * 16usize + 16usize].copy_from_slice(&p.to_le_bytes());
            }
            bytes
        };
        hasher.update(&bytes);
        let bytes = {
            let mut bytes = [0u8; 100usize];
            for (n, &p) in self.field_i8_slice.iter().enumerate() {
                bytes[n * 1usize..n * 1usize + 1usize].copy_from_slice(&p.to_le_bytes());
            }
            bytes
        };
        hasher.update(&bytes);
        let bytes = {
            let mut bytes = [0u8; 200usize];
            for (n, &p) in self.field_i16_slice.iter().enumerate() {
                bytes[n * 2usize..n * 2usize + 2usize].copy_from_slice(&p.to_le_bytes());
            }
            bytes
        };
        hasher.update(&bytes);
        let bytes = {
            let mut bytes = [0u8; 400usize];
            for (n, &p) in self.field_i32_slice.iter().enumerate() {
                bytes[n * 4usize..n * 4usize + 4usize].copy_from_slice(&p.to_le_bytes());
            }
            bytes
        };
        hasher.update(&bytes);
        let bytes = {
            let mut bytes = [0u8; 800usize];
            for (n, &p) in self.field_i64_slice.iter().enumerate() {
                bytes[n * 8usize..n * 8usize + 8usize].copy_from_slice(&p.to_le_bytes());
            }
            bytes
        };
        hasher.update(&bytes);
        let bytes = {
            let mut bytes = [0u8; 1600usize];
            for (n, &p) in self.field_i128_slice.iter().enumerate() {
                bytes[n * 16usize..n * 16usize + 16usize].copy_from_slice(&p.to_le_bytes());
            }
            bytes
        };
        hasher.update(&bytes);
        let bytes = {
            let mut bytes = [0u8; 400usize];
            for (n, &p) in self.field_f32_slice.iter().enumerate() {
                bytes[n * 4usize..n * 4usize + 4usize].copy_from_slice(&p.to_le_bytes());
            }
            bytes
        };
        hasher.update(&bytes);
        let bytes = {
            let mut bytes = [0u8; 800usize];
            for (n, &p) in self.field_f64_slice.iter().enumerate() {
                bytes[n * 8usize..n * 8usize + 8usize].copy_from_slice(&p.to_le_bytes());
            }
            bytes
        };
        hasher.update(&bytes);
        let bytes = {
            let mut bytes = [0u8; 100usize];
            for (n, &p) in self.field_bool_slice.iter().enumerate() {
                bytes[n] = p as u8;
            }
            bytes
        };
        hasher.update(&bytes);
        hasher.finalize().to_le_bytes()
    }
}
impl brec::Size for CustomBlock {
    fn size() -> u64 {
        7583u64
    }
}
impl brec::Read for CustomBlock {
    fn read<T: std::io::Read>(buf: &mut T, skip_sig: bool) -> Result<Self, brec::Error>
    where
        Self: Sized,
    {
        if !skip_sig {
            let mut sig = [0u8; 4];
            buf.read_exact(&mut sig)?;
            if sig != CUSTOMBLOCK {
                return Err(brec::Error::SignatureDismatch);
            }
        }
        let mut field_u8 = [0u8; 1];
        buf.read_exact(&mut field_u8)?;
        let field_u8 = field_u8[0];
        let mut field_u16 = [0u8; 2usize];
        buf.read_exact(&mut field_u16)?;
        let field_u16 = u16::from_le_bytes(field_u16);
        let mut field_u32 = [0u8; 4usize];
        buf.read_exact(&mut field_u32)?;
        let field_u32 = u32::from_le_bytes(field_u32);
        let mut field_u64 = [0u8; 8usize];
        buf.read_exact(&mut field_u64)?;
        let field_u64 = u64::from_le_bytes(field_u64);
        let mut field_u128 = [0u8; 16usize];
        buf.read_exact(&mut field_u128)?;
        let field_u128 = u128::from_le_bytes(field_u128);
        let mut field_i8 = [0u8; 1usize];
        buf.read_exact(&mut field_i8)?;
        let field_i8 = i8::from_le_bytes(field_i8);
        let mut field_i16 = [0u8; 2usize];
        buf.read_exact(&mut field_i16)?;
        let field_i16 = i16::from_le_bytes(field_i16);
        let mut field_i32 = [0u8; 4usize];
        buf.read_exact(&mut field_i32)?;
        let field_i32 = i32::from_le_bytes(field_i32);
        let mut field_i64 = [0u8; 8usize];
        buf.read_exact(&mut field_i64)?;
        let field_i64 = i64::from_le_bytes(field_i64);
        let mut field_i128 = [0u8; 16usize];
        buf.read_exact(&mut field_i128)?;
        let field_i128 = i128::from_le_bytes(field_i128);
        let mut field_f32 = [0u8; 4usize];
        buf.read_exact(&mut field_f32)?;
        let field_f32 = f32::from_le_bytes(field_f32);
        let mut field_f64 = [0u8; 8usize];
        buf.read_exact(&mut field_f64)?;
        let field_f64 = f64::from_le_bytes(field_f64);
        let mut field_bool = [0u8; 1usize];
        buf.read_exact(&mut field_bool)?;
        let field_bool = field_bool[0] != 0;
        let mut field_u8_slice = [0u8; 100usize];
        buf.read_exact(&mut field_u8_slice)?;
        let mut field_u16_slice_slice = [0u8; 200usize];
        buf.read_exact(&mut field_u16_slice_slice)?;
        let mut field_u16_slice = [0u16; 100usize];
        for (i, chunk) in field_u16_slice_slice.chunks_exact(2usize).enumerate() {
            field_u16_slice[i] = u16::from_le_bytes(chunk.try_into()?);
        }
        let mut field_u32_slice_slice = [0u8; 400usize];
        buf.read_exact(&mut field_u32_slice_slice)?;
        let mut field_u32_slice = [0u32; 100usize];
        for (i, chunk) in field_u32_slice_slice.chunks_exact(4usize).enumerate() {
            field_u32_slice[i] = u32::from_le_bytes(chunk.try_into()?);
        }
        let mut field_u64_slice_slice = [0u8; 800usize];
        buf.read_exact(&mut field_u64_slice_slice)?;
        let mut field_u64_slice = [0u64; 100usize];
        for (i, chunk) in field_u64_slice_slice.chunks_exact(8usize).enumerate() {
            field_u64_slice[i] = u64::from_le_bytes(chunk.try_into()?);
        }
        let mut field_u128_slice_slice = [0u8; 1600usize];
        buf.read_exact(&mut field_u128_slice_slice)?;
        let mut field_u128_slice = [0u128; 100usize];
        for (i, chunk) in field_u128_slice_slice.chunks_exact(16usize).enumerate() {
            field_u128_slice[i] = u128::from_le_bytes(chunk.try_into()?);
        }
        let mut field_i8_slice_slice = [0u8; 100usize];
        buf.read_exact(&mut field_i8_slice_slice)?;
        let mut field_i8_slice = [0i8; 100usize];
        for (i, chunk) in field_i8_slice_slice.chunks_exact(1usize).enumerate() {
            field_i8_slice[i] = i8::from_le_bytes(chunk.try_into()?);
        }
        let mut field_i16_slice_slice = [0u8; 200usize];
        buf.read_exact(&mut field_i16_slice_slice)?;
        let mut field_i16_slice = [0i16; 100usize];
        for (i, chunk) in field_i16_slice_slice.chunks_exact(2usize).enumerate() {
            field_i16_slice[i] = i16::from_le_bytes(chunk.try_into()?);
        }
        let mut field_i32_slice_slice = [0u8; 400usize];
        buf.read_exact(&mut field_i32_slice_slice)?;
        let mut field_i32_slice = [0i32; 100usize];
        for (i, chunk) in field_i32_slice_slice.chunks_exact(4usize).enumerate() {
            field_i32_slice[i] = i32::from_le_bytes(chunk.try_into()?);
        }
        let mut field_i64_slice_slice = [0u8; 800usize];
        buf.read_exact(&mut field_i64_slice_slice)?;
        let mut field_i64_slice = [0i64; 100usize];
        for (i, chunk) in field_i64_slice_slice.chunks_exact(8usize).enumerate() {
            field_i64_slice[i] = i64::from_le_bytes(chunk.try_into()?);
        }
        let mut field_i128_slice_slice = [0u8; 1600usize];
        buf.read_exact(&mut field_i128_slice_slice)?;
        let mut field_i128_slice = [0i128; 100usize];
        for (i, chunk) in field_i128_slice_slice.chunks_exact(16usize).enumerate() {
            field_i128_slice[i] = i128::from_le_bytes(chunk.try_into()?);
        }
        let mut field_f32_slice_slice = [0u8; 400usize];
        buf.read_exact(&mut field_f32_slice_slice)?;
        let mut field_f32_slice = [0f32; 100usize];
        for (i, chunk) in field_f32_slice_slice.chunks_exact(4usize).enumerate() {
            field_f32_slice[i] = f32::from_le_bytes(chunk.try_into()?);
        }
        let mut field_f64_slice_slice = [0u8; 800usize];
        buf.read_exact(&mut field_f64_slice_slice)?;
        let mut field_f64_slice = [0f64; 100usize];
        for (i, chunk) in field_f64_slice_slice.chunks_exact(8usize).enumerate() {
            field_f64_slice[i] = f64::from_le_bytes(chunk.try_into()?);
        }
        let mut field_bool_slice_slice = [0u8; 100usize];
        buf.read_exact(&mut field_bool_slice_slice)?;
        let mut field_bool_slice = [false; 100usize];
        for i in 0..100usize {
            field_bool_slice[i] = field_bool_slice_slice[i] != 0;
        }
        let mut crc = [0u8; 4];
        buf.read_exact(&mut crc)?;
        let block = CustomBlock {
            field_u8,
            field_u16,
            field_u32,
            field_u64,
            field_u128,
            field_i8,
            field_i16,
            field_i32,
            field_i64,
            field_i128,
            field_f32,
            field_f64,
            field_bool,
            field_u8_slice,
            field_u16_slice,
            field_u32_slice,
            field_u64_slice,
            field_u128_slice,
            field_i8_slice,
            field_i16_slice,
            field_i32_slice,
            field_i64_slice,
            field_i128_slice,
            field_f32_slice,
            field_f64_slice,
            field_bool_slice,
        };
        if block.crc() != crc {
            return Err(brec::Error::CrcDismatch);
        }
        Ok(block)
    }
}
impl<'a> brec::ReadFromSlice<'a> for CustomBlockReferred<'a> {
    fn read_from_slice(buf: &'a [u8], skip_sig: bool) -> Result<Self, brec::Error>
    where
        Self: Sized,
    {
        if !skip_sig {
            if buf.len() < 4 {
                return Err(brec::Error::NotEnoughtSignatureData(buf.len(), 4));
            }
            if buf[..4] != CUSTOMBLOCK {
                return Err(brec::Error::SignatureDismatch);
            }
        }
        let required = if skip_sig {
            CustomBlock::size() - 4
        } else {
            CustomBlock::size()
        } as usize;
        if buf.len() < required {
            return Err(brec::Error::NotEnoughtData(buf.len(), required));
        }
        let __sig = if skip_sig {
            &CUSTOMBLOCK
        } else {
            <&[u8; 4usize]>::try_from(&buf[0usize..4usize])?
        };
        let field_u8 = u8::from_le_bytes(buf[4usize..5usize].try_into()?);
        let field_u16 = u16::from_le_bytes(buf[5usize..7usize].try_into()?);
        let field_u32 = u32::from_le_bytes(buf[7usize..11usize].try_into()?);
        let field_u64 = u64::from_le_bytes(buf[11usize..19usize].try_into()?);
        let field_u128 = u128::from_le_bytes(buf[19usize..35usize].try_into()?);
        let field_i8 = i8::from_le_bytes(buf[35usize..36usize].try_into()?);
        let field_i16 = i16::from_le_bytes(buf[36usize..38usize].try_into()?);
        let field_i32 = i32::from_le_bytes(buf[38usize..42usize].try_into()?);
        let field_i64 = i64::from_le_bytes(buf[42usize..50usize].try_into()?);
        let field_i128 = i128::from_le_bytes(buf[50usize..66usize].try_into()?);
        let field_f32 = f32::from_le_bytes(buf[66usize..70usize].try_into()?);
        let field_f64 = f64::from_le_bytes(buf[70usize..78usize].try_into()?);
        let field_bool = u8::from_le_bytes(buf[78usize..79usize].try_into()?) == 1;
        let field_u8_slice = <&[u8; 100usize]>::try_from(&buf[79usize..179usize])?;
        let field_u16_slice = {
            let bytes = &buf[179usize..379usize];
            if bytes.as_ptr() as usize % 2usize != 0 {
                return Err(brec::Error::MisalignedPointer);
            }
            if bytes.len() != 200usize {
                return Err(brec::Error::UnexpectedSliceLength);
            }
            let slice = unsafe { &*(bytes.as_ptr() as *const [u16; 100usize]) };
            if false {
                let mut arr = [0u16; 100usize];
                for (i, &value) in slice.iter().enumerate() {
                    arr[i] = u16::from_le(value);
                }
                std::boxed::Box::leak(std::boxed::Box::new(arr))
            } else {
                slice
            }
        };
        let field_u32_slice = {
            let bytes = &buf[379usize..779usize];
            if bytes.as_ptr() as usize % 4usize != 0 {
                return Err(brec::Error::MisalignedPointer);
            }
            if bytes.len() != 400usize {
                return Err(brec::Error::UnexpectedSliceLength);
            }
            let slice = unsafe { &*(bytes.as_ptr() as *const [u32; 100usize]) };
            if false {
                let mut arr = [0u32; 100usize];
                for (i, &value) in slice.iter().enumerate() {
                    arr[i] = u32::from_le(value);
                }
                std::boxed::Box::leak(std::boxed::Box::new(arr))
            } else {
                slice
            }
        };
        let field_u64_slice = {
            let bytes = &buf[779usize..1579usize];
            if bytes.as_ptr() as usize % 8usize != 0 {
                return Err(brec::Error::MisalignedPointer);
            }
            if bytes.len() != 800usize {
                return Err(brec::Error::UnexpectedSliceLength);
            }
            let slice = unsafe { &*(bytes.as_ptr() as *const [u64; 100usize]) };
            if false {
                let mut arr = [0u64; 100usize];
                for (i, &value) in slice.iter().enumerate() {
                    arr[i] = u64::from_le(value);
                }
                std::boxed::Box::leak(std::boxed::Box::new(arr))
            } else {
                slice
            }
        };
        let field_u128_slice = {
            let bytes = &buf[1579usize..3179usize];
            if bytes.as_ptr() as usize % 16usize != 0 {
                return Err(brec::Error::MisalignedPointer);
            }
            if bytes.len() != 1600usize {
                return Err(brec::Error::UnexpectedSliceLength);
            }
            let slice = unsafe { &*(bytes.as_ptr() as *const [u128; 100usize]) };
            if false {
                let mut arr = [0u128; 100usize];
                for (i, &value) in slice.iter().enumerate() {
                    arr[i] = u128::from_le(value);
                }
                std::boxed::Box::leak(std::boxed::Box::new(arr))
            } else {
                slice
            }
        };
        let field_i8_slice = {
            let bytes = &buf[3179usize..3279usize];
            if bytes.len() != 100usize {
                return Err(brec::Error::UnexpectedSliceLength);
            }
            let slice = unsafe { &*(bytes.as_ptr() as *const [i8; 100usize]) };
            if false {
                let mut arr = [0i8; 100usize];
                for (i, &value) in slice.iter().enumerate() {
                    arr[i] = i8::from_le(value);
                }
                std::boxed::Box::leak(std::boxed::Box::new(arr))
            } else {
                slice
            }
        };
        let field_i16_slice = {
            let bytes = &buf[3279usize..3479usize];
            if bytes.as_ptr() as usize % 2usize != 0 {
                return Err(brec::Error::MisalignedPointer);
            }
            if bytes.len() != 200usize {
                return Err(brec::Error::UnexpectedSliceLength);
            }
            let slice = unsafe { &*(bytes.as_ptr() as *const [i16; 100usize]) };
            if false {
                let mut arr = [0i16; 100usize];
                for (i, &value) in slice.iter().enumerate() {
                    arr[i] = i16::from_le(value);
                }
                std::boxed::Box::leak(std::boxed::Box::new(arr))
            } else {
                slice
            }
        };
        let field_i32_slice = {
            let bytes = &buf[3479usize..3879usize];
            if bytes.as_ptr() as usize % 4usize != 0 {
                return Err(brec::Error::MisalignedPointer);
            }
            if bytes.len() != 400usize {
                return Err(brec::Error::UnexpectedSliceLength);
            }
            let slice = unsafe { &*(bytes.as_ptr() as *const [i32; 100usize]) };
            if false {
                let mut arr = [0i32; 100usize];
                for (i, &value) in slice.iter().enumerate() {
                    arr[i] = i32::from_le(value);
                }
                std::boxed::Box::leak(std::boxed::Box::new(arr))
            } else {
                slice
            }
        };
        let field_i64_slice = {
            let bytes = &buf[3879usize..4679usize];
            if bytes.as_ptr() as usize % 8usize != 0 {
                return Err(brec::Error::MisalignedPointer);
            }
            if bytes.len() != 800usize {
                return Err(brec::Error::UnexpectedSliceLength);
            }
            let slice = unsafe { &*(bytes.as_ptr() as *const [i64; 100usize]) };
            if false {
                let mut arr = [0i64; 100usize];
                for (i, &value) in slice.iter().enumerate() {
                    arr[i] = i64::from_le(value);
                }
                std::boxed::Box::leak(std::boxed::Box::new(arr))
            } else {
                slice
            }
        };
        let field_i128_slice = {
            let bytes = &buf[4679usize..6279usize];
            if bytes.as_ptr() as usize % 16usize != 0 {
                return Err(brec::Error::MisalignedPointer);
            }
            if bytes.len() != 1600usize {
                return Err(brec::Error::UnexpectedSliceLength);
            }
            let slice = unsafe { &*(bytes.as_ptr() as *const [i128; 100usize]) };
            if false {
                let mut arr = [0i128; 100usize];
                for (i, &value) in slice.iter().enumerate() {
                    arr[i] = i128::from_le(value);
                }
                std::boxed::Box::leak(std::boxed::Box::new(arr))
            } else {
                slice
            }
        };
        let field_f32_slice = {
            let bytes = &buf[6279usize..6679usize];
            if bytes.as_ptr() as usize % 4usize != 0 {
                return Err(brec::Error::MisalignedPointer);
            }
            if bytes.len() != 100usize * 4usize {
                return Err(brec::Error::UnexpectedSliceLength);
            }
            let slice = unsafe { &*(bytes.as_ptr() as *const [f32; 100usize]) };
            if false {
                let mut arr = [0f32; 100usize];
                for (i, chunk) in bytes.chunks_exact(4usize).enumerate() {
                    arr[i] = f32::from_le_bytes(
                        chunk.try_into().map_err(brec::Error::TryFromSliceError)?,
                    );
                }
                std::boxed::Box::leak(std::boxed::Box::new(arr))
            } else {
                slice
            }
        };
        let field_f64_slice = {
            let bytes = &buf[6679usize..7479usize];
            if bytes.as_ptr() as usize % 8usize != 0 {
                return Err(brec::Error::MisalignedPointer);
            }
            if bytes.len() != 100usize * 8usize {
                return Err(brec::Error::UnexpectedSliceLength);
            }
            let slice = unsafe { &*(bytes.as_ptr() as *const [f64; 100usize]) };
            if false {
                let mut arr = [0f64; 100usize];
                for (i, chunk) in bytes.chunks_exact(8usize).enumerate() {
                    arr[i] = f64::from_le_bytes(
                        chunk.try_into().map_err(brec::Error::TryFromSliceError)?,
                    );
                }
                std::boxed::Box::leak(std::boxed::Box::new(arr))
            } else {
                slice
            }
        };
        let field_bool_slice = {
            let bytes = &buf[7479usize..7579usize];
            if bytes.len() != 100usize {
                return Err(brec::Error::UnexpectedSliceLength);
            }
            unsafe { &*(bytes.as_ptr() as *const [bool; 100usize]) }
        };
        let __crc = u32::from_le_bytes(buf[7579usize..7583usize].try_into()?);
        Ok(CustomBlockReferred {
            __sig,
            field_u8,
            field_u16,
            field_u32,
            field_u64,
            field_u128,
            field_i8,
            field_i16,
            field_i32,
            field_i64,
            field_i128,
            field_f32,
            field_f64,
            field_bool,
            field_u8_slice,
            field_u16_slice,
            field_u32_slice,
            field_u64_slice,
            field_u128_slice,
            field_i8_slice,
            field_i16_slice,
            field_i32_slice,
            field_i64_slice,
            field_i128_slice,
            field_f32_slice,
            field_f64_slice,
            field_bool_slice,
            __crc,
        })
    }
}
impl brec::TryRead for CustomBlock {
    fn try_read<T: std::io::Read + std::io::Seek>(buf: &mut T) -> Result<ReadStatus<Self>, Error>
    where
        Self: Sized,
    {
        let mut sig_buf = [0u8; 4];
        let start_pos = buf.stream_position()?;
        let len = buf.seek(std::io::SeekFrom::End(0))? - start_pos;
        buf.seek(std::io::SeekFrom::Start(start_pos))?;
        if len < 4 {
            return Ok(ReadStatus::NotEnoughtDataToReadSig(4 - len));
        }
        buf.read_exact(&mut sig_buf)?;
        if sig_buf != CUSTOMBLOCK {
            buf.seek(std::io::SeekFrom::Start(start_pos))?;
            return Ok(ReadStatus::DismatchSignature);
        }
        if len < CustomBlock::size() {
            return Ok(ReadStatus::NotEnoughtData(CustomBlock::size() - len));
        }
        Ok(ReadStatus::Success(CustomBlock::read(buf, true)?))
    }
}
impl brec::TryReadBuffered for CustomBlock {
    fn try_read<T: std::io::Read>(buf: &mut T) -> Result<ReadStatus<Self>, Error>
    where
        Self: Sized,
    {
        use std::io::BufRead;
        let mut reader = std::io::BufReader::new(buf);
        let bytes = reader.fill_buf()?;
        if bytes.len() < 4 {
            return Ok(ReadStatus::NotEnoughtDataToReadSig(
                (4 - bytes.len()) as u64,
            ));
        }
        if !bytes.starts_with(&CUSTOMBLOCK) {
            return Ok(ReadStatus::DismatchSignature);
        }
        if (bytes.len() as u64) < CustomBlock::size() {
            return Ok(ReadStatus::NotEnoughtData(
                CustomBlock::size() - bytes.len() as u64,
            ));
        }
        reader.consume(4);
        let blk = CustomBlock::read(&mut reader, true);
        reader.consume(CustomBlock::size() as usize - 4);
        Ok(ReadStatus::Success(blk?))
    }
}
impl brec::Write for CustomBlock {
    fn write<T: std::io::Write>(&self, buf: &mut T) -> std::io::Result<usize> {
        let mut bytes: usize = buf.write(&CUSTOMBLOCK)?;
        bytes += buf.write(&[self.field_u8])?;
        bytes += buf.write(&self.field_u16.to_le_bytes())?;
        bytes += buf.write(&self.field_u32.to_le_bytes())?;
        bytes += buf.write(&self.field_u64.to_le_bytes())?;
        bytes += buf.write(&self.field_u128.to_le_bytes())?;
        bytes += buf.write(&self.field_i8.to_le_bytes())?;
        bytes += buf.write(&self.field_i16.to_le_bytes())?;
        bytes += buf.write(&self.field_i32.to_le_bytes())?;
        bytes += buf.write(&self.field_i64.to_le_bytes())?;
        bytes += buf.write(&self.field_i128.to_le_bytes())?;
        bytes += buf.write(&self.field_f32.to_le_bytes())?;
        bytes += buf.write(&self.field_f64.to_le_bytes())?;
        bytes += buf.write(&[self.field_bool as u8])?;
        bytes += buf.write(&self.field_u8_slice)?;
        let bts = {
            let mut bytes = [0u8; 200usize];
            for (n, &p) in self.field_u16_slice.iter().enumerate() {
                bytes[n * 2usize..n * 2usize + 2usize].copy_from_slice(&p.to_le_bytes());
            }
            bytes
        };
        bytes += buf.write(&bts)?;
        let bts = {
            let mut bytes = [0u8; 400usize];
            for (n, &p) in self.field_u32_slice.iter().enumerate() {
                bytes[n * 4usize..n * 4usize + 4usize].copy_from_slice(&p.to_le_bytes());
            }
            bytes
        };
        bytes += buf.write(&bts)?;
        let bts = {
            let mut bytes = [0u8; 800usize];
            for (n, &p) in self.field_u64_slice.iter().enumerate() {
                bytes[n * 8usize..n * 8usize + 8usize].copy_from_slice(&p.to_le_bytes());
            }
            bytes
        };
        bytes += buf.write(&bts)?;
        let bts = {
            let mut bytes = [0u8; 1600usize];
            for (n, &p) in self.field_u128_slice.iter().enumerate() {
                bytes[n * 16usize..n * 16usize + 16usize].copy_from_slice(&p.to_le_bytes());
            }
            bytes
        };
        bytes += buf.write(&bts)?;
        let bts = {
            let mut bytes = [0u8; 100usize];
            for (n, &p) in self.field_i8_slice.iter().enumerate() {
                bytes[n * 1usize..n * 1usize + 1usize].copy_from_slice(&p.to_le_bytes());
            }
            bytes
        };
        bytes += buf.write(&bts)?;
        let bts = {
            let mut bytes = [0u8; 200usize];
            for (n, &p) in self.field_i16_slice.iter().enumerate() {
                bytes[n * 2usize..n * 2usize + 2usize].copy_from_slice(&p.to_le_bytes());
            }
            bytes
        };
        bytes += buf.write(&bts)?;
        let bts = {
            let mut bytes = [0u8; 400usize];
            for (n, &p) in self.field_i32_slice.iter().enumerate() {
                bytes[n * 4usize..n * 4usize + 4usize].copy_from_slice(&p.to_le_bytes());
            }
            bytes
        };
        bytes += buf.write(&bts)?;
        let bts = {
            let mut bytes = [0u8; 800usize];
            for (n, &p) in self.field_i64_slice.iter().enumerate() {
                bytes[n * 8usize..n * 8usize + 8usize].copy_from_slice(&p.to_le_bytes());
            }
            bytes
        };
        bytes += buf.write(&bts)?;
        let bts = {
            let mut bytes = [0u8; 1600usize];
            for (n, &p) in self.field_i128_slice.iter().enumerate() {
                bytes[n * 16usize..n * 16usize + 16usize].copy_from_slice(&p.to_le_bytes());
            }
            bytes
        };
        bytes += buf.write(&bts)?;
        let bts = {
            let mut bytes = [0u8; 400usize];
            for (n, &p) in self.field_f32_slice.iter().enumerate() {
                bytes[n * 4usize..n * 4usize + 4usize].copy_from_slice(&p.to_le_bytes());
            }
            bytes
        };
        bytes += buf.write(&bts)?;
        let bts = {
            let mut bytes = [0u8; 800usize];
            for (n, &p) in self.field_f64_slice.iter().enumerate() {
                bytes[n * 8usize..n * 8usize + 8usize].copy_from_slice(&p.to_le_bytes());
            }
            bytes
        };
        bytes += buf.write(&bts)?;
        let bts = {
            let mut bytes = [0u8; 100usize];
            for (n, &p) in self.field_bool_slice.iter().enumerate() {
                bytes[n] = p as u8;
            }
            bytes
        };
        bytes += buf.write(&bts)?;
        bytes += buf.write(&self.crc())?;
        Ok(bytes)
    }
    fn write_all<T: std::io::Write>(&self, buf: &mut T) -> std::io::Result<()> {
        buf.write_all(&CUSTOMBLOCK)?;
        buf.write_all(&[self.field_u8])?;
        buf.write_all(&self.field_u16.to_le_bytes())?;
        buf.write_all(&self.field_u32.to_le_bytes())?;
        buf.write_all(&self.field_u64.to_le_bytes())?;
        buf.write_all(&self.field_u128.to_le_bytes())?;
        buf.write_all(&self.field_i8.to_le_bytes())?;
        buf.write_all(&self.field_i16.to_le_bytes())?;
        buf.write_all(&self.field_i32.to_le_bytes())?;
        buf.write_all(&self.field_i64.to_le_bytes())?;
        buf.write_all(&self.field_i128.to_le_bytes())?;
        buf.write_all(&self.field_f32.to_le_bytes())?;
        buf.write_all(&self.field_f64.to_le_bytes())?;
        buf.write_all(&[self.field_bool as u8])?;
        buf.write_all(&self.field_u8_slice)?;
        let bts = {
            let mut bytes = [0u8; 200usize];
            for (n, &p) in self.field_u16_slice.iter().enumerate() {
                bytes[n * 2usize..n * 2usize + 2usize].copy_from_slice(&p.to_le_bytes());
            }
            bytes
        };
        buf.write_all(&bts)?;
        let bts = {
            let mut bytes = [0u8; 400usize];
            for (n, &p) in self.field_u32_slice.iter().enumerate() {
                bytes[n * 4usize..n * 4usize + 4usize].copy_from_slice(&p.to_le_bytes());
            }
            bytes
        };
        buf.write_all(&bts)?;
        let bts = {
            let mut bytes = [0u8; 800usize];
            for (n, &p) in self.field_u64_slice.iter().enumerate() {
                bytes[n * 8usize..n * 8usize + 8usize].copy_from_slice(&p.to_le_bytes());
            }
            bytes
        };
        buf.write_all(&bts)?;
        let bts = {
            let mut bytes = [0u8; 1600usize];
            for (n, &p) in self.field_u128_slice.iter().enumerate() {
                bytes[n * 16usize..n * 16usize + 16usize].copy_from_slice(&p.to_le_bytes());
            }
            bytes
        };
        buf.write_all(&bts)?;
        let bts = {
            let mut bytes = [0u8; 100usize];
            for (n, &p) in self.field_i8_slice.iter().enumerate() {
                bytes[n * 1usize..n * 1usize + 1usize].copy_from_slice(&p.to_le_bytes());
            }
            bytes
        };
        buf.write_all(&bts)?;
        let bts = {
            let mut bytes = [0u8; 200usize];
            for (n, &p) in self.field_i16_slice.iter().enumerate() {
                bytes[n * 2usize..n * 2usize + 2usize].copy_from_slice(&p.to_le_bytes());
            }
            bytes
        };
        buf.write_all(&bts)?;
        let bts = {
            let mut bytes = [0u8; 400usize];
            for (n, &p) in self.field_i32_slice.iter().enumerate() {
                bytes[n * 4usize..n * 4usize + 4usize].copy_from_slice(&p.to_le_bytes());
            }
            bytes
        };
        buf.write_all(&bts)?;
        let bts = {
            let mut bytes = [0u8; 800usize];
            for (n, &p) in self.field_i64_slice.iter().enumerate() {
                bytes[n * 8usize..n * 8usize + 8usize].copy_from_slice(&p.to_le_bytes());
            }
            bytes
        };
        buf.write_all(&bts)?;
        let bts = {
            let mut bytes = [0u8; 1600usize];
            for (n, &p) in self.field_i128_slice.iter().enumerate() {
                bytes[n * 16usize..n * 16usize + 16usize].copy_from_slice(&p.to_le_bytes());
            }
            bytes
        };
        buf.write_all(&bts)?;
        let bts = {
            let mut bytes = [0u8; 400usize];
            for (n, &p) in self.field_f32_slice.iter().enumerate() {
                bytes[n * 4usize..n * 4usize + 4usize].copy_from_slice(&p.to_le_bytes());
            }
            bytes
        };
        buf.write_all(&bts)?;
        let bts = {
            let mut bytes = [0u8; 800usize];
            for (n, &p) in self.field_f64_slice.iter().enumerate() {
                bytes[n * 8usize..n * 8usize + 8usize].copy_from_slice(&p.to_le_bytes());
            }
            bytes
        };
        buf.write_all(&bts)?;
        let bts = {
            let mut bytes = [0u8; 100usize];
            for (n, &p) in self.field_bool_slice.iter().enumerate() {
                bytes[n] = p as u8;
            }
            bytes
        };
        buf.write_all(&bts)?;
        buf.write_all(&self.crc())?;
        Ok(())
    }
}
