use fs4::fs_std::FileExt;
use std::{
    fs::{File, OpenOptions},
    path::{Path, PathBuf},
    thread::sleep,
    time::{Duration, Instant},
};

use crate::*;

/// The extension appended to the target filename to create the lock file.
///
/// This file is used to coordinate exclusive access to the associated storage file.
/// For example, if the target is `data.brec`, the lock file will be `data.lock`.
pub const LOCK_EXT: &str = "lock";

/// The default polling interval in milliseconds between attempts to acquire the file lock.
///
/// Used when no custom interval is specified via [`FileStorageOptions::interval`].
pub const WAIT_INTERVAL_MS: u64 = 50;

/// Builder-style configuration for creating a [`FileWriterDef`] instance with custom locking behavior.
///
/// This helper allows you to configure timeout and retry interval settings before opening
/// the storage file. It provides a fluent interface for fine-tuning how `FileWriterDef`
/// attempts to acquire its advisory lock.
///
/// # Examples
///
/// ```ignore
/// use std::time::Duration;
///
/// let storage = FileStorageOptions::new("data.brec")
///     .timeout(Duration::from_secs(2))
///     .interval(Duration::from_millis(100))
///     .open::<MyBlock, MyBlockRef, MyPayload, MyPayloadImpl>()?;
/// ```
pub struct FileStorageOptions {
    interval: Duration,
    timeout: Option<Duration>,
    filename: PathBuf,
}

impl FileStorageOptions {
    /// Creates a new `FileStorageOptions` instance with default parameters.
    ///
    /// By default, the timeout is unset (i.e., the operation will fail immediately if the
    /// file is locked), and the polling interval is set to `WAIT_INTERVAL_MS` (typically 50 ms).
    ///
    /// # Arguments
    ///
    /// * `filename` – Path to the target storage file.
    ///
    /// # Returns
    ///
    /// A `FileStorageOptions` builder instance.
    pub fn new<P: AsRef<Path>>(filename: P) -> Self {
        Self {
            interval: Duration::from_millis(WAIT_INTERVAL_MS),
            timeout: None,
            filename: filename.as_ref().to_path_buf(),
        }
    }

    /// Sets the polling interval between lock acquisition attempts.
    ///
    /// If the lock is held by another process, this interval determines how frequently the
    /// system checks for availability. Useful for fine-tuning responsiveness vs CPU load.
    ///
    /// # Arguments
    ///
    /// * `interval` – Duration to wait between retries.
    ///
    /// # Returns
    ///
    /// The updated `FileStorageOptions` instance.
    pub fn interval(mut self, interval: Duration) -> Self {
        self.interval = interval;
        self
    }

    /// Sets the maximum amount of time to wait for the lock to become available.
    ///
    /// If the timeout elapses before the lock is acquired, the operation will fail with
    /// [`Error::TimeoutToWaitLockedFile`]. If unset, the lock must be acquired immediately.
    ///
    /// # Arguments
    ///
    /// * `timeout` – Maximum duration to wait for the lock.
    ///
    /// # Returns
    ///
    /// The updated `FileStorageOptions` instance.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Opens the target storage file using the configured lock options.
    ///
    /// This method consumes the builder and delegates to [`FileWriterDef::new`], passing
    /// in the specified filename, timeout, and retry interval.
    ///
    /// # Type Parameters
    ///
    /// * `B` – Block definition type.
    /// * `BR` – Block reference definition.
    /// * `PL` – Payload definition.
    /// * `Inner` – Payload internals.
    ///
    /// # Returns
    ///
    /// A new `FileWriterDef` instance on success, or an appropriate [`Error`] on failure.
    pub fn open<
        B: BlockDef,
        PL: PayloadDef<Inner>,
        Inner: PayloadInnerDef,
    >(
        self,
    ) -> Result<FileWriterDef<B, PL, Inner>, Error> {
        FileWriterDef::new(self.filename, self.timeout, Some(self.interval))
    }
}

/// `FileWriterDef` provides a wrapper around `WriterDef<File, ...>` that attempts to prevent
/// concurrent access to the target storage file by using a filesystem-based locking mechanism.
///
/// When a new instance is created, a `.lock` file is created next to the target file,
/// and an exclusive file lock is applied to it. This lock is respected by other instances
/// of `FileWriterDef`, effectively serializing access to the underlying storage.
///
/// Note: this is an *advisory lock*, which means it only prevents access for code that
/// explicitly respects the lock (e.g., other `brec` - based tools or processes using `fs4`).
/// It does **not** protect the file from being opened or modified by unrelated programs or
/// low-level system utilities.
///
/// `FileWriterDef` also supports an optional timeout while waiting for the lock, enabling
/// coordinated access patterns in multi-process environments.
pub struct FileWriterDef<
    B: BlockDef,
    PL: PayloadDef<Inner>,
    Inner: PayloadInnerDef,
