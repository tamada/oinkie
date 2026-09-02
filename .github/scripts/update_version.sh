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
