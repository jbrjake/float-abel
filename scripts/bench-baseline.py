#!/usr/bin/env python3
"""The per-machine bench baseline of CLAUDE.md § Performance, behind scripts/bench-baseline.sh.

`write` records the last criterion sitting on this machine into bench-baseline.json: every
benchmark id's median time per run and its element count, read from `<criterion>/**/new/`
(criterion writes a group `a/b` to a directory `a_b`), and the peak resident set of the bench
process on the end-to-end `dd_sum` rows BENCH.md § The end-to-end SUM quotes, each run alone
under `/usr/bin/time` with a criterion home of its own so the sitting's data stays as it was.
`compare` reads the same estimates and prints every row whose median per element is past the
threshold, 1.5× by default, of the recorded one, and with `--rss` re-measures the end-to-end rows
and reports those the same way. Advisory, never a gate: the exit status is 0 either way, and a
row past the line is a design question for the owner, not a broken build.

The machine key is the CPU's brand string (`Apple M5 Max`, `AMD Ryzen 7 5700U`), so one file holds
every machine the crate is measured on and a sitting replaces only its own machine's entry.
"""

import argparse
import datetime
import json
import os
import pathlib
import platform
import re
import subprocess
import sys
import tempfile

BASELINE = "bench-baseline.json"
THRESHOLD = 1.5
# The end-to-end rows whose peak resident set BENCH.md records, each matched exactly.
RSS_ROWS = [
    "dd_sum/exact/1w/serialized",
    "dd_sum/isize/1w/serialized",
    "dd_sum/exact/8w/serialized",
    "dd_sum/isize/8w/serialized",
    "dd_sum/wide/1w/serialized",
]


def machine():
    """The CPU's brand string, without a trailing `with …` clause."""
    if platform.system() == "Darwin":
        brand = subprocess.run(
            ["sysctl", "-n", "machdep.cpu.brand_string"],
            check=True,
            capture_output=True,
            text=True,
        ).stdout
    else:
        brand = ""
        with open("/proc/cpuinfo", encoding="utf-8") as cpuinfo:
            for line in cpuinfo:
                if line.lower().startswith("model name"):
                    brand = line.split(":", 1)[1]
                    break
    brand = re.sub(r"\s+with\s.*$", "", brand.strip())
    if not brand:
        sys.exit("bench-baseline: cannot name this machine; pass --machine")
    return brand


def target_dir():
    return pathlib.Path(os.environ.get("CARGO_TARGET_DIR", "target"))


def criterion_dir(args):
    if args.criterion:
        return pathlib.Path(args.criterion)
    if "CRITERION_HOME" in os.environ:
        return pathlib.Path(os.environ["CRITERION_HOME"])
    return target_dir() / "criterion"


def read_rows(criterion):
    """Every benchmark id under `criterion` with its median per run in ns and its elements."""
    rows = {}
    for bench in sorted(criterion.glob("**/new/benchmark.json")):
        with open(bench, encoding="utf-8") as f:
            meta = json.load(f)
        with open(bench.with_name("estimates.json"), encoding="utf-8") as f:
            estimates = json.load(f)
        throughput = meta.get("throughput") or {}
        elements = throughput.get("Elements")
        rows[meta["full_id"]] = {
            "median_ns": round(estimates["median"]["point_estimate"], 4),
            "elements": elements,
        }
    if not rows:
        sys.exit(f"bench-baseline: no criterion estimates under {criterion}")
    return rows


def per_element(row):
    elements = row.get("elements")
    return row["median_ns"] / elements if elements else row["median_ns"]


def dd_sum_binary(args):
    """The newest `dd_sum-*` bench binary, which `cargo bench --bench dd_sum --no-run` builds."""
    if args.dd_sum:
        return pathlib.Path(args.dd_sum)
    deps = target_dir() / "release" / "deps"
    candidates = [p for p in deps.glob("dd_sum-*") if p.suffix != ".d" and os.access(p, os.X_OK)]
    if not candidates:
        sys.exit(f"bench-baseline: no dd_sum bench binary under {deps}; pass --dd-sum")
    return max(candidates, key=lambda p: p.stat().st_mtime)


def peak_rss(binary, row):
    """Bytes of peak resident set of one row run alone, under a throwaway criterion home."""
    if platform.system() == "Darwin":
        flag, pattern = "-l", re.compile(r"^\s*(\d+)\s+maximum resident set size", re.M)
    else:
        flag, pattern = "-v", re.compile(r"Maximum resident set size \(kbytes\):\s*(\d+)", re.M)
    with tempfile.TemporaryDirectory() as home:
        env = dict(os.environ, CRITERION_HOME=home)
        done = subprocess.run(
            ["/usr/bin/time", flag, str(binary), "--bench", f"^{re.escape(row)}$"],
            env=env,
            capture_output=True,
            text=True,
            check=False,
        )
    if done.returncode != 0:
        sys.exit(f"bench-baseline: {binary} on {row} exited {done.returncode}:\n{done.stderr}")
    found = pattern.search(done.stderr)
    if not found:
        sys.exit(f"bench-baseline: no resident set size in time's output for {row}")
    if row not in done.stdout:
        sys.exit(f"bench-baseline: {binary} ran no row named {row}")
    value = int(found.group(1))
    return value if platform.system() == "Darwin" else value * 1024


