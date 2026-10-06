# oinkie-cli

This directory contains the CLI interface of `oinkie` for extracting and comparing birthmarks and calculating similarities.

## 🏃 Usage

```sh
Detects software theft by comparing birthmarks extracted from binaries

Usage: oinkie [OPTIONS] <COMMAND>

Commands:
  info     Display information about the application
  lift     Lift binary files to JSON files of an intermediate representation, using a specified lifter
  extract  Extract birthmarks from a lifted binary file (JSON format)
  compare  Compare birthmarks and output the similarity score
  review   Re-read a finished comparison: recompute the similarity of each pair from the stored function similarities
  stats    Summarise a set of birthmarks: how many of each type, how many functions each holds, and how long each function's birthmark is
  run      Extract birthmarks and compare them in one command
  mcp      Serve oinkie over the Model Context Protocol, on stdin and stdout
  help     Print this message or the help of the given subcommand(s)

Options:
  -l, --level <LEVEL>  Log level for the application [default: warn] [possible values: error, warn, info, debug, trace, off]
  -h, --help           Print help
  -V, --version        Print version
```

### `lift` command

This command lifts binary files into Oinkie-IR files (P-code JSON format), which are used as inputs for the `extract` command.

```sh
Lift binary files to JSON files of an intermediate representation, using a specified lifter

Usage: oinkie lift [OPTIONS] [FILES]...

Arguments:
  [FILES]...  Path to the binary or intermediate files to lift

Options:
  -d, --dest <DIRECTORY>          Specify the directory for putting the resultant JSON files of the lifted programs (default: './pcodes' directory) [default: pcodes]
  -r, --ir <IR>                   Intermediate representation to produce. This also picks the tool, since a representation is only produced by one of them; several representations can come from the same tool. [default: ghidra-pcode] [possible values: ghidra-pcode, ida-microcode, binary-ninja-llil, binary-ninja-mlil, binary-ninja-hlil]
  -H, --home <HOME>               Path to the installation directory of the tool behind --ir. If not specified, that tool's own environment variable (GHIDRA_HOME for Ghidra) is read, then the usual install locations are searched. The error names which variable to set.
  -i, --intermediate <DIRECTORY>  Directory for the lifter to work in, kept rather than discarded. Every lifter runs in one, since that is where its script writes; Ghidra also keeps its project there. If not specified, a temporary directory is used and deleted.
      --script <SCRIPT>           Path to a custom lifting script, replacing the built-in one. The language is that of the tool behind --ir: Java for Ghidra. It must write {input file name}.json into its working directory.
  -j, --jobs <N>                  Lift up to N files at a time (default: 1, one after another). Lifting runs a whole decompiler process per file, and several of them against a Ghidra installation whose language cache has not been built yet can corrupt it, so parallelism is opt-in. [default: 1]
  -S, --skip                      Skip if the resultant JSON file already exists
  -h, --help                      Print help (see more with '--help')
```

### `extract` command

