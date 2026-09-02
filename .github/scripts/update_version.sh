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

TO_VERSION=$1
V='[0-9]+\.[0-9]+\.[0-9]+'

replace_in() {
    file=$1
    sed -E \
        -e "s|(badge/Version-)${V}|\1${TO_VERSION}|g" \
        -e "s|(releases/tag/v)${V}|\1${TO_VERSION}|g" \
        -e "s|(oinkie:)${V}|\1${TO_VERSION}|g" \
        "$file" > "$file.tmp"
    mv "$file.tmp" "$file"
}

sed -E "s/^version = \".*\"/version = \"${TO_VERSION}\"/" Cargo.toml > Cargo.toml.tmp
mv Cargo.toml.tmp Cargo.toml

replace_in README.md
replace_in docs/content/_index.md
