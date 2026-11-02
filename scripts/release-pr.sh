#!/usr/bin/env bash
# The release PR, cut from a clone (DESIGN.md § Release). In a temporary worktree at origin/dev,
# scripts/prepare-release.sh writes the version the log counts to into Cargo.toml and Cargo.lock and
# the entry for the commits since the last tag into CHANGELOG.md; the three files are committed as
# `chore(release): vX.Y.Z`, pushed as release/next, and opened or refreshed as a pull request against
# dev with the entry as its body, under the developer's own gh login. The checkout this runs from is
# never touched: no branch is made or moved in it. No runner holds write access to the repository for
# this step, because a commit and a pull request carry no identity a runner could add; the .crate's
# attestation does, and that step is the release workflow's. With dev already carrying the version,
# nothing is pushed and the script says so.
#
#   release-pr.sh              cut or refresh the release PR against dev
#   release-pr.sh --dry-run    everything but the push and the pull request: the version, the diff
#                              and the entry, to read first
#   release-pr.sh --self-test  the fixture: against a bare origin, with gh stubbed through $GH, the
#                              PR opens, then refreshes, then has nothing to do, then a dry run
#                              pushes nothing
set -euo pipefail

GH=${GH:-gh}

cut() {
  local dry=$1
  local root tmp tree version entry pr
  root=$(git rev-parse --show-toplevel)
  tmp=$(mktemp -d)
  tree=$tmp/tree
  entry=$tmp/entry.md
  # shellcheck disable=SC2064  # $root and $tree are fixed now; the trap wants their values
  trap "git -C '$root' worktree remove --force '$tree' >/dev/null 2>&1 || true; rm -rf '$tmp'" EXIT

  git -C "$root" fetch --quiet --tags origin dev
  git -C "$root" worktree add --quiet --detach "$tree" origin/dev
  cd "$tree"

  version=$(./scripts/prepare-release.sh)
  if git diff --quiet -- Cargo.toml Cargo.lock CHANGELOG.md; then
    echo "release-pr: $version is in place on dev; no release PR to open" >&2
    return 0
  fi
  git --no-pager diff --stat -- Cargo.toml Cargo.lock CHANGELOG.md >&2

  # The commit is the three generated files on a detached head; the hooks are the developer's gates
  # for hand-written changes and do not run here. The dev→main pull request runs the whole suite.
  git add Cargo.toml Cargo.lock CHANGELOG.md
  git commit --quiet --no-verify -m "chore(release): v$version"
  awk -v v="$version" 'index($0, "## [" v "]") == 1 { p = 1; print; next } /^## \[/ { p = 0 } p { print }' CHANGELOG.md > "$entry"
  if [ ! -s "$entry" ]; then
    echo "release-pr: CHANGELOG.md carries no entry for $version" >&2
    return 1
  fi

  if [ "$dry" = 1 ]; then
    echo "release-pr: dry run; would push release/next and open or refresh the pull request 'chore(release): v$version' against dev with this body:" >&2
    cat "$entry" >&2
    return 0
  fi

  git push --quiet --force origin HEAD:refs/heads/release/next
  cd "$root"
  pr=$("$GH" pr list --head release/next --base dev --state open --json number --jq '.[0].number // empty')
  if [ -n "$pr" ]; then
    "$GH" pr edit "$pr" --title "chore(release): v$version" --body-file "$entry"
    echo "release-pr: refreshed the release PR #$pr at v$version" >&2
  else
    "$GH" pr create --base dev --head release/next --title "chore(release): v$version" --body-file "$entry"
    echo "release-pr: opened the release PR at v$version" >&2
  fi
}

