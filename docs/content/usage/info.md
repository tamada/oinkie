---
title: "Application Information (`info` command)"
description: "How to display application information and supported configurations."
date: 2026-06-22
draft: false
weight: 60
---

The **info** command displays general details about the **oinkie** application, including the version, author, and all currently supported birthmark configurations and comparison algorithms.

---

## 🏃 Usage

```sh
oinkie info [OPTIONS]
```

### Options
* `-h, --help`  
  Print help information.

---

## 📊 Example Output

Running the `info` command provides detailed listings of supported formats and algorithms in your current environment:

```text
oinkie v0.2.0
Detecting software theft, the birthmark toolkit for Ghidra Pcode, LLVM IR/BC, and Binary Ninja.

Supported Birthmark Types:
  - fc-seq (Function Calls Sequence)
  - fc-freq (Function Calls Frequency)
  - fc-set (Function Calls Set)
  - op-seq (Opcode Sequence)
  - op-freq (Opcode Frequency)
  - op-set (Opcode Set)
  - op-1gram-seq, op-1gram-freq, op-1gram-set (Opcode 1-gram variants)
  - op-2gram-seq, op-2gram-freq, op-2gram-set (Opcode 2-gram variants)
  - op-3gram-seq, op-3gram-freq, op-3gram-set (Opcode 3-gram variants)
  - op-4gram-seq, op-4gram-freq, op-4gram-set (Opcode 4-gram variants)
  - op-5gram-seq, op-5gram-freq, op-5gram-set (Opcode 5-gram variants)
  - op-6gram-seq, op-6gram-freq, op-6gram-set (Opcode 6-gram variants)
  - op-7gram-seq, op-7gram-freq, op-7gram-set (Opcode 7-gram variants)
  - op-8gram-seq, op-8gram-freq, op-8gram-set (Opcode 8-gram variants)

  info stops the listing at k = 8. Any k is accepted: op-12gram-set is a
  birthmark type, it is simply not one info can list.

Supported Similarity Algorithms:
  - cosine (Cosine Similarity)
  - dice (Dice Index)
  - euclidean (Euclidean Distance)
  - jaccard (Jaccard Index)
  - levenshtein (Levenshtein Distance)
  - lcs (Longest Common Subsequence)
  - simpson (Simpson Index)
  - weighted-jaccard (Weighted Jaccard Index)

Supported Aggregators:
  - hungarian (Bipartite Matching via Hungarian Algorithm)
  - topn:N (Average of Top N Closest Matchings)
```
