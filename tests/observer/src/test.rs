use brec::prelude::*;
use proptest::arbitrary::any;
use proptest::prelude::*;
use proptest::strategy::ValueTree;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    Mutex,
};
use tokio::task::JoinHandle;
use tokio_stream::StreamExt;
use tokio_util::sync::CancellationToken;

use crate::*;

brec::generate!();

#[derive(Clone, PartialEq, PartialOrd, Debug)]
struct WrappedPacket {
    blocks: Vec<Block>,
    payload: Option<Payload>,
}

impl From<&WrappedPacket> for Packet {
    fn from(wrapped: &WrappedPacket) -> Self {
        Packet::new(wrapped.blocks.clone(), wrapped.payload.clone())
    }
}

impl From<Packet> for WrappedPacket {
    fn from(pkg: Packet) -> Self {
        WrappedPacket {
            blocks: pkg.blocks,
            payload: pkg.payload,
        }
    }
}

impl Arbitrary for WrappedPacket {
    type Parameters = bool;

    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(no_blocks: bool) -> Self::Strategy {
        if no_blocks {
            prop::option::of(Payload::arbitrary())
                .prop_map(|payload| WrappedPacket {
                    blocks: Vec::new(),
                    payload,
                })
                .boxed()
        } else {
            (
                prop::collection::vec(Block::arbitrary(), 1..20),
                prop::option::of(Payload::arbitrary()),
            )
                .prop_map(|(blocks, payload)| WrappedPacket { blocks, payload })
                .boxed()
        }
    }
}

#[test]
fn update_reader_by_steps() -> std::io::Result<()> {
    let count = brec::storage::DEFAULT_SLOT_CAPACITY
        .saturating_mul(2)
        .saturating_add(10 + 1);
    let started = std::time::Instant::now();
    println!("Generate {count} packets...");

    let packets = gen_n::<WrappedPacket>(count);

    println!(
        "Generated {count} packets in {}s",
        started.elapsed().as_secs()
    );

    let filename = format!(
        "brec_test_update_reader_by_steps_{}.tmp",
        std::process::id()
    );
    let tmp = std::env::temp_dir().join(&filename);
    let mut wfile = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&tmp)?;

    let mut writer = Writer::new(&mut wfile)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;

    let rfile = std::fs::OpenOptions::new().read(true).open(&tmp)?;

    let mut reader = Reader::new(&rfile)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    let mut last = 0;
    let mut total = 0;
    for (idx, packet) in packets.iter().enumerate() {
        if idx % 30 == 0 || idx == packets.len() - 1 {
            let added = reader.reload().map_err(|err| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string())
            })?;
            println!("Checkpoint {idx}: added {added} packets");
            assert_eq!(added + last, idx);
            // Repeated reload should not add more packets
            let added = reader.reload().map_err(|err| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string())
            })?;
            assert_eq!(added, 0);
            assert_eq!(reader.count(), idx);
            // Try to seek to the last packet, which should be valid
            match reader.seek(last) {
                Err(Error::EmptySource) => {}
                Err(err) => panic!("Unexpected error on seek to {last} from {idx}: {err}"),
                Ok(mut iterator) => {
                    let mut read = 0;
                    // Check packets from the last checkpoint to the current index
                    for (i, expected) in packets[last..idx].iter().enumerate() {
                        read += 1;
                        let Some(pkg) = iterator.next() else {
                            panic!("Expected packet at index {} but got None", last + i);
                        };
                        if let Err(err) = pkg {
                            panic!("Error reading packet at index {}: {err}", last + i);
                        } else {
                            let pkg = pkg.unwrap();
                            let wrapped: WrappedPacket = pkg.into();
                            assert_eq!(
                                &wrapped,
                                expected,
                                "Packet mismatch at index {}: ",
                                last + i,
                            );
                        }
                    }
                    assert!(
                        iterator.next().is_none(),
                        "Expected no more packets after index {}, but got some",
                        idx - 1
                    );
                    println!(
                        "\t- has been read {} packets from index {} to {}; all packets match as expected.",
                        read,
                        last,
                        idx - 1
                    );
                    total += read;
                }
            };
            last = idx;
        }
        writer
            .insert(packet.into())
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    }
    // Last package is not expected to be read since it is added after the last checkpoint
    assert_eq!(total, packets.len() - 1);
    if let Err(err) = std::fs::remove_file(&tmp) {
        eprintln!(
            "Test PASS, but cannot remove tmp file:\nfile:{}\nerror: {err}",
            tmp.display()
        );
    }
    Ok(())
}