> {
    _filelock: File,
    inner: WriterDef<File, B, PL, Inner>,
}

impl<B: BlockDef, PL: PayloadDef<Inner>, Inner: PayloadInnerDef>
    FileWriterDef<B, PL, Inner>
{
    /// Creates a new instance of `FileWriterDef`, opening the specified storage file and
    /// acquiring an exclusive advisory lock via a `.lock` companion file.
    ///
    /// A `.lock` file is created in the same directory as the target file, and an exclusive
    /// OS-level advisory lock is applied to it. This ensures serialized access between
    /// multiple processes or instances of `FileWriterDef`, assuming they respect the locking protocol.
    ///
    /// > **Note:** The lock is advisory. It does **not** prevent external programs or non-cooperative
    /// > code from accessing or modifying the file directly.
    ///
    /// If the lock is already held by another process, this method will retry acquiring it
    /// until the lock becomes available or the specified timeout is reached.
    ///
    /// # Arguments
    ///
    /// * `filename` - Path to the target storage file.
    /// * `timeout` - Maximum duration to wait for the lock. If `None`, the function fails immediately
    ///   if the file is already locked.
    /// * `interval` - Polling interval to wait between retry attempts. If `None`, a default interval
    ///   of 50 milliseconds is used.
    ///
    /// # Errors
    ///
    /// This function returns:
    ///
    /// * [`Error::PathIsNotFile`] — if the specified path exists but is not a regular file.
    /// * [`Error::TimeoutToWaitLockedFile`] — if the lock is not acquired within the timeout period.
    /// * [`Error::FileIsLocked`] — if the lock is held and no timeout was specified.
    /// * [`Error::FailToLockFile`] — if the `.lock` file cannot be opened or the locking operation fails
    ///   due to a non-recoverable I/O error.
    /// * [`Error::Io`] — if the actual storage file cannot be opened for reading and writing.
    /// * Any error returned by [`WriterDef::new`] if initialization of the inner storage fails.
    ///
    /// # Returns
    ///
    /// A new `FileWriterDef` instance with exclusive access to the specified file, guarded
    /// by a live advisory lock. The lock is automatically released when the instance is dropped.
    pub fn new<P: AsRef<Path>>(
        filename: P,
        timeout: Option<Duration>,
        interval: Option<Duration>,
    ) -> Result<Self, Error> {
        let filename = filename.as_ref().to_path_buf();
        let filename_str = filename.to_string_lossy().to_string();
        if filename.exists() && !filename.is_file() {
            return Err(Error::PathIsNotFile(filename_str));
        }
        let lock_file = filename.with_extension(LOCK_EXT);
        let started = Instant::now();
        let interval = interval.unwrap_or(Duration::from_millis(WAIT_INTERVAL_MS));
        let wait_or_fail = || {
            if let Some(timeout) = timeout {
                if started.elapsed() >= timeout {
                    return Err(Error::TimeoutToWaitLockedFile(filename_str.clone()));
                }
                sleep(interval);
                Ok(())
            } else {
                Err(Error::FileIsLocked(filename_str.clone()))
            }
        };
        let filelock = loop {
            let file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(false)
                .open(&lock_file)
                .map_err(Error::FailToLockFile)?;
            match file.try_lock_exclusive() {
                Ok(lock) => {
                    if lock {
                        break file;
                    } else {
                        wait_or_fail()?;
                    }
                }
                Err(err) => {
                    if err.kind() == std::io::ErrorKind::WouldBlock {
                        wait_or_fail()?;
                    } else {
                        return Err(Error::FailToLockFile(err));
                    }
                }
            };
        };
        let storage_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(filename)?;
        Ok(Self {
            _filelock: filelock,
            inner: WriterDef::new(storage_file)?,
        })
    }


    /// Inserts a new packet into storage at the next available slot.
    ///
    /// # Arguments
    /// * `packet` — The `PacketDef` to be written
    ///
    /// # Returns
    /// * `Ok(())` — Packet successfully written
    /// * `Err(Error)` — If no space is found or write fails
    pub fn insert(&mut self, packet: PacketDef<B, PL, Inner>) -> Result<(), Error> {
        self.inner.insert(packet)
    }


}

