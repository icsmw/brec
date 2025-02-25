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
pub struct BlockA {
    a: u32,
    b: u64,
    c: [u8; 100],
}

#[block]
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
    let mut payloads = vec![
        Payload::PayloadA(PayloadA {
            str: String::from("(1) Hello World!"),
            num: 200,
            list: vec![
                String::from("(1) one"),
                String::from("(1) two"),
                String::from("(1) three"),
            ],
        }),
        Payload::PayloadB(PayloadB {
            str: String::from("B:(1) Hello World!"),
            num: 200,
            list: vec![
                String::from("B:(1) one"),
                String::from("B:(1) two"),
                String::from("B:(1) three"),
            ],
        }),
        Payload::PayloadA(PayloadA {
            str: String::from("(2) Hello World!"),
            num: 400,
            list: vec![
                String::from("(2) one"),
                String::from("(2) two"),
                String::from("(2) three"),
            ],
        }),
        Payload::PayloadB(PayloadB {
            str: String::from("B:(2) Hello World!"),
            num: 400,
            list: vec![
                String::from("B:(2) one"),
                String::from("B:(2) two"),
                String::from("B:(2) three"),
            ],
        }),
        Payload::PayloadA(PayloadA {
            str: String::from("(3) Hello World!"),
            num: 600,
            list: vec![
                String::from("(3) one"),
                String::from("(3) two"),
                String::from("(3) three"),
            ],
        }),
        Payload::PayloadB(PayloadB {
            str: String::from("B:(3) Hello World!"),
            num: 600,
            list: vec![
                String::from("B:(3) one"),
                String::from("B:(3) two"),
                String::from("B:(3) three"),
            ],
        }),
    ];
    for payload in payloads.iter_mut() {
        payload.write_all(&mut buffer).unwrap();
    }
    let mut restored: Vec<Payload> = Vec::new();
    let mut reader = BufReader::new(Cursor::new(buffer));
    let mut n = 0;
    loop {
        if n > payloads.len() {
            panic!("Fail to read payloads");
        }
        let header = match brec::PayloadHeader::read(&mut reader) {
            Ok(header) => header,
            Err(err) => {
                println!("{err}");
                break;
            }
        };
        println!("Read header for payload {} bytes", header.len);
        restored.push(Payload::read(&mut reader, &header).expect("Payload has been read"));
        n += 1;
    }
    assert_eq!(payloads.len(), restored.len());
    for (a, b) in payloads.iter().zip(restored.iter()) {
        if let (Payload::PayloadA(a), Payload::PayloadA(b)) = (a, b) {
            assert_eq!(a, b);
        } else if let (Payload::PayloadB(a), Payload::PayloadB(b)) = (a, b) {
            assert_eq!(a, b);
        } else {
            panic!("Payloads aren't match");
        }
    }
}

fn main() {}
