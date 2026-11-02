#!/usr/bin/env bash
# Whether miri and the fuzz targets are due on a pull request's head. ci.yml's `changes` job asks
# before the two jobs run (owner: "FUZZ NEEDS TO RUN IF IT HASN'T YET SINCE THE CODE LAST CHANGED!").
# The probes are the nightly's, half an hour under miri and ten minutes a fuzz target, and a pull
# request into main re-running them over code a green run already proved costs the owner that hour
# (owner: "I DO NOT WANT CUTTING A PR TO TAKE AN HOUR OF MY TIME just to re-run fuzz tests that are
# SUPPOSED to run every night!").
#
# A green run is a completed run of ci.yml on the head branch whose miri job and every fuzz job
# concluded success; a night that skipped the suite, or a run that skipped the probes, does not count.
# The code is what the probes build: src/, tests/, fuzz/, Cargo.toml and Cargo.lock, with the
# `version = "…"` lines ignored, so the release commit, which moves those lines alone, never makes
# the probes due by itself; a dependency moving in Cargo.lock moves its checksum line too, which is
# not ignored. The newest green run decides: the probes are due when there is none, when its commit
# cannot be fetched, or when the code differs from it.
#
#   probes-due.sh <branch> <head>   prints true when the probes are due on <head>, false when the
#                                   newest green run on <branch> tested the same code; the reason
#                                   goes to stderr. A failure prints nothing and exits non-zero, and
#                                   the caller runs them.
#   probes-due.sh --self-test       the fixture: gh stubbed through $GH over a repository with a
#                                   green run behind docs, version and code changes
set -euo pipefail

GH=${GH:-gh}
CODE=(src tests fuzz Cargo.toml Cargo.lock)
VERSION_LINE='^version = "'
# The head sha of a run whose probe jobs all concluded success, or nothing.
GREEN='if ([.jobs[] | select(.name | test("^cargo miri|^fuzz ")) | .conclusion] | length > 0 and all(. == "success")) then .headSha else empty end'

say() { echo "probes-due: $*" >&2; }

due() {
  local branch=$1 head=$2 ids id sha=
  ids=$("$GH" run list --workflow ci.yml --branch "$branch" --status success --limit 20 --json databaseId --jq '.[].databaseId')
  for id in $ids; do
    sha=$("$GH" run view "$id" --json headSha,jobs --jq "$GREEN")
    [ -n "$sha" ] && break
  done
  if [ -z "$sha" ]; then
    say "no green run of miri and the fuzz targets on $branch; they run"
    echo true
    return
  fi
  if ! git cat-file -e "$sha^{commit}" 2>/dev/null && ! git fetch --quiet --depth=1 origin "$sha" 2>/dev/null; then
    say "the green run's commit $sha is not reachable from here; they run"
    echo true
    return
  fi
  if git diff --quiet -I"$VERSION_LINE" "$sha" "$head" -- "${CODE[@]}"; then
    say "the green run at ${sha:0:12} tested this code; they are skipped"
    echo false
  else
    say "the code changed since the green run at ${sha:0:12}; they run:"
    git --no-pager diff --stat -I"$VERSION_LINE" "$sha" "$head" -- "${CODE[@]}" >&2
    echo true
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
  export GH="$tmp/gh" GH_RUNS="$tmp/runs"
  mkdir "$GH_RUNS"

  fail() { echo "self-test: $*" >&2; status=1; }

  # The stub: `run list` prints the ids in $GH_RUNS/order, newest first; `run view <id>` prints
  # $GH_RUNS/<id>, the head sha of a green run, or nothing for a run whose probes did not pass.
  cat > "$GH" <<'STUB'
#!/usr/bin/env bash
case "$1 $2" in
  "run list") [ -f "$GH_RUNS/order" ] && cat "$GH_RUNS/order" ;;
  "run view") cat "$GH_RUNS/$3" ;;