#[test]
fn update_reader_by_checkpoints() -> std::io::Result<()> {
    let count = brec::storage::DEFAULT_SLOT_CAPACITY
        .saturating_mul(2)
        .saturating_add(10 + 1);
    let started = std::time::Instant::now();
    println!("Generate {count} packets...");

    let packets = gen_n::<WrappedPacket>(count);

    println!(
        "Generated {count} packets in {}s",
        started.elapsed().as_secs()
    );

    let filename = format!(
        "brec_test_update_reader_by_checkpoints_{}.tmp",
        std::process::id()
    );
    let tmp = std::env::temp_dir().join(&filename);
    let mut wfile = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&tmp)?;

    let mut writer = Writer::new(&mut wfile)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;

    const CHECKPOINTS: [usize; 13] = [
        0,
        1,
        brec::storage::DEFAULT_SLOT_CAPACITY.saturating_div(2),
        brec::storage::DEFAULT_SLOT_CAPACITY.saturating_sub(10),
        brec::storage::DEFAULT_SLOT_CAPACITY.saturating_sub(1),
        brec::storage::DEFAULT_SLOT_CAPACITY,
        brec::storage::DEFAULT_SLOT_CAPACITY.saturating_add(1),
        brec::storage::DEFAULT_SLOT_CAPACITY.saturating_add(10),
        brec::storage::DEFAULT_SLOT_CAPACITY
            .saturating_mul(2)
            .saturating_sub(10),
        brec::storage::DEFAULT_SLOT_CAPACITY
            .saturating_mul(2)
            .saturating_sub(1),
        brec::storage::DEFAULT_SLOT_CAPACITY.saturating_mul(2),
        brec::storage::DEFAULT_SLOT_CAPACITY
            .saturating_mul(2)
            .saturating_add(1),
        brec::storage::DEFAULT_SLOT_CAPACITY
            .saturating_mul(2)
            .saturating_add(10),
    ];

    let rfile = std::fs::OpenOptions::new().read(true).open(&tmp)?;

    let mut reader = Reader::new(&rfile)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    let mut last = 0;
    let mut total = 0;
    for (idx, packet) in packets.iter().enumerate() {
        if CHECKPOINTS.contains(&idx) {
            let added = reader.reload().map_err(|err| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string())
            })?;
            println!("Checkpoint {idx}: added {added} packets");
            assert_eq!(added + last, idx);
            // Repeated reload should not add more packets
            let added = reader.reload().map_err(|err| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string())
            })?;
            assert_eq!(added, 0);
            assert_eq!(reader.count(), idx);
            // Try to seek to the last packet, which should be valid
            match reader.seek(last) {
                Err(Error::EmptySource) => {}
                Err(err) => panic!("Unexpected error on seek to {last} from {idx}: {err}"),
                Ok(mut iterator) => {
                    let mut read = 0;
                    // Check packets from the last checkpoint to the current index
                    for (i, expected) in packets[last..idx].iter().enumerate() {
                        read += 1;
                        let Some(pkg) = iterator.next() else {
                            panic!("Expected packet at index {} but got None", last + i);
                        };
                        if let Err(err) = pkg {
                            panic!("Error reading packet at index {}: {err}", last + i);
                        } else {
                            let pkg = pkg.unwrap();
                            let wrapped: WrappedPacket = pkg.into();
                            assert_eq!(
                                &wrapped,
                                expected,
                                "Packet mismatch at index {}: expected {:?}, got {:?}",
                                last + i,
                                expected,
                                wrapped
                            );
                        }
                    }
                    assert!(
                        iterator.next().is_none(),
                        "Expected no more packets after index {}, but got some",
                        idx - 1
                    );
                    println!(
                        "\t- has been read {} packets from index {} to {}; all packets match as expected.",
                        read,
                        last,
                        idx - 1
                    );
                    total += read;
                }
            };
            last = idx;
        }
        writer
            .insert(packet.into())
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    }
    // Last package is not expected to be read since it is added after the last checkpoint
    assert_eq!(total, packets.len() - 1);
    if let Err(err) = std::fs::remove_file(&tmp) {
        eprintln!(
            "Test PASS, but cannot remove tmp file:\nfile:{}\nerror: {err}",
            tmp.display()
        );
    }
    Ok(())
}

struct MySubscription {
    count: usize,
    expected: usize,
    token: CancellationToken,
}

impl MySubscription {
    pub fn new(expected: usize, token: CancellationToken) -> Self {
        Self {
            count: 0,
            expected,
            token,
        }
    }
}

impl Subscription for MySubscription {
    fn on_update(&mut self, total: usize, _added: usize) -> SubscriptionUpdate {
        if total == self.expected {
            self.token.cancel();
        }
        SubscriptionUpdate::Skip
    }
    fn on_packet(&mut self, _packet: PacketDef<(), Block, Payload, Payload>) {
        self.count += 1;
    }
}

#[derive(Default)]
struct ObserverVerificationState {
    next: usize,
    failure: Option<String>,
}

