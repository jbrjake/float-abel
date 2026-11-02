#!/usr/bin/env bash
# The release pull request, one, dev into main (DESIGN.md § Release; owner: "i want it to be a
# single PR, dev to main."). From a clone on dev, clean and at origin/dev, under the developer's own
# gh login: scripts/prepare-release.sh writes the version the log counts to into Cargo.toml and
# Cargo.lock and the entry for the commits since the last tag into CHANGELOG.md; the three files are
# committed on dev as `chore(release): vX.Y.Z` and pushed (a push to dev is just a push); then the
# pull request from dev into main is opened, or refreshed when one is open, with the entry as its
# body. Merging it is the release. No runner holds write access to the repository for any of this: a
# commit and a pull request carry no identity a runner could add; the .crate's attestation does, and
# that step is the release workflow's. With dev already carrying the version the commit is skipped
# and the pull request still opened or refreshed; with the version tagged, there is nothing to do.
#
# The release commit and its push skip the hooks. The tree is what the push gate passed at
# origin/dev, and the three files differ from it by a version line, the lockfile's own-package
# record and a changelog entry, none of which the gate reads; the pull request runs the whole suite.
#
#   release-pr.sh              commit the release onto dev, push, open or refresh the dev→main PR
#   release-pr.sh --dry-run    the version, the diff and the entry, with the three files put back
#                              and nothing pushed or opened
#   release-pr.sh --self-test  the fixture: against a bare origin with gh stubbed through $GH, the
#                              PR opens, then refreshes without a second commit, then has nothing to
#                              do once tagged; a dry run moves nothing; a dirty tree, another branch
#                              and a dev behind origin are refused
set -euo pipefail

GH=${GH:-gh}
FILES=(Cargo.toml Cargo.lock CHANGELOG.md)

refuse() { echo "release-pr: $*" >&2; exit 1; }

