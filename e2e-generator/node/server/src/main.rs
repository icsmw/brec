fn main() {}

#[cfg(test)]
mod tests {
    use brec::prelude::*;
    use futures_util::{SinkExt, StreamExt};
    use protocol::{Block, Packet, Payload, gen_n_packets};
    use std::io::Cursor;
    use std::net::SocketAddr;
    use tokio::net::TcpListener;
    use tokio_tungstenite::{accept_async, connect_async, tungstenite::Message};

    fn test_package_count() -> usize {
        std::env::var("TEST_PACKAGE_COUNT")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(1000)
    }

    #[derive(Default)]
    struct Coverage {
        block_u8: bool,
        block_u16: bool,
        block_u32: bool,
        block_i8: bool,
        block_i16: bool,
        block_i32: bool,
        block_i64: bool,
        block_i128: bool,
        block_f32: bool,
        block_f64: bool,
        block_bool: bool,
        block_enums: bool,
        block_comb: bool,
        block_u64: bool,
        block_u128: bool,
        payload_a: bool,
        payload_b: bool,
        payload_c: bool,
        payload_d: bool,
        payload_a_f32_non_zero: bool,
        payload_a_f64_non_zero: bool,
        payload_b_f32_some_non_zero: bool,
        payload_b_f64_some_non_zero: bool,
    }

    impl Coverage {
        fn complete(&self) -> bool {
            self.block_u8
                && self.block_u16
                && self.block_u32
                && self.block_i8
                && self.block_i16
                && self.block_i32
                && self.block_i64
                && self.block_i128
                && self.block_f32
                && self.block_f64
                && self.block_u64
                && self.block_u128
                && self.block_bool
                && self.block_enums
                && self.block_comb
                && self.payload_a
                && self.payload_b
                && self.payload_c
                && self.payload_d
                && self.payload_a_f32_non_zero
                && self.payload_a_f64_non_zero
                && self.payload_b_f32_some_non_zero
                && self.payload_b_f64_some_non_zero
        }

