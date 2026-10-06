#!/usr/bin/env python3
"""Checks that every internal link in the repository's Markdown resolves.

Two kinds of link live here and they do not resolve the same way, which is why
this is a script rather than an off-the-shelf checker:

- **Repository links**, in README.md and the READMEs beside the code, point at
  files. `assets/lifters/ghidra/README.md` is a path on disk, and the reader is
  someone browsing the repository.
- **Site links**, in `docs/content/`, point at rendered pages. `../run` from
  `usage/mcp.md` is the URL `/usage/run`, whose file is `usage/run.md`. There is
  no `../run` on disk, and a filesystem checker reports every one of them as
  broken.

External links are not checked here. They fail for reasons that have nothing to
do with this repository -- a site down for ten minutes would fail a push -- and
they rot on their own schedule. `link-rot.yaml` looks at those weekly and opens
an issue.

What this exists for: a directory that moves leaves every link to it pointing
at nothing, and nothing else would notice.
"""

import os
import re
import subprocess
import sys
from pathlib import Path, PurePosixPath

# [text](target), with the target stopping at whitespace or a closing paren.
INLINE = re.compile(r"\[[^\]]*\]\(\s*(<[^>]*>|[^)\s]+)")

# [label]: target, the definition half of a reference-style link. Without this
# a link written `[text][label]` is never checked, which would make the whole
# check optional -- anyone could bypass it by choosing the other syntax.
DEFINITION = re.compile(r"^ {0,3}\[[^\]]+\]:\s*(<[^>]*>|\S+)", re.MULTILINE)

# A fence opens and closes with three or more backticks or tildes.
FENCE = re.compile(r"^\s*(```+|~~~+)")

# A run of backticks: where a code span opens or closes.
TICKS = re.compile(r"`+")

CONTENT = Path("docs/content")


def without_code(text):
    """The text with code regions blanked, keeping every other offset intact.

    Link-shaped text inside a fenced example or a code span is not a link, and
    treating one as a link fails a push over documentation that is correct.
    This repository's READMEs are mostly examples, so it is a question of when
    rather than whether.

    Characters are replaced with spaces rather than removed so that offsets --
    and so the reported line numbers -- still point at the real file.
    """
    lines = text.split("\n")
    inside = False
    kept = []
    for line in lines:
        if FENCE.match(line):
            inside = not inside
            kept.append(" " * len(line))
            continue
        kept.append(" " * len(line) if inside else line)
    return without_spans("\n".join(kept))


def without_spans(text):
    """The text with every code span blanked.

    A span opens with a run of backticks and closes at the next run of the
    same length; a run with none after it is literal text, as in CommonMark.

    Walked once over the runs rather than matched with a backreference, which
    would retry from every later position after an unclosed run and so take
    time growing with the square of the file.
    """
    runs = [(m.start(), m.end()) for m in TICKS.finditer(text)]
    # For each run, the next one of the same length, found from the end.
    closer = [None] * len(runs)
    last = {}
    for i in range(len(runs) - 1, -1, -1):
        length = runs[i][1] - runs[i][0]
        closer[i] = last.get(length)
        last[length] = i
    out = list(text)
    i = 0
    while i < len(runs):
        j = closer[i]
        if j is None:
            i += 1
            continue
        start, end = runs[i][0], runs[j][1]
        out[start:end] = [c if c == "\n" else " " for c in text[start:end]]
        i = j + 1
    return "".join(out)


def tracked_markdown():
    out = subprocess.run(
        ["git", "ls-files", "-z", "*.md"], capture_output=True, text=True, check=True
    ).stdout
    return [Path(p) for p in out.split("\0") if p]


def targets(text):
    """Every internal destination in the text, inline and reference alike."""
    scannable = without_code(text)
    matches = list(INLINE.finditer(scannable)) + list(DEFINITION.finditer(scannable))
    for match in sorted(matches, key=lambda m: m.start()):
        href = match.group(1).strip("<>")
        # Anchors and queries name a place within a page, not another page.
        href = href.split("#", 1)[0].split("?", 1)[0]
        if not href:
            continue
        if re.match(r"^[a-z][a-z0-9+.-]*:", href):  # http:, mailto:, ...
            continue
        yield href, match.start()


def line_of(text, offset):
    return text.count("\n", 0, offset) + 1


def site_candidates(source, href):
    """Where a `docs/content` link points, as files that could serve that URL."""
    relative = source.relative_to(CONTENT)
    if relative.name == "_index.md":
        url_dir = PurePosixPath("/") / relative.parent
    else:
        url_dir = PurePosixPath("/") / relative.parent / relative.stem
    url = href if href.startswith("/") else str(url_dir / href)
    url = os.path.normpath(url).lstrip("/")
    base = CONTENT / url if url else CONTENT
    return [base.with_suffix(".md"), base / "_index.md", base]


def repository_candidates(source, href):
    """Where a repository link points, named the way the reader wrote it.

    Relative to the repository rather than resolved to an absolute path: the
    message is read by someone looking at the repository, and their home
    directory is neither theirs to recognise nor anyone's to print.
    """
    target = (source.parent / href).resolve()
    root = Path.cwd().resolve()
    try:
        return [target.relative_to(root)]
    except ValueError:
        # Above the repository. Kept as written rather than made absolute, so
        # the message says what the link says.
        return [Path(os.path.normpath(source.parent / href))]


def main():
    failures = []
    checked = 0
    for source in tracked_markdown():
        text = source.read_text(encoding="utf-8")
        in_site = CONTENT in source.parents
        for href, offset in targets(text):
            checked += 1
            candidates = (
                site_candidates(source, href)
                if in_site
                else repository_candidates(source, href)
            )
            if not any((Path.cwd() / c).exists() for c in candidates):
                failures.append(
                    "%s:%d: %s does not resolve (%s link; looked for %s)"
                    % (
                        source,
                        line_of(text, offset),
                        href,
                        "site" if in_site else "repository",
                        ", ".join(str(c) for c in candidates),
                    )
                )

    for failure in failures:
        print(failure, file=sys.stderr)
    print("%d internal links checked, %d broken" % (checked, len(failures)))
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
