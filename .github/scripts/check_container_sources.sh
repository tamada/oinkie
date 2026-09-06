#! /bin/bash
#
# Does every path the Containerfiles copy still exist?
#
# Nothing builds the images except a release, so a `COPY` naming a directory
# that has moved is discovered when the release is already tagged. That is
# exactly what happened: #91 moved `lifter/` under `assets/`, both
# Containerfiles went on copying `lifter`, and the break sat on main
# undetected because no push builds an image.
#
# A second's worth of shell catches it. Building the images on every push would
# too, at several minutes of Rust each.
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