cut() {
  local dry=$1
  local branch version tag entry pr before
  cd "$(git rev-parse --show-toplevel)"
  branch=$(git rev-parse --abbrev-ref HEAD)
  [ "$branch" = dev ] || refuse "on branch $branch; the release is cut from dev"
  [ -z "$(git status --porcelain)" ] || refuse "the tree is not clean; commit or stash first"
  git fetch --quiet --tags origin dev
  [ "$(git rev-parse HEAD)" = "$(git rev-parse origin/dev)" ] || refuse "dev is not at origin/dev; pull or push first"
  before=$(git rev-parse --short HEAD)

  version=$(./scripts/prepare-release.sh)
  tag=v$version
  if git rev-parse -q --verify "refs/tags/$tag" >/dev/null; then
    echo "release-pr: $tag is already released; nothing to do" >&2
    return 0
  fi
  entry=$(mktemp)
  # shellcheck disable=SC2064  # $entry is fixed now; the trap wants its value
  trap "rm -f '$entry'" EXIT
  awk -v v="$version" 'index($0, "## [" v "]") == 1 { p = 1; print; next } /^## \[/ { p = 0 } p { print }' CHANGELOG.md > "$entry"
  if [ ! -s "$entry" ]; then
    git checkout --quiet -- "${FILES[@]}"
    refuse "CHANGELOG.md carries no entry for $version"
  fi

  local pending=0
  git diff --quiet -- "${FILES[@]}" || pending=1
  if [ "$pending" = 1 ]; then
    git --no-pager diff --stat -- "${FILES[@]}" >&2
  else
    echo "release-pr: $version is in place on dev" >&2
  fi

  if [ "$dry" = 1 ]; then
    if [ "$pending" = 1 ]; then
      echo "release-pr: dry run; would commit 'chore(release): $tag' onto dev, push it, and open or refresh the pull request dev → main with this body:" >&2
    else
      echo "release-pr: dry run; would open or refresh the pull request dev → main with this body:" >&2
    fi
    cat "$entry" >&2
    git checkout --quiet -- "${FILES[@]}"
    return 0
  fi

  if [ "$pending" = 1 ]; then
    git add "${FILES[@]}"
    git commit --quiet --no-verify -m "chore(release): $tag"
    git push --quiet --no-verify origin dev
    echo "release-pr: committed and pushed 'chore(release): $tag' onto dev ($before -> $(git rev-parse --short HEAD)); the hooks were skipped, since the gate passed this tree at $before and the pull request runs the suite" >&2
  fi
  pr=$("$GH" pr list --base main --head dev --state open --json number --jq '.[0].number // empty')
  if [ -n "$pr" ]; then
    "$GH" pr edit "$pr" --title "chore(release): $tag" --body-file "$entry"
    echo "release-pr: refreshed the pull request #$pr, dev → main, at $tag" >&2
  else
    "$GH" pr create --base main --head dev --title "chore(release): $tag" --body-file "$entry"
    echo "release-pr: opened the pull request dev → main at $tag" >&2
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
  calls() { wc -l < "$GH_LOG" | tr -d ' '; }

  # The stub records every call and answers `pr list` with the number in $GH_PR, when there is one.
  # A body file is copied beside the log, since the real one dies with the script.
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
  : > "$GH_LOG"

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
  git branch main
  echo a > a && git add a && git commit --quiet -m "feat: add the sum"
  echo b > b && git add b && git commit --quiet -m "fix: refuse a bad byte"
  git push --quiet origin dev main --tags

  # 1. The first run commits the version the log counts to onto dev, pushes it, and opens the one
  #    pull request, dev into main, with the entry as its body.
  ./scripts/release-pr.sh 2>/dev/null
  [ "$(git log -1 --format=%s)" = "chore(release): v0.1.1" ] || fail "dev carries '$(git log -1 --format=%s)', not the v0.1.1 commit"
  [ "$(git rev-parse HEAD)" = "$(git rev-parse origin/dev)" ] || fail "dev was not pushed"
  [ "$(git diff --name-only HEAD~1 HEAD | tr '\n' ' ')" = "CHANGELOG.md Cargo.lock Cargo.toml " ] || fail "the release commit changes $(git diff --name-only HEAD~1 HEAD | tr '\n' ' ')"
  grep -q '^pr create --base main --head dev --title chore(release): v0.1.1 --body-file ' "$GH_LOG" || fail "no pr create in the gh log: $(cat "$GH_LOG")"
  head -1 "$GH_LOG.body" | grep -q '^## \[0.1.1\]' || fail "the body does not open with the 0.1.1 entry: $(head -1 "$GH_LOG.body")"
  grep -q 'refuse a bad byte' "$GH_LOG.body" || fail "the body lacks the fix line"
  [ -z "$(git status --porcelain)" ] || fail "the tree is not clean after the run: $(git status --porcelain)"

  # 2. With the PR open and the version in place, a second run refreshes it and commits nothing.
  echo 7 > "$GH_PR"
  local head; head=$(git rev-parse HEAD)
  ./scripts/release-pr.sh 2>/dev/null
  grep -q '^pr edit 7 --title chore(release): v0.1.1 --body-file ' "$GH_LOG" || fail "no pr edit 7 in the gh log: $(cat "$GH_LOG")"
  [ "$(grep -c '^pr create' "$GH_LOG")" = 1 ] || fail "a second pr create was made"
  [ "$(git rev-parse HEAD)" = "$head" ] || fail "the refresh made a commit"

  # 3. Once the version is tagged, there is nothing to do: no commit, no gh call.
  git tag v0.1.1 && git push --quiet origin v0.1.1
  : > "$GH_PR"
  local n; n=$(calls)
  ./scripts/release-pr.sh 2> "$tmp/out" || fail "the released run failed: $(cat "$tmp/out")"
  grep -q 'is already released' "$tmp/out" || fail "the released run did not say so: $(cat "$tmp/out")"
  [ "$(calls)" = "$n" ] || fail "the released run called gh"
  [ "$(git rev-parse HEAD)" = "$head" ] || fail "the released run made a commit"

  # 4. A dry run after another fix prints the next version and entry, puts the files back, and
  #    pushes and opens nothing.
  echo c > c && git add c && git commit --quiet -m "fix: close the edge" && git push --quiet origin dev
  head=$(git rev-parse HEAD)
  ./scripts/release-pr.sh --dry-run 2> "$tmp/out" || fail "the dry run failed: $(cat "$tmp/out")"
  grep -q "dry run; would commit 'chore(release): v0.1.2' onto dev" "$tmp/out" || fail "the dry run did not name v0.1.2: $(cat "$tmp/out")"
  grep -q 'close the edge' "$tmp/out" || fail "the dry run did not print the entry"
  [ "$(git rev-parse HEAD)" = "$head" ] || fail "the dry run made a commit"
  [ "$(git rev-parse origin/dev)" = "$head" ] || fail "the dry run moved origin/dev"
  [ -z "$(git status --porcelain)" ] || fail "the dry run left the tree dirty: $(git status --porcelain)"
  [ "$(calls)" = "$n" ] || fail "the dry run called gh"

  # 5. A dirty tree, another branch and a dev behind origin are refused before anything is written.
  echo x > x
  ./scripts/release-pr.sh 2> "$tmp/out" && fail "a dirty tree was not refused"
  grep -q 'not clean' "$tmp/out" || fail "the dirty tree's refusal: $(cat "$tmp/out")"
  rm x
  git checkout --quiet -b topic
  ./scripts/release-pr.sh 2> "$tmp/out" && fail "another branch was not refused"
  grep -q 'on branch topic' "$tmp/out" || fail "the branch's refusal: $(cat "$tmp/out")"
  git checkout --quiet dev
  git reset --quiet --hard HEAD~1
  ./scripts/release-pr.sh 2> "$tmp/out" && fail "a dev behind origin was not refused"
  grep -q 'not at origin/dev' "$tmp/out" || fail "the stale dev's refusal: $(cat "$tmp/out")"
  git reset --quiet --hard origin/dev
  [ "$(git rev-parse origin/dev)" = "$head" ] || fail "a refusal moved origin/dev"
  [ "$(calls)" = "$n" ] || fail "a refusal called gh"

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
