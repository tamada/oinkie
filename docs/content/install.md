---
title: "⚓️ Install"
description: "How to install oinkie from source or download release binaries."
date: 2026-06-22
draft: false
---

To install **oinkie**, you can build it from source using Rust's package manager, Cargo, or download pre-compiled binaries from the release page.

---

## 🛠️ Prerequisites

Before installing **oinkie**, ensure you have the following prerequisites installed on your system:

### 1. Rust Toolchain
Since **oinkie** is written in Rust (using the 2024 edition), you will need the Rust toolchain installed:
- [Install Rust/Cargo](https://www.rust-lang.org/tools/install)

### 2. Ghidra (for Binary Lifting)
To analyze binaries, **oinkie** relies on Ghidra for lifting compiled machine code to Pcode.
- [Download Ghidra](https://ghidra-sre.org/)
- **Java Development Kit (JDK):** Ghidra requires a compatible JDK (typically JDK 17 or later).
- **Environment Variable:** Set the `GHIDRA_HOME` environment variable to the directory where Ghidra is installed.
  ```sh
  export GHIDRA_HOME=/path/to/ghidra
  ```

---

## 📦 Building from Source

You can clone the repository and compile **oinkie** directly on your machine.

### 1. Clone the Repository
```sh
git clone https://github.com/tamada/oinkie.git
cd oinkie
```

### 2. Build the Project
Compile the CLI executable using Cargo:
```sh
cargo build --release
```
The compiled binary will be available at `./target/release/oinkie`.

### 3. Install Globally
To install the `oinkie` command to your Cargo binary directory (usually `~/.cargo/bin` which should be in your `PATH`):
```sh
cargo install --path .
```

---

## 🚀 Verifying the Installation

To verify that **oinkie** has been successfully installed, check the version and help information:

```sh
oinkie --version
```

To view all available commands and options, run:

```sh
oinkie --help
```
