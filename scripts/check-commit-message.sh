#!/usr/bin/env bash
# The Conventional Commits gate. The version and the changelog are derived from the log (DESIGN.md
# § Release), so a subject the version count cannot read is a wrong version later, not a style nit.
#
#   check-commit-message.sh <file>          the commit-msg hook: the message git is about to record
#   check-commit-message.sh --range <A..B>  every non-merge subject in the range (a PR's commits)
#   check-commit-message.sh --subject <S>   one subject (a PR's title)
#   check-commit-message.sh --self-test     the fixtures: every good subject passes, every bad one fails
#
# A subject is `type(scope)!: subject`: a type from the baseline's list, an optional lowercase
# scope, an optional `!` for a breaking change, then a subject that starts lowercase (an imperative
# verb, never a capitalised noun phrase) and carries no trailing period. `Merge …`, `Revert "…"`
# and autosquash's `fixup!`/`squash!` prefixes pass, because git writes them.
set -euo pipefail

check_subject() {
  local s=$1
  case "$s" in
    'Merge '*|'Revert "'*|'fixup! '*|'squash! '*) return 0 ;;
  esac
  local types='feat|fix|docs|test|bench|ci|chore|build|refactor|perf|revert'
  # The pattern lives in a variable: bash 3.2 (macOS) mis-parses an inline `=~` pattern with
  # parentheses, and a quoted one would match literally. The tail is checked apart from the regex
  # so that `.` never has to match a multibyte character under a C locale.
  local re="^(${types})(\\([a-z0-9][a-z0-9/_.,-]*\\))?!?: [a-z0-9]"
  [[ $s =~ $re ]] || return 1
  [[ $s != *. ]] || return 1
  return 0
}

fail() {
  echo "check-commit-message: FAIL: $1" >&2
  echo "  want: type(scope)!: subject — types feat fix docs test bench ci chore build refactor perf revert; a lowercase subject with no trailing period" >&2
  exit 1
}

self_test() {
  local good=(
    'feat(wide): add the 34-limb sum'
    'fix: refuse a non-canonical byte'
    'chore(meshwork): close the task on the green job'
    'feat!: drop the f32 path'
    'feat(exact)!: widen the compact word'
    'docs(release): record the ruling: the version counted from the first commit'
    'build(deps): bump actions/checkout from 4 to 5'
    'refactor(exact/heap): split the block'
    'refactor(wide,fixed): move the operator impls into ops submodules'
    'bench: add the crossover rows'
    'perf(simd): fold two limbs per lane'
    'ci(release): run ci on dev'
    'test(laws): cross the cap edge'
    'chore: release v0.1.0'
    'revert: feat(wide): add the 34-limb sum'
    'Revert "feat(wide): add the 34-limb sum"'
    'Merge pull request #12 from jbrjake/dev'
    'fixup! feat(wide): add the 34-limb sum'
    'squash! fix: refuse a non-canonical byte'
  )
  local bad=(
    'Add the sum'
    'feat add the sum'
    'feat: Add the sum'
    'feat:add the sum'
    'feature: add the sum'
    'feat(Wide): add the sum'
    'feat(wide) add the sum'
    'feat: add the sum.'
    'feat: '
    'feat:'
    ''
    'WIP'
    'feat(): add the sum'
    'feat (wide): add the sum'
    'FEAT: add the sum'
  )
  local status=0 s
  for s in "${good[@]}"; do
    if ! check_subject "$s"; then
      echo "self-test: good subject refused: '$s'" >&2
      status=1
    fi
  done
  for s in "${bad[@]}"; do
    if check_subject "$s"; then
      echo "self-test: bad subject accepted: '$s'" >&2
      status=1
    fi
  done
  if [ "$status" -eq 0 ]; then
    echo "check-commit-message: self-test OK (${#good[@]} good, ${#bad[@]} bad)"
  fi
  return "$status"
}

case "${1:-}" in
  --self-test)
    self_test
    ;;
  --subject)
    [ "$#" -eq 2 ] || fail "--subject takes one argument"
    check_subject "$2" || fail "'$2'"
    ;;
  --range)
    [ "$#" -eq 2 ] || fail "--range takes one argument, A..B"
    status=0
    while IFS= read -r line; do
      hash=${line%% *}
      subject=${line#* }
      if ! check_subject "$subject"; then
        echo "check-commit-message: FAIL $hash: '$subject'" >&2
        status=1
      fi
    done < <(git log --no-merges --format='%h %s' "$2")
    [ "$status" -eq 0 ] || fail "a commit in $2 is not Conventional"
    echo "check-commit-message: OK ($2)"
    ;;
  "")
    fail "no message file, --range, --subject or --self-test given"
    ;;
  *)
    [ -f "$1" ] || fail "no such message file: $1"
    # The first line git will keep: comments and leading blank lines are stripped at commit.
    subject=$(grep -v '^#' "$1" | sed -n '/./{p;q;}')
    check_subject "$subject" || fail "'$subject'"
    ;;
esac