struct VerifyingSubscription {
    expected: Vec<WrappedPacket>,
    state: Arc<Mutex<ObserverVerificationState>>,
    token: CancellationToken,
}

impl VerifyingSubscription {
    fn new(
        expected: Vec<WrappedPacket>,
        state: Arc<Mutex<ObserverVerificationState>>,
        token: CancellationToken,
    ) -> Self {
        Self {
            expected,
            state,
            token,
        }
    }
}

impl Subscription for VerifyingSubscription {
    fn on_update(&mut self, _total: usize, _added: usize) -> SubscriptionUpdate {
        SubscriptionUpdate::Read
    }

    fn on_packet(&mut self, packet: PacketDef<(), Block, Payload, Payload>) {
        let wrapped: WrappedPacket = packet.into();
        let mut state = self.state.lock().unwrap();
        if state.failure.is_some() {
            self.token.cancel();
            return;
        }
        let idx = state.next;
        let Some(expected) = self.expected.get(idx) else {
            state.failure = Some(format!(
                "Observer emitted an unexpected extra packet at index {idx}: {wrapped:?}"
            ));
            self.token.cancel();
            return;
        };
        if &wrapped != expected {
            state.failure = Some(format!(
                "Packet mismatch at index {idx}: expected {expected:?}, got {wrapped:?}"
            ));
            self.token.cancel();
            return;
        }
        state.next += 1;
        if state.next == self.expected.len() {
            self.token.cancel();
        }
    }

    fn on_error(&mut self, err: &brec::Error) -> SubscriptionErrorAction {
        let mut state = self.state.lock().unwrap();
        state.failure = Some(format!("Observer returned an error: {err}"));
        self.token.cancel();
        SubscriptionErrorAction::Stop
    }

    fn on_stopped(&mut self, reason: Option<brec::Error>) {
        if let Some(reason) = reason {
            let mut state = self.state.lock().unwrap();
            if state.failure.is_none() {
                state.failure = Some(format!("Observer stopped unexpectedly: {reason}"));
            }
            self.token.cancel();
        }
    }

    fn on_aborted(&mut self) {
        let mut state = self.state.lock().unwrap();
        if state.failure.is_none() && state.next != self.expected.len() {
            state.failure = Some(format!(
                "Observer aborted after reading {} of {} packets",
                state.next,
                self.expected.len()
            ));
        }
    }
}

#[tokio::test]
async fn observer() -> std::io::Result<()> {
    let count = brec::storage::DEFAULT_SLOT_CAPACITY
        .saturating_mul(2)
        .saturating_add(10 + 1);
    let started = std::time::Instant::now();
    println!("Generate {count} packets...");

    let packets = gen_n::<WrappedPacket>(count);

    println!(
        "Generated {count} packets in {}s",
        started.elapsed().as_secs()
    );

    let filename = format!("brec_test_observer_{}.tmp", std::process::id());
    let tmp = std::env::temp_dir().join(&filename);
    let mut wfile = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&tmp)?;

    let token = CancellationToken::new();
    let options =
        FileObserverOptions::new(&tmp).subscribe(MySubscription::new(count, token.clone()));
    let mut observer = FileObserver::new(options)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    let handle: JoinHandle<Result<(), Error>> = tokio::task::spawn(async move {
        let mut writer = Writer::new(&mut wfile)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
        for (idx, packet) in packets.iter().enumerate() {
            if idx % 30 == 0 {
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            }
            if let Err(err) = writer.insert(packet.into()) {
                eprintln!("Error during writing: {err}");
                return Err(err);
            }
        }
        Ok(())
    });
    token.cancelled().await;
    let _ = handle.await;
    observer.shutdown().await;

    if let Err(err) = std::fs::remove_file(&tmp) {
        eprintln!(
            "Test PASS, but cannot remove tmp file:\nfile:{}\nerror: {err}",
            tmp.display()
        );
    }
    Ok(())
}