esac
exit 0
STUB
  chmod +x "$GH"

  git init --quiet "$tmp/repo"
  cd "$tmp/repo"
  mkdir -p src docs
  printf '[package]\nname = "fixture"\nversion = "0.0.0"\nedition = "2021"\n' > Cargo.toml
  : > src/lib.rs
  echo '# fixture' > README.md
  echo '# Changelog' > CHANGELOG.md
  echo notes > docs/notes.md
  cargo generate-lockfile --quiet --offline
  git add -A && git commit --quiet -m "build: lay the crate down"
  local green; green=$(git rev-parse HEAD)

  # The decision on stdout, the reason on stderr.
  check() { # <case> <expected> <reason>
    local name=$1 want=$2 reason=$3 got
    got=$("$here/probes-due.sh" dev HEAD 2> "$tmp/err") || fail "$name: the script failed: $(cat "$tmp/err")"
    [ "$got" = "$want" ] || fail "$name: printed '$got', not $want ($(cat "$tmp/err"))"
    grep -q "$reason" "$tmp/err" || fail "$name: the reason does not say '$reason': $(cat "$tmp/err")"
  }

  # 1. No green run on the branch: due.
  check "no run" true "no green run"

  # 2. A green run at this very commit: not due.
  echo 1 > "$GH_RUNS/order"
  echo "$green" > "$GH_RUNS/1"
  check "same commit" false "tested this code"

  # 3. The release commit's shape: docs, the changelog and the version lines of both manifests move,
  #    and nothing else. Not due.
  echo more >> README.md
  echo more >> docs/notes.md
  echo '## [0.1.0]' >> CHANGELOG.md
  printf '[package]\nname = "fixture"\nversion = "0.1.0"\nedition = "2021"\n' > Cargo.toml
  cargo generate-lockfile --quiet --offline
  git add -A && git commit --quiet -m "chore(release): v0.1.0"
  git diff --quiet "$green" HEAD -- Cargo.toml Cargo.lock && fail "the fixture's release commit moved no version line"
  check "release commit" false "tested this code"

  # 4. The code moves: due, and the reason names the file.
  echo 'pub fn f() {}' > src/lib.rs
  git add -A && git commit --quiet -m "feat: add f"
  check "code moved" true "src/lib.rs"
  git reset --quiet --hard HEAD~1

  # 5. The manifest moves beyond its version line: due.
  printf '[features]\nx = []\n' >> Cargo.toml
  git add -A && git commit --quiet -m "feat: add a feature"
  check "manifest moved" true "Cargo.toml"
  git reset --quiet --hard HEAD~1

  # 6. A newer run whose probes did not pass is passed over for the green run behind it.
  printf '2\n1\n' > "$GH_RUNS/order"
  : > "$GH_RUNS/2"
  check "newer run not green" false "tested this code"

  # 7. The newest green run decides: green at the commit that moved the code, the head one docs
  #    commit past it. Not due, where the older green run alone would have said due.
  echo 'pub fn f() {}' > src/lib.rs
  git add -A && git commit --quiet -m "feat: add f"
  git rev-parse HEAD > "$GH_RUNS/2"
  echo again >> README.md
  git add -A && git commit --quiet -m "docs: say more"
  check "newest green run decides" false "tested this code"

  # 8. A green run at a commit that is not here and cannot be fetched: due.
  echo 3 > "$GH_RUNS/order"
  printf '%040d\n' 0 > "$GH_RUNS/3"
  check "unreachable commit" true "not reachable"

  # 9. gh failing leaves nothing on stdout and a non-zero exit, which the caller reads as due.
  printf '#!/usr/bin/env bash\nexit 1\n' > "$GH"
  local out=
  if out=$("$here/probes-due.sh" dev HEAD 2>/dev/null); then
    fail "a failing gh did not fail the script"
  fi
  [ -z "$out" ] || fail "a failing gh printed '$out'"

  if [ "$status" -eq 0 ]; then
    echo "probes-due: self-test OK"
  fi
  return "$status"
)

case "${1:-}" in
  --self-test) self_test ;;
  --help|-h|"") echo "probes-due: usage: probes-due.sh <branch> <head> | --self-test" >&2; exit 2 ;;
  *) [ $# -eq 2 ] || { echo "probes-due: usage: probes-due.sh <branch> <head> | --self-test" >&2; exit 2; }
     due "$1" "$2" ;;
esac
