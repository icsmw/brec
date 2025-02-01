use brec::*;

#[block]
struct MyBlock {
    field: u8,
    log_level: u8,
}

#[block]
struct MyBlock2 {
    field: u8,
    log_level: u8,
}

#[block]
struct MyBlock1 {
    field: u8,
    log_level: u8,
}

include_generated!();

fn main() {
    println!("Hello, world!");
}
