use super::block::{BlockFile, BlockInterfaceFile, BlockSupportFile};
use super::client::ClientFile;
use super::file::JavaFile;
use super::packet::PacketFile;
use super::payload::{
    DefaultPayloadFile, HelperTypeFile, PayloadFile, PayloadInterfaceFile, PayloadSupportFile,
};
use crate::*;
use brec_scheme::SchemeFile;
use std::path::{Path, PathBuf};

pub struct JavaFiles<'a> {
    model: &'a Model,
    scheme: SchemeFile,
}

impl<'a> JavaFiles<'a> {
    pub fn new(model: &'a Model, scheme: SchemeFile) -> Self {
        Self { model, scheme }
    }

    pub fn write_to(&self, out: &Path) -> Result<(), Error> {
        for file in self.files()? {
            file.write_to(out)?;
        }
        Ok(())
    }

    pub fn paths(model: &'a Model, scheme: SchemeFile) -> Result<Vec<PathBuf>, Error> {
        Self::new(model, scheme)
            .files()
            .map(|files| files.into_iter().map(|file| file.path).collect())
    }

    fn files(&self) -> Result<Vec<JavaFile>, Error> {
        let mut files = vec![
            ClientFile::new(self.model).file()?,
            PacketFile::new(self.model).file()?,
            BlockInterfaceFile::new(self.model).file()?,
            BlockSupportFile::new(self.model, &self.scheme).file()?,
            PayloadInterfaceFile::new(self.model).file()?,
            PayloadSupportFile::new(self.model, &self.scheme).file()?,
        ];

        for block in &self.scheme.blocks {
            files.push(BlockFile::new(self.model, block).file()?);
        }

        for payload in &self.scheme.config.default_payloads {
            match payload.as_str() {
                "Bytes" => files
                    .push(DefaultPayloadFile::new(self.model, "Bytes", "byte[]", "Bytes").file()?),
                "String" => files.push(
                    DefaultPayloadFile::new(self.model, "StringPayload", "String", "String")
                        .file()?,
                ),
                _ => {}
            }
        }

        for ty in &self.scheme.types {
            files.push(HelperTypeFile::new(self.model, ty).file()?);
        }

        for payload in &self.scheme.payloads {
            files.push(PayloadFile::new(self.model, payload).file()?);
        }

        Ok(files)
    }
}
