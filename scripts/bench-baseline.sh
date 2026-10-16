#!/usr/bin/env bash
# The per-machine bench baseline, CLAUDE.md § Performance. `write` records this machine's last
# criterion sitting (every id's median per run and the end-to-end rows' peak resident set) into
# bench-baseline.json; `compare` prints every row past 1.5× of the recorded one. Advisory, never a
# gate, and never run from a hook. The work is scripts/bench-baseline.py; run with --help for the
# flags (--criterion, --machine, --threshold, --rss, --all).
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
exec python3 -I scripts/bench-baseline.py "$@"
