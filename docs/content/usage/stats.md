---
title: "Summarising Birthmarks (`stats` command)"
description: "How many birthmarks of which type, how many functions each holds, and how long each function's birthmark is."
date: 2026-09-30
draft: false
weight: 65
---

The **stats** command summarises a set of extracted birthmarks: how many there are of each type, how many functions each program holds, and how long each function's birthmark is. It is what you would otherwise write a script for when describing a dataset in a paper, or when choosing a threshold on birthmark length.

---

## 📏 Functions and elements

A birthmark file is one program, and it holds one entry per **function**. Each function's birthmark is made of **elements**: operations, calls or \\(k\\)-grams. The two counts are kept apart by name:

| | counts | in a `seq` birthmark | in a `set` birthmark | in a `freq` birthmark |
| --- | --- | --- | --- | --- |
| **functions** | entries in one birthmark file | | | |
| **elements** | items in one function's birthmark: the *birthmark length* | the sequence length | distinct items | distinct keys |

For the `freq` shapes, the total of the counts is reported separately, as **occurrences**.

---

## 🏃 Usage

```sh
oinkie stats [OPTIONS] <PATHS>...
```

### Arguments

* `<PATHS>...`  
  Birthmark files, or directories holding them. A directory contributes the `*.json` directly inside it. A file that does not read as a birthmark (a lifted program, say) is skipped with a warning, and the number skipped is part of the output.

### Options

* `-f, --format <FORMAT>`  
  `json`, `csv` or `markdown`. `[default: markdown]`
* `-o, --output <FILE>`  
  Write to `FILE` rather than to standard output.
* `-r, --recursive`  
  Descend into the subdirectories of the given directories.
* `--per-file`  
  Report each birthmark file as well as each group.
* `-t, --top <N>`  
  Report the \\(N\\) most frequent elements of each group. For `set` birthmarks the count is the number of functions containing the element.

---

## 📊 What is reported

**One row per group**, a group being birthmarks of the same intermediate representation and type, which are the ones that can be compared with each other:

* `files`: birthmark files in the group
* `functions`: functions per file
* `elements`: elements per function, over every function in the group
* `empty`: functions with no elements, as a count and a ratio
* `vocabulary`: distinct elements across the group
* `occurrences`: the total of the counts, `freq` shapes only

`functions` and `elements` are given as min, Q1, median, Q3, max, mean and standard deviation. The standard deviation is the population one, since the input is the whole set rather than a sample of one; quartiles interpolate linearly, as R's and NumPy's defaults do.

**With `--per-file`**, one row per birthmark file as well: its functions, its empty functions, and its elements per function.

### Formats

* **`markdown`** prints every table requested, rounded to two decimals, ready to read or paste.
* **`json`** always carries the groups, the per-file rows and the skipped files, at full precision.
* **`csv`** holds one table: the groups, or the per-file rows with `--per-file`, or the most frequent elements with `--top`. Giving both `--per-file` and `--top` with `-f csv` is refused.

---

## 💡 Example

```sh
oinkie extract -b op-seq -d birthmarks/op-seq pcodes/*.json
oinkie extract -b op-set -d birthmarks/op-set pcodes/*.json
oinkie stats -r birthmarks --top 3
```

```markdown
## Elements per function

| ir | birthmark | min | Q1 | median | Q3 | max | mean | sd | empty | empty % | vocabulary | occurrences |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| ghidra-pcode | op-seq | 3 | 4.00 | 4.00 | 5.00 | 7 | 4.60 | 1.36 | 0 | 0.0% | 8 | – |
| ghidra-pcode | op-set | 3 | 3.00 | 3.00 | 3.00 | 5 | 3.40 | 0.80 | 0 | 0.0% | 8 | – |
```
