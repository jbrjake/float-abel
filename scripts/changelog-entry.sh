#!/usr/bin/env bash
# The CHANGELOG entry for one release, rendered from the log by this script alone: the project's
# posture is no dependency it does not need, in the build as in the crate (owner: "THIS IS A
# MINIMALIST PROJECT THAT IS SUPPOSED TO HAVE NO DEPENDENCIES IT DOESN'T NEED"). Commits are grouped
# by Conventional Commits type, newest first, the scope in italics and a breaking change marked;
# chore(meshwork) and chore(release) commits are the store's and the release's own and stay out.
#
#   changelog-entry.sh <version> <from>..<to> [<date>]  the entry for the commits in the range
#   changelog-entry.sh --self-test                       the fixture renders to the expected entry
set -euo pipefail

REPO_URL=https://github.com/jbrjake/float-abel

# Reads commit records on stdin, newest first, each opened by a record separator (0x1e) and holding
# the subject, a unit separator (0x1f) and the body (git log --format='%x1e%s%x1f%b'), and prints
# the entry for $1 (the version) counted from $2 (the tag it follows) on $3 (the date).
render() {
  awk -v version="$1" -v from="$2" -v date="$3" -v repo="$REPO_URL" '
    BEGIN {
      RS = "\036"; FS = "\037"
      norder = split("Added Fixed Performance Changed Documentation Tests Benchmarks Build CI Other Reverted", order, " ")
      g["feat"] = "Added"; g["fix"] = "Fixed"; g["perf"] = "Performance"; g["refactor"] = "Changed"
      g["docs"] = "Documentation"; g["test"] = "Tests"; g["bench"] = "Benchmarks"; g["build"] = "Build"
      g["ci"] = "CI"; g["chore"] = "Other"; g["revert"] = "Reverted"
    }
    NF == 0 { next }
    {
      s = $1; body = $2
      scope = ""
      breaking = (body ~ /(^|\n)BREAKING[ -]CHANGE: /)
      if (match(s, /^[a-z]+(\([^)]*\))?!?: /)) {
        head = substr(s, 1, RLENGTH); text = substr(s, RLENGTH + 1)
        type = head; sub(/[(!:].*$/, "", type)
        if (match(head, /\([^)]*\)/)) scope = substr(head, RSTART + 1, RLENGTH - 2)
        if (head ~ /!: $/) breaking = 1
        if (type == "chore" && (scope == "meshwork" || scope == "release")) next
        group = (type in g) ? g[type] : "Other"
      } else {
        text = s; group = "Other"
      }
      line = "- "
      if (scope != "") line = line "*(" scope ")* "
      if (breaking) line = line "[**breaking**] "
      lines[group] = lines[group] line text "\n"
    }
    END {
      printf "## [%s](%s/compare/%s...v%s) - %s\n", version, repo, from, version, date
      for (i = 1; i <= norder; i++) {
        grp = order[i]
        if (grp in lines) printf "\n### %s\n%s", grp, lines[grp]
      }
    }
  '
}

self_test() {
  local want got
  want='## [0.29.3]('"$REPO_URL"'/compare/v0.0.0...v0.29.3) - 2026-11-02

### Added
- *(wide)* add Wide, the 34-limb exact f64 sum
- add the compact tier

### Fixed
- *(exact)* close the limb cap under retraction at its edge
- refuse a non-canonical byte

### Performance
- *(exact)* inline the operator chain

### Changed
- *(exact)* [**breaking**] widen the compact word
- *(wide,fixed)* move the operator impls into ops submodules

### Documentation
- *(readme)* rewrite the install section

### Tests
- *(laws)* cross the cap edge

### Benchmarks
- add the crossover rows

### Build
- *(deps)* bump actions/checkout from 4 to 5

### CI
- [**breaking**] run ci on dev

### Other
- release v0.1.0
- a subject nobody shaped

### Reverted
- feat(wide): add Wide'
  got=$(
    for rec in \
      'feat(wide): add Wide, the 34-limb exact f64 sum|' \
      'chore(meshwork): close a task|' \
      'fix(exact): close the limb cap under retraction at its edge|' \
      'refactor(exact)!: widen the compact word|' \
      'docs(readme): rewrite the install section|' \
      'chore(release): v0.29.2|' \
      'perf(exact): inline the operator chain|' \
      'test(laws): cross the cap edge|' \
      'ci: run ci on dev|Closes #9\nBREAKING CHANGE: the gate moved' \
      'bench: add the crossover rows|' \
      'feat: add the compact tier|' \
      'refactor(wide,fixed): move the operator impls into ops submodules|' \
      'build(deps): bump actions/checkout from 4 to 5|' \
      'chore: release v0.1.0|' \
      'a subject nobody shaped|' \
      'revert: feat(wide): add Wide|' \
      'fix: refuse a non-canonical byte|'
    do printf '\036%s\037%b' "${rec%%|*}" "${rec#*|}"; done | render 0.29.3 v0.0.0 2026-11-02
  )
  if [ "$got" != "$want" ]; then
    echo "self-test: the rendered entry differs from the expected one:" >&2
    diff <(printf '%s\n' "$want") <(printf '%s\n' "$got") >&2 || true
    return 1
  fi
  echo "changelog-entry: self-test OK"
}

case "${1:-}" in
  --self-test) self_test ;;
  "") echo "changelog-entry: usage: changelog-entry.sh <version> <from>..<to> [<date>] | --self-test" >&2; exit 2 ;;
  *)
    version=$1
    range=$2
    date=${3:-$(date -u +%F)}
    from=${range%%..*}
    git log --no-merges --format='%x1e%s%x1f%b' "$range" | render "$version" "$from" "$date"
    ;;
esac
