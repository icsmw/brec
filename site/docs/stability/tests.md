
The stability of `brec` is ensured through two levels of testing.

## Functional Testing

To test reading, writing, parsing, filtering, and other functions, `brec` uses `proptest` to generate **random values** for predefined (structurally consistent) blocks and payloads. Blocks and payloads are tested **both separately and in combination** as part of packet testing. 

Packets are constructed with **randomly generated blocks and payloads**. Additionally, the ability of `brec` tools to **reliably read and write randomly generated blocks** is also tested, specifically focusing on `Storage<S: std::io::Read + std::io::Write + std::io::Seek>` and `PacketBufReader`.

In total, **over 40 GB of test data** is generated for this type of testing.

## Macro Testing

To validate the behavior of the `block` and `payload` macros, `brec` also uses `proptest`, but this time it **not only generates random data but also randomly constructs block and payload structures**.

Each randomly generated set of structures is saved as a separate crate. After generating these test cases, each one is **compiled and executed** to ensure stability. Specifically, all randomly generated packets **must be successfully encoded and subsequently decoded without errors**.