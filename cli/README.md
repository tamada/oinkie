# oinkie-cli

This directory contains the CLI interface of `oinkie` for extracting and comparing birthmarks and calculating similarities.

## 🏃 Usage

```sh
Birthmarking toolkit for Ghidra P-Code

Usage: oinkie [OPTIONS] <COMMAND>

Commands:
  info         Display information about the application
  lift         Lift binary files to P-code JSON files using a specified lifter
  extract      Extract birthmarks from a lifted binary file (JSON format)
  compare      Compare birthmarks and output the similarity score
  reaggregate  Reaggregate the element-wise similarity scores and recalculate the birthmark-wise similarity score
  run          Extract birthmarks and compare them in one command
  help         Print this message or the help of the given subcommand(s)

Options:
  -l, --level <LEVEL>  Log level for the application [default: warn]
                       [possible values: error, warn, info, debug, trace, off]
  -h, --help           Print help
  -V, --version        Print version
```

### `lift` command

This command lifts binary files into Oinkie-IR files (P-code JSON format), which are used as inputs for the `extract` command.

```sh
Lift binary files to P-code JSON files using a specified lifter

Usage: oinkie lift [OPTIONS] [FILES]...

Arguments:
  [FILES]...  Path to the binary or intermediate files to lift

Options:
  -d, --dest <DIRECTORY>
          Specify the directory for putting the resultant JSON files for the lifted P-code
          (default: './pcodes' directory) [default: pcodes]
  -l, --lifter-type <LIFTER_TYPE>
          Specify the lifter type [default: ghidra]
          [possible values: ghidra, llvm, binary-ninja]
  -H, --home <HOME>
          Specify the path to the home directory of the lifter (e.g., GHIDRA_HOME for Ghidra).
          If not specified, the environment variable (e.g., GHIDRA_HOME) or default paths are searched.
  -i, --intermediate <DIRECTORY>
          Directory to keep intermediate files like Ghidra project directories.
          If not specified, a temporary directory is used and deleted.
      --script <SCRIPT>
          Path to a custom lifting script. Interpretation depends on the lifter type.
          For Ghidra, it's the path to a Java script.
  -S, --skip
          Skip if the resultant JSON file already exists
  -h, --help
          Print help
```

### `extract` command

This command extracts birthmarks from the given lifted binary files.
The lifted binary files are obtained by [`lift` command](#lift-command)

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
          k-grams are written with the k in the name: 'op-3gram-set'. Any k parses, not
          only the ones 'oinkie info' lists.
          The full birthmark types can be found by running 'oinkie info'. [default: op-seq]
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

Usage: oinkie compare [OPTIONS] [JSON_FILES]...

Arguments:
  [JSON_FILES]...  Path to the birthmark JSON files to compare

Options:
  -a, --algorithm <ALGORITHM>
      Specify the similarity calculation algorithm. [default: jaccard]
      [possible values: cosine, dice, euclidean, jaccard, levenshtein, lcs, simpson, weighted-jaccard]
  -A, --aggregator <METHOD>
      Specify the aggregator for combining element-wise similarity scores into a birthmark-wise similarity score.
      Available:
      - hungarian  Use the Hungarian algorithm to find the optimal matching between elements of two birthmarks,
                   maximizing the total similarity score.
      - topn:N     For each element in the first birthmark, consider only the top N most similar elements in the
                   second birthmark when calculating the overall similarity score. This can reduce noise from less
                   relevant matches and focus on the most significant similarities. [default: hungarian]
  -s, --strategy <STRATEGY>
      Specify the pairing strategy for comparing files. [default: all-and-self]
      [possible values: all-and-self, all, self-coverage, adjacent, first-vs-others]
  -d, --dest <DIRECTORY>
      Specify the destination directory for the comparing results [default: similarities]
  -S, --skip
      Skip if the similarity file already exists for the pair of birthmarks
  -h, --help
      Print help (see more with '--help')
```

### `reaggregate` command

Reaggregate the element-wise similarity scores and recalculate the birthmark-wise similarity score.

```sh
Reaggregate the element-wise similarity scores and recalculate the birthmark-wise similarity score

Usage: oinkie reaggregate [OPTIONS] <SCORE_DIRECTORY>

Arguments:
  <SCORE_DIRECTORY>  Path to the directory containing the element-wise similarity scores

Options:
  -A, --aggregator <METHOD>
          Specify the aggregator for combining element-wise similarity scores into a birthmark-wise similarity score.
          Available:
          - hungarian  Use the Hungarian algorithm to find the optimal matching between elements of two birthmarks,
                       maximizing the total similarity score.
          - topn:N     For each element in the first birthmark, consider only the top N most similar elements in the
                       second birthmark when calculating the overall similarity score. This can reduce noise from less
                       relevant matches and focus on the most significant similarities. [default: hungarian]
  -d, --dest-file <RESULT.CSV>
          Specify the result CSV file of the comparing results to reaggregate.
          The file contains the birthmark-wise similarity score list. [default: reaggregate.csv]
  -h, --help
          Print help
```

### `run` command

```sh
Extract birthmarks and compare them in one command

Usage: oinkie run [OPTIONS] [FILES]...

Arguments:
  [FILES]...  Path to the JSON files

Options:
  -a, --analysis <ANALYSIS>
          Analysis to run, as '{birthmark}-{algorithm}' -- for example 'op-set-jaccard' or 'op-3gram-freq-cosine'.
          Run 'oinkie info' for the birthmarks and the algorithms they pair with. Any k
          parses in a k-gram name, not only the ones listed. [default: op-set-jaccard]
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