#[cfg(test)]
mod tests {
    use crate::*;
    use crate::{storage::writer::FileStorageOptions, tests::*};
    use std::{
        env::temp_dir,
        fs,
        sync::mpsc::{channel, Receiver, Sender},
        thread::{sleep, spawn},
        time::Duration,
    };

    #[test]
    fn success() {
        let filename = temp_dir().join("test_brec_filestorage_success_lock.bin");
        if filename.exists() {
            fs::remove_file(&filename).expect("Test file has been removed");
        }
        let filename_a = filename.clone();
        let (tx, rx): (Sender<()>, Receiver<()>) = channel();
        let a = spawn(move || {
            let a = FileWriterDef::<TestBlock, TestPayload, TestPayload>::new(
                filename_a, None, None,
            )
            .expect("Storage A has been created");
            tx.send(()).expect("Signal has been send");
            sleep(Duration::from_millis(100));
            drop(a);
            true
        });
        let b = spawn(move || {
            rx.recv().expect("Signal  has been gotten");

            FileWriterDef::<TestBlock, TestPayload, TestPayload>::new(
                &filename,
                Some(Duration::from_millis(300)),
                None,
            )
            .is_ok()
        });
        let a = a.join().expect("Storage A has been created");
        let b = b.join().expect("Storage B has been created");
        assert_eq!(a, b);
    }

    #[test]
    fn success_with_opt() {
        let filename = temp_dir().join("test_brec_filestorage_success_lock_opt.bin");
        if filename.exists() {
            fs::remove_file(&filename).expect("Test file has been removed");
        }
        let filename_a = filename.clone();
        let (tx, rx): (Sender<()>, Receiver<()>) = channel();
        let a = spawn(move || {
            let a = FileStorageOptions::new(filename_a)
                .open::<TestBlock, TestPayload, TestPayload>()
                .expect("Storage A has been created");
            tx.send(()).expect("Signal has been send");
            sleep(Duration::from_millis(100));
            drop(a);
            true
        });
        let b = spawn(move || {
            rx.recv().expect("Signal  has been gotten");
            FileStorageOptions::new(filename)
                .timeout(Duration::from_millis(300))
                .open::<TestBlock, TestPayload, TestPayload>()
                .is_ok()
        });
        let a = a.join().expect("Storage A has been created");
        let b = b.join().expect("Storage B has been created");
        assert_eq!(a, b);
    }

    #[test]
    fn timeout() {
        let filename = temp_dir().join("test_brec_filestorage_timeout.bin");
        if filename.exists() {
            fs::remove_file(&filename).expect("Test file has been removed");
        }
        let filename_a = filename.clone();
        let (tx, rx): (Sender<()>, Receiver<()>) = channel();
        let a = spawn(move || {
            let a = FileWriterDef::<TestBlock, TestPayload, TestPayload>::new(
                filename_a, None, None,
            )
            .expect("Storage A has been created");
            tx.send(()).expect("Signal has been send");
            sleep(Duration::from_millis(500));
            drop(a);
            true
        });
        let b = spawn(move || {
            rx.recv().expect("Signal  has been gotten");
            FileWriterDef::<TestBlock, TestPayload, TestPayload>::new(
                &filename,
                Some(Duration::from_millis(100)),
                None,
            )
            .is_ok()
        });
        let a = a.join().expect("Storage A has been created");
        let b = b.join().expect("Storage B has been created");
        assert!(a);
        assert!(!b);
    }

    #[test]
    fn fail() {
        let filename = temp_dir().join("test_brec_filestorage_fail.bin");
        if filename.exists() {
            fs::remove_file(&filename).expect("Test file has been removed");
        }
        let filename_a = filename.clone();
        let (tx, rx): (Sender<()>, Receiver<()>) = channel();
        let a = spawn(move || {
            let a = FileWriterDef::<TestBlock, TestPayload, TestPayload>::new(
                filename_a, None, None,
            )
            .expect("Storage A has been created");
            tx.send(()).expect("Signal has been send");
            sleep(Duration::from_millis(500));
            drop(a);
            true
        });
        let b = spawn(move || {
            rx.recv().expect("Signal  has been gotten");
            FileWriterDef::<TestBlock, TestPayload, TestPayload>::new(
                &filename, None, None,
            )
            .is_ok()
        });
        let a = a.join().expect("Storage A has been created");
        let b = b.join().expect("Storage B has been created");
        assert!(a);
        assert!(!b);
    }
}
