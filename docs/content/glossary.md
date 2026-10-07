---
title: "📖 Glossary"
description: "The terms oinkie uses, each defined once."
date: 2026-10-06
draft: false
---

{{< katex >}}

The terms below are the ones the commands, their `--help`, this site and the library use. Each is defined here once; where a term was used loosely in the past, the old wording is given so that it can be recognised.

The order follows the work: a binary is **lifted**, a **birthmark** is extracted from it, two birthmarks are **compared**, and the comparison is **reviewed**.

---

## 🔧 Lifting

### Lift

Turning a binary into a [lifted program](#lifted-program), with a decompiler doing the reading. `oinkie lift` runs the [lifter](#lifter) and writes one JSON file per binary.

### Lifter

The tool that does the lifting: Ghidra, IDA or Binary Ninja, driven headless by a script oinkie ships for each.

### Intermediate representation (IR)

The language a lifter translates machine code into, independent of the processor it came from: `ghidra-pcode`, `ida-microcode`, `binary-ninja-llil`, `binary-ninja-mlil` or `binary-ninja-hlil` (`--ir`). Each IR comes from one lifter. Birthmarks are comparable only when they come from the same IR.

### Lifted program

The JSON `lift` writes: the program's functions, each as the sequence of IR operations it is made of, and the names of the functions it calls. What `extract` and `run` take.

---

## 🧬 Birthmarks

### Birthmark

What is extracted from one lifted program, and compared in its place: one [function](#function) for each function of the program. Written to a [birthmark file](#birthmark-file).

### Function

One function's birthmark: the [elements](#element) extracted from one function of the program. A birthmark holds one per function, in the program's order. The number of functions is how many entries the birthmark has.

### Element

One item of a function's birthmark: an IR operation (`op`), the name of a called function (`fc`), or a [k-gram](#k-gram) of operations. The number of elements in one function's birthmark is its *birthmark length*, which is what [`--min-elements`](#min-elements) counts and what `stats` reports as `elements`. In a `set` it counts distinct elements, and in a `freq` distinct keys; the total of a `freq`'s counts is reported separately, as *occurrences*.

*Not* a function: "element-wise similarity", used before v0.8.0 for what is a [function similarity](#function-similarity), is retired.

### k-gram

\\(k\\) consecutive operations taken as one element, for \\(k \geq 1\\): `op-3gram-*` takes every run of three. A k-gram keeps some of the order that a `set` or a `freq` of single operations loses.

### Shape

How a function's elements are kept:

* `seq` — in order, duplicates included;
* `set` — each distinct element once, without order;
* `freq` — each distinct element with how often it occurs.

Older pages call this the *structure* or *representation*.

### Birthmark type

What to extract: the kind of element and the [shape](#shape), named `{element}-{shape}` — `op-seq`, `fc-set`, `op-3gram-freq` (`-b`, `--birthmark-type`). Birthmarks are comparable only when they are of the same type.

### Empty function, empty birthmark

A function with no elements, and a birthmark with no functions. Both still compare. Two empty birthmarks have a [similarity](#similarity) of 1.0, and an empty one against one that is not, 0.0; every [algorithm](#algorithm) scores empty functions by the same rule. A perfect match between programs that should have nothing in common is therefore a reason to look at the inputs. `extract` warns when every function of an `fc-*` birthmark is empty — a program that calls nothing, or whose calls the lifter could not resolve — since two such birthmarks agree completely.

---

## ⚖️ Comparing

### Algorithm

How two functions' elements are compared into a number between 0 and 1: `jaccard`, `dice`, `simpson`, `cosine`, `euclidean`, `levenshtein`, `lcs`, `weighted-jaccard` (`-a`, `--algorithm`). Each works on some [shapes](#shape) and not others; `oinkie info` lists which.

### Analysis

A [birthmark type](#birthmark-type) and an [algorithm](#algorithm) together, named `{birthmark type}-{algorithm}` — `op-set-jaccard`, `op-3gram-seq-levenshtein` (`-a`, `--analysis` of `run`). Only pairings the algorithm supports are analyses.

### Function similarity

The algorithm's number for one function of one birthmark against one function of the other, between 0 and 1. A comparison computes one for every such pair of functions: the *function similarity matrix*, which the [pair CSV](#pair-csv) stores.

Formerly *element-wise similarity* or *element-wise score*.

### Aggregator

How a pair's function similarities become its one [similarity](#similarity) (`-A`, `--aggregator`):

* `hungarian` (the default) — the one-to-one matching of functions that maximises the total, by the Hungarian algorithm, and the mean of the matched similarities over as many functions as the larger birthmark has;
* `topn:N` — each function's best match in the other birthmark, on both sides and not necessarily one-to-one, and the mean of the \\(N\\) best of those from each side. `topn:all`, or `topn`, takes them all.

When one birthmark has more functions than the other, the difference counts as zeros. Under `hungarian`, each function of the larger birthmark left without a partner counts 0, and the mean is over the larger birthmark's functions. Under `topn`, the smaller side's list of best matches is filled out with zeros to the larger side's length, which `topn:all` averages in and a small \\(N\\) mostly leaves out. So under `hungarian` and `topn:all`, a small program does not score as a copy of a large one merely because all of it appears there.

### Similarity

The number a comparison gives a pair of birthmarks, between 0 and 1, from its function similarities by the [aggregator](#aggregator). A high similarity suggests that one program is a copy of the other; it does not prove it.

Where a file or an option says *score* — the [score directory](#score-directory), the `similarity` column of a [summary](#summary) — it means this. Formerly also *birthmark-wise*, *program-wide* or *program-wise similarity score*.

### Pair

Two birthmarks compared with each other. Which pairs are compared is the [pairing strategy](#pairing-strategy)'s choice, and each is numbered in the order it chose them.

### Pairing strategy

Which pairs of the given files to compare (`-s`, `--strategy`): `all-and-self`, `all`, `self-coverage`, `adjacent`, `first-vs-others` or `last-vs-others`. `oinkie info` says what each is for.

---

## 📁 Files

### Birthmark file

A birthmark as JSON, named `{stem}_{hash}.json` after the lifted program it came from, so that two programs with the same name do not overwrite each other. Written by `extract`, and by `run` into the score directory's `birthmarks/`. Its functions are under the key `elements`, a name kept from before the terms above were settled, so that every file written before still reads.

### Score directory

Where `compare` and `run` write a comparison (`-d`): one [pair CSV](#pair-csv) per pair and a [summary](#summary), and from `run` the birthmarks it compared, under `birthmarks/`. What `review` reads.

### Pair CSV

One pair's comparison, `00000.csv` and onwards by the pair's number: its similarity and how long it took, the two birthmarks it compared and where their files are, and the function similarity matrix. Also called the *score CSV*.

### Summary

One row per pair — its number, similarity, the two programs and how long it took — then the total duration: `results.csv` in a score directory, and the file `review` writes (`review.csv` by default). With `--min-elements`, a last line records the threshold.

---

## 🔍 Reviewing

### Review

Re-reading a finished comparison: recomputing each pair's similarity from the function similarities its pair CSV stored, under another [aggregator](#aggregator) or without the shortest functions, without comparing anything again. `oinkie review`; named `reaggregate` before v0.8.0.

### Min elements

The threshold `review --min-elements` drops functions below before aggregating, given as a count (`5`) or as a multiple of the mean birthmark length over every function of every birthmark in the score directory (`0.3x`). It is recorded in the summary twice: as *given*, and as *resolved* to a count. Scores under different thresholds are not comparable with each other.

---

## 📊 Describing

### Group

In `stats`, the birthmarks of one IR and one birthmark type: the ones that can be compared with each other, and so the unit every statistic is taken over.

### Vocabulary

In `stats`, the number of distinct elements across a group.
