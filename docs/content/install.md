---
title: "⚓️ Install"
description: "How to install oinkie from crates.io, from source or as release binaries, and how to depend on it as a library."
date: 2026-06-22
draft: false
---

Install **oinkie** from [crates.io](https://crates.io/crates/oinkie) with Rust's package manager, Cargo, build it from source, or download pre-compiled binaries from the [release page](https://github.com/tamada/oinkie/releases). To use it from Rust code instead, see [As a library](#-as-a-library).

---

## 🛠️ Prerequisites

Before installing **oinkie**, ensure you have the following prerequisites installed on your system:

### 1. Rust Toolchain

Since **oinkie** is written in Rust (using the 2024 edition), you will need the Rust toolchain installed:
- [Install Rust/Cargo](https://www.rust-lang.org/tools/install)

### 2. Ghidra (for Binary Lifting)

To analyze binaries, **oinkie** relies on Ghidra for lifting compiled machine code to Pcode.
- [Download Ghidra](https://ghidra-sre.org/)
- **Java Development Kit (JDK):** JDK 21 or later, which is what Ghidra 12's own
  class files need. A JDK that is too old does not say so: Ghidra asks for a
  path instead, and in a non-interactive shell that surfaces as
  `Unable to prompt user for JDK path, no TTY detected`. Setting `JAVA_HOME`
  is what makes it name the version it rejected.
- **Environment Variable:** Set the `GHIDRA_HOME` environment variable to the directory where Ghidra is installed.
  ```sh
  export GHIDRA_HOME=/path/to/ghidra
  ```

---

## 📦 From crates.io

```sh
cargo install oinkie
```

This installs the `oinkie` command into Cargo's binary directory (usually `~/.cargo/bin`, which should be in your `PATH`). The MCP server is behind a feature of its own; to have `oinkie mcp` as well:

```sh
cargo install oinkie --features mcp
```

---

## 📦 From the Release Page

Each [release](https://github.com/tamada/oinkie/releases) carries a pre-compiled `oinkie`, with the MCP server, for six platforms:

| System | x86_64 | arm64 |
| --- | --- | --- |
| Linux | `oinkie-X.Y.Z_amd64_linux.tar.gz` | `oinkie-X.Y.Z_arm64_linux.tar.gz` |
| macOS | `oinkie-X.Y.Z_amd64_darwin.tar.gz` | `oinkie-X.Y.Z_arm64_darwin.tar.gz` |
| Windows | `oinkie-X.Y.Z_amd64_windows.zip` | `oinkie-X.Y.Z_arm64_windows.zip` |

Each archive holds the binary (`oinkie.exe` on Windows), `README.md`, `LICENSE` and the shell completions. Put the binary somewhere on your `PATH`. Windows binaries are released from v0.8.1.

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

## 📚 As a library

The `oinkie` crate is the library the command is built on. Its default `cli` feature carries what only the command needs -- the argument parser, the logger and the progress bars -- so a program that uses the library leaves it out:

```sh
cargo add oinkie --no-default-features
```

The README's [Using oinkie as a library](https://github.com/tamada/oinkie#-using-oinkie-as-a-library) has an example, and the API is documented on [docs.rs](https://docs.rs/oinkie).

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
