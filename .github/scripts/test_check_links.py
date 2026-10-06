#!/usr/bin/env python3
"""What check_links.py must and must not see.

Run by `python3 .github/scripts/test_check_links.py`, and on every push beside
the check itself. It exists because the first version of the checker was a
regex over the raw text, which both missed reference-style links -- making the
whole check optional, since anyone could bypass it by choosing that syntax --
and read link-shaped text inside code examples as real links, which fails a
push over documentation that is correct.

Neither had happened yet. The second is the one that would have: this
repository's READMEs are mostly fenced examples.
"""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from check_links import targets  # noqa: E402


def found(text):
    return [href for href, _ in targets(text)]


CASES = [
    ("an inline link is seen", "See [the guide](guide.md).", ["guide.md"]),
    (
        "a reference definition is seen",
        "See [the guide][setup].\n\n[setup]: setup.md\n",
        ["setup.md"],
    ),
    (
        "a fenced example is not a link",
        "Before.\n\n```markdown\n[text](missing.md)\n```\n\nAfter.\n",
        [],
    ),
    (
        "a tilde fence is not a link either",
        "~~~\n[text](missing.md)\n~~~\n",
        [],
    ),
    (
        "a code span is not a link",
        "Write `[text](missing.md)` in your README.",
        [],
    ),
    (
        "a fence does not swallow what follows it",
        "```\n[a](in-fence.md)\n```\n\n[b](after-fence.md)\n",
        ["after-fence.md"],
    ),
    ("an external link is not ours", "[site](https://example.com/x.md)", []),
    ("an anchor names a place in this page", "[top](#heading)", []),
    (
        "a span closes only at a run of its own length",
        "``a ` [in](in-span.md) b`` [out](after-span.md)",
        ["after-span.md"],
    ),
    (
        "an unclosed run is text, not the start of a span",
        "a stray ` then [b](after-stray.md)",
        ["after-stray.md"],
    ),
    (
        "a span may run over a line ending",
        "`a\n[in](in-span.md)` [out](after-span.md)",
        ["after-span.md"],
    ),
    (
        "an anchor on a file still names the file",
        "[a section](other.md#heading)",
        ["other.md"],
    ),
]


def main():
    failures = 0
    for name, text, expected in CASES:
        actual = found(text)
        if actual != expected:
            print("FAIL %s: expected %r, got %r" % (name, expected, actual))
            failures += 1
        else:
            print("ok   %s" % name)
    print("%d cases, %d failed" % (len(CASES), failures))
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
