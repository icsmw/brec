use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use crate::tests::*;
use proc_macro2::TokenStream;
use proptest::prelude::*;
use quote::{format_ident, quote};
use uuid::Uuid;

#[derive(Debug)]
pub(crate) struct TCrate {
    packets: Vec<Packet>,
    folder: Uuid,
}

impl Arbitrary for TCrate {
    type Parameters = ();

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> Self::Strategy {
        prop::collection::vec(Packet::arbitrary_with(()), 1..10)
            .prop_map(move |packets| TCrate {
                packets,
                folder: Uuid::new_v4(),
            })
            .boxed()
    }
}

impl TCrate {
    pub fn write_all<P: AsRef<Path>>(&self, dest: P) -> std::io::Result<()> {
        let crate_path = dest.as_ref().join(self.folder.to_string());
        let src_path = crate_path.join("src");
        fs::create_dir_all(&src_path)?;
        let mut file = File::create(crate_path.join("Cargo.toml"))?;
        file.write_all(include_str!("./cargo.toml.test").as_bytes())?;
        let mut file = File::create(crate_path.join("build.rs"))?;
        file.write_all(include_str!("./build.rs.test").as_bytes())?;
        let mut file = File::create(src_path.join("main.rs"))?;
        file.write_all(self.generate().to_string().as_bytes())?;
        Ok(())
    }

    fn generate(&self) -> TokenStream {
        let declarations = self
            .packets
            .iter()
            .map(|pkg| pkg.declaration(()))
            .collect::<Vec<TokenStream>>();
        let mut packets_refs = Vec::new();
        let insts = self
            .packets
            .iter()
            .map(|pkg| {
                let name = format_ident!("{}", pkg.name);
                packets_refs.push(name.clone());
                let instance = pkg.instance(());
                quote! { let #name = #instance; }
            })
            .collect::<Vec<TokenStream>>();

        quote! {
            use brec::*;

            #(#declarations)*

            brec::include_generated!();

            fn main() {
                #(#insts)*

                let mut packets = vec![
                    #(#packets_refs,)*
                ];

                let mut buffer: Vec<u8> = Vec::new();

                for packet in packets.iter_mut() {
                    packet.write_all(&mut buffer).expect("Data is written");
                }

                let mut restored: Vec<Packet> = Vec::new();
                let mut inner = std::io::BufReader::new(std::io::Cursor::new(buffer));
                let mut reader: PacketBufReader<_, std::io::BufWriter<Vec<u8>>> =
                    PacketBufReader::new(&mut inner);

                loop {
                    match reader.read() {
                        Ok(next) => match next {
                            NextPacket::Found(packet) => restored.push(packet),
                            NextPacket::NotFound => {
                                // Data will be refilled with next call
                            }
                            NextPacket::NotEnoughData(_needed) => {
                                // Data will be refilled with next call
                            }
                            NextPacket::NoData => {
                                break;
                            }
                            NextPacket::Skipped => {
                                //
                            }
                        },
                        Err(err) => {
                            panic!("{err}");
                        }
                    };
                }
                println!("Has been read: {} from {}", restored.len(), packets.len());
            }
        }
    }
}
