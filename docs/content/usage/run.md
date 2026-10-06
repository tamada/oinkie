---
title: "5. All-in-One Execution (`run` command)"
description: "How to extract and compare birthmarks using a single execution command."
date: 2026-06-22
draft: false
weight: 10
---

{{< katex >}}

The **run** command is an all-in-one command designed to perform both **extraction** and **comparison** of software birthmarks in a single transaction. This is ideal for quick checks or automated CI pipelines, saving the intermediate step of writing extracted birthmarks manually to separate folders.

---

## 🏃 Usage

```sh
oinkie run [OPTIONS] [FILES]...
```

### Arguments

* `<FILES>...`  
  Paths to the OIR JSON files (previously generated via the `lift` command) to extract and compare.

### Options

* `-a, --analysis <ANALYSIS>`  
  The combination of birthmark-type, representation, and similarity-algorithm to evaluate.  
  `[default: op-set-jaccard]`  
  *Refer below for a list of common configuration formats.*
* `-A, --aggregator <METHOD>`  
  Specify the method for combining the function similarities of a pair into its single similarity. `[default: hungarian]`  
  *Refer below for a full list of aggregators.*
* `-s, --strategy <STRATEGY>`  
  The pairing strategy to use when comparing files. `[default: all-and-self]`  
  `[possible values: all-and-self, all, self-coverage, adjacent, first-vs-others, last-vs-others]`
  *Refer a full list in [compare](../compare) subcommand.*
* `-d, --dest <DIRECTORY>`  
  Destination directory for output CSV files containing the similarity results. `[default: similarities]`  
  The birthmarks the comparisons were made from are written into its `birthmarks/` subdirectory, one per input, and each pair's CSV names the two it used -- so the directory holds everything [`review`](../review) needs to re-read the comparison.
* `-S, --skip`  
  Reuse what an earlier run left in the destination. A birthmark already in `birthmarks/` is reused if it is of the analysis's birthmark type, and extracted again otherwise. A pair's CSV is reused only if both birthmarks it was computed from were reused; otherwise the pair is compared again, so a CSV never names a birthmark it was not computed from.

---

## 🔬 Specifying Analysis Modes (`--analysis`)

The format for `--analysis` strings is structured as:
```
<element_type>-<structural_representation>-<similarity_algorithm>
```

For instance:
* `op-set-jaccard`: Extract opcode (`op`) as a unique set (`set`) and compare using Jaccard Similarity (`jaccard`).
* `fc-freq-cosine`: Extract function calls (`fc`) as a frequency vector (`freq`) and compare using Cosine Similarity (`cosine`).
* `op-3gram-seq-levenshtein`: Extract opcode 3-grams as a sequence and compare using Levenshtein distance.

### Full List of Supported Configurations

You can use any valid combination of elements, structures, and matching algorithms. Run `oinkie info` or consult the `--help` command for a comprehensive list of all possible analysis parameters on your system. Typical options include:

* `fc-freq-cosine`, `fc-set-dice`, `fc-freq-euclidean`, `fc-set-jaccard`, `fc-seq-levenshtein`, `fc-seq-lcs`, `fc-set-simpson`, `fc-freq-weightedjaccard`
* `op-freq-cosine`, `op-set-dice`, `op-freq-euclidean`, `op-set-jaccard`, `op-seq-levenshtein`, `op-seq-lcs`, `op-set-simpson`, `op-freq-weightedjaccard`
* \\(k\\)-gram variations (`op-1gram`, `op-2gram`, ...) combined with `set`, `seq`, or
  `freq`, and matching algorithms. `oinkie info` and shell completion stop listing
  at \\(k = 8\\), but any \\(k\\) is accepted: `op-12gram-set-jaccard` runs.
