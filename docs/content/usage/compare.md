---
title: "3. Comparing Birthmarks (`compare` command)"
description: "How to compare birthmarks and calculate software similarities."
date: 2026-06-22
draft: false
weight: 30
---

{{< katex >}}

The **compare** command performs pairwise comparison between extracted birthmarks to measure their similarity. This step calculates a score between \\(0.0\\) (entirely different) and \\(1.0\\) (identical), indicating the level of structural similarity between the programs.

---

## 🏃 Usage

```sh
oinkie compare [OPTIONS] [JSON_FILES]...
```

### Arguments

* `<JSON_FILES>...`  
  Paths to the birthmark JSON files (generated using the `extract` command) to compare.

### Options

* `-a, --algorithm <ALGORITHM>`  
  Specify the similarity calculation algorithm to compare birthmarks. `[default: jaccard]`  
  *Refer below for a full list of algorithms.*
* `-A, --aggregator <METHOD>`  
  Specify the method for combining the function similarities of a pair into its single similarity. `[default: hungarian]`  
  *Refer below for a full list of aggregators.*
* `-s, --strategy <STRATEGY>`  
  The pairing strategy to use when comparing files. `[default: all-and-self]`  
  `[possible values: all-and-self, all, self-coverage, adjacent, first-vs-others, last-vs-others]`
  *Refer below for a full list of strategies.*
* `-d, --dest <DIRECTORY>`  
  The output directory where comparison results are saved (typically as CSV files). `[default: similarities]`
* `-S, --skip`  
  Skip the comparison if the output file already exists for the current file pair.
* `--threads <N>`  
  The number of threads to compute on, for every part of the command that runs in parallel. `[default: one per core]`

---

## 🧦 Pairing Strategies (`--strategy`)

When comparing multiple files, you can configure which file pairs are compared using the following strategy options:

| Strategy | Description |
| :--- | :--- |
| **`all-and-self`** | Compare every file with every other file, including self-comparisons. |
| **`all`** | Compare every file with every other file, excluding self-comparisons. |
| **`self-coverage`** | Compare each file only with itself (useful for baselines). |
| **`adjacent`** | Compare adjacent files in the input list (e.g., \\(F_1\\) vs \\(F_2\\), \\(F_2\\) vs \\(F_3\\)). |
| **`first-vs-others`** | Compare the first file in the arguments list against all remaining files. |
| **`last-vs-others`** | Compare the last file in the arguments list against all remaining files. |

---

## 🪞 Similarity Calculation Algorithms (`--algorithm`)

The mathematical algorithms used to compare two birthmark properties:

* **`cosine`** (Cosine Similarity)
* **`dice`** (Dice Index)
* **`euclidean`** (Euclidean Distance Similarity)
* **`jaccard`** (Jaccard Index)
* **`jensen-shannon`** (Jensen–Shannon Divergence): compares the proportions of each element in the two functions, whatever their sizes, as \\(1 - \\sqrt{\\mathrm{JSD}}\\), the divergence taken in bits
* **`levenshtein`** (Levenshtein Distance Similarity)
* **`lcs`** (Longest Common Subsequence Similarity)
* **`simpson`** (Simpson Index / Overlap Coefficient)
* **`tanimoto`** (Tanimoto Coefficient): \\(a \\cdot b / (|a|^2 + |b|^2 - a \\cdot b)\\) over the counts; on sets it would be Jaccard's index, so it takes frequencies only
* **`weighted-jaccard`** (Weighted Jaccard Index)

---

## 🏗️ Similarity Aggregators (`--aggregator`)

Because software birthmarks are extracted at a **function level**, comparing two programs involves comparing sets of functions. **oinkie** uses aggregators to resolve these function similarities into the single similarity of the pair:

### 1. `hungarian` (Default)

Uses the **Hungarian Algorithm** to pair the functions of program A with those of program B one to one, choosing the pairing whose similarities add up to the most, and averages the paired similarities. A function is not always paired with its own best match: two functions may share a best match, and only one of them can have it.

### 2. `topn:N`

Takes each function's best match in the other program, on both sides, and averages the \\(N\\) best of those from each side. The matching need not be one-to-one. This reduces noise from minor or unrelated function matches.

### Pairs of different sizes

When one program has more functions than the other, the functions the smaller one cannot match count as zeros, so they lower the similarity instead of dropping out of it. Program A with 2 functions against program B with 3:

|  | B₁ | B₂ | B₃ |
| --- | --- | --- | --- |
| **A₁** | 0.9 | 0.2 | 0.1 |
| **A₂** | 0.3 | 0.8 | 0.0 |

* `hungarian` pairs A₁ with B₁ and A₂ with B₂, leaves B₃ without a partner, and averages over B's three functions: \\((0.9 + 0.8 + 0) / 3 \\approx 0.567\\).
* `topn:all` takes A's best matches, 0.9 and 0.8, filled out with a 0 to B's three, and B's, 0.9, 0.8 and 0.1: \\((0.9 + 0.8 + 0 + 0.9 + 0.8 + 0.1) / 6 \\approx 0.583\\).
* `topn:1` takes the single best from each side, 0.9 and 0.9, and scores 0.9: a small \\(N\\) looks only at the strongest matches, whatever is left over.

So under `hungarian` and `topn:all`, a small program does not score as a copy of a large one merely because all of it appears there. The [glossary](../../glossary#aggregator) states the rule once for every command that aggregates.
