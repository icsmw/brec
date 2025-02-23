#[brec::payload(bincode)]
#[derive(serde::Deserialize, serde::Serialize)]
struct PayloadA {
    pub str: String,
    pub num: u32,
    pub list: Vec<String>,
}

#[brec::payload(bincode)]
#[derive(serde::Deserialize, serde::Serialize)]
struct PayloadB {
    pub str: String,
    pub num: u32,
    pub list: Vec<String>,
}

#[brec::block]
struct BlockA {
    a: u32,
    b: u64,
    c: [u8; 100],
}

#[brec::block]
struct BlockB {
    aa: i32,
    bb: i64,
    cc: [u8; 100],
}

brec::include_generated!();

#[test]
fn write_read() {
    use brec::prelude::*;
    let mut buffer: Vec<u8> = Vec::new();
    let mut packets = vec![
        Packet::new(
            vec![
                Block::BlockA(BlockA {
                    a: 1,
                    b: 2,
                    c: [0u8; 100],
                }),
                Block::BlockB(BlockB {
                    aa: 1,
                    bb: 2,
                    cc: [0u8; 100],
                }),
            ],
            None,
        ),
        Packet::new(
            vec![
                Block::BlockA(BlockA {
                    a: 3,
                    b: 4,
                    c: [0u8; 100],
                }),
                Block::BlockB(BlockB {
                    aa: 3,
                    bb: 4,
                    cc: [0u8; 100],
                }),
            ],
            None,
        ),
        Packet::new(
            vec![
                Block::BlockA(BlockA {
                    a: 4,
                    b: 5,
                    c: [0u8; 100],
                }),
                Block::BlockB(BlockB {
                    aa: 4,
                    bb: 5,
                    cc: [0u8; 100],
                }),
            ],
            None,
        ),
    ];
    for pkg in packets.iter_mut() {
        pkg.write_all(&mut buffer).unwrap();
    }
}
