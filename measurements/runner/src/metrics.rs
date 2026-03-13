use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};
use sysinfo::{Pid, ProcessesToUpdate, System};

#[derive(Debug)]
pub struct Tracker {
    pid: Pid,
    cpu_ms: u64,
    rss_kb: u64,
    stop: Arc<AtomicBool>,
    peak_rss_kb: Arc<AtomicU64>,
    handle: Option<JoinHandle<()>>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Usage {
    pub cpu_ms: u64,
    pub rss_kb: u64,
    pub peak_rss_kb: u64,
}

impl Tracker {
    pub fn start() -> Self {
        let pid = Pid::from_u32(std::process::id());
        let (cpu_ms, rss_kb) = read_process_snapshot(pid).unwrap_or((0, 0));
        let stop = Arc::new(AtomicBool::new(false));
        let peak_rss_kb = Arc::new(AtomicU64::new(rss_kb));
        let stop_inner = Arc::clone(&stop);
        let peak_inner = Arc::clone(&peak_rss_kb);
        let handle = thread::spawn(move || {
            let mut sys = System::new();
            while !stop_inner.load(Ordering::Relaxed) {
                let _ = sys.refresh_processes(ProcessesToUpdate::Some(&[pid]), true);
                if let Some(process) = sys.process(pid) {
                    let rss = process.memory() / 1024;
                    let _ = peak_inner.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |prev| {
                        if rss > prev { Some(rss) } else { None }
                    });
                }
                thread::sleep(Duration::from_millis(5));
            }
        });

        Self {
            pid,
            cpu_ms,
            rss_kb,
            stop,
            peak_rss_kb,
            handle: Some(handle),
        }
    }

    fn stop_thread(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }

    pub fn finish(mut self) -> Usage {
        self.stop_thread();
        let (end_cpu_ms, end_rss_kb) = read_process_snapshot(self.pid).unwrap_or((self.cpu_ms, self.rss_kb));
        let peak = self.peak_rss_kb.load(Ordering::Relaxed);
        Usage {
            cpu_ms: end_cpu_ms.saturating_sub(self.cpu_ms),
            rss_kb: end_rss_kb.saturating_sub(self.rss_kb),
            peak_rss_kb: peak.saturating_sub(self.rss_kb),
        }
    }
}

impl Drop for Tracker {
    fn drop(&mut self) {
        self.stop_thread();
    }
}

fn read_process_snapshot(pid: Pid) -> Option<(u64, u64)> {
    let mut sys = System::new();
    let _ = sys.refresh_processes(ProcessesToUpdate::Some(&[pid]), true);
    let process = sys.process(pid)?;
    Some((process.accumulated_cpu_time(), process.memory() / 1024))
}
