---
title: "🏃 Usage"
description: "Detailed guide on using oinkie CLI for software birthmarking."
date: 2026-06-22
draft: false
---

{{< katex >}}

The `oinkie` command-line utility provides all the subcommands required to perform the entire software birthmarking process—from lifting binary files, extracting birthmarks, to comparing similarities.

The terms these pages use — birthmark, function, element, similarity and the rest — are each defined once in the [glossary](../glossary).

---

## 🚀 General Help and Subcommands

To view the basic usage and options of the `oinkie` CLI:

```sh
Detects software theft by comparing birthmarks extracted from binaries

Usage: oinkie [OPTIONS] <COMMAND>

Commands:
  info     Display information about the application
  lift     Lift binary files to JSON files of an intermediate representation, using a specified
           lifter
  extract  Extract birthmarks from a lifted binary file (JSON format)
  compare  Compare birthmarks and output the similarity score
  review   Re-read a finished comparison: recompute the similarity of each pair from the stored
           function similarities
  stats    Summarise a set of birthmarks: how many of each type, how many functions each holds, and
           how long each function's birthmark is
  run      Extract birthmarks and compare them in one command
  mcp      Serve oinkie over the Model Context Protocol, on stdin and stdout
  help     Print this message or the help of the given subcommand(s)

Options:
  -l, --level <LEVEL>  Log level for the application [default: warn] [possible values: error, warn,
                       info, debug, trace, off]
      --no-progress    Draw no progress bars, as when stderr is kept in a log or a CI output
  -h, --help           Print help
  -V, --version        Print version
```

---

## 🏃 Steps in detail

The toolkit's operations are divided into distinct stages. You can execute them individually to examine results at each stage, or run them all at once with the `run` command:

* **[Displaying Application Info](info)**  
  Query general details, supported birthmark models, and similarity algorithms.

* **[Summarising Birthmarks](stats)**  
  Count the birthmarks of each type, the functions each holds, and how long each function's birthmark is.

* **[Serving to an Agent](mcp)**  
  Expose the steps below over the Model Context Protocol, so an agent can run them.

1. **[Lifting Binaries to Pcode (OIR)](lift)**  
   Convert your raw executable or compiled binary files into the Oinkie Intermediate Representation (OIR) JSON format using Ghidra.
   
2. **[Extracting Birthmarks](extract)**  
   Analyze the generated OIR JSON files to extract specific birthmarks based on opcodes, function calls, or \\(k\\)-grams.
   
3. **[Comparing Birthmarks](compare)**  
   Compare extracted birthmarks between pairs of files using chosen similarity algorithms and matching heuristics.
   
4. **[Reviewing Scores](review)**  
   Recalculate the similarity of each pair from saved function similarities.
   
5. **[All-in-One Execution (Run)](run)**  
   Execute extraction and comparison together in a single command.

---

## ⚙️ Threads, memory and time

Comparing is where the resources go, and how much depends on the pairs:

* **Memory.** Comparing a pair holds one number for every pair of their functions, 8 bytes each: two birthmarks of 5,000 functions take 200 MB, two of 50,000 take 20 GB. Several pairs are compared at once, so the largest pairs and the number of threads together set the peak.
* **Time.** It grows with the same count of function pairs, and with the algorithm: `lcs` and `levenshtein` work through both functions' sequences for every pair of functions, the set and frequency algorithms only through what the two hold. On a 10-core machine, two birthmarks of 5,000 functions compare in a few seconds under `jaccard` and in under a minute under `lcs`. The `hungarian` aggregator adds its own share on large pairs; `topn:N` adds almost none.
* **Threads.** `compare`, `run`, `review`, `extract`, `stats` and `mcp` compute on one thread per core by default. `--threads N` bounds all of it to \\(N\\) threads -- the pairs, the files, and the rows of each pair's matrix share them -- and lowers memory with it. Nothing else sets the number; an environment variable such as `RAYON_NUM_THREADS` has no effect.
* **`lift --jobs` is a different knob.** It counts decompiler processes, each a whole Ghidra, Binary Ninja or IDA run, and is bounded by their memory rather than by cores. Lifting is serial unless `--jobs` says otherwise; see [lift](lift).

