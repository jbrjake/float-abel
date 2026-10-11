#!/usr/bin/env bash
# Rust files target 500 lines (warn) with a hard ceiling of 750 (fail). Takes the files to check as
# arguments; with none, checks every tracked and untracked .rs file.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

files=()
if [ "$#" -gt 0 ]; then
  files=("$@")
else
  while IFS= read -r f; do files+=("$f"); done < <(git ls-files --cached --others --exclude-standard -- '*.rs')
fi

status=0
for f in ${files[@]+"${files[@]}"}; do
  [ -f "$f" ] || continue
  case "$f" in *.rs) ;; *) continue ;; esac
  lines=$(wc -l <"$f" | tr -d ' ')
  if [ "$lines" -gt 750 ]; then
    echo "check-file-length: FAIL $f is $lines lines (hard ceiling 750: split it)" >&2
    status=1
  elif [ "$lines" -gt 500 ]; then
    echo "check-file-length: warn $f is $lines lines (target 500)" >&2
  fi
done
exit "$status"
