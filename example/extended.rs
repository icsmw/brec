#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use brec::*;
struct MyBlock {
    field: u8,
    log_level: u8,
}
#[repr(C)]
struct MyBlockReferred<'a>
where
    Self: Sized,
{
    __sig: &'a [u8; 4usize],
    field: u8,
    log_level: u8,
    __crc: u32,
}
#[automatically_derived]
impl<'a> ::core::fmt::Debug for MyBlockReferred<'a>
where
    Self: Sized,
{
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field4_finish(
            f,
            "MyBlockReferred",
            "__sig",
            &self.__sig,
            "field",
            &self.field,
            "log_level",
            &self.log_level,
            "__crc",
            &&self.__crc,
        )
    }
}
impl<'a> From<MyBlockReferred<'a>> for MyBlock {
    fn from(packet: MyBlockReferred<'a>) -> Self {
        MyBlock {
            field: packet.field,
            log_level: packet.log_level,
        }
    }
}
const MYBLOCK: [u8; 4] = [254u8, 32u8, 165u8, 251u8];
impl Signature for MyBlockReferred<'_> {
    fn sig() -> &'static [u8; 4] {
        &MYBLOCK
    }
}
impl brec::Crc for MyBlock {
    fn crc(&self) -> [u8; 4] {
        let mut hasher = brec::crc32fast::Hasher::new();
        hasher.update(&[self.field]);
        hasher.update(&[self.log_level]);
        hasher.finalize().to_le_bytes()
    }
}
impl brec::Size for MyBlock {
    fn size(&self) -> usize {
        10usize
    }
}
impl brec::Read for MyBlock {
    fn read<T: std::io::Read>(buf: &mut T) -> Result<Self, brec::Error>
    where
        Self: Sized,
    {
        let mut sig = [0u8; 4];
        buf.read_exact(&mut sig)?;
        if sig != MYBLOCK {
            return Err(brec::Error::SignatureDismatch);
        }
        let mut field = [0u8; 1];
        buf.read_exact(&mut field)?;
        let field = field[0];
        let mut log_level = [0u8; 1];
        buf.read_exact(&mut log_level)?;
        let log_level = log_level[0];
        let mut crc = [0u8; 4];
        buf.read_exact(&mut crc)?;
        let packet = MyBlock { field, log_level };
        if packet.crc() != crc {
            return Err(brec::Error::CrcDismatch);
        }
        Ok(packet)
    }
}
impl<'a> brec::ReadFromSlice<'a> for MyBlockReferred<'a> {
    fn read_from_slice(buf: &'a [u8]) -> Result<Self, brec::Error>
    where
        Self: Sized,
    {
        if buf.len() < 4 {
            return Err(brec::Error::NotEnoughtSignatureData(buf.len(), 4));
        }
        if buf[..4] != MYBLOCK {
            return Err(brec::Error::SignatureDismatch);
        }
        if buf.len() < std::mem::size_of::<MyBlock>() {
            return Err(
                brec::Error::NotEnoughtData(buf.len(), std::mem::size_of::<MyBlock>()),
            );
        }
        let __sig = <&[u8; 4usize]>::try_from(&buf[0usize..4usize])?;
        let field = u8::from_le_bytes(buf[4usize..5usize].try_into()?);
        let log_level = u8::from_le_bytes(buf[5usize..6usize].try_into()?);
        let __crc = u32::from_le_bytes(buf[6usize..10usize].try_into()?);
        Ok(MyBlockReferred {
            __sig,
            field,
            log_level,
            __crc,
        })
    }
}
impl brec::Write for MyBlock {
    fn write<T: std::io::Write>(&self, buf: &mut T) -> std::io::Result<usize> {
        Ok(
            buf.write(&MYBLOCK)? + buf.write(&[self.field])?
                + buf.write(&[self.log_level])? + buf.write(&self.crc())?,
        )
    }
    fn write_all<T: std::io::Write>(&self, buf: &mut T) -> std::io::Result<()> {
        buf.write_all(&MYBLOCK)?;
        buf.write_all(&[self.field])?;
        buf.write_all(&[self.log_level])?;
        buf.write_all(&self.crc())?;
        Ok(())
    }
}
struct MyBlock2 {
    field: u8,
    log_level: u8,
}
#[repr(C)]
struct MyBlock2Referred<'a>
where
    Self: Sized,
{
    __sig: &'a [u8; 4usize],
    field: u8,
    log_level: u8,
    __crc: u32,
}
#[automatically_derived]
impl<'a> ::core::fmt::Debug for MyBlock2Referred<'a>
where
    Self: Sized,
{
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field4_finish(
            f,
            "MyBlock2Referred",
            "__sig",
            &self.__sig,
            "field",
            &self.field,
            "log_level",
            &self.log_level,
            "__crc",
            &&self.__crc,
        )
    }
}
impl<'a> From<MyBlock2Referred<'a>> for MyBlock {
    fn from(packet: MyBlock2Referred<'a>) -> Self {
        MyBlock {
            field: packet.field,
            log_level: packet.log_level,
        }
    }
}
const MYBLOCK2: [u8; 4] = [2u8, 174u8, 37u8, 243u8];
impl Signature for MyBlock2Referred<'_> {
    fn sig() -> &'static [u8; 4] {
        &MYBLOCK2
    }
}
impl brec::Crc for MyBlock2 {
    fn crc(&self) -> [u8; 4] {
        let mut hasher = brec::crc32fast::Hasher::new();
        hasher.update(&[self.field]);
        hasher.update(&[self.log_level]);
        hasher.finalize().to_le_bytes()
    }
}
impl brec::Size for MyBlock2 {
    fn size(&self) -> usize {
        10usize
    }
}
impl brec::Read for MyBlock2 {
    fn read<T: std::io::Read>(buf: &mut T) -> Result<Self, brec::Error>
    where
        Self: Sized,
    {
        let mut sig = [0u8; 4];
        buf.read_exact(&mut sig)?;
        if sig != MYBLOCK2 {
            return Err(brec::Error::SignatureDismatch);
        }
        let mut field = [0u8; 1];
        buf.read_exact(&mut field)?;
        let field = field[0];
        let mut log_level = [0u8; 1];
        buf.read_exact(&mut log_level)?;
        let log_level = log_level[0];
        let mut crc = [0u8; 4];
        buf.read_exact(&mut crc)?;
        let packet = MyBlock2 { field, log_level };
        if packet.crc() != crc {
            return Err(brec::Error::CrcDismatch);
        }
        Ok(packet)
    }
}
impl<'a> brec::ReadFromSlice<'a> for MyBlock2Referred<'a> {
    fn read_from_slice(buf: &'a [u8]) -> Result<Self, brec::Error>
    where
        Self: Sized,
    {
        if buf.len() < 4 {
            return Err(brec::Error::NotEnoughtSignatureData(buf.len(), 4));
        }
        if buf[..4] != MYBLOCK2 {
            return Err(brec::Error::SignatureDismatch);
        }
        if buf.len() < std::mem::size_of::<MyBlock2>() {
            return Err(
                brec::Error::NotEnoughtData(buf.len(), std::mem::size_of::<MyBlock2>()),
            );
        }
        let __sig = <&[u8; 4usize]>::try_from(&buf[0usize..4usize])?;
        let field = u8::from_le_bytes(buf[4usize..5usize].try_into()?);
        let log_level = u8::from_le_bytes(buf[5usize..6usize].try_into()?);
        let __crc = u32::from_le_bytes(buf[6usize..10usize].try_into()?);
        Ok(MyBlock2Referred {
            __sig,
            field,
            log_level,
            __crc,
        })
    }
}
impl brec::Write for MyBlock2 {
    fn write<T: std::io::Write>(&self, buf: &mut T) -> std::io::Result<usize> {
        Ok(
            buf.write(&MYBLOCK2)? + buf.write(&[self.field])?
                + buf.write(&[self.log_level])? + buf.write(&self.crc())?,
        )
    }
    fn write_all<T: std::io::Write>(&self, buf: &mut T) -> std::io::Result<()> {
        buf.write_all(&MYBLOCK2)?;
        buf.write_all(&[self.field])?;
        buf.write_all(&[self.log_level])?;
        buf.write_all(&self.crc())?;
        Ok(())
    }
}
struct MyBlock1 {
    field: u8,
    log_level: u8,
}
#[repr(C)]
struct MyBlock1Referred<'a>
where
    Self: Sized,
{
    __sig: &'a [u8; 4usize],
    field: u8,
    log_level: u8,
    __crc: u32,
}
#[automatically_derived]
impl<'a> ::core::fmt::Debug for MyBlock1Referred<'a>
where
    Self: Sized,
{
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field4_finish(
            f,
            "MyBlock1Referred",
            "__sig",
            &self.__sig,
            "field",
            &self.field,
            "log_level",
            &self.log_level,
            "__crc",
            &&self.__crc,
        )
    }
}
impl<'a> From<MyBlock1Referred<'a>> for MyBlock {
    fn from(packet: MyBlock1Referred<'a>) -> Self {
        MyBlock {
            field: packet.field,
            log_level: packet.log_level,
        }
    }
}
const MYBLOCK1: [u8; 4] = [183u8, 2u8, 127u8, 115u8];
impl Signature for MyBlock1Referred<'_> {
    fn sig() -> &'static [u8; 4] {
        &MYBLOCK1
    }
}
impl brec::Crc for MyBlock1 {
    fn crc(&self) -> [u8; 4] {
        let mut hasher = brec::crc32fast::Hasher::new();
        hasher.update(&[self.field]);
        hasher.update(&[self.log_level]);
        hasher.finalize().to_le_bytes()
    }
}
impl brec::Size for MyBlock1 {
    fn size(&self) -> usize {
        10usize
    }
}
impl brec::Read for MyBlock1 {
    fn read<T: std::io::Read>(buf: &mut T) -> Result<Self, brec::Error>
    where
        Self: Sized,
    {
        let mut sig = [0u8; 4];
        buf.read_exact(&mut sig)?;
        if sig != MYBLOCK1 {
            return Err(brec::Error::SignatureDismatch);
        }
        let mut field = [0u8; 1];
        buf.read_exact(&mut field)?;
        let field = field[0];
        let mut log_level = [0u8; 1];
        buf.read_exact(&mut log_level)?;
        let log_level = log_level[0];
        let mut crc = [0u8; 4];
        buf.read_exact(&mut crc)?;
        let packet = MyBlock1 { field, log_level };
        if packet.crc() != crc {
            return Err(brec::Error::CrcDismatch);
        }
        Ok(packet)
    }
}
impl<'a> brec::ReadFromSlice<'a> for MyBlock1Referred<'a> {
    fn read_from_slice(buf: &'a [u8]) -> Result<Self, brec::Error>
    where
        Self: Sized,
    {
        if buf.len() < 4 {
            return Err(brec::Error::NotEnoughtSignatureData(buf.len(), 4));
        }
        if buf[..4] != MYBLOCK1 {
            return Err(brec::Error::SignatureDismatch);
        }
        if buf.len() < std::mem::size_of::<MyBlock1>() {
            return Err(
                brec::Error::NotEnoughtData(buf.len(), std::mem::size_of::<MyBlock1>()),
            );
        }
        let __sig = <&[u8; 4usize]>::try_from(&buf[0usize..4usize])?;
        let field = u8::from_le_bytes(buf[4usize..5usize].try_into()?);
        let log_level = u8::from_le_bytes(buf[5usize..6usize].try_into()?);
        let __crc = u32::from_le_bytes(buf[6usize..10usize].try_into()?);
        Ok(MyBlock1Referred {
            __sig,
            field,
            log_level,
            __crc,
        })
    }
}
impl brec::Write for MyBlock1 {
    fn write<T: std::io::Write>(&self, buf: &mut T) -> std::io::Result<usize> {
        Ok(
            buf.write(&MYBLOCK1)? + buf.write(&[self.field])?
                + buf.write(&[self.log_level])? + buf.write(&self.crc())?,
        )
    }
    fn write_all<T: std::io::Write>(&self, buf: &mut T) -> std::io::Result<()> {
        buf.write_all(&MYBLOCK1)?;
        buf.write_all(&[self.field])?;
        buf.write_all(&[self.log_level])?;
        buf.write_all(&self.crc())?;
        Ok(())
    }
}
pub(crate) enum Block {
    MyBlock(MyBlock),
    MyBlock2(MyBlock2),
    MyBlock1(MyBlock1),
}
fn main() {
    {
        ::std::io::_print(format_args!("Hello, world!\n"));
    };
}
