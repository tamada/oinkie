#!/bin/sh
# The library's public API is the one committed in .github/public-api.txt.
#
#   public_api.sh             compare, and fail on any difference
#   public_api.sh --update    rewrite the file from the code
#   public_api.sh --install   install what the other two need
#
# A new `pub`, a removed one or a changed signature then shows up as a line in
# the pull request's diff, where a reviewer sees it, instead of reaching
# crates.io unnoticed (#133). Updating the file is the deliberate act of
# saying the change is meant.
#
# cargo-public-api reads rustdoc's JSON output, which only nightly produces,
# and each version of it reads one version of that format. Both are pinned
# here, together, and only here: an unpinned nightly changes the format under
# a pinned tool and fails a pull request that changed nothing. Raise them
# together, and rerun --update in case the rendering changed.
set -eu

readonly NIGHTLY=nightly-2026-10-03
readonly TOOL_VERSION=0.52.0
readonly FILE=.github/public-api.txt

case "${1:-}" in
--install)
    rustup toolchain install "$NIGHTLY" --profile minimal
    cargo install cargo-public-api --version "$TOOL_VERSION" --locked
    exit 0
    ;;
--update | "") ;;
*)
    echo "usage: $0 [--update | --install]" >&2
    exit 2
    ;;
esac

# `+$NIGHTLY` on cargo is what pins it: called from a stable cargo,
# cargo-public-api switches to whatever toolchain is named `nightly`.
#
# -ss leaves out blanket and auto-trait impls (`impl<T> From<T> for T`,
# `impl Send for ...`), which change with the compiler rather than with this
# crate. --no-default-features because the library has no API behind a
# feature, and building without the command line's dependencies is faster.
now=$(cargo +"$NIGHTLY" public-api -ss --no-default-features --color never) || {
    echo "$0: cargo-public-api failed; is it installed? $0 --install" >&2
    exit 2
}

if [ "${1:-}" = "--update" ]; then
    printf '%s\n' "$now" >"$FILE"
    echo "updated $FILE"
    exit 0
fi

# diff is the last command of the pipeline, so its status is the pipeline's.
if ! printf '%s\n' "$now" | diff -u "$FILE" -; then
    echo >&2
    echo "$0: the public API is not the one in $FILE." >&2
    echo "  If the change is meant, run $0 --update and commit the file." >&2
    exit 1
fi
echo "ok: the public API is the one in $FILE"
