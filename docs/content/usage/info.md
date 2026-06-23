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
  - op-uni-gram-seq,   op-uni-gram-freq,   op-uni-gram-set   (Opcode 1-gram variants)
  - op-bi-gram-seq,    op-bi-gram-freq,    op-bi-gram-set    (Opcode 2-gram variants)
  - op-tri-gram-seq,   op-tri-gram-freq,   op-tri-gram-set   (Opcode 3-gram variants)
  - op-quad-gram-seq,  op-quad-gram-freq,  op-quad-gram-set  (Opcode 4-gram variants)
  - op-penta-gram-seq, op-penta-gram-freq, op-penta-gram-set (Opcode 5-gram variants)
  - op-hexa-gram-seq,  op-hexa-gram-freq,  op-hexa-gram-set  (Opcode 6-gram variants)
  - op-hepta-gram-seq, op-hepta-gram-freq, op-hepta-gram-set (Opcode 7-gram variants)
  - op-octa-gram-seq,  op-octa-gram-freq,  op-octa-gram-set  (Opcode 8-gram variants)

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
