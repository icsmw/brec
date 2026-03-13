use num_format::{Locale, ToFormattedString};
use std::{
    collections::HashMap,
    fmt,
    io::Write,
    path::Path,
    sync::{Mutex, OnceLock},
};

static REPORT: OnceLock<Mutex<Report>> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReportColumn {
    Payload,
    Platform,
    Case,
    Bytes,
    Rows,
    TimeMs,
    AvgUsPerRow,
    RateMbitS,
    CpuMs,
    RssKb,
    PeakRssKb,
    Iterations,
}

impl ReportColumn {
    fn id(self) -> &'static str {
        match self {
            Self::Payload => "payload",
            Self::Platform => "platform",
            Self::Case => "case",
            Self::Bytes => "bytes",
            Self::Rows => "rows",
            Self::TimeMs => "time_ms",
            Self::AvgUsPerRow => "avg_us_per_row",
            Self::RateMbitS => "rate_mbit_s",
            Self::CpuMs => "cpu_ms",
            Self::RssKb => "rss_kb",
            Self::PeakRssKb => "peak_rss_kb",
            Self::Iterations => "iterations",
        }
    }
}

impl fmt::Display for ReportColumn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Payload => "Payload",
                Self::Platform => "Platform",
                Self::Case => "Case",
                Self::Bytes => "Bytes",
                Self::Rows => "Rows",
                Self::TimeMs => "Time, ms",
                Self::AvgUsPerRow => "Avg, us/row",
                Self::RateMbitS => "Rate, Mbit/s",
                Self::CpuMs => "CPU, ms",
                Self::RssKb => "RSS+, Kb",
                Self::PeakRssKb => "PeakRSS+, Kb",
                Self::Iterations => "Iterations",
            }
        )
    }
}

pub fn add(payload: PayloadKind, pl: Platform, case: TestCase, res: TestResults) {
    REPORT
        .get_or_init(|| Mutex::new(Report::default()))
        .lock()
        .expect("Failed to lock report")
        .add(payload, pl, case, res);
}

pub fn write_csv<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let report = REPORT
        .get_or_init(|| Mutex::new(Report::default()))
        .lock()
        .expect("Failed to lock report");

    let mut rows = report
        .results
        .iter()
        .flat_map(|(payload, platforms)| {
            platforms.iter().flat_map(move |(platform, cases)| {
                cases
                    .iter()
                    .map(move |(case, results)| (*payload, platform, case, results))
            })
        })
        .collect::<Vec<_>>();

    rows.sort_by(|(payload_a, platform_a, case_a, _), (payload_b, platform_b, case_b, _)| {
        payload_a
            .cmp(payload_b)
            .then_with(|| platform_a.sort_key().cmp(&platform_b.sort_key()))
            .then_with(|| platform_a.to_string().cmp(&platform_b.to_string()))
            .then_with(|| case_a.cmp(case_b))
    });

    let mut file = std::fs::File::create(path)?;
    let csv_header = [
        ReportColumn::Payload,
        ReportColumn::Platform,
        ReportColumn::Case,
        ReportColumn::Bytes,
        ReportColumn::Rows,
        ReportColumn::TimeMs,
        ReportColumn::AvgUsPerRow,
        ReportColumn::RateMbitS,
        ReportColumn::CpuMs,
        ReportColumn::RssKb,
        ReportColumn::PeakRssKb,
        ReportColumn::Iterations,
    ]
    .iter()
    .map(|column| column.id())
    .collect::<Vec<_>>()
    .join(",");
    writeln!(file, "{csv_header}")?;

    for (payload, platform, case, results) in rows {
        let size: u128 =
            results.iter().map(|r| r.size as u128).sum::<u128>() / results.len() as u128;
        let count: u128 = results.iter().map(|r| r.count as u128).sum::<u128>() / results.len() as u128;
        let time_ns: u128 = results.iter().map(|r| r.time).sum::<u128>() / results.len() as u128;
        let cpu_ms: u128 =
            results.iter().map(|r| r.cpu_ms as u128).sum::<u128>() / results.len() as u128;
        let rss_kb: u128 =
            results.iter().map(|r| r.rss_kb as u128).sum::<u128>() / results.len() as u128;
        let peak_rss_kb: u128 =
            results.iter().map(|r| r.peak_rss_kb as u128).sum::<u128>() / results.len() as u128;
        let iterations = results.len();

        let avg_us_per_row = if count == 0 {
            0.0
        } else {
            (time_ns as f64 / 1_000.0) / count as f64
        };
        let rate_mbit_s = if time_ns == 0 {
            0.0
        } else {
            (size as f64 * 8.0 * 1_000.0) / time_ns as f64
        };
        let time_ms = (time_ns + 500_000) / 1_000_000;

        writeln!(
            file,
            "\"{}\",\"{}\",\"{}\",{},{},{},{:.2},{:.2},{},{},{},{}",
            payload,
            platform,
            case,
            size,
            count,
            time_ms,
            avg_us_per_row,
            rate_mbit_s,
            cpu_ms,
            rss_kb,
            peak_rss_kb,
            iterations
        )?;
    }

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PayloadKind {
    Record,
    RecordBincode,
    RecordCrypt,
    Borrowed,
}

