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
  Specify the method for combining individual element-wise (function-to-function) similarities into a single program-wide similarity score. `[default: hungarian]`  
  *Refer below for a full list of aggregators.*
* `-s, --strategy <STRATEGY>`  
  The pairing strategy to use when comparing files. `[default: all-and-self]`  
  `[possible values: all-and-self, all, self-coverage, adjacent, first-vs-others, last-vs-others]`
  *Refer below for a full list of strategies.*
* `-d, --dest <DIRECTORY>`  
  The output directory where comparison results are saved (typically as CSV files). `[default: similarities]`
* `-S, --skip`  
  Skip the comparison if the output file already exists for the current file pair.

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
* **`levenshtein`** (Levenshtein Distance Similarity)
* **`lcs`** (Longest Common Subsequence Similarity)
* **`simpson`** (Simpson Index / Overlap Coefficient)
* **`weighted-jaccard`** (Weighted Jaccard Index)

---

## 🏗️ Similarity Aggregators (`--aggregator`)

Because software birthmarks are extracted at a **function level**, comparing two programs involves comparing sets of functions. **oinkie** uses aggregators to resolve these function-to-function similarities into a single program-level similarity score:

### 1. `hungarian` (Default)
Uses the **Hungarian Algorithm** to find the optimal global matching between the functions of program A and program B. This ensures that every function is paired up with its most likely match in the other program, maximizing the overall global similarity score.

### 2. `topn:N`
For each function in the first program, it considers only the top \\(N\\) most similar functions in the second program when calculating the overall average similarity score. This reduces noise from minor or unrelated function matches.
