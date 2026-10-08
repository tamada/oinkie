# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_oinkie_global_optspecs
    string join \n l/level= no-progress h/help V/version
end

function __fish_oinkie_needs_command
    # Figure out if the current invocation already has a command.
    set -l cmd (commandline -opc)
    set -e cmd[1]
    argparse -s (__fish_oinkie_global_optspecs) -- $cmd 2>/dev/null
    or return
    if set -q argv[1]
        # Also print the command, so this can be used to figure out what it is.
        echo $argv[1]
        return 1
    end
    return 0
end

function __fish_oinkie_using_subcommand
    set -l cmd (__fish_oinkie_needs_command)
    test -z "$cmd"
    and return 1
    contains -- $cmd[1] $argv
end

# The aggregators. topn:N and matched:T take any count or threshold; a few
# are offered as examples.
function __fish_oinkie_aggregators
    printf '%s\t%s\n' \
        hungarian 'one-to-one matching that maximises the total (the default)' \
        topn:all "each function's best match, averaged over all of them" \
        topn:1 'the single best match from each side' \
        topn:3 'the 3 best matches from each side' \
        topn:5 'the 5 best matches from each side' \
        containment 'hungarian, averaged over the smaller side' \
        matched:0.8 'share of functions matched at 0.8 or above' \
        matched:0.9 'share of functions matched at 0.9 or above' \
        matched:1 'share of functions matched exactly' \
        weighted 'hungarian, each function counting by its size'
end

# --min-elements: a count, or a ratio of the mean followed by x. A fraction
# without the x is refused, so one typed is completed with it.
function __fish_oinkie_min_elements
    set -l token (commandline -ct)
    if test -z "$token"
        printf '%s\n' 1 2 3 5 10 0.1x 0.3x 0.5x 1x
    else if string match -qr '^[0-9]+$' -- $token
        printf '%s\n' $token {$token}x
    else if string match -qr '^[0-9]*\.[0-9]*$' -- $token
        echo {$token}x
    end
end

