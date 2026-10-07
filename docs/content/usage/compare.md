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

### 3. `containment`

The pairing of `hungarian`, averaged over the **smaller** program's functions rather than the larger's: how much of the smaller program is found in the larger one. Code copied whole into a larger program scores 1.0, where `hungarian` scores it at the share of the larger program it makes up. A high score therefore says the smaller program is *contained* in the larger, not that the two are alike; read it with the two programs' sizes in mind. It follows Broder's distinction between the containment and the resemblance of documents (1997).

### 4. `matched:T`

The pairing of `hungarian`, counting the pairs whose similarity is at least \\(T\\), over the larger program's functions: the proportion of functions that have a counterpart that close. "62% of the functions have a counterpart at 0.9 or above" is easier to state in a report than a mean similarity. \\(T\\) is above 0 and at most 1; `matched:0.9`, for instance.

### Pairs of different sizes

When one program has more functions than the other, the functions the smaller one cannot match count as zeros, so they lower the similarity instead of dropping out of it. Program A with 2 functions against program B with 3:

|  | B₁ | B₂ | B₃ |
| --- | --- | --- | --- |
| **A₁** | 0.9 | 0.2 | 0.1 |
| **A₂** | 0.3 | 0.8 | 0.0 |

* `hungarian` pairs A₁ with B₁ and A₂ with B₂, leaves B₃ without a partner, and averages over B's three functions: \\((0.9 + 0.8 + 0) / 3 \\approx 0.567\\).
* `topn:all` takes A's best matches, 0.9 and 0.8, filled out with a 0 to B's three, and B's, 0.9, 0.8 and 0.1: \\((0.9 + 0.8 + 0 + 0.9 + 0.8 + 0.1) / 6 \\approx 0.583\\).
* `topn:1` takes the single best from each side, 0.9 and 0.9, and scores 0.9: a small \\(N\\) looks only at the strongest matches, whatever is left over.
* `containment` takes the pairing `hungarian` found and averages it over A's two functions: \\((0.9 + 0.8) / 2 = 0.85\\). B₃ does not count against it.
* `matched:0.85` counts the paired similarities of at least 0.85 -- only 0.9 -- over B's three functions: \\(1 / 3 \\approx 0.333\\). `matched:0.8` counts both, \\(2 / 3\\).

So under `hungarian`, `topn:all` and `matched:T`, a small program does not score as a copy of a large one merely because all of it appears there; under `containment`, it does, which is what `containment` is for. The [glossary](../../glossary#aggregator) states the rule once for every command that aggregates.
