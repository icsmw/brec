#!/usr/bin/env python3
from __future__ import annotations

import argparse
import csv
import os
from pathlib import Path

# Avoid permission issues in restricted environments.
os.environ.setdefault("MPLCONFIGDIR", "/tmp/matplotlib")

import matplotlib.pyplot as plt


PRIMARY_METRICS = (
    ("avg_us_per_row", "Avg, us/row"),
    ("rate_mbit_s", "Rate, Mbit/s"),
)

CASE_READ_WRITE = ("Writing", "Reading")
CASE_FILTERING = ("Filtering",)
BORROWED_PAYLOAD = "Borrowed blocks only (referred path benchmark)"

PLATFORM_ORDER = {
    "Plain text": 0,
    "Crypt Plain text": 1,
    "JSON": 2,
    "Crypt JSON": 3,
    "Protobuf": 4,
    "FlatBuffers": 5,
    "FlatBuffers Owned": 6,
    "Brec Storage": 7,
    "Brec Stream": 8,
}

PAYLOAD_SLUG = {
    "Small record with a few fields and a text message": "small_record",
    "Large record with many fields and a key/value collection": "large_record",
    "Large record with many fields and a key/value collection with encryption": "large_record_crypt",
    "Borrowed blocks only (referred path benchmark)": "borrowed_blocks_only",
}

CASE_COLORS = {
    "Writing": "#f58518",   # orange
    "Reading": "#4c78a8",   # blue
    "Filtering": "#54a24b", # green
}

FONT_PLATFORM = 12
FONT_DELTA = 12
FONT_LEGEND = 13
FONT_PANEL_TITLE = 15
FONT_SUPTITLE = 14


def load_rows(path: Path) -> list[dict[str, str]]:
    with path.open("r", encoding="utf-8", newline="") as fh:
        return list(csv.DictReader(fh))


def to_float(value: str) -> float:
    try:
        return float(value)
    except ValueError:
        return 0.0


def sorted_platforms(rows: list[dict[str, str]]) -> list[str]:
    platforms = {row["platform"] for row in rows}
    return sorted(platforms, key=lambda p: (PLATFORM_ORDER.get(p, 999), p))


def metric_values(
    lookup: dict[tuple[str, str], dict[str, str]],
    platforms: list[str],
    case: str,
    metric_key: str,
) -> list[float]:
    return [to_float(lookup.get((platform, case), {}).get(metric_key, "0")) for platform in platforms]


def set_platform_ticks(ax: plt.Axes, x: list[int], platforms: list[str]) -> None:
    ax.set_xticks(x)
    ax.set_xticklabels(platforms, rotation=0, ha="center", fontsize=FONT_PLATFORM)


def avg_size_mb_values(
    lookup: dict[tuple[str, str], dict[str, str]],
    platforms: list[str],
    cases: tuple[str, ...],
) -> list[float]:
    values: list[float] = []
    for platform in platforms:
        samples = [
            to_float(lookup.get((platform, case), {}).get("bytes", "0"))
            for case in cases
            if (platform, case) in lookup
        ]
        size_bytes = (sum(samples) / len(samples)) if samples else 0.0
        values.append(size_bytes / (1024.0 * 1024.0))
    return values


def cleanup_old_svgs(out_dir: Path) -> list[Path]:
    removable: list[Path] = []
    for slug in PAYLOAD_SLUG.values():
        removable.append(out_dir / f"{slug}_read_write.svg")
        removable.append(out_dir / f"{slug}_filtering.svg")

    deleted: list[Path] = []
    for path in removable:
        if path.exists():
            path.unlink()
            deleted.append(path)
    return deleted


def choose_baseline(values: list[float], higher_is_better: bool) -> float:
    if not values:
        return 0.0
    return max(values) if higher_is_better else min(values)


