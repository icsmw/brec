fn main() {}

#[cfg(test)]
mod tests {
    use brec::prelude::*;
    use futures_util::{SinkExt, StreamExt};
    use protocol::{Packet, gen_n_packets};
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
        let packets = gen_n_packets(count);

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
        let packets = gen_n_packets(count);
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
