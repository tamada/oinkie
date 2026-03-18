# oinkie-cli

This directory contains the CLI interface of `oinkie` for extracting and comparing birthmarks and calculating similarities.

## 🏃 Usage

```sh
Birthmarking toolkit for Ghidra P-Code

Usage: oinkie [OPTIONS] <COMMAND>

Commands:
  compare  Compare birthmarks and output the similarity score
  extract  Extract birthmarks from a lifted binary file (JSON format)
  run      Extract birthmarks and compare them in one command
  info     Display information about the application
  help     Print this message or the help of the given subcommand(s)

Options:
  -l, --level <LEVEL>  Log level for the application [default: warn]
                       [possible values: error, warn, info, debug, trace, off]
  -h, --help           Print help
  -V, --version        Print version
```

### `extract` command

```sh
Extract birthmarks from a lifted binary file (JSON format)

Usage: oinkie extract [OPTIONS] [FILES]...

Arguments:
  [FILES]...  Path to the JSON files to extract birthmarks from

Options:
  -d, --dest <DEST>
          Specify the directory for putting the resultant JSON files for the extracted birthmarks
          (default: './birthmarks' directory) [default: birthmarks]
  -b, --birthmark-type <BIRTHMARK_TYPE>
          Type of birthmark to extract.
          fc (Function Calls) and op (Opcode) with set, seq, and freq variants are supported.
          For example, 'op-seq' extracts the sequence of operations as a birthmark,
          while 'fc-freq' extracts the frequency of function calls.
          The full birthmark types cann be found by running 'oinkie info'. [default: op-seq]
  -S, --skip
          Skip the resultant birthmark file is already exists
  -B, --binary-type <BINARY_TYPE>
          Type of binary. Current version only supports Ghidra JSON format [default: ghidra]
          [possible values: ghidra, llvm, binary-ninja]
  -h, --help
          Print help
```

### `compare` command

```sh
Compare birthmarks and output the similarity score

Usage: oinkie compare [OPTIONS] [FILES]...

Arguments:
  [FILES]...  Path to the birthmark JSON files to compare

Options:
  -a, --algorithm <ALGORITHM>
          Specify the similarity calculation algorithm. [default: jaccard]
          [possible values: cosine, dice, euclidean, jaccard, levenshtein, lcs, simpson, weighted-jaccard]
  -s, --strategy <STRATEGY>
          Specify the pairing strategy for comparing files. [default: all-and-self]
          [possible values: all-and-self, all, self-coverage, adjacent, first-vs-others]
  -d, --dest <DEST>
          Specify the destination directory for the comparing results [default: similarities]
  -S, --skip
          Skip if the similarity file already exists for the pair of birthmarks
  -h, --help
          Print help (see more with '--help')
```

### `run` command

```sh
Extract birthmarks and compare them in one command

Usage: oinkie run [OPTIONS] [FILES]...

Arguments:
  [FILES]...  Path to the JSON files

Options:
  -a, --analysis <ANALYSIS> 
          Similarity algorithm to use [default: op-set-jaccard] [possible values: fc-freq-cosine, fc-set-dice, fc-freq-euclidean, fc-set-jaccard, fc-seq-levenshtein, fc-seq-lcs, fc-set-simpson, fc-freq-weightedjaccard, op-freq-cosine, op-set-dice, op-freq-euclidean, op-set-jaccard, op-seq-levenshtein, op-seq-lcs, op-set-simpson, op-freq-weightedjaccard, op1gram-set-dice, op1gram-set-jaccard, op1gram-set-simpson, op1gram-seq-levenshtein, op1gram-seq-lcs, op1gram-freq-cosine, op1gram-freq-euclidean, op1gram-freq-weightedjaccard, op2gram-set-dice, op2gram-set-jaccard, op2gram-set-simpson, op2gram-seq-levenshtein, op2gram-seq-lcs, op2gram-freq-cosine, op2gram-freq-euclidean, op2gram-freq-weightedjaccard, op3gram-set-dice, op3gram-set-jaccard, op3gram-set-simpson, op3gram-seq-levenshtein, op3gram-seq-lcs, op3gram-freq-cosine, op3gram-freq-euclidean, op3gram-freq-weightedjaccard, op4gram-set-dice, op4gram-set-jaccard, op4gram-set-simpson, op4gram-seq-levenshtein, op4gram-seq-lcs, op4gram-freq-cosine, op4gram-freq-euclidean, op4gram-freq-weightedjaccard, op5gram-set-dice, op5gram-set-jaccard, op5gram-set-simpson, op5gram-seq-levenshtein, op5gram-seq-lcs, op5gram-freq-cosine, op5gram-freq-euclidean, op5gram-freq-weightedjaccard, op6gram-set-dice, op6gram-set-jaccard, op6gram-set-simpson, op6gram-seq-levenshtein, op6gram-seq-lcs, op6gram-freq-cosine, op6gram-freq-euclidean, op6gram-freq-weightedjaccard]
  -s, --strategy <STRATEGY>
          Pairing strategy for file comparisons [default: all-and-self]
          [possible values: all-and-self, all, self-coverage, adjacent, first-vs-others]
  -d, --dest <DEST>
          Destination path for the output CSV file (default: 'similarities' directory [default: similarities]
  -S, --skip
          Skip if the similarity file already exists for the pair of birthmarks
  -h, --help
          Print help (see more with '--help')
```