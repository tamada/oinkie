---
title: "oinkie 🐽🐷🐖"
description: "Detecting software theft, the birthmark toolkit for Ghidra Pcode, LLVM IR/BC, and Binary Ninja."
date: 2026-06-22
draft: false
outputs:
  - html
---

{{< keywordList markdownify="true" >}}
[![Version](https://shields.io/badge/Version-0.2.1-blue)](https://github.com/tamada/oinkie/releases/tag/v0.2.1)
[![License-MIT](https://shields.io/badge/License-MIT-blue)](https://github.com/tamada/oinkie/blob/main/LICENSE) 
[![Coverage Status](https://coveralls.io/repos/github/tamada/oinkie/badge.svg)](https://coveralls.io/github/tamada/oinkie)
[![Docker](https://img.shields.io/badge/Container-quay.io/tama5/oinkie:0.2.1-blue?logo=docker)](https://quay.io/repository/tama5/oinkie)
{{< /keywordList >}}


[ [🗣️ Overview](#-overview) | [🚶 Procedures of Birthmarking](#-procedures-of-birthmarking) | [🧭 Navigation](#-navigation) ]

{{< figure src="oinkie.png" alt="oinkie logo" width="200px" class="mx-auto my-4" >}}

**oinkie** is a software birthmark toolkit designed for detecting software theft. It extracts unique characteristics (birthmarks) of software from binary formats and compares them to identify suspected plagiarism.

Currently, **oinkie** supports extracting and comparing birthmarks from Ghidra Pcode, and planned support includes LLVM IR/BC and Binary Ninja.

---

## 🗣️ Overview

Software theft is difficult to detect because it is conducted stealthily, and the source code of stolen software remains private. Compilers and their options sensitively alter the binary formats (including executables) of software, and the problem is further complicated by the vast amount of software worldwide. Therefore, we need a method to detect software theft targeting binary formats from large software repositories.

To solve this, Tamada et al. proposed the concept of **software birthmarking** in 2004. A software birthmark refers to the native characteristics of programs and allows for comparison between them. The similarities of the two birthmarks reflect how similar the original programs are.

This toolkit extracts these birthmarks from binary code and compares them to calculate similarities. High similarity suggests that one program is suspected of being a copy of the other.

---

## 🚶 Procedures of Birthmarking

To examine software birthmarks with `oinkie`, we apply the following 5-step workflow:

1. **Collect** the binary files to be examined.
2. **Lift** the binary files to an intermediate representation (IR), such as Ghidra Pcode.
3. **Extract** the birthmarks from the lifted IR files.
4. **Compare** the birthmarks to calculate similarities.
5. **Analyze** the similarity results to determine if software theft is suspected.

{{< figure src="procedures.png" alt="Overview of the birthmarking process" class="mx-auto my-6" >}}

---

## 🧭 Navigation

- To get started and install `oinkie`, see [⚓️ Install](/install),
- To learn how to use the CLI and its subcommands, see [🏃 Usage](/usage),
- To read about the scientific background, origin, and research papers, see [About](/about), and
- To learn the trend of the software birthmarks, see [Academic](/academic).
