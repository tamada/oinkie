# oinkie over MCP

`oinkie mcp` serves the birthmark pipeline over the [Model Context Protocol](https://modelcontextprotocol.io),
on stdin and stdout, so an agent can ask whether two programs are alike without
a person chaining `extract` and `compare` by hand.

## 🧱 Building it

The subcommand is behind a cargo feature, so a build that does not want a
server does not carry its dependencies:

```sh
cargo build --release --features mcp
```

The released binaries and both container images are built with it, so this is
only needed when building from source.

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

`claude_desktop_config.json` is the same shape, with absolute paths, since the
desktop app does not start in your project:

```json
{
  "mcpServers": {
    "oinkie": {
      "command": "/usr/local/bin/oinkie",
      "args": ["mcp", "--root", "/path/to/your/work"]
    }
  }
}
```

### 🐳 In a container

Both images end in `ENTRYPOINT ["oinkie"]`, so either is already an MCP server.
Use **`light`**: the tools here never lift, so `full`'s Ghidra and JDK are a
gigabyte of dead weight on this path.

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

`-i` keeps stdin open, which is the whole transport. **Do not add `-t`** — a
TTY rewrites what passes through it and breaks the JSON-RPC framing.

## 🧰 The tools

| tool | what it is for |
| --- | --- |
| `oinkie_info` | the vocabulary: birthmark types, algorithms, analysis names, strategies, aggregators |
| `oinkie_run` | lifted programs in, a similarity per pair out — the whole question in one call |
| `oinkie_extract` | write birthmarks, which `oinkie_compare` takes |
| `oinkie_compare` | score birthmarks, to try another algorithm without re-reading the programs |
| `oinkie_reaggregate` | recompute scores in a directory under a different aggregator |

Ask `oinkie_info` first. The names are precise — `op-3gram-freq-cosine` is a
real one and `op-3gram-cosine` is not — and its lists are generated from the
same code that parses them, so they cannot drift from what is accepted.

A destination directory is optional for `oinkie_run` and `oinkie_compare`, and
is written in the shape `oinkie run -d` produces. That is what lets a directory
one of them wrote be handed straight to `oinkie_reaggregate`.

## 📁 `--root`

Every path the tools are given, input and output alike, has to resolve inside a
`--root`. Repeat the option for more than one; it defaults to the working
directory.

The paths reaching these tools are written by a language model rather than by
you, and a model that has the wrong directory in mind will say so with a path
rather than a question. `--root` is what keeps that a refusal instead of a file
written somewhere surprising.

## 🚫 Lifting is not here

There is no `oinkie_lift`, and there is not going to be one. Run
[`oinkie lift`](../../assets/lifters/ghidra/README.md) yourself and point the
tools at what it produced — on a host with Ghidra installed, or in the `full`
image, which bundles it.

It is the one step whose shape does not fit a tool call. It starts a whole
decompiler process per binary, and how long that takes is set by the binary
rather than by the request: a hello-world lifts in a few seconds, and something
worth comparing takes considerably longer, with the client blocked on a single
call for the whole of it. `--script` is the other half — a replacement lifting
script is arbitrary code inside Ghidra, which is a reasonable thing to put in a
person's hands and not in a model's.

None of that says lifting is dangerous. A model with a shell can run `oinkie
lift` like anyone else, and it is welcome to. The line here is about which step
belongs inside a request and response, not about what may be run.

## 🩺 When something is wrong

The server speaks JSON-RPC on stdout and nothing else; logs and errors go to
stderr. If a client reports a protocol error, the first thing to check is
whether something else in the environment is writing to stdout.

A refused name comes back with the library's own words, which name the spelling
that was meant:

```
op-seq-euclidean: euclidean operates on frequency vectors; use op-freq-euclidean.
```

A refused path names the directories that are allowed. A request for more than
500 pairs is refused before anything is read — `all-and-self` over *n* files is
\(n(n+1)/2\) pairs — and `max_pairs` raises that deliberately.