def measure_rss(args):
    binary = dd_sum_binary(args)
    rss = {}
    for row in RSS_ROWS:
        rss[row] = peak_rss(binary, row)
        print(f"{row}: {rss[row] / 1e9:.2f} GB peak resident set", file=sys.stderr)
    return rss


def load_baseline(path):
    if not path.exists():
        return {}
    with open(path, encoding="utf-8") as f:
        return json.load(f)


def first_line(argv):
    """A command's first line of output, or `unknown` when it is absent or fails: a sitting's
    tree may be a synced copy with no git, and rustc may be off the login PATH."""
    try:
        done = subprocess.run(argv, capture_output=True, text=True, check=False)
    except FileNotFoundError:
        return "unknown"
    return done.stdout.strip().split("\n", 1)[0] if done.returncode == 0 else "unknown"


def commit(args):
    return args.commit or first_line(["git", "rev-parse", "--short", "HEAD"])


def rustc():
    return first_line(["rustc", "--version"])


def write(args):
    key = args.machine or machine()
    rows = read_rows(criterion_dir(args))
    entry = {
        "commit": commit(args),
        "taken": datetime.date.today().isoformat(),
        "os": f"{platform.system()} {platform.release()}",
        "rustc": rustc(),
        "rows": rows,
    }
    if not args.no_rss:
        entry["rss_bytes"] = measure_rss(args)
    path = pathlib.Path(args.baseline)
    baseline = load_baseline(path)
    baseline[key] = entry
    with open(path, "w", encoding="utf-8") as f:
        json.dump(baseline, f, indent=2, sort_keys=True, ensure_ascii=False)
        f.write("\n")
    print(f"{path}: {key} at {entry['commit']}, {len(rows)} rows", file=sys.stderr)


def report(title, pairs, threshold, unit, show_all):
    """Rows past the threshold, worst first; with `show_all`, every row."""
    past = []
    for name, before, after in pairs:
        ratio = after / before if before else float("inf")
        if show_all or ratio > threshold:
            past.append((ratio, name, before, after))
    past.sort(reverse=True)
    if not past:
        print(f"{title}: every row within {threshold}× of the baseline")
        return
    print(f"{title}: {len(past)} row(s){'' if show_all else f' past {threshold}×'}")
    width = max(len(name) for _, name, _, _ in past)
    for ratio, name, before, after in past:
        print(f"  {name:<{width}}  {before:>12.4f}  {after:>12.4f} {unit}  {ratio:>6.2f}×")


def compare(args):
    key = args.machine or machine()
    baseline = load_baseline(pathlib.Path(args.baseline)).get(key)
    if baseline is None:
        sys.exit(f"bench-baseline: no entry for {key} in {args.baseline}; run `write` there first")
    rows = read_rows(criterion_dir(args))
    recorded = baseline["rows"]
    pairs = [
        (name, per_element(recorded[name]), per_element(rows[name]))
        for name in sorted(rows)
        if name in recorded
    ]
    print(f"{key}: baseline at {baseline['commit']} ({baseline['taken']}), {len(pairs)} rows compared")
    report("medians per element", pairs, args.threshold, "ns", args.all)
    for name in sorted(set(rows) - set(recorded)):
        print(f"  new, not in the baseline: {name}")
    for name in sorted(set(recorded) - set(rows)):
        print(f"  in the baseline, not measured: {name}")
    if args.rss:
        recorded_rss = baseline.get("rss_bytes", {})
        measured = measure_rss(args)
        pairs = [
            (row, recorded_rss[row] / 1e9, measured[row] / 1e9)
            for row in RSS_ROWS
            if row in recorded_rss
        ]
        report("peak resident set", pairs, args.threshold, "GB", args.all)


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n", 1)[0])
    parser.add_argument("verb", choices=["write", "compare"])
    parser.add_argument("--baseline", default=BASELINE, help="the json file (default: %(default)s)")
    parser.add_argument("--machine", help="the machine key (default: the CPU brand string)")
    parser.add_argument("--criterion", help="criterion's output dir (default: <target>/criterion)")
    parser.add_argument("--dd-sum", help="the dd_sum bench binary (default: the newest built)")
    parser.add_argument("--commit", help="the commit to record (default: git rev-parse)")
    parser.add_argument("--threshold", type=float, default=THRESHOLD, help="ratio past which a row is reported (default: %(default)s)")
    parser.add_argument("--no-rss", action="store_true", help="write: skip the peak resident set rows")
    parser.add_argument("--rss", action="store_true", help="compare: re-measure the peak resident set rows too")
    parser.add_argument("--all", action="store_true", help="compare: print every row, not only those past the threshold")
    args = parser.parse_args()
    if args.verb == "write":
        write(args)
    else:
        compare(args)


if __name__ == "__main__":
    main()
