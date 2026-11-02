#!/usr/bin/env bash
# The version, counted from 0.0.0 over every commit since the first (DESIGN.md § Release; owner:
# "YOU NEED TO BE COUNTING FROM 0.0.0 ON FROM THE VERY FIRST COMMIT"). Conventional Commits' own
# semver mapping, applied to each commit in order: a breaking change (`!` after the type, or a
# `BREAKING CHANGE:` footer) adds one to the major and zeroes the rest; else a `feat` adds one to the
# minor and zeroes the patch; else a `fix` adds one to the patch; every other type leaves the version
# alone. Merge commits are skipped. Nobody types the number: Cargo.toml is written from this, and the
# release refuses a Cargo.toml that disagrees with it.
#
#   version-from-log.sh [<rev>]      the version at <rev> (default HEAD), from the root commit
#   version-from-log.sh --self-test  the fixtures: each sequence of commits reaches the version it must
set -euo pipefail

# Reads commit records on stdin, oldest first, each opened by a record separator (0x1e) and holding
# the subject, a unit separator (0x1f) and the body, and prints the version they count to. git log
# emits the shape with --format='%x1e%s%x1f%b', and the body keeps its newlines, so a footer is a
# line of the body.
count() {
  awk '
    BEGIN { RS = "\036"; FS = "\037"; major = 0; minor = 0; patch = 0 }
    {
      subject = $1; body = $2
      breaking = (subject ~ /^[a-z]+(\([^)]*\))?!: /) || (body ~ /(^|\n)BREAKING[ -]CHANGE: /)
      if (breaking) { major++; minor = 0; patch = 0 }
      else if (subject ~ /^feat(\([^)]*\))?: /) { minor++; patch = 0 }
      else if (subject ~ /^fix(\([^)]*\))?: /) { patch++ }
    }
    END { printf "%d.%d.%d\n", major, minor, patch }
  '
}

self_test() {
  local status=0
  # Each case: expected version, then the commits oldest first; `|` separates subject from body.
  check() {
    local want=$1; shift
    local got rec
    got=$(for rec in "$@"; do printf '\036%s\037%b' "${rec%%|*}" "${rec#*|}"; done | count)
    if [ "$got" != "$want" ]; then
      echo "self-test: want $want, got $got for: $*" >&2
      status=1
    fi
  }
  check 0.0.0 'chore: initial commit|'
  check 0.0.0 'docs: write the README|' 'test: cover the edge|' 'ci: pin the actions|' 'perf: inline the chain|' 'refactor: split|' 'bench: add rows|' 'build: lay the crate down|'
  check 0.1.0 'chore: initial|' 'feat: add the sum|'
  check 0.0.1 'fix: refuse a bad byte|'
  check 0.1.1 'feat: add the sum|' 'fix: refuse a bad byte|'
  check 0.1.2 'feat: add the sum|' 'fix: one|' 'fix: two|'
  check 0.2.0 'feat: add the sum|' 'fix: one|' 'feat(wide): add Wide|'
  check 0.2.1 'feat: a|' 'fix: b|' 'feat(wide,fixed): c|' 'fix(exact/heap): d|'
  check 1.0.0 'feat!: drop the f32 path|'
  check 1.0.0 'feat: a|' 'fix: b|' 'refactor(exact)!: widen the word|'
  check 1.0.0 'feat: a|' 'fix: b|' 'feat: c|BREAKING CHANGE: the wire format moved'
  check 1.0.0 'feat: a|' 'fix: b|' 'fix: c|Closes #4\nBREAKING-CHANGE: the cap moved'
  check 1.1.0 'feat!: a|' 'feat: b|'
  check 1.0.1 'feat!: a|' 'fix: b|'
  check 2.0.0 'feat!: a|' 'fix: b|' 'chore!: c|'
  check 0.1.0 'feat: a|' 'Merge pull request #1 from jbrjake/dev|' 'Revert "fix: b"|' 'fixup! feat: a|'
  check 0.1.0 'feat: a|' 'feature: not a type|' 'Feat: capitalised|' 'feat:no space|' 'fix - dashed|'
  check 0.1.0 'feat: a|A body that merely mentions a BREAKING CHANGE: in prose is still a feat'
  if [ "$status" -eq 0 ]; then
    echo "version-from-log: self-test OK"
  fi
  return "$status"
}

case "${1:-}" in
  --self-test) self_test ;;
  *)
    rev=${1:-HEAD}
    git log --reverse --topo-order --no-merges --format='%x1e%s%x1f%b' "$rev" | count
    ;;
esac
