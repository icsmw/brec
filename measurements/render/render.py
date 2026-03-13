#!/usr/bin/env python3
from __future__ import annotations

import os
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path


REQUIRED_ENV_KEYS = (
    "BREC_TEST_MEASUREMENTS_ITERATIONS",
    "BREC_TEST_MEASUREMENTS_ITERATIONS_CRYPT",
    "BREC_TEST_MEASUREMENTS_PACKAGES",
    "BREC_TEST_MEASUREMENTS_RECORDS",
    "BREC_SESSION_REUSE_LIMIT",
    "BREC_DECRYPT_CACHE_LIMIT",
)

RUNNER_MANIFEST = "measurements/runner/Cargo.toml"
CHARTS_SCRIPT = "measurements/render/charts.py"
CHARTS_OUT_DIR = "site/docs/assets"


def repo_root() -> Path:
    return Path(__file__).resolve().parents[2]


def build_csv_path(root: Path) -> Path:
    ts = datetime.now(timezone.utc).strftime("%Y%m%d_%H%M%S")
    out_dir = root / "measurements" / "results"
    out_dir.mkdir(parents=True, exist_ok=True)
    return out_dir / f"{ts}.cvs"


def read_required_env() -> dict[str, str]:
    values: dict[str, str] = {}
    missing: list[str] = []

    for key in REQUIRED_ENV_KEYS:
        value = os.environ.get(key, "").strip()
        if not value:
            missing.append(key)
        else:
            values[key] = value

    if missing:
        print("Missing required env vars:")
        for key in missing:
            print(f"- {key}")
        sys.exit(2)

    return values


def run() -> int:
    root = repo_root()
    csv_path = build_csv_path(root)
    env_values = read_required_env()

    child_env = os.environ.copy()
    child_env.update(env_values)
    child_env["BREC_TEST_MEASUREMENTS_CVS"] = str(csv_path)

    test_cmd = [
        "cargo",
        "test",
        "--manifest-path",
        RUNNER_MANIFEST,
        "--release",
        "--",
        "--nocapture",
    ]

    print("Running:", " ".join(test_cmd))
    print(f"CSV output: {csv_path}")

    completed = subprocess.run(test_cmd, cwd=root, env=child_env)
    if completed.returncode != 0:
        return completed.returncode

    print(f"Done. Measurements saved to: {csv_path}")

    chart_cmd = [
        sys.executable,
        CHARTS_SCRIPT,
        "--csv",
        str(csv_path),
        "--out-dir",
        CHARTS_OUT_DIR,
    ]
    print("Rendering charts:", " ".join(chart_cmd))
    charts = subprocess.run(chart_cmd, cwd=root)
    return charts.returncode


if __name__ == "__main__":
    raise SystemExit(run())
