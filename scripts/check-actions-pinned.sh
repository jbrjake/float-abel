#!/usr/bin/env bash
# Every `uses:` in a workflow names a full commit hash, never a tag or a branch, because a tag can
# be moved to code nobody reviewed (CLAUDE.md § Standing decisions). The tag the hash stood for
# goes in a trailing comment, which this check does not read. Takes the workflow files to check as
# arguments; with none, checks every tracked and untracked file under .github/workflows/.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

files=()
if [ "$#" -gt 0 ]; then
  files=("$@")
else
  while IFS= read -r f; do files+=("$f"); done < <(git ls-files --cached --others --exclude-standard -- '.github/workflows/*.yml' '.github/workflows/*.yaml')
fi

status=0
for f in ${files[@]+"${files[@]}"}; do
  [ -f "$f" ] || continue
  case "$f" in .github/workflows/*.yml|.github/workflows/*.yaml) ;; *) continue ;; esac
  # A `uses:` whose ref is anything but forty hex digits. A local action (`./path`) carries no ref.
  if bad=$(grep -nE '^\s*(-\s*)?uses:\s*[^./ ][^ ]*@' "$f" | grep -vE '@[0-9a-f]{40}(\s|$)'); then
    echo "check-actions-pinned: FAIL $f names an action by a tag or branch:" >&2
    printf '%s\n' "$bad" >&2
    status=1
  fi
done
exit "$status"
