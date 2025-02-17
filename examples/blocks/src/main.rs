#[cfg(test)]
mod block;

// mod bincode_ex;
#[cfg(test)]
mod block_with_enum;
mod extended_blocks_1;
// mod extended_block;
// mod extended_blocks_2;
// #[brec::payload(bincode)]
// #[derive(serde::Deserialize, serde::Serialize)]
// struct PayloadA {
//     pub str: String,
//     pub num: u32,
//     pub list: Vec<String>,
// }

// #[brec::payload(bincode)]
// #[derive(serde::Deserialize, serde::Serialize)]
// struct PayloadB {
//     pub str: String,
//     pub num: u32,
//     pub list: Vec<String>,
// }

// #[brec::block]
// struct BlockA {
//     a: u32,
//     b: u64,
//     c: [u8; 100],
// }

// brec::include_generated!();

// #[brec::block]
// struct BlockC {
//     aa: i32,
//     bb: i64,
//     cc: [u8; 100],
// }

// brec::include_generated!();

fn main() {
    println!("This is just an example. No sence to run it ;)");
}
