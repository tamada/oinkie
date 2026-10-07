#!/bin/sh
# The library does not depend on the command line.
#
# `cargo add oinkie --no-default-features` builds the library alone, and none
# of the crates below belongs in that build: they are the command line's
# argument parser, its output naming, its logger and progress bar, its MCP
# server, and its completion generator. Rust already stops the
# library from `use`-ing the binary's modules; what it does not stop is one of
# these creeping back in as a dependency of the library itself.
#
# Two runs, because a check that finds nothing has to be shown capable of
# finding something. With default features every one of these must appear --
# `cli` pulls them in, and `mcp` and `gencomp` add the rest -- so if the
# pattern ever stops matching, that half fails instead of this script
# reporting a clean library it never looked at.
set -eu

readonly CLI_ONLY="clap clap_complete env_logger indicatif rmcp sha2 tokio"

# Prints the crates of $CLI_ONLY that `cargo tree` lists, one per line.
# `--prefix none` puts each crate's name first on its line, so the name is
# compared whole rather than searched for inside other names.
#
# `--target all`, because this runs on one platform and the policy holds on
# every one: without it cargo resolves for the host alone, and a dependency
# declared only for Windows or macOS would pass here while every user on that
# platform pulled it in.
listed() {
    tree=$(cargo tree -p oinkie -e normal --target all --prefix none "$@") || {
        echo "$0: cargo tree $* failed" >&2
        exit 2
    }
    for crate in $CLI_ONLY; do
        if printf '%s\n' "$tree" | awk -v c="$crate" '$1 == c { found = 1 } END { exit !found }'; then
            echo "$crate"
        fi
    done
}

found=$(listed --no-default-features)
if [ -n "$found" ]; then
    echo "$0: the library alone depends on the command line's crates:" >&2
    printf '%s\n' "$found" | sed 's/^/  /' >&2
    exit 1
fi

# The self-check: everything must be listed once the features that need it
# are on.
missing=""
present=$(listed --features mcp,gencomp)
for crate in $CLI_ONLY; do
    if ! printf '%s\n' "$present" | grep -qx "$crate"; then
        missing="$missing $crate"
    fi
done
if [ -n "$missing" ]; then
    echo "$0: not found even with the command line's features on:$missing" >&2
    echo "  -- the check above would have reported a clean library without looking" >&2
    exit 1
fi

echo "ok: the library alone depends on none of: $CLI_ONLY"
