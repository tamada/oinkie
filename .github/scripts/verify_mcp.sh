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
# Usage: .github/scripts/verify_mcp.sh <path to oinkie>

set -euo pipefail

readonly BIN="${1:?usage: $0 <path to oinkie>}"
readonly EXPECTED="oinkie_compare oinkie_extract oinkie_info oinkie_reaggregate oinkie_run"

session() {
    printf '%s\n' \
        '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2026-07-28","capabilities":{},"clientInfo":{"name":"verify_mcp","version":"0"}}}' \
        '{"jsonrpc":"2.0","method":"notifications/initialized"}' \
        '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}'
}

out=$(mktemp)
err=$(mktemp)
trap 'rm -f "$out" "$err"' EXIT

session | "$BIN" mcp > "$out" 2> "$err" || {
    echo "$0: $BIN mcp exited non-zero" >&2
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
    echo "$0: $BIN does not serve the expected tools" >&2
    echo "  got:  ${got:-<none>}" >&2
    echo "  want: $EXPECTED" >&2
    exit 1
fi

echo "ok: $BIN serves $got"
