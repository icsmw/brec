use num_format::{Locale, ToFormattedString};
use std::{
    collections::HashMap,
    fmt,
    sync::{Mutex, OnceLock},
};

static REPORT: OnceLock<Mutex<Report>> = OnceLock::new();

pub fn add(payload: PayloadKind, pl: Platform, case: TestCase, res: TestResults) {
    REPORT
        .get_or_init(|| Mutex::new(Report::default()))
        .lock()
        .expect("Failed to lock report")
        .add(payload, pl, case, res);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PayloadKind {
    Record,
    RecordBincode,
}

impl PayloadKind {
    pub fn file_tag(self) -> &'static str {
        match self {
            Self::Record => "record",
            Self::RecordBincode => "record_bincode",
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Platform {
    Text,
    Json,
    Storage,
    BinStream,
    StreamedStorage,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TestCase {
    Reading,
    Filtering,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TestResults {
    pub size: u64,
    pub count: usize,
    pub time: u128,
    pub cpu_ms: u64,
    pub rss_kb: u64,
    pub peak_rss_kb: u64,
}

#[derive(Debug, Default)]
pub struct Report {
    pub results: HashMap<PayloadKind, HashMap<Platform, HashMap<TestCase, Vec<TestResults>>>>,
}

impl Report {
    pub fn add(&mut self, payload: PayloadKind, pl: Platform, case: TestCase, res: TestResults) {
        self.results
            .entry(payload)
            .or_default()
            .entry(pl)
            .or_default()
            .entry(case)
            .or_default()
            .push(res);
        println!("{self}");
    }
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (idx, (payload, platforms)) in self.results.iter().enumerate() {
            if idx > 0 {
                writeln!(f)?;
            }
            writeln!(f, "Payload: {payload}")?;

            let mut cells = Vec::new();
            let mut widths = (0, 0, 0, 0, 8, 7, 7, 8, 10); // platform, case, size, count, time, cpu, rss, peak, iterations

            platforms.iter().for_each(|(pl, cases)| {
                cases.iter().for_each(|(case, results)| {
                    let size: u128 = results.iter().map(|r| r.size as u128).sum::<u128>()
                        / results.len() as u128;
                    let count: u128 = results.iter().map(|r| r.count as u128).sum::<u128>()
                        / results.len() as u128;
                    let time: u128 =
                        results.iter().map(|r| r.time).sum::<u128>() / results.len() as u128;
                    let cpu_ms: u128 = results.iter().map(|r| r.cpu_ms as u128).sum::<u128>()
                        / results.len() as u128;
                    let rss_kb: u128 = results.iter().map(|r| r.rss_kb as u128).sum::<u128>()
                        / results.len() as u128;
                    let peak_rss_kb: u128 =
                        results.iter().map(|r| r.peak_rss_kb as u128).sum::<u128>()
                            / results.len() as u128;
                    let iterations = results.len();

                    let pl_s = pl.to_string();
                    let case_s = case.to_string();
                    let size_s = if size > 1024 * 1024 {
                        format!(
                            "{} Mb",
                            (size / (1024 * 1024)).to_formatted_string(&Locale::en)
                        )
                    } else if size > 1024 {
                        format!("{} Kb", (size / 1024).to_formatted_string(&Locale::en))
                    } else {
                        format!("{} B", size.to_formatted_string(&Locale::en))
                    };
                    let count_s = count.to_formatted_string(&Locale::en);
                    let time_s = time.to_formatted_string(&Locale::en);
                    let cpu_s = cpu_ms.to_formatted_string(&Locale::en);
                    let rss_s = rss_kb.to_formatted_string(&Locale::en);
                    let peak_s = peak_rss_kb.to_formatted_string(&Locale::en);
                    let iterations_s = iterations.to_formatted_string(&Locale::en);

                    widths.0 = widths.0.max(pl_s.len());
                    widths.1 = widths.1.max(case_s.len());
                    widths.2 = widths.2.max(size_s.len());
                    widths.3 = widths.3.max(count_s.len());
                    widths.4 = widths.4.max(time_s.len());
                    widths.5 = widths.5.max(cpu_s.len());
                    widths.6 = widths.6.max(rss_s.len());
                    widths.7 = widths.7.max(peak_s.len());
                    widths.8 = widths.8.max(iterations_s.len());

                    cells.push((
                        pl_s,
                        case_s,
                        size_s,
                        count_s,
                        time_s,
                        cpu_s,
                        rss_s,
                        peak_s,
                        iterations_s,
                    ));
                });
            });

            widths.0 = widths.0.max("Platform".len());
            widths.1 = widths.1.max("Case".len());
            widths.2 = widths.2.max("Bytes".len());
            widths.3 = widths.3.max("Rows".len());
            widths.4 = widths.4.max("Time, ms".len());
            widths.5 = widths.5.max("CPU, ms".len());
            widths.6 = widths.6.max("RSS+, Kb".len());
            widths.7 = widths.7.max("PeakRSS+, Kb".len());
            widths.8 = widths.8.max("Iterations".len());

            let (w_pl, w_case, w_size, w_count, w_time, w_cpu, w_rss, w_peak, w_iterations) =
                widths;

            let header = format!(
                "| {:^w_pl$} | {:^w_case$} | {:^w_size$} | {:^w_count$} | {:^w_time$} | {:^w_cpu$} | {:^w_rss$} | {:^w_peak$} | {:^w_iterations$} |",
                "Platform",
                "Case",
                "Bytes",
                "Rows",
                "Time, ms",
                "CPU, ms",
                "RSS+, Kb",
                "PeakRSS+, Kb",
                "Iterations",
                w_pl = w_pl,
                w_case = w_case,
                w_size = w_size,
                w_count = w_count,
                w_time = w_time,
                w_cpu = w_cpu,
                w_rss = w_rss,
                w_peak = w_peak,
                w_iterations = w_iterations,
            );

            let separator = format!(
                "+-{pl:-<w_pl$}-+-{case:-<w_case$}-+-{size:-<w_size$}-+-{count:-<w_count$}-+-{time:-<w_time$}-+-{cpu:-<w_cpu$}-+-{rss:-<w_rss$}-+-{peak:-<w_peak$}-+-{iterations:-<w_iterations$}-+",
                pl = "",
                case = "",
                size = "",
                count = "",
                time = "",
                cpu = "",
                rss = "",
                peak = "",
                iterations = "",
                w_pl = w_pl,
                w_case = w_case,
                w_size = w_size,
                w_count = w_count,
                w_time = w_time,
                w_cpu = w_cpu,
                w_rss = w_rss,
                w_peak = w_peak,
                w_iterations = w_iterations
            );

            writeln!(f, "{separator}")?;
            writeln!(f, "{header}")?;
            writeln!(f, "{separator}")?;

            for (pl, case, size, count, time, cpu, rss, peak, iterations) in cells {
                writeln!(
                    f,
                    "| {:<w_pl$} | {:<w_case$} | {:>w_size$} | {:>w_count$} | {:>w_time$} | {:>w_cpu$} | {:>w_rss$} | {:>w_peak$} | {:>w_iterations$} |",
                    pl,
                    case,
                    size,
                    count,
                    time,
                    cpu,
                    rss,
                    peak,
                    iterations,
                    w_pl = w_pl,
                    w_case = w_case,
                    w_size = w_size,
                    w_count = w_count,
                    w_time = w_time,
                    w_cpu = w_cpu,
                    w_rss = w_rss,
                    w_peak = w_peak,
                    w_iterations = w_iterations
                )?;
            }

            writeln!(f, "{separator}")?;
        }
        Ok(())
    }
}

impl fmt::Display for PayloadKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Record => "Record",
                Self::RecordBincode => "RecordBincode",
            }
        )
    }
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Storage => "Storage",
                Self::BinStream => "Binary stream",
                Self::Text => "Plant text",
                Self::Json => "JSON",
                Self::StreamedStorage => "Streamed storage",
            }
        )
    }
}

impl fmt::Display for TestCase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Reading => "Reading",
                Self::Filtering => "Filtering",
            }
        )
    }
}