        fn missing_list(&self) -> Vec<&'static str> {
            let mut missing = Vec::new();
            if !self.block_u8 {
                missing.push("BlockU8");
            }
            if !self.block_u16 {
                missing.push("BlockU16");
            }
            if !self.block_u32 {
                missing.push("BlockU32");
            }
            if !self.block_i8 {
                missing.push("BlockI8");
            }
            if !self.block_i16 {
                missing.push("BlockI16");
            }
            if !self.block_i32 {
                missing.push("BlockI32");
            }
            if !self.block_i64 {
                missing.push("BlockI64");
            }
            if !self.block_i128 {
                missing.push("BlockI128");
            }
            if !self.block_f32 {
                missing.push("BlockF32");
            }
            if !self.block_f64 {
                missing.push("BlockF64");
            }
            if !self.block_u64 {
                missing.push("BlockU64");
            }
            if !self.block_u128 {
                missing.push("BlockU128");
            }
            if !self.block_bool {
                missing.push("BlockBool");
            }
            if !self.block_enums {
                missing.push("BlockEnums");
            }
            if !self.block_comb {
                missing.push("BlockCombination");
            }
            if !self.payload_a {
                missing.push("PayloadA");
            }
            if !self.payload_b {
                missing.push("PayloadB");
            }
            if !self.payload_c {
                missing.push("PayloadC");
            }
            if !self.payload_d {
                missing.push("PayloadD");
            }
            if !self.payload_a_f32_non_zero {
                missing.push("PayloadA.field_f32!=0");
            }
            if !self.payload_a_f64_non_zero {
                missing.push("PayloadA.field_f64!=0");
            }
            if !self.payload_b_f32_some_non_zero {
                missing.push("PayloadB.field_f32=Some(!=0)");
            }
            if !self.payload_b_f64_some_non_zero {
                missing.push("PayloadB.field_f64=Some(!=0)");
            }
            missing
        }
    }

    fn measure_coverage(packets: &[Packet]) -> Coverage {
        let mut c = Coverage::default();
        for packet in packets {
            for block in &packet.blocks {
                match block {
                    Block::BlockU8(_) => c.block_u8 = true,
                    Block::BlockU16(_) => c.block_u16 = true,
                    Block::BlockU32(_) => c.block_u32 = true,
                    Block::BlockI8(_) => c.block_i8 = true,
                    Block::BlockI16(_) => c.block_i16 = true,
                    Block::BlockI32(_) => c.block_i32 = true,
                    Block::BlockI64(_) => c.block_i64 = true,
                    Block::BlockI128(_) => c.block_i128 = true,
                    Block::BlockF32(_) => c.block_f32 = true,
                    Block::BlockF64(_) => c.block_f64 = true,
                    Block::BlockU64(_) => c.block_u64 = true,
                    Block::BlockU128(_) => c.block_u128 = true,
                    Block::BlockBool(_) => c.block_bool = true,
                    Block::BlockEnums(_) => c.block_enums = true,
                    Block::BlockCombination(_) => c.block_comb = true,
                }
            }

            if let Some(payload) = &packet.payload {
                match payload {
                    Payload::PayloadA(p) => {
                        c.payload_a = true;
                        if p.field_f32 != 0.0 {
                            c.payload_a_f32_non_zero = true;
                        }
                        if p.field_f64 != 0.0 {
                            c.payload_a_f64_non_zero = true;
                        }
                    }
                    Payload::PayloadB(p) => {
                        c.payload_b = true;
                        if p.field_f32.is_some_and(|v| v != 0.0) {
                            c.payload_b_f32_some_non_zero = true;
                        }
                        if p.field_f64.is_some_and(|v| v != 0.0) {
                            c.payload_b_f64_some_non_zero = true;
                        }
                    }
                    Payload::PayloadC(_) => c.payload_c = true,
                    Payload::PayloadD(_) => c.payload_d = true,
                    Payload::String(_) => {}
                    Payload::Bytes(_) => {}
                }
            }
        }
        c
    }

    fn gen_n_packets_with_coverage(count: usize) -> Vec<Packet> {
        const MAX_ATTEMPTS: usize = 64;
        let mut last_missing = Vec::new();

        for _ in 0..MAX_ATTEMPTS {
            let packets = gen_n_packets(count);
            let coverage = measure_coverage(&packets);
            if coverage.complete() {
                return packets;
            }
            last_missing = coverage.missing_list();
        }

        panic!(
            "failed to build packet set with required napi coverage after {} attempts; missing: {}",
            MAX_ATTEMPTS,
            last_missing.join(", ")
        );
    }

    fn pack(mut packet: Packet) -> Result<Vec<u8>, String> {
        let mut out = Vec::new();
        packet
            .write_all(&mut out, &mut ())
            .map_err(|e| format!("write packet failed: {e}"))?;
        Ok(out)
    }

    fn unpack(bytes: &[u8]) -> Result<Packet, String> {
        Packet::read(&mut Cursor::new(bytes), &mut ())
            .map_err(|e| format!("read packet failed: {e}"))
    }

    async fn run_server(listener: TcpListener, packets: Vec<Packet>) -> Result<(), String> {
        let (stream, peer) = listener
            .accept()
            .await
            .map_err(|e| format!("accept failed: {e}"))?;

        let mut ws = accept_async(stream)
            .await
            .map_err(|e| format!("handshake failed: {e}"))?;

        let total = packets.len();
        println!("[server-test] client connected: {peer}, total packets: {total}");

        let mut pending = packets;
        let mut processed: usize = 0;
        let mut sent_bytes: usize = 0;
        let mut recv_bytes: usize = 0;
        while let Some(packet) = pending.pop() {
            let expected_bytes = pack(packet)?;
            let expected_packet = unpack(&expected_bytes)?;
            sent_bytes = sent_bytes.saturating_add(expected_bytes.len());

            ws.send(Message::Binary(expected_bytes.clone().into()))
                .await
                .map_err(|e| format!("send failed: {e}"))?;

            let message = ws
                .next()
                .await
                .ok_or_else(|| "client disconnected before echo".to_string())?
                .map_err(|e| format!("read ws frame failed: {e}"))?;

            let received = match message {
                Message::Binary(bytes) => bytes,
                Message::Close(_) => {
                    return Err(format!(
                        "client sent close frame before finishing at idx={} of {}",
                        processed + 1,
                        total
                    ));
                }
                other => {
                    return Err(format!("unexpected frame from client: {other:?}"));
                }
            };
            recv_bytes = recv_bytes.saturating_add(received.len());

            let received_packet = unpack(received.as_ref())?;
            let normalized_expected = pack(expected_packet)?;
            let normalized_received = pack(received_packet)?;

            if normalized_expected != normalized_received {
                return Err("packet mismatch after echo".to_string());
            }

            // Packet is verified; remove from queue by not pushing it back.
            processed = processed.saturating_add(1);
            if processed % 10 == 0 || processed == total {
                println!("[server-test] progress: {processed}/{total}");
            }
        }

        ws.send(Message::Close(None))
            .await
            .map_err(|e| format!("send close failed: {e}"))?;

        if processed != total {
            return Err(format!(
                "verification incomplete: processed={processed}, expected={total}"
            ));
        }
        println!(
            "SERVER_SUMMARY packets={} sent_bytes={} recv_bytes={} verified=true",
            processed, sent_bytes, recv_bytes
        );

        Ok(())
    }

    fn e2e_bind_addr() -> Result<SocketAddr, String> {
        std::env::var("TEST_WS_ADDR")
            .ok()
            .unwrap_or_else(|| "127.0.0.1:19001".to_string())
            .parse::<SocketAddr>()
            .map_err(|e| format!("invalid TEST_WS_ADDR: {e}"))
    }

    async fn run_echo_client(addr: SocketAddr) -> Result<(), String> {
        let url = format!("ws://{addr}");
        let (mut ws, _) = connect_async(url)
            .await
            .map_err(|e| format!("client connect failed: {e}"))?;

        while let Some(message) = ws.next().await {
            let message = message.map_err(|e| format!("client read failed: {e}"))?;

            match message {
                Message::Binary(bytes) => {
                    ws.send(Message::Binary(bytes))
                        .await
                        .map_err(|e| format!("client echo send failed: {e}"))?;
                }
                Message::Close(_) => break,
                _ => {}
            }
        }

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn server_roundtrip_binary_packets() -> Result<(), Box<dyn std::error::Error>> {
        let count = test_package_count();
        let packets = gen_n_packets_with_coverage(count);

        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;

        let server_task = tokio::spawn(run_server(listener, packets));
        let client_task = tokio::spawn(run_echo_client(addr));

        let client_res = client_task.await?;
        if let Err(err) = client_res {
            return Err(err.into());
        }

        let server_res = server_task.await?;
        if let Err(err) = server_res {
            return Err(err.into());
        }

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn server_roundtrip_binary_packets_external_client(
    ) -> Result<(), Box<dyn std::error::Error>> {
        if std::env::var("BROWSER_E2E").ok().as_deref() != Some("1") {
            eprintln!("BROWSER_E2E != 1, skip external-client test");
            return Ok(());
        }

        let count = test_package_count();
        let packets = gen_n_packets_with_coverage(count);
        let addr = e2e_bind_addr()?;
        let listener = TcpListener::bind(addr).await?;
        let local = listener.local_addr()?;

        println!("READY_WS_ADDR={local}");
        run_server(listener, packets)
            .await
            .map_err(std::io::Error::other)?;
        Ok(())
    }
}
