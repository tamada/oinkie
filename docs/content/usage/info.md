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
=========== Oinkie Info ============
Oinkie is a tool for detecting the code theft with Ghidra P-code as birthmarks.
The birthmark is a unique characteristic of a program that can be used to identify it.
Oinkie extracts birthmarks from given codes and compares them to calculate the similarities.
============ Birthmarks =============
- fc-seq                the sequence of method calls in a program
- fc-freq               the frequency of method calls in a program
- fc-set                the set of method calls in a program
- op-seq                the sequence of operations in a program
- op-set                the set of operations in a program
- op-freq               the frequency of operations in a program
- op-1gram-seq          the sequence of 1-grams of operations in a program
- op-2gram-seq          the sequence of 2-grams of operations in a program
- op-3gram-seq          the sequence of 3-grams of operations in a program
- op-4gram-seq          the sequence of 4-grams of operations in a program
- op-5gram-seq          the sequence of 5-grams of operations in a program
- op-6gram-seq          the sequence of 6-grams of operations in a program
- op-7gram-seq          the sequence of 7-grams of operations in a program
- op-8gram-seq          the sequence of 8-grams of operations in a program
- op-1gram-freq         the frequency of 1-grams of operations in a program
- op-2gram-freq         the frequency of 2-grams of operations in a program
- op-3gram-freq         the frequency of 3-grams of operations in a program
- op-4gram-freq         the frequency of 4-grams of operations in a program
- op-5gram-freq         the frequency of 5-grams of operations in a program
- op-6gram-freq         the frequency of 6-grams of operations in a program
- op-7gram-freq         the frequency of 7-grams of operations in a program
- op-8gram-freq         the frequency of 8-grams of operations in a program
- op-1gram-set          the set of 1-grams of operations in a program
- op-2gram-set          the set of 2-grams of operations in a program
- op-3gram-set          the set of 3-grams of operations in a program
- op-4gram-set          the set of 4-grams of operations in a program
- op-5gram-set          the set of 5-grams of operations in a program
- op-6gram-set          the set of 6-grams of operations in a program
- op-7gram-set          the set of 7-grams of operations in a program
- op-8gram-set          the set of 8-grams of operations in a program
======== Compare Algorithms ========
- cosine                Cosine similarity based on term frequency vectors. Available: seq and freq
- dice                  Dice coefficient. Available: seq, set and freq
- euclidean             Euclidean distance between term frequency vectors. Available: seq and freq
- jaccard               Jaccard index. Available: seq, set and freq
- levenshtein           Levenshtein distance. Available: seq
- lcs                   Longest Common Subsequence (LCS). Available: seq
- simpson               Simpson's coefficient. Available: seq, set and freq
- weighted-jaccard      Weighted Jaccard index based on term frequency vectors. Available: seq and freq
```

The k-gram listing stops at \\(k = 8\\). That bound is only what `info` shows:
any \\(k\\) is accepted, so `op-12gram-set` is a birthmark type even though it is
not listed.

`info` does not list the aggregators, which are given to `-A/--aggregator` on
`compare`, `run` and `reaggregate` rather than named in an analysis. They are
`hungarian` and `topn:N` (or `topn:all`); see those commands' `--help`.