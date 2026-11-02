#!/usr/bin/env bash
# Writes the release the log counts to (DESIGN.md § Release): the version into Cargo.toml and
# Cargo.lock, and the CHANGELOG.md entry for the commits since the last release tag under the
# `## [Unreleased]` line. Nobody types the number: scripts/version-from-log.sh counts it from the
# first commit, scripts/changelog-entry.sh renders the entry, and the release workflow refuses a
# Cargo.toml that disagrees with the count. Prints the version. With the tree already carrying it,
# changes nothing and says so. The optional date is for a reproducible run; the default is today, UTC.
#
#   prepare-release.sh [<date>]
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

version=$(./scripts/version-from-log.sh)
last=$(git describe --tags --abbrev=0 --match 'v*' HEAD)
current=$(awk -F'"' '/^version = "/ { print $2; exit }' Cargo.toml)

if [ "$current" = "$version" ] && grep -q "^## \[$version\]" CHANGELOG.md; then
  echo "prepare-release: $version is in place (Cargo.toml and CHANGELOG.md carry it)" >&2
  echo "$version"
  exit 0
fi
if [ "$last" = "v$version" ]; then
  echo "prepare-release: the log counts to $version and $last is already released; nothing to prepare" >&2
  echo "$version"
  exit 0
fi

# The package's version line is the first `version = "…"` in Cargo.toml, under [package].
awk -v v="$version" 'BEGIN { done = 0 } /^version = "/ && !done { print "version = \"" v "\""; done = 1; next } { print }' Cargo.toml > Cargo.toml.release
mv Cargo.toml.release Cargo.toml
# Cargo.lock's own-package records follow. Only the workspace members move; the index is read because
# a runner has none cached, and nothing is downloaded.
cargo update --workspace --quiet

# The entry goes through a file: macOS awk takes no newline in a -v string.
entry=$(mktemp)
./scripts/changelog-entry.sh "$version" "$last..HEAD" ${1:+"$1"} > "$entry"
awk -v f="$entry" '{ print } /^## \[Unreleased\]$/ { print ""; while ((getline line < f) > 0) print line; close(f) }' CHANGELOG.md > CHANGELOG.md.release
mv CHANGELOG.md.release CHANGELOG.md
rm -f "$entry"

echo "prepare-release: $current -> $version, the entry since $last written" >&2
echo "$version"
