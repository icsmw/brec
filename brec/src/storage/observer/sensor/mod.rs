mod error;

use crossbeam::channel::{Receiver, Sender};
use notify::{recommended_watcher, Event, RecursiveMode, Result as NotifyResult, Watcher};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
};
use tracing::error;

pub use error::SensorError;

#[derive(Clone, Copy, Debug)]
pub struct Wake {
    pub size: u64,
}

impl Wake {
    pub fn new(size: u64) -> Self {
        Self { size }
    }
}

pub struct Sensor {
    target: PathBuf,
    locked: Arc<AtomicBool>,
    processed_len: Arc<AtomicU64>,
    tx: Sender<Wake>,
    _watcher: notify::RecommendedWatcher,
}

impl Sensor {
    pub fn new(target: impl AsRef<Path>) -> Result<(Self, Receiver<Wake>), SensorError> {
        let target = target.as_ref().to_path_buf();
        if !target.is_file() {
            return Err(SensorError::NotFile(target.to_string_lossy().to_string()));
        }

        let (tx, rx): (Sender<Wake>, Receiver<Wake>) = crossbeam::channel::bounded(1);

        let locked = Arc::new(AtomicBool::new(false));
        let processed_len = Arc::new(AtomicU64::new(0));

        let inner_locked = locked.clone();
        let inner_processed_len = processed_len.clone();
        let inner_target = target.clone();
        let inner_tx = tx.clone();

        let mut watcher = recommended_watcher(move |res: NotifyResult<Event>| {
            if res.is_err() {
                return;
            }
            if let Err(err) = Sensor::emit(
                &inner_target,
                &inner_tx,
                inner_locked.as_ref(),
                inner_processed_len.as_ref(),
            ) {
                error!("Error emitting wake: {}", err);
            }
        })?;

        watcher.watch(&target, RecursiveMode::NonRecursive)?;

        let sensor = Self {
            target,
            locked,
            processed_len,
            tx,
            _watcher: watcher,
        };

        Ok((sensor, rx))
    }

    /// Owner acknowledges the wake and reports it successfully read up to `until` byte offset.
    /// Sensor unlocks and, if there is already more data beyond `until`, emits Wake immediately.
    pub fn processed(&self, until: u64) -> Result<(), SensorError> {
        fetch_max_u64(&self.processed_len, until);

        self.locked.store(false, Ordering::Release);

        Self::emit(&self.target, &self.tx, &self.locked, &self.processed_len)
    }

    fn emit(
        target: &Path,
        tx: &Sender<Wake>,
        locked: &AtomicBool,
        processed_len: &AtomicU64,
    ) -> Result<(), SensorError> {
        let size = fs::metadata(target)?.len();

        if size <= processed_len.load(Ordering::Acquire) {
            return Ok(());
        }

        if locked
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed)
            .is_err()
        {
            return Ok(());
        }

        match tx.try_send(Wake::new(size)) {
            Ok(_) | Err(crossbeam::channel::TrySendError::Full(_)) => {
                // Prevent blocking in notify callback.
                Ok(())
            }
            Err(crossbeam::channel::TrySendError::Disconnected(_)) => {
                Err(SensorError::Disconnected)
            }
        }
    }
}

fn fetch_max_u64(store: &AtomicU64, checked: u64) {
    let mut current = store.load(Ordering::Relaxed);
    while checked > current {
        match store.compare_exchange_weak(current, checked, Ordering::Release, Ordering::Relaxed) {
            Ok(_) => return,
            Err(next) => current = next,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs::{File, OpenOptions},
        io::Write,
        time::Duration,
    };
    use tempfile::tempdir;

    fn append_and_sync(path: &Path, bytes: &[u8]) {
        let mut f = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .expect("open for append");
        f.write_all(bytes).expect("write_all");
        f.flush().expect("flush");
        // sync_all helps to make size/metadata observable promptly on some FS/OS
        f.sync_all().expect("sync_all");
    }

    #[test]
    fn emits_wake_on_append() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("store.bin");

        // Create empty file.
        File::create(&path).expect("create file");

        let (_sensor, rx) = Sensor::new(&path).expect("sensor new");

        // Append some bytes to trigger notify.
        append_and_sync(&path, b"hello");

        let wake = rx
            .recv_timeout(Duration::from_secs(2))
            .expect("expected wake after append");

        assert!(
            wake.size >= 5,
            "wake.size should be >= appended bytes, got {}",
            wake.size
        );
    }

    #[test]
    fn blocks_until_processed_then_retriggers_if_grown() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("store.bin");
        File::create(&path).expect("create file");

        let (sensor, rx) = Sensor::new(&path).expect("sensor new");

        // 1) First append -> should get first wake.
        append_and_sync(&path, b"12345");
        let w1 = rx
            .recv_timeout(Duration::from_secs(2))
            .expect("expected first wake");
        assert!(w1.size >= 5);

        // 2) While NOT calling processed(), sensor is locked.
        // Append more bytes; we should NOT receive a new wake yet.
        append_and_sync(&path, b"67890");
        let no_wake = rx.recv_timeout(Duration::from_millis(200));
        assert!(no_wake.is_err(), "should not emit second wake while locked");

        // 3) Now acknowledge processing up to first wake size.
        // processed() performs an immediate emit() check and should trigger if file grew.
        sensor.processed(w1.size).expect("processed");

        let w2 = rx
            .recv_timeout(Duration::from_secs(2))
            .expect("expected second wake after processed");

        assert!(
            w2.size >= w1.size + 5,
            "second wake should reflect grown file: w1={}, w2={}",
            w1.size,
            w2.size
        );
    }
}
