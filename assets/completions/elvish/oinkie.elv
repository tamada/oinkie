
use builtin;
use str;

set edit:completion:arg-completer[oinkie] = {|@words|
    fn spaces {|n|
        builtin:repeat $n ' ' | str:join ''
    }
    fn cand {|text desc|
        edit:complex-candidate $text &display=$text' '(spaces (- 14 (wcswidth $text)))$desc
    }
    var command = 'oinkie'
    for word $words[1..-1] {
        if (str:has-prefix $word '-') {
            break
        }
        set command = $command';'$word
    }
    var completions = [
        &'oinkie'= {
            cand -l 'Log level for the application'
            cand --level 'Log level for the application'
            cand --no-progress 'Draw no progress bars, as when stderr is kept in a log or a CI output'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
            cand -V 'Print version'
            cand --version 'Print version'
            cand info 'Display information about the application'
            cand lift 'Lift binary files to JSON files of an intermediate representation, using a specified lifter'
            cand extract 'Extract birthmarks from a lifted binary file (JSON format)'
            cand compare 'Compare birthmarks and output the similarity score'
            cand review 'Re-read a finished comparison: recompute the similarity of each pair from the stored function similarities'
            cand stats 'Summarise a set of birthmarks: how many of each type, how many functions each holds, and how long each function''s birthmark is'
            cand run 'Extract birthmarks and compare them in one command'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'oinkie;info'= {
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'oinkie;lift'= {
            cand -d 'Specify the directory for putting the resultant JSON files of the lifted programs (default: ''./pcodes'' directory)'
            cand --dest 'Specify the directory for putting the resultant JSON files of the lifted programs (default: ''./pcodes'' directory)'
            cand -r 'Intermediate representation to produce. This also picks the tool, since a representation is only produced by one of them; several representations can come from the same tool.'
            cand --ir 'Intermediate representation to produce. This also picks the tool, since a representation is only produced by one of them; several representations can come from the same tool.'
            cand -H 'Path to the installation directory of the tool behind --ir. If not specified, that tool''s own environment variable (GHIDRA_HOME for Ghidra) is read, then the usual install locations are searched. The error names which variable to set.'
            cand --home 'Path to the installation directory of the tool behind --ir. If not specified, that tool''s own environment variable (GHIDRA_HOME for Ghidra) is read, then the usual install locations are searched. The error names which variable to set.'
            cand -i 'Directory for the lifter to work in, kept rather than discarded. Every lifter runs in one, since that is where its script writes; Ghidra also keeps its project there. If not specified, a temporary directory is used and deleted.'
            cand --intermediate 'Directory for the lifter to work in, kept rather than discarded. Every lifter runs in one, since that is where its script writes; Ghidra also keeps its project there. If not specified, a temporary directory is used and deleted.'
            cand --script 'Path to a custom lifting script, replacing the built-in one. The language is that of the tool behind --ir: Java for Ghidra. It must write {input file name}.json into its working directory.'
            cand -j 'Lift up to N files at a time (default: 1, one after another). Lifting runs a whole decompiler process per file, and several of them against a Ghidra installation whose language cache has not been built yet can corrupt it, so parallelism is opt-in.'
            cand --jobs 'Lift up to N files at a time (default: 1, one after another). Lifting runs a whole decompiler process per file, and several of them against a Ghidra installation whose language cache has not been built yet can corrupt it, so parallelism is opt-in.'
            cand -S 'Skip if the resultant JSON file already exists'
            cand --skip 'Skip if the resultant JSON file already exists'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'oinkie;extract'= {
            cand -d 'Specify the directory for putting the resultant JSON files for the extracted birthmarks (default: ''./birthmarks'' directory)'
            cand --dest 'Specify the directory for putting the resultant JSON files for the extracted birthmarks (default: ''./birthmarks'' directory)'
            cand -b 'Type of birthmark to extract, such as ''op-seq'' or ''fc-freq''; ''oinkie info'' lists them'
            cand --birthmark-type 'Type of birthmark to extract, such as ''op-seq'' or ''fc-freq''; ''oinkie info'' lists them'
            cand -S 'Skip the resultant birthmark file is already exists'
            cand --skip 'Skip the resultant birthmark file is already exists'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'oinkie;compare'= {
            cand -a 'Specify the similarity calculation algorithm.'
            cand --algorithm 'Specify the similarity calculation algorithm.'
            cand -A 'Aggregator combining a pair''s function similarities into its similarity: hungarian, topn:N, containment, matched:T or weighted'
            cand --aggregator 'Aggregator combining a pair''s function similarities into its similarity: hungarian, topn:N, containment, matched:T or weighted'
            cand -s 'Pairing strategy for comparing files'
            cand --strategy 'Pairing strategy for comparing files'
            cand -d 'Destination directory for the results'
            cand --dest 'Destination directory for the results'
            cand -S 'Skip if the similarity file already exists for the pair of birthmarks'
            cand --skip 'Skip if the similarity file already exists for the pair of birthmarks'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'oinkie;review'= {
            cand -A 'Aggregator combining a pair''s function similarities into its similarity: hungarian, topn:N, containment, matched:T or weighted'
            cand --aggregator 'Aggregator combining a pair''s function similarities into its similarity: hungarian, topn:N, containment, matched:T or weighted'
            cand -d 'Specify the result CSV file of the comparing results to review. The file lists the similarity of each pair.'
            cand --dest-file 'Specify the result CSV file of the comparing results to review. The file lists the similarity of each pair.'
            cand --min-elements 'Drop the functions with fewer elements than this before aggregating: N elements, or R times the mean (Rx)'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'oinkie;stats'= {
            cand -f 'Output format'
            cand --format 'Output format'
            cand -o 'Write the statistics to FILE rather than to standard output'
            cand --output 'Write the statistics to FILE rather than to standard output'
            cand -t 'Report the N most frequent elements of each group. With -f csv this replaces the summary table.'
            cand --top 'Report the N most frequent elements of each group. With -f csv this replaces the summary table.'
            cand -r 'Descend into the subdirectories of the given directories'
            cand --recursive 'Descend into the subdirectories of the given directories'
            cand --per-file 'Report each birthmark file as well as each group. With -f csv this replaces the summary table.'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'oinkie;run'= {
            cand -a 'Analysis to run, as ''{birthmark}-{algorithm}'' -- for example ''op-set-jaccard'' or ''op-3gram-freq-cosine'''
            cand --analysis 'Analysis to run, as ''{birthmark}-{algorithm}'' -- for example ''op-set-jaccard'' or ''op-3gram-freq-cosine'''
            cand -s 'Pairing strategy for file comparisons'
            cand --strategy 'Pairing strategy for file comparisons'
            cand -d 'Destination path for the output CSV file (default: ''similarities'' directory'
            cand --dest 'Destination path for the output CSV file (default: ''similarities'' directory'
            cand -A 'Aggregator combining a pair''s function similarities into its similarity: hungarian, topn:N, containment, matched:T or weighted'
            cand --aggregator 'Aggregator combining a pair''s function similarities into its similarity: hungarian, topn:N, containment, matched:T or weighted'
            cand -S 'Skip if the similarity file already exists for the pair of birthmarks'
            cand --skip 'Skip if the similarity file already exists for the pair of birthmarks'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'oinkie;help'= {
            cand info 'Display information about the application'
            cand lift 'Lift binary files to JSON files of an intermediate representation, using a specified lifter'
            cand extract 'Extract birthmarks from a lifted binary file (JSON format)'
            cand compare 'Compare birthmarks and output the similarity score'
            cand review 'Re-read a finished comparison: recompute the similarity of each pair from the stored function similarities'
            cand stats 'Summarise a set of birthmarks: how many of each type, how many functions each holds, and how long each function''s birthmark is'
            cand run 'Extract birthmarks and compare them in one command'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'oinkie;help;info'= {
        }
        &'oinkie;help;lift'= {
        }
        &'oinkie;help;extract'= {
        }
        &'oinkie;help;compare'= {
        }
        &'oinkie;help;review'= {
        }
        &'oinkie;help;stats'= {
        }
        &'oinkie;help;run'= {
        }
        &'oinkie;help;help'= {
        }
    ]
    $completions[$command]
}
