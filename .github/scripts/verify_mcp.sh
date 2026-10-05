#! /bin/bash
#
# Does this oinkie actually serve MCP?
#
# `publish.yaml` builds the release binaries with `--features mcp`, and nothing
# looked at the result. A flag lost in an edit, or a feature that stops
# resolving, would ship an oinkie with no `mcp` subcommand and the first anyone
# would know of it is a client failing to start.
#
# Speaks the protocol rather than grepping `--help`: a subcommand that exists
# and does not work would pass that.
#
# Usage:
#   .github/scripts/verify_mcp.sh <path to oinkie>
#   .github/scripts/verify_mcp.sh --image <tag>
#   .github/scripts/verify_mcp.sh --image <tag> --no-args
#
# The image form asks the same question of a container, which is how the
# documentation tells people to run this -- and `docker run -i` without a TTY
# is itself part of what is being checked.
#
# --no-args passes no command at all, so the image's own CMD has to be the
# right one. That is the whole of what `mcp-image` adds over `light-image`, and
# the only other place it is read is a client's startup.

set -euo pipefail

if [ "${1:-}" = "--image" ]; then
    readonly IMAGE="${2:?usage: $0 --image <tag> [--no-args]}"
    if [ "${3:-}" = "--no-args" ]; then
        readonly SUBJECT="image $IMAGE with no arguments"
        run() { docker run -i --rm "$IMAGE"; }
    else
        readonly SUBJECT="image $IMAGE"
        run() { docker run -i --rm "$IMAGE" mcp; }
    fi
else
    readonly SUBJECT="${1:?usage: $0 <path to oinkie> | --image <tag>}"
    run() { "$1" mcp; }
fi

readonly EXPECTED="oinkie_compare oinkie_extract oinkie_info oinkie_review oinkie_run oinkie_stats"

session() {
    printf '%s\n' \
        '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2026-07-28","capabilities":{},"clientInfo":{"name":"verify_mcp","version":"0"}}}' \
        '{"jsonrpc":"2.0","method":"notifications/initialized"}' \
        '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}'
}

out=$(mktemp)
err=$(mktemp)
trap 'rm -f "$out" "$err"' EXIT

session | run "$@" > "$out" 2> "$err" || {
    echo "$0: $SUBJECT did not serve a session" >&2
    sed 's/^/  /' "$err" >&2
    exit 1
}

# stdout is the JSON-RPC channel; anything else on it corrupts a session.
while IFS= read -r line; do
    printf '%s' "$line" | jq -e . > /dev/null 2>&1 || {
        echo "$0: stdout carried something that is not JSON:" >&2
        echo "  $line" >&2
        exit 1
    }
done < "$out"

got=$(jq -r 'select(.id == 2) | .result.tools[].name' "$out" | sort | tr '\n' ' ')
got="${got% }"

if [ "$got" != "$EXPECTED" ]; then
    echo "$0: $SUBJECT does not serve the expected tools" >&2
    echo "  got:  ${got:-<none>}" >&2
    echo "  want: $EXPECTED" >&2
    exit 1
fi

echo "ok: $SUBJECT serves $got"
