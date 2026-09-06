#! /bin/bash
#
# Does every path the Containerfiles copy still exist?
#
# This is the shallow half of a pair. CI builds the *light* image on every
# push, which covers everything about its build context and rather more; the
# *full* image is not built until a release, because what it adds is a 400 MB
# Ghidra download whose URL and checksum change only when someone edits them.
#
# So this exists for the full image, and costs a second. Without it, a `COPY`
# there naming a directory that has moved is discovered when the release is
# already tagged -- which is what happened: #91 moved `lifter/` under
# `assets/`, both Containerfiles went on copying `lifter`, and nothing looked.
#
# Usage: .github/scripts/check_container_sources.sh

set -euo pipefail

status=0

for containerfile in containers/*/Containerfile; do
    # `COPY --from=…` copies out of an earlier stage rather than the build
    # context, so its source is not a path in this repository.
    while read -r src; do
        if [ ! -e "$src" ]; then
            echo "$containerfile: COPY $src -- no such path in the repository" >&2
            status=1
        fi
    done < <(grep -E '^COPY ' "$containerfile" | grep -v -- '--from=' | awk '{print $2}')
done

[ "$status" -eq 0 ] && echo "ok: every COPY source exists"
exit "$status"
