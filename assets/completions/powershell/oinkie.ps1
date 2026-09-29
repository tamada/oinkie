
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'oinkie' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'oinkie'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-') -or
                $element.Value -eq $wordToComplete) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'oinkie' {
            [CompletionResult]::new('-l', '-l', [CompletionResultType]::ParameterName, 'Log level for the application')
            [CompletionResult]::new('--level', '--level', [CompletionResultType]::ParameterName, 'Log level for the application')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('info', 'info', [CompletionResultType]::ParameterValue, 'Display information about the application')
            [CompletionResult]::new('lift', 'lift', [CompletionResultType]::ParameterValue, 'Lift binary files to JSON files of an intermediate representation, using a specified lifter')
            [CompletionResult]::new('extract', 'extract', [CompletionResultType]::ParameterValue, 'Extract birthmarks from a lifted binary file (JSON format)')
            [CompletionResult]::new('compare', 'compare', [CompletionResultType]::ParameterValue, 'Compare birthmarks and output the similarity score')
            [CompletionResult]::new('reaggregate', 'reaggregate', [CompletionResultType]::ParameterValue, 'Reaggregate the element-wise similarity scores and recalculate the birthmark-wise similarity score')
            [CompletionResult]::new('stats', 'stats', [CompletionResultType]::ParameterValue, 'Summarise a set of birthmarks: how many of each type, how many functions each holds, and how long each function''s birthmark is')
            [CompletionResult]::new('run', 'run', [CompletionResultType]::ParameterValue, 'Extract birthmarks and compare them in one command')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'oinkie;info' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'oinkie;lift' {
            [CompletionResult]::new('-d', '-d', [CompletionResultType]::ParameterName, 'Specify the directory for putting the resultant JSON files of the lifted programs (default: ''./pcodes'' directory)')
            [CompletionResult]::new('--dest', '--dest', [CompletionResultType]::ParameterName, 'Specify the directory for putting the resultant JSON files of the lifted programs (default: ''./pcodes'' directory)')
            [CompletionResult]::new('-r', '-r', [CompletionResultType]::ParameterName, 'Intermediate representation to produce. This also picks the tool, since a representation is only produced by one of them; several representations can come from the same tool.')
            [CompletionResult]::new('--ir', '--ir', [CompletionResultType]::ParameterName, 'Intermediate representation to produce. This also picks the tool, since a representation is only produced by one of them; several representations can come from the same tool.')
            [CompletionResult]::new('-H', '-H ', [CompletionResultType]::ParameterName, 'Path to the installation directory of the tool behind --ir. If not specified, that tool''s own environment variable (GHIDRA_HOME for Ghidra) is read, then the usual install locations are searched. The error names which variable to set.')
            [CompletionResult]::new('--home', '--home', [CompletionResultType]::ParameterName, 'Path to the installation directory of the tool behind --ir. If not specified, that tool''s own environment variable (GHIDRA_HOME for Ghidra) is read, then the usual install locations are searched. The error names which variable to set.')
            [CompletionResult]::new('-i', '-i', [CompletionResultType]::ParameterName, 'Directory for the lifter to work in, kept rather than discarded. Every lifter runs in one, since that is where its script writes; Ghidra also keeps its project there. If not specified, a temporary directory is used and deleted.')
            [CompletionResult]::new('--intermediate', '--intermediate', [CompletionResultType]::ParameterName, 'Directory for the lifter to work in, kept rather than discarded. Every lifter runs in one, since that is where its script writes; Ghidra also keeps its project there. If not specified, a temporary directory is used and deleted.')
            [CompletionResult]::new('--script', '--script', [CompletionResultType]::ParameterName, 'Path to a custom lifting script, replacing the built-in one. The language is that of the tool behind --ir: Java for Ghidra. It must write {input file name}.json into its working directory.')
            [CompletionResult]::new('-j', '-j', [CompletionResultType]::ParameterName, 'Lift up to N files at a time (default: 1, one after another). Lifting runs a whole decompiler process per file, and several of them against a Ghidra installation whose language cache has not been built yet can corrupt it, so parallelism is opt-in.')
            [CompletionResult]::new('--jobs', '--jobs', [CompletionResultType]::ParameterName, 'Lift up to N files at a time (default: 1, one after another). Lifting runs a whole decompiler process per file, and several of them against a Ghidra installation whose language cache has not been built yet can corrupt it, so parallelism is opt-in.')
            [CompletionResult]::new('-S', '-S ', [CompletionResultType]::ParameterName, 'Skip if the resultant JSON file already exists')
            [CompletionResult]::new('--skip', '--skip', [CompletionResultType]::ParameterName, 'Skip if the resultant JSON file already exists')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'oinkie;extract' {
            [CompletionResult]::new('-d', '-d', [CompletionResultType]::ParameterName, 'Specify the directory for putting the resultant JSON files for the extracted birthmarks (default: ''./birthmarks'' directory)')
            [CompletionResult]::new('--dest', '--dest', [CompletionResultType]::ParameterName, 'Specify the directory for putting the resultant JSON files for the extracted birthmarks (default: ''./birthmarks'' directory)')
            [CompletionResult]::new('-b', '-b', [CompletionResultType]::ParameterName, 'Type of birthmark to extract. fc (Function Calls) and op (Opcode) with set, seq, and freq variants are supported. For example, ''op-seq'' extracts the sequence of operations as a birthmark, while ''fc-freq'' extracts the frequency of function calls. k-grams are written with the k in the name: ''op-3gram-set''. Any k parses, not only the ones ''oinkie info'' lists. The full birthmark types can be found by running ''oinkie info''.')
            [CompletionResult]::new('--birthmark-type', '--birthmark-type', [CompletionResultType]::ParameterName, 'Type of birthmark to extract. fc (Function Calls) and op (Opcode) with set, seq, and freq variants are supported. For example, ''op-seq'' extracts the sequence of operations as a birthmark, while ''fc-freq'' extracts the frequency of function calls. k-grams are written with the k in the name: ''op-3gram-set''. Any k parses, not only the ones ''oinkie info'' lists. The full birthmark types can be found by running ''oinkie info''.')
            [CompletionResult]::new('-S', '-S ', [CompletionResultType]::ParameterName, 'Skip the resultant birthmark file is already exists')
            [CompletionResult]::new('--skip', '--skip', [CompletionResultType]::ParameterName, 'Skip the resultant birthmark file is already exists')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'oinkie;compare' {
            [CompletionResult]::new('-a', '-a', [CompletionResultType]::ParameterName, 'Specify the similarity calculation algorithm.')
            [CompletionResult]::new('--algorithm', '--algorithm', [CompletionResultType]::ParameterName, 'Specify the similarity calculation algorithm.')
            [CompletionResult]::new('-A', '-A ', [CompletionResultType]::ParameterName, 'Specify the aggregator for combining element-wise similarity scores into a birthmark-wise similarity score. Available: - hungarian  Use the Hungarian algorithm to find the optimal matching between elements of two birthmarks,              maximizing the total similarity score. - topn:N     For each element in the first birthmark, consider only the top N most similar elements in the              second birthmark when calculating the overall similarity score. This can reduce noise from less              relevant matches and focus on the most significant similarities.')
            [CompletionResult]::new('--aggregator', '--aggregator', [CompletionResultType]::ParameterName, 'Specify the aggregator for combining element-wise similarity scores into a birthmark-wise similarity score. Available: - hungarian  Use the Hungarian algorithm to find the optimal matching between elements of two birthmarks,              maximizing the total similarity score. - topn:N     For each element in the first birthmark, consider only the top N most similar elements in the              second birthmark when calculating the overall similarity score. This can reduce noise from less              relevant matches and focus on the most significant similarities.')
            [CompletionResult]::new('-s', '-s', [CompletionResultType]::ParameterName, 'Specify the pairing strategy for comparing files.')
            [CompletionResult]::new('--strategy', '--strategy', [CompletionResultType]::ParameterName, 'Specify the pairing strategy for comparing files.')
            [CompletionResult]::new('-d', '-d', [CompletionResultType]::ParameterName, 'Specify the destination directory for the comparing results')
            [CompletionResult]::new('--dest', '--dest', [CompletionResultType]::ParameterName, 'Specify the destination directory for the comparing results')
            [CompletionResult]::new('-S', '-S ', [CompletionResultType]::ParameterName, 'Skip if the similarity file already exists for the pair of birthmarks')
            [CompletionResult]::new('--skip', '--skip', [CompletionResultType]::ParameterName, 'Skip if the similarity file already exists for the pair of birthmarks')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'oinkie;reaggregate' {
            [CompletionResult]::new('-A', '-A ', [CompletionResultType]::ParameterName, 'Specify the aggregator for combining element-wise similarity scores into a birthmark-wise similarity score. Available: - hungarian  Use the Hungarian algorithm to find the optimal matching between elements of two birthmarks,              maximizing the total similarity score. - topn:N     For each element in the first birthmark, consider only the top N most similar elements in the              second birthmark when calculating the overall similarity score. This can reduce noise from less              relevant matches and focus on the most significant similarities.')
            [CompletionResult]::new('--aggregator', '--aggregator', [CompletionResultType]::ParameterName, 'Specify the aggregator for combining element-wise similarity scores into a birthmark-wise similarity score. Available: - hungarian  Use the Hungarian algorithm to find the optimal matching between elements of two birthmarks,              maximizing the total similarity score. - topn:N     For each element in the first birthmark, consider only the top N most similar elements in the              second birthmark when calculating the overall similarity score. This can reduce noise from less              relevant matches and focus on the most significant similarities.')
            [CompletionResult]::new('-d', '-d', [CompletionResultType]::ParameterName, 'Specify the result CSV file of the comparing results to reaggregate. The file contains the birthmark-wise similarity score list.')
            [CompletionResult]::new('--dest-file', '--dest-file', [CompletionResultType]::ParameterName, 'Specify the result CSV file of the comparing results to reaggregate. The file contains the birthmark-wise similarity score list.')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'oinkie;stats' {
            [CompletionResult]::new('-f', '-f', [CompletionResultType]::ParameterName, 'Output format')
            [CompletionResult]::new('--format', '--format', [CompletionResultType]::ParameterName, 'Output format')
            [CompletionResult]::new('-o', '-o', [CompletionResultType]::ParameterName, 'Write the statistics to FILE rather than to standard output')
            [CompletionResult]::new('--output', '--output', [CompletionResultType]::ParameterName, 'Write the statistics to FILE rather than to standard output')
            [CompletionResult]::new('-t', '-t', [CompletionResultType]::ParameterName, 'Report the N most frequent elements of each group. With -f csv this replaces the summary table.')
            [CompletionResult]::new('--top', '--top', [CompletionResultType]::ParameterName, 'Report the N most frequent elements of each group. With -f csv this replaces the summary table.')
            [CompletionResult]::new('-r', '-r', [CompletionResultType]::ParameterName, 'Descend into the subdirectories of the given directories')
            [CompletionResult]::new('--recursive', '--recursive', [CompletionResultType]::ParameterName, 'Descend into the subdirectories of the given directories')
            [CompletionResult]::new('--per-file', '--per-file', [CompletionResultType]::ParameterName, 'Report each birthmark file as well as each group. With -f csv this replaces the summary table.')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'oinkie;run' {
            [CompletionResult]::new('-a', '-a', [CompletionResultType]::ParameterName, 'Analysis to run, as ''{birthmark}-{algorithm}'' -- for example ''op-set-jaccard'' or ''op-3gram-freq-cosine''. Run ''oinkie info'' for the birthmarks and the algorithms they pair with. Any k parses in a k-gram name, not only the ones listed.')
            [CompletionResult]::new('--analysis', '--analysis', [CompletionResultType]::ParameterName, 'Analysis to run, as ''{birthmark}-{algorithm}'' -- for example ''op-set-jaccard'' or ''op-3gram-freq-cosine''. Run ''oinkie info'' for the birthmarks and the algorithms they pair with. Any k parses in a k-gram name, not only the ones listed.')
            [CompletionResult]::new('-s', '-s', [CompletionResultType]::ParameterName, 'Pairing strategy for file comparisons')
            [CompletionResult]::new('--strategy', '--strategy', [CompletionResultType]::ParameterName, 'Pairing strategy for file comparisons')
            [CompletionResult]::new('-d', '-d', [CompletionResultType]::ParameterName, 'Destination path for the output CSV file (default: ''similarities'' directory')
            [CompletionResult]::new('--dest', '--dest', [CompletionResultType]::ParameterName, 'Destination path for the output CSV file (default: ''similarities'' directory')
            [CompletionResult]::new('-A', '-A ', [CompletionResultType]::ParameterName, 'Specify the aggregator for combining element-wise similarity scores into a birthmark-wise similarity score. Available: - hungarian  Use the Hungarian algorithm to find the optimal matching between elements of two birthmarks,              maximizing the total similarity score. - topn:N     For each element in the first birthmark, consider only the top N most similar elements in the              second birthmark when calculating the overall similarity score. This can reduce noise from less              relevant matches and focus on the most significant similarities. available topn:N or topn:all (same as topn).')
            [CompletionResult]::new('--aggregator', '--aggregator', [CompletionResultType]::ParameterName, 'Specify the aggregator for combining element-wise similarity scores into a birthmark-wise similarity score. Available: - hungarian  Use the Hungarian algorithm to find the optimal matching between elements of two birthmarks,              maximizing the total similarity score. - topn:N     For each element in the first birthmark, consider only the top N most similar elements in the              second birthmark when calculating the overall similarity score. This can reduce noise from less              relevant matches and focus on the most significant similarities. available topn:N or topn:all (same as topn).')
            [CompletionResult]::new('-S', '-S ', [CompletionResultType]::ParameterName, 'Skip if the similarity file already exists for the pair of birthmarks')
            [CompletionResult]::new('--skip', '--skip', [CompletionResultType]::ParameterName, 'Skip if the similarity file already exists for the pair of birthmarks')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'oinkie;help' {
            [CompletionResult]::new('info', 'info', [CompletionResultType]::ParameterValue, 'Display information about the application')
            [CompletionResult]::new('lift', 'lift', [CompletionResultType]::ParameterValue, 'Lift binary files to JSON files of an intermediate representation, using a specified lifter')
            [CompletionResult]::new('extract', 'extract', [CompletionResultType]::ParameterValue, 'Extract birthmarks from a lifted binary file (JSON format)')
            [CompletionResult]::new('compare', 'compare', [CompletionResultType]::ParameterValue, 'Compare birthmarks and output the similarity score')
            [CompletionResult]::new('reaggregate', 'reaggregate', [CompletionResultType]::ParameterValue, 'Reaggregate the element-wise similarity scores and recalculate the birthmark-wise similarity score')
            [CompletionResult]::new('stats', 'stats', [CompletionResultType]::ParameterValue, 'Summarise a set of birthmarks: how many of each type, how many functions each holds, and how long each function''s birthmark is')
            [CompletionResult]::new('run', 'run', [CompletionResultType]::ParameterValue, 'Extract birthmarks and compare them in one command')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'oinkie;help;info' {
            break
        }
        'oinkie;help;lift' {
            break
        }
        'oinkie;help;extract' {
            break
        }
        'oinkie;help;compare' {
            break
        }
        'oinkie;help;reaggregate' {
            break
        }
        'oinkie;help;stats' {
            break
        }
        'oinkie;help;run' {
            break
        }
        'oinkie;help;help' {
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