self_test() (
  set -euo pipefail
  local here tmp status=0
  here=$(cd "$(dirname "$0")" && pwd)
  # The commit gate runs this from a git hook, which exports GIT_INDEX_FILE, GIT_DIR and more; a
  # fixture repository that inherited them would read and write the enclosing one's index.
  while IFS='=' read -r v _; do unset "$v"; done < <(env | grep -o '^GIT_[A-Za-z0-9_]*=')
  tmp=$(mktemp -d)
  trap 'rm -rf "$tmp"' EXIT
  export GIT_AUTHOR_NAME=fixture GIT_AUTHOR_EMAIL=fixture@example.invalid
  export GIT_COMMITTER_NAME=fixture GIT_COMMITTER_EMAIL=fixture@example.invalid
  export GH="$tmp/gh" GH_LOG="$tmp/gh.log" GH_PR="$tmp/gh.pr"

  fail() { echo "self-test: $*" >&2; status=1; }

  # The stub records every call and answers `pr list` with the number in $GH_PR, when there is one.
  # A body file is copied beside the log, since the real one dies with the script's temp dir.
  cat > "$GH" <<'STUB'
#!/usr/bin/env bash
printf '%s\n' "$*" >> "$GH_LOG"
case "$1 $2" in
  "pr list") [ -s "$GH_PR" ] && cat "$GH_PR" ;;
  "pr create"|"pr edit") while [ $# -gt 0 ]; do [ "$1" = --body-file ] && cp "$2" "$GH_LOG.body"; shift; done ;;
esac
exit 0
STUB
  chmod +x "$GH"

  git init --quiet --bare "$tmp/origin.git"
  git init --quiet "$tmp/clone"
  cd "$tmp/clone"
  git checkout --quiet -b dev
  git remote add origin "$tmp/origin.git"
  mkdir -p src scripts
  printf '[package]\nname = "fixture"\nversion = "0.0.0"\nedition = "2021"\n' > Cargo.toml
  : > src/lib.rs
  printf '# Changelog\n\n## [Unreleased]\n' > CHANGELOG.md
  cp "$here"/version-from-log.sh "$here"/changelog-entry.sh "$here"/prepare-release.sh "$here"/release-pr.sh scripts/
  cargo generate-lockfile --quiet --offline
  git add -A && git commit --quiet -m "build: lay the crate down"
  git tag v0.0.0
  echo a > a && git add a && git commit --quiet -m "feat: add the sum"
  echo b > b && git add b && git commit --quiet -m "fix: refuse a bad byte"
  git push --quiet origin dev --tags

  # 1. A first run opens the PR at the version the log counts to, with the three files on release/next
  #    and the entry as the body, and leaves the clone's tree and branches alone.
  ./scripts/release-pr.sh 2>/dev/null
  git fetch --quiet origin release/next
  [ "$(git log -1 --format=%s origin/release/next)" = "chore(release): v0.1.1" ] || fail "release/next carries '$(git log -1 --format=%s origin/release/next)', not the v0.1.1 commit"
  [ "$(git diff --name-only origin/dev origin/release/next | tr '\n' ' ')" = "CHANGELOG.md Cargo.lock Cargo.toml " ] || fail "release/next changes $(git diff --name-only origin/dev origin/release/next | tr '\n' ' ')"
  grep -q '^pr create --base dev --head release/next --title chore(release): v0.1.1 --body-file ' "$GH_LOG" || fail "no pr create in the gh log: $(cat "$GH_LOG")"
  head -1 "$GH_LOG.body" | grep -q '^## \[0.1.1\]' || fail "the body does not open with the 0.1.1 entry: $(head -1 "$GH_LOG.body")"
  grep -q 'refuse a bad byte' "$GH_LOG.body" || fail "the body lacks the fix line"
  [ -z "$(git status --porcelain)" ] || fail "the clone's tree moved: $(git status --porcelain)"
  [ -z "$(git branch --list release/next)" ] || fail "a local release/next branch was made"
  [ -z "$(git worktree list | sed 1d)" ] || fail "a worktree was left behind: $(git worktree list)"

  # 2. With the PR open, a second run refreshes it rather than opening another.
  echo 7 > "$GH_PR"
  ./scripts/release-pr.sh 2>/dev/null
  grep -q '^pr edit 7 --title chore(release): v0.1.1 --body-file ' "$GH_LOG" || fail "no pr edit 7 in the gh log: $(cat "$GH_LOG")"
  grep -c '^pr create' "$GH_LOG" | grep -qx 1 || fail "a second pr create was made"

  # 3. Once dev carries the version, there is nothing to do: no push, no gh call.
  git merge --quiet --no-ff -m "Merge pull request #7 from fixture/release/next" origin/release/next
  git push --quiet origin dev
  : > "$GH_PR"
  local before after
  before=$(wc -l < "$GH_LOG")
  ./scripts/release-pr.sh 2> "$tmp/out" || fail "the in-place run failed: $(cat "$tmp/out")"
  grep -q 'is in place on dev' "$tmp/out" || fail "the in-place run did not say so: $(cat "$tmp/out")"
  after=$(wc -l < "$GH_LOG")
  [ "$before" = "$after" ] || fail "the in-place run called gh"

  # 4. A dry run after another fix prints the next version and entry and pushes nothing.
  echo c > c && git add c && git commit --quiet -m "fix: close the edge"
  git push --quiet origin dev
  ./scripts/release-pr.sh --dry-run 2> "$tmp/out" || fail "the dry run failed: $(cat "$tmp/out")"
  grep -q "dry run; would push release/next and open or refresh the pull request 'chore(release): v0.1.2'" "$tmp/out" || fail "the dry run did not name v0.1.2: $(cat "$tmp/out")"
  grep -q 'close the edge' "$tmp/out" || fail "the dry run did not print the entry"
  git fetch --quiet origin release/next
  [ "$(git log -1 --format=%s origin/release/next)" = "chore(release): v0.1.1" ] || fail "the dry run moved release/next"
  [ "$(wc -l < "$GH_LOG")" = "$after" ] || fail "the dry run called gh"

  if [ "$status" -eq 0 ]; then
    echo "release-pr: self-test OK"
  fi
  return "$status"
)

case "${1:-}" in
  --self-test) self_test ;;
  --dry-run) cut 1 ;;
  "") cut 0 ;;
  *) echo "release-pr: usage: release-pr.sh [--dry-run | --self-test]" >&2; exit 2 ;;
esac