def add_delta_labels(
    ax: plt.Axes,
    bars,
    values: list[float],
    baseline: float,
    higher_is_better: bool,
) -> None:
    if baseline <= 0:
        return
    for bar, value in zip(bars, values):
        if higher_is_better:
            # For throughput metrics lower values are worse than the best (max).
            if value >= baseline:
                continue
            delta_pct = ((baseline - value) / baseline) * 100.0
            label = f"-{delta_pct:.1f}%"
        else:
            # For latency/resource metrics higher values are worse than the best (min).
            if value <= baseline:
                continue
            delta_pct = ((value - baseline) / baseline) * 100.0
            label = f"+{delta_pct:.1f}%"

        ax.annotate(
            label,
            (bar.get_x() + bar.get_width() / 2, bar.get_height()),
            textcoords="offset points",
            xytext=(0, 4),
            ha="center",
            va="bottom",
            fontsize=FONT_DELTA,
        )


def draw_chart(payload: str, rows: list[dict[str, str]], cases: tuple[str, ...], subtitle: str, out_file: Path) -> None:
    case_rows = [row for row in rows if row["case"] in cases]
    if not case_rows:
        return

    platforms = sorted_platforms(case_rows)
    if not platforms:
        return

    lookup: dict[tuple[str, str], dict[str, str]] = {
        (row["platform"], row["case"]): row for row in case_rows
    }

    if len(cases) > 1:
        fig = plt.figure(figsize=(18, 14), constrained_layout=True)
        grid = fig.add_gridspec(3, 2, height_ratios=[1, 1, 1.2])
        primary_axes = [
            fig.add_subplot(grid[0, :]),
            fig.add_subplot(grid[1, :]),
        ]
        resource_axes = [
            fig.add_subplot(grid[2, 0]),
            fig.add_subplot(grid[2, 1]),
        ]
    else:
        fig = plt.figure(figsize=(18, 12), constrained_layout=True)
        grid = fig.add_gridspec(3, 1, height_ratios=[1, 1, 1.2])
        primary_axes = [
            fig.add_subplot(grid[0, 0]),
            fig.add_subplot(grid[1, 0]),
        ]
        resource_axes = [fig.add_subplot(grid[2, 0])]

    bar_width = 0.38 if len(cases) > 1 else 0.7
    x = list(range(len(platforms)))

    # Primary metric panels.
    for metric_idx, ((metric_key, metric_label), ax) in enumerate(zip(PRIMARY_METRICS, primary_axes)):
        size_bars = None

        for case_idx, case in enumerate(cases):
            if len(cases) == 1:
                positions = x
            else:
                shift = (case_idx - (len(cases) - 1) / 2.0) * bar_width
                positions = [val + shift for val in x]

            values = metric_values(lookup, platforms, case, metric_key)
            bars = ax.bar(
                positions,
                values,
                width=bar_width,
                label=case,
                color=CASE_COLORS.get(case),
            )
            higher_is_better = metric_key == "rate_mbit_s"
            baseline = choose_baseline(values, higher_is_better)
            add_delta_labels(ax, bars, values, baseline, higher_is_better)

        # In read/write mode add file size bars to Avg panel on secondary axis.
        if metric_key == "avg_us_per_row" and len(cases) > 1:
            size_mb_values = avg_size_mb_values(lookup, platforms, cases)

            ax_right = ax.twinx()
            size_bars = ax_right.bar(
                x,
                size_mb_values,
                width=0.14,
                color="#7f7f7f",
                alpha=0.45,
                label="Size, Mb",
                zorder=1,
            )
            ax_right.set_ylabel("Size, Mb", color="#7f7f7f")
            ax_right.tick_params(axis="y", labelcolor="#7f7f7f")

        ax.set_ylabel(metric_label)
        ax.grid(axis="y", alpha=0.25)
        set_platform_ticks(ax, x, platforms)

        if metric_idx == 0:
            handles, labels = ax.get_legend_handles_labels()
            if size_bars is not None:
                handles.append(size_bars)
                labels.append("Size, Mb")
            legend = ax.legend(handles, labels, loc="upper right", fontsize=FONT_LEGEND)
            legend.set_title("Scenario", prop={"size": FONT_LEGEND})

    # Combined resource panel(s):
    # left Y axis = CPU, right Y axis = RSS/PeakRSS.
    for case, ax in zip(cases, resource_axes):
        cpu_values = metric_values(lookup, platforms, case, "cpu_ms")
        rss_values = metric_values(lookup, platforms, case, "rss_kb")
        peak_values = metric_values(lookup, platforms, case, "peak_rss_kb")

        bars_cpu = ax.bar(x, cpu_values, width=0.28, color="#4c78a8", alpha=0.9, label="CPU, ms")
        ax.set_ylabel("CPU, ms", color="#4c78a8")
        ax.tick_params(axis="y", labelcolor="#4c78a8")
        set_platform_ticks(ax, x, platforms)
        ax.grid(axis="y", alpha=0.25)
        ax.set_title(case, fontsize=FONT_PANEL_TITLE)

        ax_right = ax.twinx()
        rss_positions = [val + 0.22 for val in x]
        peak_positions = [val + 0.44 for val in x]
        bars_rss = ax_right.bar(
            rss_positions,
            rss_values,
            width=0.18,
            color="#f58518",
            alpha=0.85,
            label="RSS+, Kb",
        )
        bars_peak = ax_right.bar(
            peak_positions,
            peak_values,
            width=0.18,
            color="#e45756",
            alpha=0.85,
            label="PeakRSS+, Kb",
        )
        ax_right.set_ylabel("RSS/PeakRSS, Kb", color="#e45756")
        ax_right.tick_params(axis="y", labelcolor="#e45756")

        handles = [bars_cpu, bars_rss, bars_peak]
        labels = [h.get_label() for h in handles]
        ax.legend(handles, labels, loc="upper left", fontsize=FONT_PLATFORM)

    fig.suptitle(f"{payload}\\n{subtitle}", fontsize=FONT_SUPTITLE)
    fig.savefig(out_file, format="svg")
    plt.close(fig)