complete -c oinkie -n "__fish_oinkie_needs_command" -s l -l level -d 'Log level for the application' -r -f -a "error\t''
warn\t''
info\t''
debug\t''
trace\t''
off\t''"
complete -c oinkie -n "__fish_oinkie_needs_command" -l no-progress -d 'Draw no progress bars, as when stderr is kept in a log or a CI output'
complete -c oinkie -n "__fish_oinkie_needs_command" -s h -l help -d 'Print help'
complete -c oinkie -n "__fish_oinkie_needs_command" -s V -l version -d 'Print version'
complete -c oinkie -n "__fish_oinkie_needs_command" -f -a "info" -d 'Display information about the application'
complete -c oinkie -n "__fish_oinkie_needs_command" -f -a "lift" -d 'Lift binary files to JSON files of an intermediate representation, using a specified lifter'
complete -c oinkie -n "__fish_oinkie_needs_command" -f -a "extract" -d 'Extract birthmarks from a lifted binary file (JSON format)'
complete -c oinkie -n "__fish_oinkie_needs_command" -f -a "compare" -d 'Compare birthmarks and output the similarity score'
complete -c oinkie -n "__fish_oinkie_needs_command" -f -a "review" -d 'Re-read a finished comparison: recompute the similarity of each pair from the stored function similarities'
complete -c oinkie -n "__fish_oinkie_needs_command" -f -a "stats" -d 'Summarise a set of birthmarks: how many of each type, how many functions each holds, and how long each function\'s birthmark is'
complete -c oinkie -n "__fish_oinkie_needs_command" -f -a "run" -d 'Extract birthmarks and compare them in one command'
complete -c oinkie -n "__fish_oinkie_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c oinkie -n "__fish_oinkie_using_subcommand info" -s h -l help -d 'Print help'
complete -c oinkie -n "__fish_oinkie_using_subcommand lift" -s d -l dest -d 'Specify the directory for putting the resultant JSON files of the lifted programs (default: \'./pcodes\' directory)' -r -f -a "(__fish_complete_directories)"
complete -c oinkie -n "__fish_oinkie_using_subcommand lift" -s r -l ir -d 'Intermediate representation to produce. This also picks the tool, since a representation is only produced by one of them; several representations can come from the same tool.' -r -f -a "ghidra-pcode\t'Ghidra\'s P-Code, as refined by the decompiler — what `HighFunction` yields, rather than raw lifted P-Code'
ida-microcode\t'The Hex-Rays microcode -- which runs IDA Pro\'s decompiler, so on an installation with a cloud decompiler each function is sent to Hex-Rays\' servers'
binary-ninja-llil\t'Binary Ninja\'s Low Level IL: one expression per machine instruction, registers and flags still explicit'
binary-ninja-mlil\t'Binary Ninja\'s Medium Level IL: stack and registers resolved into variables, calls carrying their parameters'
binary-ninja-hlil\t'Binary Ninja\'s High Level IL: control flow recovered, the level its decompiler output is rendered from'"
complete -c oinkie -n "__fish_oinkie_using_subcommand lift" -s H -l home -d 'Path to the installation directory of the tool behind --ir. If not specified, that tool\'s own environment variable (GHIDRA_HOME for Ghidra) is read, then the usual install locations are searched. The error names which variable to set.' -r -f -a "(__fish_complete_directories)"
complete -c oinkie -n "__fish_oinkie_using_subcommand lift" -s i -l intermediate -d 'Directory for the lifter to work in, kept rather than discarded. Every lifter runs in one, since that is where its script writes; Ghidra also keeps its project there. If not specified, a temporary directory is used and deleted.' -r -f -a "(__fish_complete_directories)"
complete -c oinkie -n "__fish_oinkie_using_subcommand lift" -l script -d 'Path to a custom lifting script, replacing the built-in one. The language is that of the tool behind --ir: Java for Ghidra. It must write {input file name}.json into its working directory.' -r -F
complete -c oinkie -n "__fish_oinkie_using_subcommand lift" -s j -l jobs -d 'Lift up to N files at a time (default: 1, one after another). Lifting runs a whole decompiler process per file, and several of them against a Ghidra installation whose language cache has not been built yet can corrupt it, so parallelism is opt-in.' -r -f -a "(seq 1 (getconf _NPROCESSORS_ONLN 2>/dev/null; or echo 8))"
complete -c oinkie -n "__fish_oinkie_using_subcommand lift" -s S -l skip -d 'Skip if the resultant JSON file already exists'
complete -c oinkie -n "__fish_oinkie_using_subcommand lift" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c oinkie -n "__fish_oinkie_using_subcommand extract" -s d -l dest -d 'Specify the directory for putting the resultant JSON files for the extracted birthmarks (default: \'./birthmarks\' directory)' -r -f -a "(__fish_complete_directories)"
complete -c oinkie -n "__fish_oinkie_using_subcommand extract" -s b -l birthmark-type -d 'Type of birthmark to extract, such as \'op-seq\' or \'fc-freq\'; \'oinkie info\' lists them' -r -f -a "fc-seq\t'the sequence of method calls in a program'
fc-freq\t'the frequency of method calls in a program'
fc-set\t'the set of method calls in a program'
op-seq\t'the sequence of operations in a program'
op-set\t'the set of operations in a program'
op-freq\t'the frequency of operations in a program'
op-1gram-seq\t'the sequence of 1-grams of operations in a program'
op-2gram-seq\t'the sequence of 2-grams of operations in a program'
op-3gram-seq\t'the sequence of 3-grams of operations in a program'
op-4gram-seq\t'the sequence of 4-grams of operations in a program'
op-5gram-seq\t'the sequence of 5-grams of operations in a program'
op-6gram-seq\t'the sequence of 6-grams of operations in a program'
op-7gram-seq\t'the sequence of 7-grams of operations in a program'
op-8gram-seq\t'the sequence of 8-grams of operations in a program'
op-1gram-freq\t'the frequency of 1-grams of operations in a program'
op-2gram-freq\t'the frequency of 2-grams of operations in a program'
op-3gram-freq\t'the frequency of 3-grams of operations in a program'
op-4gram-freq\t'the frequency of 4-grams of operations in a program'
op-5gram-freq\t'the frequency of 5-grams of operations in a program'
op-6gram-freq\t'the frequency of 6-grams of operations in a program'
op-7gram-freq\t'the frequency of 7-grams of operations in a program'
op-8gram-freq\t'the frequency of 8-grams of operations in a program'
op-1gram-set\t'the set of 1-grams of operations in a program'
op-2gram-set\t'the set of 2-grams of operations in a program'
op-3gram-set\t'the set of 3-grams of operations in a program'
op-4gram-set\t'the set of 4-grams of operations in a program'
op-5gram-set\t'the set of 5-grams of operations in a program'
op-6gram-set\t'the set of 6-grams of operations in a program'
op-7gram-set\t'the set of 7-grams of operations in a program'
op-8gram-set\t'the set of 8-grams of operations in a program'"
complete -c oinkie -n "__fish_oinkie_using_subcommand extract" -l threads -d 'Number of threads to compute on [default: one per core]' -r -f -a "(seq 1 (getconf _NPROCESSORS_ONLN 2>/dev/null; or echo 8))"
complete -c oinkie -n "__fish_oinkie_using_subcommand extract" -s S -l skip -d 'Skip the resultant birthmark file is already exists'
complete -c oinkie -n "__fish_oinkie_using_subcommand extract" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c oinkie -n "__fish_oinkie_using_subcommand compare" -s a -l algorithm -d 'Specify the similarity calculation algorithm.' -r -f -a "cosine\t'Cosine similarity based on term frequency vectors. Available: seq and freq'
dice\t'Dice coefficient. Available: seq, set and freq'
euclidean\t'Euclidean distance between term frequency vectors. Available: seq and freq'
jaccard\t'Jaccard index. Available: seq, set and freq'
jensen-shannon\t'Jensen-Shannon divergence between element distributions, as 1 - sqrt(JSD). Available: seq and freq'
levenshtein\t'Levenshtein distance. Available: seq'
lcs\t'Longest Common Subsequence (LCS). Available: seq'
simpson\t'Simpson\'s coefficient. Available: seq, set and freq'
tanimoto\t'Tanimoto coefficient over term frequency vectors. Available: seq and freq'
weighted-jaccard\t'Weighted Jaccard index based on term frequency vectors. Available: seq and freq'"
complete -c oinkie -n "__fish_oinkie_using_subcommand compare" -s A -l aggregator -d 'Aggregator combining a pair\'s function similarities into its similarity: hungarian, topn:N, containment, matched:T or weighted' -r -f -a "(__fish_oinkie_aggregators)"
complete -c oinkie -n "__fish_oinkie_using_subcommand compare" -s s -l strategy -d 'Pairing strategy for comparing files' -r -f -a "all-and-self\t'All possible combinations including self-comparisons ($_nC_2 + n$). Used for full matrix visualization or comprehensive heatmaps'
all\t'Compares all possible combinations ($_nC_2$). Used for comprehensive validation of accuracy (False Positive / True Positive)'
self-coverage\t'Compares each file with itself ($n$). Used for sanity checks to ensure identical files yield a similarity score of 1.0'
adjacent\t'Compares only adjacent pairs in the list ($n-1$). Useful for comparing sequential versions (e.g., v1.0 vs v1.1, v1.1 vs v1.2)'
first-vs-others\t'Compares a specific reference file against all other files ($n-1$). Compares first item and all other items. Useful for comparing a baseline version against multiple variants'
last-vs-others\t'Compares a specific reference file against all other files ($n-1$). Compares the last item and all other items. Useful for comparing a baseline version against multiple variants'"
complete -c oinkie -n "__fish_oinkie_using_subcommand compare" -s d -l dest -d 'Destination directory for the results' -r -f -a "(__fish_complete_directories)"
complete -c oinkie -n "__fish_oinkie_using_subcommand compare" -l threads -d 'Number of threads to compute on [default: one per core]' -r -f -a "(seq 1 (getconf _NPROCESSORS_ONLN 2>/dev/null; or echo 8))"
complete -c oinkie -n "__fish_oinkie_using_subcommand compare" -s S -l skip -d 'Skip if the similarity file already exists for the pair of birthmarks'
complete -c oinkie -n "__fish_oinkie_using_subcommand compare" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c oinkie -n "__fish_oinkie_using_subcommand review" -s A -l aggregator -d 'Aggregator combining a pair\'s function similarities into its similarity: hungarian, topn:N, containment, matched:T or weighted' -r -f -a "(__fish_oinkie_aggregators)"
complete -c oinkie -n "__fish_oinkie_using_subcommand review" -s d -l dest-file -d 'Specify the result CSV file of the comparing results to review. The file lists the similarity of each pair.' -r -f -a "(__fish_complete_suffix .csv)"
complete -c oinkie -n "__fish_oinkie_using_subcommand review" -l min-elements -d 'Drop the functions with fewer elements than this before aggregating: N elements, or R times the mean (Rx)' -r -f -a "(__fish_oinkie_min_elements)"
complete -c oinkie -n "__fish_oinkie_using_subcommand review" -l threads -d 'Number of threads to compute on [default: one per core]' -r -f -a "(seq 1 (getconf _NPROCESSORS_ONLN 2>/dev/null; or echo 8))"
complete -c oinkie -n "__fish_oinkie_using_subcommand review" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c oinkie -n "__fish_oinkie_using_subcommand stats" -s f -l format -d 'Output format' -r -f -a "json\t'Every table at once: the summary, the per-file rows and the skipped files'
csv\t'One table: the summary, or the per-file rows with --per-file, or the most frequent elements with --top'
markdown\t'Tables to read, or to paste into a paper'"
complete -c oinkie -n "__fish_oinkie_using_subcommand stats" -s o -l output -d 'Write the statistics to FILE rather than to standard output' -r -F
complete -c oinkie -n "__fish_oinkie_using_subcommand stats" -s t -l top -d 'Report the N most frequent elements of each group. With -f csv this replaces the summary table.' -r -f
complete -c oinkie -n "__fish_oinkie_using_subcommand stats" -l threads -d 'Number of threads to compute on [default: one per core]' -r -f -a "(seq 1 (getconf _NPROCESSORS_ONLN 2>/dev/null; or echo 8))"
complete -c oinkie -n "__fish_oinkie_using_subcommand stats" -s r -l recursive -d 'Descend into the subdirectories of the given directories'
complete -c oinkie -n "__fish_oinkie_using_subcommand stats" -l per-file -d 'Report each birthmark file as well as each group. With -f csv this replaces the summary table.'
complete -c oinkie -n "__fish_oinkie_using_subcommand stats" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c oinkie -n "__fish_oinkie_using_subcommand run" -s a -l analysis -d 'Analysis to run, as \'{birthmark}-{algorithm}\' -- for example \'op-set-jaccard\' or \'op-3gram-freq-cosine\'' -r -f -a "fc-seq-levenshtein\t''
fc-seq-lcs\t''
fc-freq-cosine\t''
fc-freq-euclidean\t''
fc-freq-jensenshannon\t''
fc-freq-tanimoto\t''
fc-freq-weightedjaccard\t''
fc-set-dice\t''
fc-set-jaccard\t''
fc-set-simpson\t''
op-seq-levenshtein\t''
op-seq-lcs\t''
op-set-dice\t''
op-set-jaccard\t''
op-set-simpson\t''
op-freq-cosine\t''
op-freq-euclidean\t''
op-freq-jensenshannon\t''
op-freq-tanimoto\t''
op-freq-weightedjaccard\t''
op-1gram-seq-levenshtein\t''
op-1gram-seq-lcs\t''
op-2gram-seq-levenshtein\t''
op-2gram-seq-lcs\t''
op-3gram-seq-levenshtein\t''
op-3gram-seq-lcs\t''
op-4gram-seq-levenshtein\t''
op-4gram-seq-lcs\t''
op-5gram-seq-levenshtein\t''
op-5gram-seq-lcs\t''
op-6gram-seq-levenshtein\t''
op-6gram-seq-lcs\t''
op-7gram-seq-levenshtein\t''
op-7gram-seq-lcs\t''
op-8gram-seq-levenshtein\t''
op-8gram-seq-lcs\t''
op-1gram-freq-cosine\t''
op-1gram-freq-euclidean\t''
op-1gram-freq-jensenshannon\t''
op-1gram-freq-tanimoto\t''
op-1gram-freq-weightedjaccard\t''
op-2gram-freq-cosine\t''
op-2gram-freq-euclidean\t''
op-2gram-freq-jensenshannon\t''
op-2gram-freq-tanimoto\t''
op-2gram-freq-weightedjaccard\t''
op-3gram-freq-cosine\t''
op-3gram-freq-euclidean\t''
op-3gram-freq-jensenshannon\t''
op-3gram-freq-tanimoto\t''
op-3gram-freq-weightedjaccard\t''
op-4gram-freq-cosine\t''
op-4gram-freq-euclidean\t''
op-4gram-freq-jensenshannon\t''
op-4gram-freq-tanimoto\t''
op-4gram-freq-weightedjaccard\t''
op-5gram-freq-cosine\t''
op-5gram-freq-euclidean\t''
op-5gram-freq-jensenshannon\t''
op-5gram-freq-tanimoto\t''
op-5gram-freq-weightedjaccard\t''
op-6gram-freq-cosine\t''
op-6gram-freq-euclidean\t''
op-6gram-freq-jensenshannon\t''
op-6gram-freq-tanimoto\t''
op-6gram-freq-weightedjaccard\t''
op-7gram-freq-cosine\t''
op-7gram-freq-euclidean\t''
op-7gram-freq-jensenshannon\t''
op-7gram-freq-tanimoto\t''
op-7gram-freq-weightedjaccard\t''
op-8gram-freq-cosine\t''
op-8gram-freq-euclidean\t''
op-8gram-freq-jensenshannon\t''
op-8gram-freq-tanimoto\t''
op-8gram-freq-weightedjaccard\t''
op-1gram-set-dice\t''
op-1gram-set-jaccard\t''
op-1gram-set-simpson\t''
op-2gram-set-dice\t''
op-2gram-set-jaccard\t''
op-2gram-set-simpson\t''
op-3gram-set-dice\t''
op-3gram-set-jaccard\t''
op-3gram-set-simpson\t''
op-4gram-set-dice\t''
op-4gram-set-jaccard\t''
op-4gram-set-simpson\t''
op-5gram-set-dice\t''
op-5gram-set-jaccard\t''
op-5gram-set-simpson\t''
op-6gram-set-dice\t''
op-6gram-set-jaccard\t''
op-6gram-set-simpson\t''
op-7gram-set-dice\t''
op-7gram-set-jaccard\t''
op-7gram-set-simpson\t''
op-8gram-set-dice\t''
op-8gram-set-jaccard\t''
op-8gram-set-simpson\t''"
complete -c oinkie -n "__fish_oinkie_using_subcommand run" -s s -l strategy -d 'Pairing strategy for file comparisons' -r -f -a "all-and-self\t'All possible combinations including self-comparisons ($_nC_2 + n$). Used for full matrix visualization or comprehensive heatmaps'
all\t'Compares all possible combinations ($_nC_2$). Used for comprehensive validation of accuracy (False Positive / True Positive)'
self-coverage\t'Compares each file with itself ($n$). Used for sanity checks to ensure identical files yield a similarity score of 1.0'
adjacent\t'Compares only adjacent pairs in the list ($n-1$). Useful for comparing sequential versions (e.g., v1.0 vs v1.1, v1.1 vs v1.2)'
first-vs-others\t'Compares a specific reference file against all other files ($n-1$). Compares first item and all other items. Useful for comparing a baseline version against multiple variants'
last-vs-others\t'Compares a specific reference file against all other files ($n-1$). Compares the last item and all other items. Useful for comparing a baseline version against multiple variants'"
complete -c oinkie -n "__fish_oinkie_using_subcommand run" -s d -l dest -d 'Destination path for the output CSV file (default: \'similarities\' directory' -r -f -a "(__fish_complete_directories)"
complete -c oinkie -n "__fish_oinkie_using_subcommand run" -s A -l aggregator -d 'Aggregator combining a pair\'s function similarities into its similarity: hungarian, topn:N, containment, matched:T or weighted' -r -f -a "(__fish_oinkie_aggregators)"
complete -c oinkie -n "__fish_oinkie_using_subcommand run" -l threads -d 'Number of threads to compute on [default: one per core]' -r -f -a "(seq 1 (getconf _NPROCESSORS_ONLN 2>/dev/null; or echo 8))"
complete -c oinkie -n "__fish_oinkie_using_subcommand run" -s S -l skip -d 'Skip if the similarity file already exists for the pair of birthmarks'
complete -c oinkie -n "__fish_oinkie_using_subcommand run" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c oinkie -n "__fish_oinkie_using_subcommand help; and not __fish_seen_subcommand_from info lift extract compare review reaggregate stats run help" -f -a "info" -d 'Display information about the application'
complete -c oinkie -n "__fish_oinkie_using_subcommand help; and not __fish_seen_subcommand_from info lift extract compare review reaggregate stats run help" -f -a "lift" -d 'Lift binary files to JSON files of an intermediate representation, using a specified lifter'
complete -c oinkie -n "__fish_oinkie_using_subcommand help; and not __fish_seen_subcommand_from info lift extract compare review reaggregate stats run help" -f -a "extract" -d 'Extract birthmarks from a lifted binary file (JSON format)'
complete -c oinkie -n "__fish_oinkie_using_subcommand help; and not __fish_seen_subcommand_from info lift extract compare review reaggregate stats run help" -f -a "compare" -d 'Compare birthmarks and output the similarity score'
complete -c oinkie -n "__fish_oinkie_using_subcommand help; and not __fish_seen_subcommand_from info lift extract compare review reaggregate stats run help" -f -a "review" -d 'Re-read a finished comparison: recompute the similarity of each pair from the stored function similarities'
complete -c oinkie -n "__fish_oinkie_using_subcommand help; and not __fish_seen_subcommand_from info lift extract compare review reaggregate stats run help" -f -a "reaggregate" -d 'The old name of `review`, kept only to say so. Hidden from help and completion; remove it in the next minor'
complete -c oinkie -n "__fish_oinkie_using_subcommand help; and not __fish_seen_subcommand_from info lift extract compare review reaggregate stats run help" -f -a "stats" -d 'Summarise a set of birthmarks: how many of each type, how many functions each holds, and how long each function\'s birthmark is'
complete -c oinkie -n "__fish_oinkie_using_subcommand help; and not __fish_seen_subcommand_from info lift extract compare review reaggregate stats run help" -f -a "run" -d 'Extract birthmarks and compare them in one command'
complete -c oinkie -n "__fish_oinkie_using_subcommand help; and not __fish_seen_subcommand_from info lift extract compare review reaggregate stats run help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'

# Positional arguments: the files and directories each command takes.
complete -c oinkie -n "__fish_oinkie_using_subcommand extract compare run stats" -f -a "(__fish_complete_suffix .json)"
complete -c oinkie -n "__fish_oinkie_using_subcommand review" -f -a "(__fish_complete_directories)"
