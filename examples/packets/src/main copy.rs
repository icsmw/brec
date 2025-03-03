use brec::prelude::*;

#[payload(bincode)]
#[derive(serde::Deserialize, serde::Serialize, PartialEq, PartialOrd, Debug)]
pub struct PayloadA {
    pub str: String,
    pub num: u32,
    pub list: Vec<String>,
}

#[payload(bincode)]
#[derive(serde::Deserialize, serde::Serialize, PartialEq, PartialOrd, Debug)]
pub struct PayloadB {
    pub str: String,
    pub num: u32,
    pub list: Vec<String>,
}

#[block]
#[derive(Debug, PartialEq, PartialOrd)]
pub struct BlockA {
    a: u32,
    b: u64,
    c: [u8; 100],
}

#[block]
#[derive(Debug, PartialEq, PartialOrd)]
pub struct BlockB {
    aa: i32,
    bb: i64,
    cc: [u8; 100],
}

brec::include_generated!();

#[test]
fn write_read() {
    use brec::prelude::*;
    use std::io::{BufReader, Cursor};
    let mut buffer: Vec<u8> = Vec::new();
    let mut packets = vec![
        Packet::new(
            vec![
                Block::BlockA(BlockA {
                    a: 1,
                    b: 2,
                    c: [3u8; 100],
                }),
                Block::BlockB(BlockB {
                    aa: 11,
                    bb: 22,
                    cc: [4u8; 100],
                }),
                Block::BlockA(BlockA {
                    a: 111,
                    b: 222,
                    c: [4u8; 100],
                }),
            ],
            Some(Payload::PayloadA(PayloadA {
                str: String::from("(1) Hello World!"),
                num: 200,
                list: vec![
                    String::from("(1) one"),
                    String::from("(1) two"),
                    String::from("(1) three"),
                ],
            })),
        ),
        Packet::new(
            vec![Block::BlockB(BlockB {
                aa: 11,
                bb: 22,
                cc: [4u8; 100],
            })],
            Some(Payload::PayloadB(PayloadB {
                str: String::from("B:(1) Hello World!"),
                num: 200,
                list: vec![
                    String::from("B:(1) one"),
                    String::from("B:(1) two"),
                    String::from("B:(1) three"),
                ],
            })),
        ),
        Packet::new(
            vec![Block::BlockA(BlockA {
                a: 1,
                b: 2,
                c: [3u8; 100],
            })],
            Some(Payload::PayloadA(PayloadA {
                str: String::from("(2) Hello World!"),
                num: 400,
                list: vec![
                    String::from("(2) one"),
                    String::from("(2) two"),
                    String::from("(2) three"),
                ],
            })),
        ),
        Packet::new(
            vec![
                Block::BlockA(BlockA {
                    a: 1,
                    b: 2,
                    c: [3u8; 100],
                }),
                Block::BlockA(BlockA {
                    a: 111,
                    b: 222,
                    c: [4u8; 100],
                }),
            ],
            Some(Payload::PayloadB(PayloadB {
                str: String::from("B:(2) Hello World!"),
                num: 400,
                list: vec![
                    String::from("B:(2) one"),
                    String::from("B:(2) two"),
                    String::from("B:(2) three"),
                ],
            })),
        ),
        Packet::new(
            vec![
                Block::BlockA(BlockA {
                    a: 1,
                    b: 2,
                    c: [3u8; 100],
                }),
                Block::BlockB(BlockB {
                    aa: 11,
                    bb: 22,
                    cc: [4u8; 100],
                }),
                Block::BlockB(BlockB {
                    aa: 11,
                    bb: 22,
                    cc: [4u8; 100],
                }),
                Block::BlockA(BlockA {
                    a: 111,
                    b: 222,
                    c: [4u8; 100],
                }),
            ],
            Some(Payload::PayloadA(PayloadA {
                str: String::from("(3) Hello World!"),
                num: 600,
                list: vec![
                    String::from("(3) one"),
                    String::from("(3) two"),
                    String::from("(3) three"),
                ],
            })),
        ),
        Packet::new(
            vec![],
            Some(Payload::PayloadB(PayloadB {
                str: String::from("B:(3) Hello World!"),
                num: 600,
                list: vec![
                    String::from("B:(3) one"),
                    String::from("B:(3) two"),
                    String::from("B:(3) three"),
                ],
            })),
        ),
    ];
    for packet in packets.iter_mut() {
        let before = buffer.len();
        packet.write_all(&mut buffer).unwrap();
        println!("Written packet: {} bytes", buffer.len() - before);
    }
    let mut restored: Vec<Packet> = Vec::new();
    let mut inner = BufReader::new(Cursor::new(buffer));
    let mut reader: PacketBufReader<_, std::io::BufWriter<Vec<u8>>> =
        PacketBufReader::new(&mut inner);
    println!("start reading");
    loop {
        match reader.read() {
            Ok(next) => match next {
                NextPacket::Found(packet) => restored.push(packet),
                _ => {
                    break;
                }
            },
            Err(err) => {
                println!("{err}");
                break;
            }
        };
        println!("Read packet");
    }
    assert_eq!(packets.len(), restored.len());
    for (a, b) in packets.iter().zip(restored.iter()) {
        for (a, b) in a.blocks.iter().zip(b.blocks.iter()) {
            assert_eq!(a, b);
        }
        if let (Some(a), Some(b)) = (a.payload.as_ref(), b.payload.as_ref()) {
            assert_eq!(a, b);
        } else if a.payload.is_some() || b.payload.is_some() {
            panic!("Payloads are dismatch")
        }
    }
}

fn main() {}
