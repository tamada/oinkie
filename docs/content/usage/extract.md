---
title: "2. Extracting Birthmarks (`extract` command)"
description: "How to extract birthmarks from Oinkie IR (P-code JSON files)."
date: 2026-06-22
draft: false
weight: 40
---

{{< katex >}}

The **extract** command analyzes the lifted OIR (JSON) files and extracts specified types of birthmarks. A software birthmark represents a distinct structural or semantic characteristic of a program (like instruction types or external function dependencies) that remains stable across compilation variations.

---

## 🏃 Usage

```sh
oinkie extract [OPTIONS] [FILES]...
```

### Arguments
* `<FILES>...`  
  Path to the OIR JSON files (previously generated via the `lift` command) from which to extract birthmarks.

### Options
* `-d, --dest <DIRECTORY>`  
  Specify the destination directory for saving the extracted birthmark files. Defaults to the `./birthmarks` directory. `[default: birthmarks]`
* `-b, --birthmark-type <BIRTHMARK_TYPE>`  
  The type and representation structure of the birthmark to extract. Defaults to `op-seq`. `[default: op-seq]`  
  *Refer below for a full list of supported birthmark types.*
* `-S, --skip`  
  Skip the extraction process if the output birthmark file already exists.

---

## 🧪 Birthmark Types and Structural Representation

Software birthmarks are categorized by their **underlying element types** and their **representation structures**.

### 1. Element Types

* **`op` (Opcode):** Focuses on the types of P-code instructions being executed (e.g., `INT_ADD`, `COPY`, `CALL`).
* **`fc` (Function Calls):** Focuses on calls made to external systems, libraries, or internal functions (e.g., `_printf`, `_atoll`).
* **Opcode \\(k\\)-grams (`op-1gram` to `op-8gram`):** Focuses on sequences of sequential P-code operations of length \\(k\\). For example, `op-3gram` processes sliding windows of 3 instructions.

### 2. Structural Representations

For any given element type, **oinkie** can format the collection using one of three structures:

* **`seq` (Sequence):** Retains the absolute order of elements as they appear in each function. Comparison is sensitive to the exact execution path/order.
* **`freq` (Frequency):** Stores elements along with their respective frequency of occurrence (histogram-like). This abstracts away the exact order but retains density/quantity characteristics.
* **`set` (Set):** Keeps unique elements without duplicate values or order information. This is highly robust against instruction shuffling.

### Common Combinations (Examples)

* **`op-seq`** (Default): Sequential instruction listing.
* **`fc-freq`**: How many times each function is called.
* **`op-3gram-seq`**: Sequences of 3 sequential operations.
* **`op-set`**: The set of unique opcodes used.

To list all possible birthmark configurations supported by your current installation, run:

```sh
oinkie info
```
