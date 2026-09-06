---
title: "Serving to an Agent (`mcp` command)"
description: "How to serve oinkie over the Model Context Protocol so an agent can drive the birthmarking pipeline."
date: 2026-09-05
draft: false
weight: 70
---

{{< katex >}}

The **mcp** command serves the birthmark pipeline over the
[Model Context Protocol](https://modelcontextprotocol.io), on stdin and stdout.
An agent can then ask whether two programs are alike without a person chaining
`extract` and `compare` by hand.

The subcommand is behind a cargo feature. The released binaries and both
container images carry it; building from source needs it asked for:

```sh
cargo build --release --features mcp
```

---

## ⚙️ Configuring a client

`.mcp.json`, in a project directory:

```json
{
  "mcpServers": {
    "oinkie": {
      "command": "oinkie",
      "args": ["mcp", "--root", "."]
    }
  }
}
```

For a container, use the **light** image — the tools here never lift, so the
Ghidra in the full image is a gigabyte of dead weight:

```json
{
  "mcpServers": {
    "oinkie": {
      "command": "docker",
      "args": [
        "run", "-i", "--rm",
        "-v", "/path/to/your/work:/work",
        "quay.io/tama5/oinkie:light",
        "mcp", "--root", "/work"
      ]
    }
  }
}
```

`-i` keeps stdin open, which is the whole transport. Do **not** add `-t`: a TTY
rewrites what passes through it and breaks the JSON-RPC framing.

---

## 🧰 The tools

| tool | what it is for |
| --- | --- |
| `oinkie_info` | the vocabulary: birthmark types, algorithms, analysis names, strategies, aggregators |
| `oinkie_run` | lifted programs in, a similarity per pair out — the whole question in one call |
| `oinkie_extract` | write birthmarks, which `oinkie_compare` takes |
| `oinkie_compare` | score birthmarks, to try another algorithm without re-reading the programs |
| `oinkie_reaggregate` | recompute scores in a directory under a different aggregator |

An agent should ask `oinkie_info` first. The names are precise, and its lists
are generated from the same code that parses them, so they cannot drift from
what is accepted.

A destination directory is optional for `oinkie_run` and `oinkie_compare`, and
is written in the shape [`run -d`](../run) produces — which is what lets a
directory one of them wrote be handed straight to `oinkie_reaggregate`.

---

## 📁 Confining what it touches

Every path the tools are given, input and output alike, has to resolve inside a
`--root`. Repeat the option for more than one; it defaults to the working
directory.

The paths reaching these tools are written by a language model rather than by
you, and a model with the wrong directory in mind will say so with a path
rather than a question. `--root` is what keeps that a refusal instead of a file
written somewhere surprising.

---

## 🚫 Lifting is not offered

There is no `oinkie_lift`. Run [`oinkie lift`](../lift) yourself and point the
tools at what it produced.

Three reasons, any one of which would be enough:

- it runs a whole decompiler process per binary, for minutes at a time, which no
  client's timeout accommodates;
- `--script` takes a replacement lifting script, which is arbitrary code running
  inside Ghidra — not something to hand to a model; and
- `analyzeHeadless` exits successfully even when its script throws, so failure
  has to be inferred rather than reported.

---

## 🩺 When something is wrong

The server speaks JSON-RPC on stdout and nothing else; logs and errors go to
stderr. If a client reports a protocol error, check whether something else in
the environment is writing to stdout.

A refused name comes back in the library's own words, naming the spelling that
was meant:

```
op-seq-euclidean: euclidean operates on frequency vectors; use op-freq-euclidean.
```

A refused path names the directories that are allowed. A request for more than
500 pairs is refused before anything is read — `all-and-self` over \\(n\\) files
is \\(n(n+1)/2\\) pairs — and `max_pairs` raises that deliberately.
