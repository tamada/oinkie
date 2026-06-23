---
title: "🏃 Usage"
description: "Detailed guide on using oinkie CLI for software birthmarking."
date: 2026-06-22
draft: false
---

{{< katex >}}

The `oinkie` command-line utility provides all the subcommands required to perform the entire software birthmarking process—from lifting binary files, extracting birthmarks, to comparing similarities.

---

## 🚀 General Help and Subcommands

To view the basic usage and options of the `oinkie` CLI:

```sh
Birthmarking toolkit for Ghidra P-Code

Usage: oinkie [OPTIONS] <COMMAND>

Commands:
  info         Display information about the application
  lift         Lift binary files to P-code JSON files using a specified lifter
  extract      Extract birthmarks from a lifted binary file (JSON format)
  compare      Compare birthmarks and output the similarity score
  reaggregate  Reaggregate the element-wise similarity scores and recalculate the birthmark-wise similarity score
  run          Extract birthmarks and compare them in one command
  help         Print this message or the help of the given subcommand(s)

Options:
  -l, --level <LEVEL>  Log level for the application [default: warn]
                       [possible values: error, warn, info, debug, trace, off]
  -h, --help           Print help
  -V, --version        Print version
```

---

## 🏃 Steps in detail

The toolkit's operations are divided into distinct stages. You can execute them individually to examine results at each stage, or run them all at once with the `run` command:

* **[Displaying Application Info](info)**  
  Query general details, supported birthmark models, and similarity algorithms.

1. **[Lifting Binaries to Pcode (OIR)](lift)**  
   Convert your raw executable or compiled binary files into the Oinkie Intermediate Representation (OIR) JSON format using Ghidra.
   
2. **[Extracting Birthmarks](extract)**  
   Analyze the generated OIR JSON files to extract specific birthmarks based on opcodes, function calls, or \\(k\\)-grams.
   
3. **[Comparing Birthmarks](compare)**  
   Compare extracted birthmarks between pairs of files using chosen similarity algorithms and matching heuristics.
   
4. **[Reaggregating Scores](reaggregate)**  
   Recalculate program-wide similarity scores from saved element-wise similarity scores.
   
5. **[All-in-One Execution (Run)](run)**  
   Execute extraction and comparison together in a single command.