#[tokio::test]
async fn observer_reads_packets_one_by_one() -> std::io::Result<()> {
    let count = brec::storage::DEFAULT_SLOT_CAPACITY
        .saturating_mul(2)
        .saturating_add(1);
    let started = std::time::Instant::now();
    println!("Generate {count} packets...");

    let packets = gen_n::<WrappedPacket>(count);

    println!(
        "Generated {count} packets in {}s",
        started.elapsed().as_secs()
    );

    let filename = format!("brec_test_observer_one_by_one_{}.tmp", std::process::id());
    let tmp = std::env::temp_dir().join(&filename);
    let mut wfile = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&tmp)?;

    let token = CancellationToken::new();
    let state = Arc::new(Mutex::new(ObserverVerificationState::default()));
    let options = FileObserverOptions::new(&tmp).subscribe(VerifyingSubscription::new(
        packets.clone(),
        state.clone(),
        token.clone(),
    ));
    let mut observer = FileObserver::new(options)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;

    let handle: JoinHandle<Result<(), Error>> = tokio::task::spawn(async move {
        let mut writer = Writer::new(&mut wfile)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
        for packet in &packets {
            writer.insert(packet.into())?;
            tokio::time::sleep(tokio::time::Duration::from_millis(3)).await;
        }
        Ok(())
    });

    token.cancelled().await;
    let writer_result = handle
        .await
        .map_err(|err| std::io::Error::other(err.to_string()))?;
    writer_result
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    observer.shutdown().await;

    let state = state.lock().unwrap();
    if let Some(failure) = &state.failure {
        panic!("{failure}");
    }
    assert_eq!(state.next, count, "Observer did not read all packets");

    if let Err(err) = std::fs::remove_file(&tmp) {
        eprintln!(
            "Test PASS, but cannot remove tmp file:\nfile:{}\nerror: {err}",
            tmp.display()
        );
    }
    Ok(())
}

#[tokio::test]
async fn observer_stream_reads_packets_while_writing() -> std::io::Result<()> {
    let count = brec::storage::DEFAULT_SLOT_CAPACITY
        .saturating_mul(2)
        .saturating_add(1);
    let started = std::time::Instant::now();
    println!("Generate {count} packets...");

    let packets = gen_n::<WrappedPacket>(count);

    println!(
        "Generated {count} packets in {}s",
        started.elapsed().as_secs()
    );

    let filename = format!("brec_test_observer_stream_{}.tmp", std::process::id());
    let tmp = std::env::temp_dir().join(&filename);
    let mut wfile = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&tmp)?;

    let mut stream =
        FileObserverStream::new(&tmp)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;

    let expected = packets.clone();
    let done = Arc::new(AtomicBool::new(false));
    let done_writer = done.clone();
    let handle: JoinHandle<Result<(), Error>> = tokio::task::spawn(async move {
        let mut writer = Writer::new(&mut wfile)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
        println!("Start writing...");
        for (idx, packet) in packets.iter().enumerate() {
            writer.insert(packet.into())?;
            if idx == 0 {
                tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
            } else if idx % 50 == 0 {
                tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
            }
        }
        println!("Written {} packets", packets.len());
        done_writer.store(true, Ordering::SeqCst);
        Ok(())
    });

    let mut read = 0usize;
    let mut announced_total = 0usize;
    let mut first_packet_before_finish = false;
    println!("Start reading...");
    while read < expected.len() {
        let event = tokio::time::timeout(tokio::time::Duration::from_secs(10), stream.next())
            .await
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::TimedOut, "observer stream timeout"))?;
        let Some(event) = event else {
            panic!("Observer stream closed unexpectedly after reading {read} packets");
        };
        match event {
            brec::FileObserverEvent::Update { total, .. } => {
                assert!(
                    total >= read,
                    "Observer announced total {total}, but {read} packets were already read"
                );
                announced_total = total;
            }
            brec::FileObserverEvent::Packet(packet) => {
                assert!(
                    announced_total > read,
                    "Packet at index {read} arrived before observer announced it via Update"
                );
                if read == 0 {
                    first_packet_before_finish = !done.load(Ordering::SeqCst);
                    assert!(
                        first_packet_before_finish,
                        "The first packet was emitted only after writing had already finished"
                    );
                }
                let wrapped: WrappedPacket = packet.into();
                assert_eq!(
                    wrapped, expected[read],
                    "Packet mismatch at index {read}"
                );
                read += 1;
            }
            brec::FileObserverEvent::Error(err) => {
                panic!("Observer stream returned an error event: {err}");
            }
            brec::FileObserverEvent::Stopped(reason) => {
                panic!("Observer stream stopped unexpectedly: {reason:?}");
            }
            brec::FileObserverEvent::Aborted => {
                panic!("Observer stream aborted unexpectedly");
            }
        }
    }
    println!("Read {read} packets");
    assert!(first_packet_before_finish);
    let writer_result = handle
        .await
        .map_err(|err| std::io::Error::other(err.to_string()))?;
    writer_result
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
    stream.shutdown().await;

    if let Err(err) = std::fs::remove_file(&tmp) {
        eprintln!(
            "Test PASS, but cannot remove tmp file:\nfile:{}\nerror: {err}",
            tmp.display()
        );
    }
    Ok(())
}

fn gen_n<T: Arbitrary>(n: usize) -> Vec<T> {
    let mut runner = proptest::test_runner::TestRunner::default();
    let strat = any::<T>();

    (0..n)
        .map(|_| strat.new_tree(&mut runner).unwrap().current())
        .collect()
}
