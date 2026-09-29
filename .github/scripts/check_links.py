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

What this exists for: `cli/mcp/README.md` pointed at `lifter/README.md` for the
whole of v0.5.0, after #91 moved that directory, and nothing noticed.
"""

import os
import re
import subprocess
import sys
from pathlib import Path, PurePosixPath

# [text](target), with the target stopping at whitespace or a closing paren.
LINK = re.compile(r"\[[^\]]*\]\(\s*(<[^>]*>|[^)\s]+)")

CONTENT = Path("docs/content")


def tracked_markdown():
    out = subprocess.run(
        ["git", "ls-files", "-z", "*.md"], capture_output=True, text=True, check=True
    ).stdout
    return [Path(p) for p in out.split("\0") if p]


def targets(text):
    for match in LINK.finditer(text):
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
