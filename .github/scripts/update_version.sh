#! /bin/sh
#
# Rewrites the version this repository claims about itself.
#
# Each replacement is anchored to what precedes the version, rather than
# matching the previous version string anywhere it appears. It used to be
# `sed "s/${FROM_VERSION}/${TO_VERSION}/g"` over the whole file, which
# rewrites any text that happens to contain the old number: going 0.3.0 to
# 0.4.0 would have turned the NII CRID 1572824500.3.007232 of a cited paper
# into ...500.4.007232, silently, in the released README (#75).
#
# Matching the shape of a version rather than the literal old one also makes
# this idempotent, so it does not matter whether Cargo.toml still holds the
# previous version when it runs.

set -eu

# Every path below is relative to the repository root, and always was -- run
# from anywhere else, this used to fail at the first sed with
# `Cargo.toml: No such file or directory`. Going there rather than saying so
# removes the precondition instead of documenting it.
cd "$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"

# cargo is needed for the lock file below. Said here, because `cargo: not
# found` three-quarters of the way through a release does not explain itself.
command -v cargo > /dev/null || {
    echo "$0: cargo is not on PATH, and Cargo.lock cannot be brought along without it" >&2
    exit 1
}

usage() {
    echo "usage: $0 <version>        e.g. $0 0.4.0" >&2
    echo "  the leading v of a tag name is accepted and ignored" >&2
    exit 1
}

[ $# -eq 1 ] || usage

# The caller is the release workflow, which derives this from the branch name
# `releases/vX.Y.Z`. Checked anyway: an unchecked value reaches the right-hand
# side of a `sed` s/// and could rewrite these files into anything, and this
# runs on a release branch where the result is committed and pushed.
TO_VERSION=$(printf '%s' "$1" | sed -E 's/^v//')
case $TO_VERSION in
    *[!0-9.]* | *..* | .* | *. ) usage ;;
esac
echo "$TO_VERSION" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+$' || usage

V='[0-9]+\.[0-9]+\.[0-9]+'
TMP=

# sed writes beside the file it is rewriting, so a failure part-way through
# must not leave that behind for the commit step to pick up.
cleanup() { [ -z "$TMP" ] || rm -f "$TMP"; }
trap cleanup EXIT

rewrite() {
    file=$1
    shift
    TMP="$file.tmp"
    sed -E "$@" "$file" > "$TMP"
    mv "$TMP" "$file"
    TMP=
}

rewrite Cargo.toml -e "s/^version = \".*\"/version = \"${TO_VERSION}\"/"

for f in README.md docs/content/_index.md; do
    rewrite "$f" \
        -e "s|(badge/Version-)${V}|\1${TO_VERSION}|g" \
        -e "s|(releases/tag/v)${V}|\1${TO_VERSION}|g" \
        -e "s|(oinkie:)${V}|\1${TO_VERSION}|g"
done

# Cargo.lock names this package's version too, and the container builds with
# `cargo build --locked`, which refuses to reconcile a lock that disagrees with
# the manifest -- it exits 101 rather than fixing it. Leaving the lock behind
# therefore breaks the release it is preparing.
#
# `--workspace` is what keeps this to a single line: it limits the update to
# this package's own entry, and cargo says as much while it runs -- "60
# unchanged dependencies". `cargo generate-lockfile` would also sync the lock
# and is the wrong tool, because it rebuilds the whole thing and dependency
# resolutions can move with it.
#
# Not `--offline`. It was here, on the reasoning that a release runner should
# resolve nothing from the network, and it made the bump fail on the only
# machine that matters: a fresh runner has no crates.io index, and `--offline`
# forbids fetching one, so resolution cannot succeed at all (#102). It passed
# every local test because a developer's registry is already warm.
cargo update --workspace