impl PayloadKind {
    pub fn file_tag(self) -> &'static str {
        match self {
            Self::Record => "record",
            Self::RecordBincode => "record_bincode",
            Self::RecordCrypt => "record_crypt",
            Self::Borrowed => "borrowed",
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Platform {
    Text,
    TextCrypt,
    Json,
    JsonCrypt,
    Protobuf,
    FlatBuffers,
    FlatBuffersOwned,
    BrecStorage,
    BrecStream,
}

impl Platform {
    fn sort_key(&self) -> usize {
        match self {
            Self::Text => 0,
            Self::TextCrypt => 0,
            Self::Json => 1,
            Self::JsonCrypt => 1,
            Self::Protobuf => 2,
            Self::FlatBuffers => 3,
            Self::FlatBuffersOwned => 4,
            Self::BrecStorage => 5,
            Self::BrecStream => 6,
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TestCase {
    Writing,
    Reading,
    Filtering,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TestResults {
    pub size: u64,
    pub count: usize,
    // Internal precision: nanoseconds.
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
            let mut widths = (0, 0, 0, 0, 8, 11, 11, 7, 7, 8, 10); // platform, case, size, count, time, avg, rate, cpu, rss, peak, iterations

            let mut ordered_rows = platforms
                .iter()
                .flat_map(|(pl, cases)| {
                    cases.iter().map(move |(case, results)| (pl, case, results))
                })
                .collect::<Vec<_>>();

            ordered_rows.sort_by(|(pl_a, case_a, _), (pl_b, case_b, _)| {
                pl_a.sort_key()
                    .cmp(&pl_b.sort_key())
                    .then_with(|| pl_a.to_string().cmp(&pl_b.to_string()))
                    .then_with(|| case_a.cmp(case_b))
            });

            ordered_rows.iter().for_each(|(pl, case, results)| {
                let size: u128 =
                    results.iter().map(|r| r.size as u128).sum::<u128>() / results.len() as u128;
                let count: u128 =
                    results.iter().map(|r| r.count as u128).sum::<u128>() / results.len() as u128;
                let time: u128 =
                    results.iter().map(|r| r.time).sum::<u128>() / results.len() as u128;
                let cpu_ms: u128 =
                    results.iter().map(|r| r.cpu_ms as u128).sum::<u128>() / results.len() as u128;
                let rss_kb: u128 =
                    results.iter().map(|r| r.rss_kb as u128).sum::<u128>() / results.len() as u128;
                let peak_rss_kb: u128 = results.iter().map(|r| r.peak_rss_kb as u128).sum::<u128>()
                    / results.len() as u128;
                let iterations = results.len();
                let avg_us_per_row = if count == 0 {
                    0.0
                } else {
                    (time as f64 / 1_000.0) / count as f64
                };
                let mbit_per_sec = if time == 0 {
                    0.0
                } else {
                    (size as f64 * 8.0 * 1_000.0) / time as f64
                };

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
                let time_ms = (time + 500_000) / 1_000_000;
                let time_s = time_ms.to_formatted_string(&Locale::en);
                let avg_s = format!("{avg_us_per_row:.2}");
                let rate_s = format!("{mbit_per_sec:.2}");
                let cpu_s = cpu_ms.to_formatted_string(&Locale::en);
                let rss_s = rss_kb.to_formatted_string(&Locale::en);
                let peak_s = peak_rss_kb.to_formatted_string(&Locale::en);
                let iterations_s = iterations.to_formatted_string(&Locale::en);

                widths.0 = widths.0.max(pl_s.len());
                widths.1 = widths.1.max(case_s.len());
                widths.2 = widths.2.max(size_s.len());
                widths.3 = widths.3.max(count_s.len());
                widths.4 = widths.4.max(time_s.len());
                widths.5 = widths.5.max(avg_s.len());
                widths.6 = widths.6.max(rate_s.len());
                widths.7 = widths.7.max(cpu_s.len());
                widths.8 = widths.8.max(rss_s.len());
                widths.9 = widths.9.max(peak_s.len());
                widths.10 = widths.10.max(iterations_s.len());
                cells.push((
                    pl_s,
                    case_s,
                    size_s,
                    count_s,
                    time_s,
                    avg_s,
                    rate_s,
                    cpu_s,
                    rss_s,
                    peak_s,
                    iterations_s,
                ));
            });

            widths.0 = widths.0.max(ReportColumn::Platform.to_string().len());
            widths.1 = widths.1.max(ReportColumn::Case.to_string().len());
            widths.2 = widths.2.max(ReportColumn::Bytes.to_string().len());
            widths.3 = widths.3.max(ReportColumn::Rows.to_string().len());
            widths.4 = widths.4.max(ReportColumn::TimeMs.to_string().len());
            widths.5 = widths.5.max(ReportColumn::AvgUsPerRow.to_string().len());
            widths.6 = widths.6.max(ReportColumn::RateMbitS.to_string().len());
            widths.7 = widths.7.max(ReportColumn::CpuMs.to_string().len());
            widths.8 = widths.8.max(ReportColumn::RssKb.to_string().len());
            widths.9 = widths.9.max(ReportColumn::PeakRssKb.to_string().len());
            widths.10 = widths.10.max(ReportColumn::Iterations.to_string().len());

            let (
                w_pl,
                w_case,
                w_size,
                w_count,
                w_time,
                w_avg,
                w_rate,
                w_cpu,
                w_rss,
                w_peak,
                w_iterations,
            ) = widths;

            let header_platform = ReportColumn::Platform.to_string();
            let header_case = ReportColumn::Case.to_string();
            let header_bytes = ReportColumn::Bytes.to_string();
            let header_rows = ReportColumn::Rows.to_string();
            let header_time = ReportColumn::TimeMs.to_string();
            let header_avg = ReportColumn::AvgUsPerRow.to_string();
            let header_rate = ReportColumn::RateMbitS.to_string();
            let header_cpu = ReportColumn::CpuMs.to_string();
            let header_rss = ReportColumn::RssKb.to_string();
            let header_peak = ReportColumn::PeakRssKb.to_string();
            let header_iterations = ReportColumn::Iterations.to_string();

            let header = format!(
                "| {:^w_pl$} | {:^w_case$} | {:^w_size$} | {:^w_count$} | {:^w_time$} | {:^w_avg$} | {:^w_rate$} | {:^w_cpu$} | {:^w_rss$} | {:^w_peak$} | {:^w_iterations$} |",
                header_platform,
                header_case,
                header_bytes,
                header_rows,
                header_time,
                header_avg,
                header_rate,
                header_cpu,
                header_rss,
                header_peak,
                header_iterations,
                w_pl = w_pl,
                w_case = w_case,
                w_size = w_size,
                w_count = w_count,
                w_time = w_time,
                w_avg = w_avg,
                w_rate = w_rate,
                w_cpu = w_cpu,
                w_rss = w_rss,
                w_peak = w_peak,
                w_iterations = w_iterations,
            );

            let separator = format!(
                "+-{pl:-<w_pl$}-+-{case:-<w_case$}-+-{size:-<w_size$}-+-{count:-<w_count$}-+-{time:-<w_time$}-+-{avg:-<w_avg$}-+-{rate:-<w_rate$}-+-{cpu:-<w_cpu$}-+-{rss:-<w_rss$}-+-{peak:-<w_peak$}-+-{iterations:-<w_iterations$}-+",
                pl = "",
                case = "",
                size = "",
                count = "",
                time = "",
                avg = "",
                rate = "",
                cpu = "",
                rss = "",
                peak = "",
                iterations = "",
                w_pl = w_pl,
                w_case = w_case,
                w_size = w_size,
                w_count = w_count,
                w_time = w_time,
                w_avg = w_avg,
                w_rate = w_rate,
                w_cpu = w_cpu,
                w_rss = w_rss,
                w_peak = w_peak,
                w_iterations = w_iterations
            );

            writeln!(f, "{separator}")?;
            writeln!(f, "{header}")?;
            writeln!(f, "{separator}")?;

            for (pl, case, size, count, time, avg, rate, cpu, rss, peak, iterations) in cells {
                writeln!(
                    f,
                    "| {:<w_pl$} | {:<w_case$} | {:>w_size$} | {:>w_count$} | {:>w_time$} | {:>w_avg$} | {:>w_rate$} | {:>w_cpu$} | {:>w_rss$} | {:>w_peak$} | {:>w_iterations$} |",
                    pl,
                    case,
                    size,
                    count,
                    time,
                    avg,
                    rate,
                    cpu,
                    rss,
                    peak,
                    iterations,
                    w_pl = w_pl,
                    w_case = w_case,
                    w_size = w_size,
                    w_count = w_count,
                    w_time = w_time,
                    w_avg = w_avg,
                    w_rate = w_rate,
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
                Self::Record => "Small record with a few fields and a text message",
                Self::RecordBincode => {
                    "Large record with many fields and a key/value collection"
                }
                Self::RecordCrypt => {
                    "Large record with many fields and a key/value collection with encryption"
                }
                Self::Borrowed => "Borrowed blocks only (referred path benchmark)",
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
                Self::Text => "Plain text",
                Self::TextCrypt => "Crypt Plain text",
                Self::Json => "JSON",
                Self::JsonCrypt => "Crypt JSON",
                Self::Protobuf => "Protobuf",
                Self::FlatBuffers => "FlatBuffers",
                Self::FlatBuffersOwned => "FlatBuffers Owned",
                Self::BrecStorage => "Brec Storage",
                Self::BrecStream => "Brec Stream",
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
                Self::Writing => "Writing",
                Self::Reading => "Reading",
                Self::Filtering => "Filtering",
            }
        )
    }
}
