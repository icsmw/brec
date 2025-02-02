use brec::*;
mod test;
mod test2;
#[block]
pub struct MyBlock {
    field: u8,
    log_level: u8,
}

#[block]
pub struct MyBlock2 {
    field: u8,
    log_level: u8,
}

#[block]
pub struct MyBlock1 {
    field: u8,
    log_level: u8,
}

include_generated!();

fn main() {
    println!("Hello, world!");
}