This command extracts birthmarks from the given lifted binary files.
The lifted binary files are obtained by [`lift` command](#lift-command)

```sh
Extract birthmarks from a lifted binary file (JSON format)

Usage: oinkie extract [OPTIONS] [JSON_FILES]...

Arguments:
  [JSON_FILES]...  Path to the JSON files to extract birthmarks from

Options:
  -d, --dest <DIRECTORY>
          Specify the directory for putting the resultant JSON files for the extracted birthmarks (default: './birthmarks' directory) [default: birthmarks]
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
  -h, --help
          Print help
```

### `compare` command

```sh
Compare birthmarks and output the similarity score

Usage: oinkie compare [OPTIONS] [JSON_FILES]...

Arguments:
  [JSON_FILES]...
          Path to the birthmark JSON files to compare

Options:
  -a, --algorithm <ALGORITHM>
          Specify the similarity calculation algorithm.

          Possible values:
          - cosine:           Cosine similarity based on term frequency vectors. Available: seq and freq
          - dice:             Dice coefficient. Available: seq, set and freq
          - euclidean:        Euclidean distance between term frequency vectors. Available: seq and freq
          - jaccard:          Jaccard index. Available: seq, set and freq
          - levenshtein:      Levenshtein distance. Available: seq
          - lcs:              Longest Common Subsequence (LCS). Available: seq
          - simpson:          Simpson's coefficient. Available: seq, set and freq
          - weighted-jaccard: Weighted Jaccard index based on term frequency vectors. Available: seq and freq
          
          [default: jaccard]

  -A, --aggregator <METHOD>
          Specify the aggregator for combining the function similarities of a pair into its similarity.
          Available:
          - hungarian  Use the Hungarian algorithm to find the optimal one-to-one matching between the functions of
                       two birthmarks, maximizing the total similarity.
          - topn:N     Take each function's best match in the other birthmark, and average the N best of those from
                       each side. Matches need not be one-to-one. This can reduce noise from less relevant matches
                       and focus on the most significant similarities.
          
          [default: hungarian]

  -s, --strategy <STRATEGY>
          Specify the pairing strategy for comparing files.

          Possible values:
          - all-and-self:    All possible combinations including self-comparisons ($_nC_2 + n$). Used for full matrix visualization or comprehensive heatmaps
          - all:             Compares all possible combinations ($_nC_2$). Used for comprehensive validation of accuracy (False Positive / True Positive)
          - self-coverage:   Compares each file with itself ($n$). Used for sanity checks to ensure identical files yield a similarity score of 1.0
          - adjacent:        Compares only adjacent pairs in the list ($n-1$). Useful for comparing sequential versions (e.g., v1.0 vs v1.1, v1.1 vs v1.2)
          - first-vs-others: Compares a specific reference file against all other files ($n-1$). Compares first item and all other items. Useful for comparing a baseline version against multiple variants
          - last-vs-others:  Compares a specific reference file against all other files ($n-1$). Compares the last item and all other items. Useful for comparing a baseline version against multiple variants
          
          [default: all-and-self]

  -d, --dest <DIRECTORY>
          Specify the destination directory for the comparing results
          
          [default: similarities]

  -S, --skip
          Skip if the similarity file already exists for the pair of birthmarks

  -h, --help
          Print help (see a summary with '-h')
```

### `review` command

Re-read a finished comparison: recompute its birthmark-wise similarity scores from the stored element-wise scores.

```sh
Re-read a finished comparison: recompute the similarity of each pair from the stored function similarities

Usage: oinkie review [OPTIONS] <SCORE_DIRECTORY>

Arguments:
  <SCORE_DIRECTORY>  Path to the score directory: the pair CSVs that compare or run wrote, holding the function similarities

Options:
  -A, --aggregator <METHOD>     Specify the aggregator for combining the function similarities of a pair into its similarity.
                                Available:
                                - hungarian  Use the Hungarian algorithm to find the optimal one-to-one matching between the functions of
                                             two birthmarks, maximizing the total similarity.
                                - topn:N     Take each function's best match in the other birthmark, and average the N best of those from
                                             each side. Matches need not be one-to-one. This can reduce noise from less relevant matches
                                             and focus on the most significant similarities. [default: hungarian]
  -d, --dest-file <RESULT.CSV>  Specify the result CSV file of the comparing results to review.
                                The file lists the similarity of each pair. [default: review.csv]
      --min-elements <N|Rx>     Drop the functions with fewer elements than this before aggregating.
                                N is a count of elements; Rx is R times the mean count over every function of every birthmark
                                in the directory (e.g. 0.3x). Needs the birthmarks the comparisons name. The threshold is
                                recorded on the last line of the summary.
  -h, --help                    Print help
```

### `stats` command

Summarise a set of birthmarks. A birthmark file holds one entry per **function**, and each function's birthmark is made of **elements**; the command reports functions per file and elements per function (the birthmark length) for each group of birthmarks sharing an intermediate representation and a type.

```sh
Summarise a set of birthmarks: how many of each type, how many functions each holds, and how long each function's birthmark is

Usage: oinkie stats [OPTIONS] <PATHS>...

Arguments:
  <PATHS>...
          Birthmark files, or directories holding them. A directory contributes the *.json directly inside it;
          a file that does not read as a birthmark is skipped with a warning and counted.

Options:
  -f, --format <FORMAT>
          Output format

          Possible values:
          - json:     Every table at once: the summary, the per-file rows and the skipped files
          - csv:      One table: the summary, or the per-file rows with --per-file, or the most frequent elements with --top
          - markdown: Tables to read, or to paste into a paper
          
          [default: markdown]

  -o, --output <FILE>
          Write the statistics to FILE rather than to standard output

  -r, --recursive
          Descend into the subdirectories of the given directories

      --per-file
          Report each birthmark file as well as each group. With -f csv this replaces the summary table.

  -t, --top <N>
          Report the N most frequent elements of each group. With -f csv this replaces the summary table.

  -h, --help
          Print help (see a summary with '-h')
```

### `run` command

```sh
Extract birthmarks and compare them in one command

Usage: oinkie run [OPTIONS] [FILES]...

Arguments:
  [FILES]...
          Path to the JSON files

Options:
  -a, --analysis <ANALYSIS>
          Analysis to run, as '{birthmark}-{algorithm}' -- for example 'op-set-jaccard' or 'op-3gram-freq-cosine'.
          Run 'oinkie info' for the birthmarks and the algorithms they pair with. Any k
          parses in a k-gram name, not only the ones listed.
          
          [default: op-set-jaccard]

  -s, --strategy <STRATEGY>
          Pairing strategy for file comparisons

          Possible values:
          - all-and-self:    All possible combinations including self-comparisons ($_nC_2 + n$). Used for full matrix visualization or comprehensive heatmaps
          - all:             Compares all possible combinations ($_nC_2$). Used for comprehensive validation of accuracy (False Positive / True Positive)
          - self-coverage:   Compares each file with itself ($n$). Used for sanity checks to ensure identical files yield a similarity score of 1.0
          - adjacent:        Compares only adjacent pairs in the list ($n-1$). Useful for comparing sequential versions (e.g., v1.0 vs v1.1, v1.1 vs v1.2)
          - first-vs-others: Compares a specific reference file against all other files ($n-1$). Compares first item and all other items. Useful for comparing a baseline version against multiple variants
          - last-vs-others:  Compares a specific reference file against all other files ($n-1$). Compares the last item and all other items. Useful for comparing a baseline version against multiple variants
          
          [default: all-and-self]

  -d, --dest <DEST>
          Destination path for the output CSV file (default: 'similarities' directory
          
          [default: similarities]

  -A, --aggregator <METHOD>
          Specify the aggregator for combining the function similarities of a pair into its similarity.
          Available:
          - hungarian  Use the Hungarian algorithm to find the optimal one-to-one matching between the functions of
                       two birthmarks, maximizing the total similarity.
          - topn:N     Take each function's best match in the other birthmark, and average the N best of those from
                       each side. Matches need not be one-to-one. This can reduce noise from less relevant matches
                       and focus on the most significant similarities. available topn:N or topn:all (same as topn).
          
          [default: hungarian]

  -S, --skip
          Skip if the similarity file already exists for the pair of birthmarks

  -h, --help
          Print help (see a summary with '-h')
```

### `mcp` command

Serves the pipeline over the Model Context Protocol, on stdin and stdout, so an
agent can drive it. Behind the `mcp` cargo feature; the released binaries and
both container images are built with it.

```sh
Serve oinkie over the Model Context Protocol, on stdin and stdout

Usage: oinkie mcp [OPTIONS]

Options:
  -r, --root <DIRECTORY>  Directory the tools may read and write under. May be given more than once.
                          Every path a tool is handed, input and output alike, has to resolve inside one of
                          these; anything else is refused. Defaults to the working directory.
                          The paths the tools receive are written by a language model rather than by you,
                          which is the whole reason this exists.
  -h, --help              Print help
```

The tools it offers, how to configure a client, and why lifting is not among
them are in [`mcp/README.md`](mcp/README.md).
