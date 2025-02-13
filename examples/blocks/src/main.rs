#[cfg(test)]
mod block;

// mod bincode_ex;
#[cfg(test)]
mod block_with_enum;
// mod extended;

#[brec::payload(bincode)]
#[derive(serde::Deserialize, serde::Serialize)]
struct MyPayload {
    pub str: String,
    pub num: u32,
    pub list: Vec<String>,
}

fn main() {
    println!("This is just an example. No sence to run it ;)");
}
