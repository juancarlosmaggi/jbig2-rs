#!/usr/bin/env python3
import os
import re
import subprocess
import time
from pathlib import Path


def parse_profile(stderr):
    profile_re = re.compile(
        r"^(?P<label>[^\s].*?)\s+(?P<total>[0-9.]+)\s+ms\s+"
        r"(?P<count>[0-9]+)x\s+avg\s+(?P<avg>[0-9.]+)\s+ms$"
    )
    labels = {}
    total_ms = None
    for line in stderr.splitlines():
        if line.startswith("Decode profile (total "):
            m = re.search(r"total\s+([0-9.]+)\s+ms", line)
            if m:
                total_ms = float(m.group(1))
            continue
        m = profile_re.match(line.strip())
        if not m:
            continue
        label = m.group("label").strip()
        total = float(m.group("total"))
        count = int(m.group("count"))
        labels[label] = (total, count)
    return total_ms, labels


def main():
    root = Path(__file__).resolve().parents[1]
    ubc = root / "tests" / "resources" / "ubc"
    out_dir = Path("/tmp/jbig2-profile")
    out_dir.mkdir(parents=True, exist_ok=True)
    runs = int(os.environ.get("JBIG2_PROFILE_RUNS", "5"))
    if runs < 1:
        raise SystemExit("JBIG2_PROFILE_RUNS must be >= 1")

    files = sorted(ubc.glob("*.jb2"))
    if not files:
        raise SystemExit("no jb2 files found")

    aggregate = {}
    file_totals = {}
    per_file_labels = {}

    total_files = len(files)
    for run_idx in range(1, runs + 1):
        run_start = time.time()
        for idx, path in enumerate(files, 1):
            progress = int((idx / total_files) * 30)
            bar = "#" * progress + "-" * (30 - progress)
            elapsed = time.time() - run_start
            avg_per = elapsed / idx if idx else 0.0
            eta = max(0.0, avg_per * (total_files - idx))
            status = (
                f"[run {run_idx}/{runs}] "
                f"[{idx}/{total_files}] [{bar}] {path.name} "
                f"(elapsed {elapsed:.1f}s, eta {eta:.1f}s)"
            )
            print(f"\r{status}", end="", flush=True)
            out_path = out_dir / (path.stem + ".bin")
            cmd = [
                "cargo",
                "run",
                "--example",
                "decode_file",
                "--release",
                "--quiet",
                "--",
                "--profile",
                str(path),
                str(out_path),
            ]
            proc = subprocess.run(cmd, cwd=root, capture_output=True, text=True)
            if proc.returncode != 0:
                print("failed", path.name)
                print(proc.stderr)
                raise SystemExit(1)
            print(f"\r{status}", end="", flush=True)

            total_ms, labels = parse_profile(proc.stderr)
            if total_ms is None:
                raise SystemExit(f"no profile total for {path.name}")
            file_totals[path.name] = file_totals.get(path.name, 0.0) + total_ms

            for label, (total, count) in labels.items():
                if label == "total_decode":
                    continue
                entry = aggregate.setdefault(label, [0.0, 0])
                entry[0] += total
                entry[1] += count

            per_label = per_file_labels.setdefault(path.name, {})
            for label, (total, _count) in labels.items():
                if label == "total_decode":
                    continue
                per_label[label] = per_label.get(label, 0.0) + total

    aggregate_rows = []
    for label, (total, count) in aggregate.items():
        total_avg = total / runs
        count_avg = int(round(count / runs)) if runs > 0 else 0
        avg = total_avg / count_avg if count_avg else 0.0
        aggregate_rows.append((total_avg, label, count_avg, avg))
    aggregate_rows.sort(reverse=True)
    file_totals_rows = [
        (total / runs, name) for name, total in file_totals.items()
    ]
    file_totals_rows.sort(reverse=True)
    per_file_top = []
    for name, labels in per_file_labels.items():
        if not labels:
            continue
        top_label, top_total = max(labels.items(), key=lambda kv: kv[1])
        per_file_top.append((top_total / runs, top_label, name))
    per_file_top.sort(reverse=True)

    report_path = root / "PROFILE_REPORT.md"
    with report_path.open("w", encoding="ascii") as f:
        f.write("# JBIG2 Decode Profiling Report\n\n")
        f.write(
            "Profiling run across UBC fixtures using "
            "`cargo run --example decode_file --release --profile`.\n\n"
        )
        f.write(f"Averaged over {runs} run(s).\n\n")
        f.write("## Slowest Decoder Sections (Aggregated)\n\n")
        f.write("| Rank | Label | Total ms | Calls | Avg ms |\n")
        f.write("| --- | --- | --- | --- | --- |\n")
        for i, (total, label, count, avg) in enumerate(aggregate_rows[:15], 1):
            f.write(f"| {i} | {label} | {total:.3f} | {count} | {avg:.3f} |\n")

        f.write("\n## Slowest Files (Total Decode)\n\n")
        f.write("| Rank | File | Total ms |\n")
        f.write("| --- | --- | --- |\n")
        for i, (total, name) in enumerate(file_totals_rows[:15], 1):
            f.write(f"| {i} | {name} | {total:.3f} |\n")

        f.write("\n## Per-File Top Hotspot (Excludes total_decode)\n\n")
        f.write("| Rank | File | Top Label | Total ms |\n")
        f.write("| --- | --- | --- | --- |\n")
        for i, (total, label, name) in enumerate(per_file_top[:15], 1):
            f.write(f"| {i} | {name} | {label} | {total:.3f} |\n")

    print(f"wrote {report_path}")


if __name__ == "__main__":
    main()
