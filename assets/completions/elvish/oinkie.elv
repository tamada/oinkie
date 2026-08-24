
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
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
            cand info 'Display information about the application'
            cand lift 'Lift binary files to P-code JSON files using a specified lifter'
            cand extract 'Extract birthmarks from a lifted binary file (JSON format)'
            cand compare 'Compare birthmarks and output the similarity score'
            cand reaggregate 'Reaggregate the element-wise similarity scores and recalculate the birthmark-wise similarity score'
            cand run 'Extract birthmarks and compare them in one command'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'oinkie;info'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'oinkie;lift'= {
            cand -d 'Specify the directory for putting the resultant JSON files for the lifted P-code (default: ''./pcodes'' directory)'
            cand --dest 'Specify the directory for putting the resultant JSON files for the lifted P-code (default: ''./pcodes'' directory)'
            cand -l 'Specify the lifter type'
            cand --lifter-type 'Specify the lifter type'
            cand -H 'Specify the path to the home directory of the lifter (e.g., GHIDRA_HOME for Ghidra). If not specified, the environment variable (e.g., GHIDRA_HOME) or default paths are searched.'
            cand --home 'Specify the path to the home directory of the lifter (e.g., GHIDRA_HOME for Ghidra). If not specified, the environment variable (e.g., GHIDRA_HOME) or default paths are searched.'
            cand -i 'Directory to keep intermediate files like Ghidra project directories. If not specified, a temporary directory is used and deleted.'
            cand --intermediate 'Directory to keep intermediate files like Ghidra project directories. If not specified, a temporary directory is used and deleted.'
            cand --script 'Path to a custom lifting script. Interpretation depends on the lifter type. For Ghidra, it''s the path to a Java script.'
            cand -S 'Skip if the resultant JSON file already exists'
            cand --skip 'Skip if the resultant JSON file already exists'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'oinkie;extract'= {
            cand -d 'Specify the directory for putting the resultant JSON files for the extracted birthmarks (default: ''./birthmarks'' directory)'
            cand --dest 'Specify the directory for putting the resultant JSON files for the extracted birthmarks (default: ''./birthmarks'' directory)'
            cand -b 'Type of birthmark to extract. fc (Function Calls) and op (Opcode) with set, seq, and freq variants are supported. For example, ''op-seq'' extracts the sequence of operations as a birthmark, while ''fc-freq'' extracts the frequency of function calls. The full birthmark types cann be found by running ''oinkie info''.'
            cand --birthmark-type 'Type of birthmark to extract. fc (Function Calls) and op (Opcode) with set, seq, and freq variants are supported. For example, ''op-seq'' extracts the sequence of operations as a birthmark, while ''fc-freq'' extracts the frequency of function calls. The full birthmark types cann be found by running ''oinkie info''.'
            cand -S 'Skip the resultant birthmark file is already exists'
            cand --skip 'Skip the resultant birthmark file is already exists'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'oinkie;compare'= {
            cand -a 'Specify the similarity calculation algorithm.'
            cand --algorithm 'Specify the similarity calculation algorithm.'
            cand -A 'Specify the aggregator for combining element-wise similarity scores into a birthmark-wise similarity score. Available: - hungarian  Use the Hungarian algorithm to find the optimal matching between elements of two birthmarks,              maximizing the total similarity score. - topn:N     For each element in the first birthmark, consider only the top N most similar elements in the              second birthmark when calculating the overall similarity score. This can reduce noise from less              relevant matches and focus on the most significant similarities.'
            cand --aggregator 'Specify the aggregator for combining element-wise similarity scores into a birthmark-wise similarity score. Available: - hungarian  Use the Hungarian algorithm to find the optimal matching between elements of two birthmarks,              maximizing the total similarity score. - topn:N     For each element in the first birthmark, consider only the top N most similar elements in the              second birthmark when calculating the overall similarity score. This can reduce noise from less              relevant matches and focus on the most significant similarities.'
            cand -s 'Specify the pairing strategy for comparing files.'
            cand --strategy 'Specify the pairing strategy for comparing files.'
            cand -d 'Specify the destination directory for the comparing results'
            cand --dest 'Specify the destination directory for the comparing results'
            cand -S 'Skip if the similarity file already exists for the pair of birthmarks'
            cand --skip 'Skip if the similarity file already exists for the pair of birthmarks'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'oinkie;reaggregate'= {
            cand -A 'Specify the aggregator for combining element-wise similarity scores into a birthmark-wise similarity score. Available: - hungarian  Use the Hungarian algorithm to find the optimal matching between elements of two birthmarks,              maximizing the total similarity score. - topn:N     For each element in the first birthmark, consider only the top N most similar elements in the              second birthmark when calculating the overall similarity score. This can reduce noise from less              relevant matches and focus on the most significant similarities.'
            cand --aggregator 'Specify the aggregator for combining element-wise similarity scores into a birthmark-wise similarity score. Available: - hungarian  Use the Hungarian algorithm to find the optimal matching between elements of two birthmarks,              maximizing the total similarity score. - topn:N     For each element in the first birthmark, consider only the top N most similar elements in the              second birthmark when calculating the overall similarity score. This can reduce noise from less              relevant matches and focus on the most significant similarities.'
            cand -d 'Specify the result CSV file of the comparing results to reaggregate. The file contains the birthmark-wise similarity score list.'
            cand --dest-file 'Specify the result CSV file of the comparing results to reaggregate. The file contains the birthmark-wise similarity score list.'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'oinkie;run'= {
            cand -a 'Similarity algorithm to use'
            cand --analysis 'Similarity algorithm to use'
            cand -s 'Pairing strategy for file comparisons'
            cand --strategy 'Pairing strategy for file comparisons'
            cand -d 'Destination path for the output CSV file (default: ''similarities'' directory'
            cand --dest 'Destination path for the output CSV file (default: ''similarities'' directory'
            cand -A 'Specify the aggregator for combining element-wise similarity scores into a birthmark-wise similarity score. Available: - hungarian  Use the Hungarian algorithm to find the optimal matching between elements of two birthmarks,              maximizing the total similarity score. - topn:N     For each element in the first birthmark, consider only the top N most similar elements in the              second birthmark when calculating the overall similarity score. This can reduce noise from less              relevant matches and focus on the most significant similarities. available topn:N or topn:all (same as topn).'
            cand --aggregator 'Specify the aggregator for combining element-wise similarity scores into a birthmark-wise similarity score. Available: - hungarian  Use the Hungarian algorithm to find the optimal matching between elements of two birthmarks,              maximizing the total similarity score. - topn:N     For each element in the first birthmark, consider only the top N most similar elements in the              second birthmark when calculating the overall similarity score. This can reduce noise from less              relevant matches and focus on the most significant similarities. available topn:N or topn:all (same as topn).'
            cand -S 'Skip if the similarity file already exists for the pair of birthmarks'
            cand --skip 'Skip if the similarity file already exists for the pair of birthmarks'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'oinkie;help'= {
            cand info 'Display information about the application'
            cand lift 'Lift binary files to P-code JSON files using a specified lifter'
            cand extract 'Extract birthmarks from a lifted binary file (JSON format)'
            cand compare 'Compare birthmarks and output the similarity score'
            cand reaggregate 'Reaggregate the element-wise similarity scores and recalculate the birthmark-wise similarity score'
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
        &'oinkie;help;reaggregate'= {
        }
        &'oinkie;help;run'= {
        }
        &'oinkie;help;help'= {
        }
    ]
    $completions[$command]
}
