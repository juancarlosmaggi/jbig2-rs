#!/usr/bin/env python3
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

    files = sorted(ubc.glob("*.jb2"))
    if not files:
        raise SystemExit("no jb2 files found")

    aggregate = {}
    file_totals = []
    per_file_top = []

    total_files = len(files)
    run_start = time.time()
    for idx, path in enumerate(files, 1):
        progress = int((idx / total_files) * 30)
        bar = "#" * progress + "-" * (30 - progress)
        elapsed = time.time() - run_start
        avg_per = elapsed / idx if idx else 0.0
        eta = max(0.0, avg_per * (total_files - idx))
        status = (
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
        file_totals.append((total_ms, path.name))

        for label, (total, count) in labels.items():
            if label == "total_decode":
                continue
            entry = aggregate.setdefault(label, [0.0, 0])
            entry[0] += total
            entry[1] += count

        filtered = {k: v for k, v in labels.items() if k != "total_decode"}
        if filtered:
            top = max(filtered.items(), key=lambda kv: kv[1][0])
            per_file_top.append((top[1][0], top[0], path.name))

    aggregate_rows = []
    for label, (total, count) in aggregate.items():
        avg = total / count if count else 0.0
        aggregate_rows.append((total, label, count, avg))
    aggregate_rows.sort(reverse=True)
    file_totals.sort(reverse=True)
    per_file_top.sort(reverse=True)

    report_path = root / "PROFILE_REPORT.md"
    with report_path.open("w", encoding="ascii") as f:
        f.write("# JBIG2 Decode Profiling Report\n\n")
        f.write(
            "Profiling run across UBC fixtures using "
            "`cargo run --example decode_file --release --profile`.\n\n"
        )
        f.write("## Slowest Decoder Sections (Aggregated)\n\n")
        f.write("| Rank | Label | Total ms | Calls | Avg ms |\n")
        f.write("| --- | --- | --- | --- | --- |\n")
        for i, (total, label, count, avg) in enumerate(aggregate_rows[:15], 1):
            f.write(f"| {i} | {label} | {total:.3f} | {count} | {avg:.3f} |\n")

        f.write("\n## Slowest Files (Total Decode)\n\n")
        f.write("| Rank | File | Total ms |\n")
        f.write("| --- | --- | --- |\n")
        for i, (total, name) in enumerate(file_totals[:15], 1):
            f.write(f"| {i} | {name} | {total:.3f} |\n")

        f.write("\n## Per-File Top Hotspot (Excludes total_decode)\n\n")
        f.write("| Rank | File | Top Label | Total ms |\n")
        f.write("| --- | --- | --- | --- |\n")
        for i, (total, label, name) in enumerate(per_file_top[:15], 1):
            f.write(f"| {i} | {name} | {label} | {total:.3f} |\n")

    print(f"wrote {report_path}")


if __name__ == "__main__":
    main()