def build_charts(csv_path: Path, out_dir: Path) -> list[Path]:
    rows = load_rows(csv_path)
    out_dir.mkdir(parents=True, exist_ok=True)

    deleted = cleanup_old_svgs(out_dir)
    if deleted:
        print(f"Removed {len(deleted)} old chart(s)")

    by_payload: dict[str, list[dict[str, str]]] = {}
    for row in rows:
        by_payload.setdefault(row["payload"], []).append(row)

    written: list[Path] = []
    for payload, payload_rows in sorted(by_payload.items()):
        slug = PAYLOAD_SLUG.get(payload, payload.lower().replace(" ", "_"))

        rw_file = out_dir / f"{slug}_read_write.svg"
        draw_chart(payload, payload_rows, CASE_READ_WRITE, "Read / Write", rw_file)
        if rw_file.exists():
            written.append(rw_file)

        if payload == BORROWED_PAYLOAD:
            continue

        filtering_file = out_dir / f"{slug}_filtering.svg"
        draw_chart(payload, payload_rows, CASE_FILTERING, "Filtering (zero-copy path)", filtering_file)
        if filtering_file.exists():
            written.append(filtering_file)

    return written


def main() -> int:
    parser = argparse.ArgumentParser(description="Render benchmark charts from CSV/CVS")
    parser.add_argument("--csv", required=True, help="Path to measurements .cvs file")
    parser.add_argument("--out-dir", required=True, help="Output directory for svg charts")
    args = parser.parse_args()

    csv_path = Path(args.csv).resolve()
    out_dir = Path(args.out_dir).resolve()

    if not csv_path.exists():
        raise SystemExit(f"CSV file not found: {csv_path}")

    written = build_charts(csv_path, out_dir)
    print(f"Generated {len(written)} chart(s):")
    for path in written:
        print(f"- {path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
