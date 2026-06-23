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
  - op1gram-seq, op1gram-freq, op1gram-set (Opcode 1-gram variants)
  - op2gram-seq, op2gram-freq, op2gram-set (Opcode 2-gram variants)
  - op3gram-seq, op3gram-freq, op3gram-set (Opcode 3-gram variants)
  - op4gram-seq, op4gram-freq, op4gram-set (Opcode 4-gram variants)
  - op5gram-seq, op5gram-freq, op5gram-set (Opcode 5-gram variants)
  - op6gram-seq, op6gram-freq, op6gram-set (Opcode 6-gram variants)

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
  - hungarian (Bipartite Bipartite Matching via Hungarian Algorithm)
  - topn:N (Average of Top N Closest Matchings)
```
