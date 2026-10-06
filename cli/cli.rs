use std::path::{Path, PathBuf};
use std::str::FromStr;

pub use crate::values::Analysis;
use crate::values::{AnalysisParser, BirthmarkTypeParser};
use crate::vocabulary::{algorithm_parser, ir_parser, strategy_parser};
use clap::ValueEnum;
use oinkie::Result;
use oinkie::birthmarks::{AnalysisType, BirthmarkType};
use oinkie::compare::{Aggregator, Algorithm, Comparator, PairingStrategy};
use oinkie::extract::Extractor;
use oinkie::lift::Ir;

#[derive(Debug, clap::Parser)]
#[command(version, about)]
pub struct OinkieOpts {
    #[clap(subcommand)]
    pub command: OinkieCommand,

    #[clap(short, long, value_enum, default_value_t = LogLevel::Warn, value_name = "LEVEL", ignore_case = true, help = "Log level for the application")]
    pub level: LogLevel,
}

/// Separated from [`OinkieOpts::init`] so that it can be tested. `init`
/// itself calls `env_logger::Builder::init`, which panics if a logger is
/// already installed, so it can be called at most once per test binary --
/// which would leave five of these six arms unreachable from a test.
fn filter_level(level: &LogLevel) -> log::LevelFilter {
    match level {
        LogLevel::Debug => log::LevelFilter::Debug,
        LogLevel::Info => log::LevelFilter::Info,
        LogLevel::Warn => log::LevelFilter::Warn,
        LogLevel::Error => log::LevelFilter::Error,
        LogLevel::Trace => log::LevelFilter::Trace,
        LogLevel::Off => log::LevelFilter::Off,
    }
}

impl OinkieOpts {
    pub fn init(&self) -> Result<()> {
        let filter = filter_level(&self.level);
        env_logger::Builder::new().filter_level(filter).init();
        Ok(())
    }
}

#[derive(Debug, clap::Parser, ValueEnum, Clone)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
    Off,
}

#[derive(Debug, clap::Parser)]
pub enum OinkieCommand {
    #[command(name = "info", about = "Display information about the application")]
    Info,

    #[command(
        name = "lift",
        about = "Lift binary files to JSON files of an intermediate representation, using a specified lifter"
    )]
    Lift(LiftOpts),

    #[command(
        name = "extract",
        about = "Extract birthmarks from a lifted binary file (JSON format)"
    )]
    Extract(ExtractOpts),

    #[command(
        name = "compare",
        about = "Compare birthmarks and output the similarity score"
    )]
    Compare(CompareOpts),

    #[command(
        name = "review",
        about = "Re-read a finished comparison: recompute the similarity of each pair from the stored function similarities"
    )]
    Review(ReviewOpts),

    /// The old name of `review`, kept only to say so. Hidden from help and
    /// completion; remove it in the next minor.
    ///
    /// It takes whatever it is given, so that `oinkie reaggregate -A topn:3 ...`
    /// reaches the message instead of failing on an option it never declared.
    #[command(
        name = "reaggregate",
        hide = true,
        disable_help_flag = true,
        trailing_var_arg = true
    )]
    Reaggregate {
        #[clap(allow_hyphen_values = true, num_args = 0..)]
        args: Vec<String>,
    },

    #[command(
        name = "stats",
        about = "Summarise a set of birthmarks: how many of each type, how many functions each holds, and how long each function's birthmark is"
    )]
    Stats(StatsOpts),

    #[command(
        name = "run",
        about = "Extract birthmarks and compare them in one command"
    )]
    Run(RunOpts),

    #[cfg(feature = "mcp")]
    #[command(
        name = "mcp",
        about = "Serve oinkie over the Model Context Protocol, on stdin and stdout"
    )]
    Mcp(McpOpts),
}

/// Options for the MCP server.
#[cfg(feature = "mcp")]
#[derive(Debug, clap::Parser)]
pub struct McpOpts {
    #[clap(
        short = 'r',
        long = "root",
        value_name = "DIRECTORY",
        help = "Directory the tools may read and write under. May be given more than once.
Every path a tool is handed, input and output alike, has to resolve inside one of
these; anything else is refused. Defaults to the working directory.
The paths the tools receive are written by a language model rather than by you,
which is the whole reason this exists."
    )]
    roots: Vec<PathBuf>,
}

#[cfg(feature = "mcp")]
impl McpOpts {
    pub fn roots(&self) -> &[PathBuf] {
        &self.roots
    }
}

#[derive(Debug, clap::Parser)]
pub struct LiftOpts {
    #[clap(
        short,
        long,
        default_value = "pcodes",
        value_name = "DIRECTORY",
        help = "Specify the directory for putting the resultant JSON files of the lifted programs (default: './pcodes' directory)"
    )]
    dest: PathBuf,

    #[clap(
        short = 'r',
        long,
        value_parser = ir_parser(),
        default_value_t = Ir::default(),
        help = "Intermediate representation to produce. This also picks the tool, since a representation is only produced by one of them; several representations can come from the same tool."
    )]
    ir: Ir,

    #[clap(
        short = 'H',
        long,
        value_name = "HOME",
        help = "Path to the installation directory of the tool behind --ir. If not specified, that tool's own environment variable (GHIDRA_HOME for Ghidra) is read, then the usual install locations are searched. The error names which variable to set."
    )]
    home: Option<PathBuf>,

    #[clap(
        short = 'i',
        long = "intermediate",
        value_name = "DIRECTORY",
        help = "Directory for the lifter to work in, kept rather than discarded. Every lifter runs in one, since that is where its script writes; Ghidra also keeps its project there. If not specified, a temporary directory is used and deleted."
    )]
    intermediate_dir: Option<PathBuf>,

    #[clap(
        long,
        value_name = "SCRIPT",
        help = "Path to a custom lifting script, replacing the built-in one. The language is that of the tool behind --ir: Java for Ghidra. It must write {input file name}.json into its working directory."
    )]
    script: Option<PathBuf>,

    #[clap(
        short = 'j',
        long,
        default_value = "1",
        value_name = "N",
        value_parser = parse_jobs,
        help = "Lift up to N files at a time (default: 1, one after another). Lifting runs a whole decompiler process per file, and several of them against a Ghidra installation whose language cache has not been built yet can corrupt it, so parallelism is opt-in."
    )]
    jobs: std::num::NonZeroUsize,

    #[clap(
        short = 'S',
        long,
        default_value_t = false,
        help = "Skip if the resultant JSON file already exists"
    )]
    skip: bool,

    #[clap(
        index = 1,
        value_name = "FILES",
        help = "Path to the binary or intermediate files to lift"
    )]
    files: Vec<PathBuf>,
}

/// Parses `--jobs`, rejecting zero in terms of the option rather than of the
/// type behind it: clap's own message for `NonZeroUsize` is "number would be
/// zero for non-zero type", which is about Rust and not about lifting.
fn parse_jobs(s: &str) -> std::result::Result<std::num::NonZeroUsize, String> {
    match s.parse::<usize>() {
        Ok(n) => std::num::NonZeroUsize::new(n).ok_or_else(|| "must be at least 1".to_string()),
        Err(e) => Err(e.to_string()),
    }
}

impl LiftOpts {
    pub fn dest(&self) -> &Path {
        &self.dest
    }

    pub fn ir(&self) -> Ir {
        self.ir
    }

    pub fn home(&self) -> Option<&Path> {
        self.home.as_deref()
    }

    pub fn intermediate_dir(&self) -> Option<&Path> {
        self.intermediate_dir.as_deref()
    }

    pub fn script(&self) -> Option<&Path> {
        self.script.as_deref()
    }

    pub fn is_skip(&self) -> bool {
        self.skip
    }

    /// How many files may be lifted at a time.
    pub fn jobs(&self) -> usize {
        self.jobs.get()
    }

    pub fn iter(&self) -> impl Iterator<Item = &PathBuf> {
        self.files.iter()
    }

    pub fn len(&self) -> usize {
        self.files.len()
    }
}

#[derive(Debug, clap::Parser)]
pub struct ExtractOpts {
    #[clap(
        short,
        long,
        default_value = "birthmarks",
        value_name = "DIRECTORY",
        help = "Specify the directory for putting the resultant JSON files for the extracted birthmarks (default: './birthmarks' directory)"
    )]
    dest: PathBuf,

    #[clap(short, long, value_name = "BIRTHMARK_TYPE", value_parser = BirthmarkTypeParser, default_value = "op-seq", hide_possible_values = true, help = "Type of birthmark to extract.
fc (Function Calls) and op (Opcode) with set, seq, and freq variants are supported.
For example, 'op-seq' extracts the sequence of operations as a birthmark,
while 'fc-freq' extracts the frequency of function calls.
k-grams are written with the k in the name: 'op-3gram-set'. Any k parses, not
only the ones 'oinkie info' lists.
The full birthmark types can be found by running 'oinkie info'.")]
    birthmark_type: BirthmarkType,

    #[clap(
        short = 'S',
        long,
        default_value_t = false,
        help = "Skip the resultant birthmark file is already exists"
    )]
    skip: bool,

    #[clap(
        index = 1,
        value_name = "JSON_FILES",
        help = "Path to the JSON files to extract birthmarks from"
    )]
    files: Vec<PathBuf>,
}

impl ExtractOpts {
    pub fn dest(&self) -> &Path {
        &self.dest
    }

    pub fn extractor(&self) -> Extractor {
        Extractor::new(self.birthmark_type.clone())
    }

    pub fn len(&self) -> usize {
        self.files.len()
    }

    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &PathBuf> {
        self.files.iter()
    }

    pub fn is_skip(&self) -> bool {
        self.skip
    }
}

#[derive(Debug, clap::Parser)]
pub struct ReviewOpts {
    #[clap(
        short = 'A',
        long,
        default_value = "hungarian",
        value_name = "METHOD",
        ignore_case = true,
        help = "Specify the aggregator for combining the function similarities of a pair into its similarity.
Available:
- hungarian  Use the Hungarian algorithm to find the optimal one-to-one matching between the functions of
             two birthmarks, maximizing the total similarity.
- topn:N     Take each function's best match in the other birthmark, and average the N best of those from
             each side. Matches need not be one-to-one. This can reduce noise from less relevant matches
             and focus on the most significant similarities."
    )]
    aggregator: Aggregator,

    #[clap(
        short,
        long,
        value_name = "RESULT.CSV",
        help = "Specify the result CSV file of the comparing results to review.
The file lists the similarity of each pair.",
        default_value = "review.csv"
    )]
    dest_file: PathBuf,

    #[clap(
        index = 1,
        value_name = "SCORE_DIRECTORY",
        help = "Path to the score directory: the pair CSVs that compare or run wrote, holding the function similarities"
    )]
    score_directory: PathBuf,

    #[clap(
        long,
        value_name = "N|Rx",
        help = "Drop the functions with fewer elements than this before aggregating.
N is a count of elements; Rx is R times the mean count over every function of every birthmark
in the directory (e.g. 0.3x). Needs the birthmarks the comparisons name. The threshold is
recorded on the last line of the summary."
    )]
    min_elements: Option<MinElements>,
}

impl ReviewOpts {
    pub fn aggregator(&self) -> &Aggregator {
        &self.aggregator
    }

    pub fn score_directory(&self) -> &Path {
        &self.score_directory
    }

    pub fn dest_file(&self) -> &PathBuf {
        &self.dest_file
    }

    pub fn min_elements(&self) -> Option<&crate::cli::MinElements> {
        self.min_elements.as_ref()
    }
}

/// The smallest function `--min-elements` keeps: a count of elements, or a
/// multiple of the mean count over the directory (#128).
///
/// One option with two spellings rather than two options, so that there is no
/// state in which both are given. A bare `0.3` is refused rather than read
/// either way: as a count it would mean 0.3 elements, and as a ratio it would
/// be the one place a number without its `x` meant a multiple.
#[derive(Debug, Clone, PartialEq)]
pub enum MinElements {
    Count(usize),
    Ratio(f64),
}

impl FromStr for MinElements {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let s = s.trim();
        if let Some(ratio) = s.strip_suffix('x') {
            return match ratio.parse::<f64>() {
                Ok(r) if r.is_finite() && r >= 0.0 => Ok(MinElements::Ratio(r)),
                _ => Err(format!(
                    "{s}: a ratio of the mean is a number of zero or more followed by x, such as 0.3x"
                )),
            };
        }
        match s.parse::<usize>() {
            Ok(n) => Ok(MinElements::Count(n)),
            Err(_) if s.parse::<f64>().is_ok() => Err(format!(
                "{s}: a count of elements is a whole number; for {s} times the mean, write {s}x"
            )),
            Err(_) => Err(format!(
                "{s}: expected a count of elements, such as 5, or a ratio of the mean, such as 0.3x"
            )),
        }
    }
}

impl std::fmt::Display for MinElements {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MinElements::Count(n) => write!(f, "{n}"),
            MinElements::Ratio(r) => write!(f, "{r}x"),
        }
    }
}

/// The formats `stats` writes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum StatsFormat {
    /// Every table at once: the summary, the per-file rows and the skipped files.
    Json,
    /// One table: the summary, or the per-file rows with --per-file, or the
    /// most frequent elements with --top.
    Csv,
    /// Tables to read, or to paste into a paper.
    Markdown,
}

#[derive(Debug, clap::Parser)]
pub struct StatsOpts {
    #[clap(
        short,
        long,
        value_enum,
        default_value_t = StatsFormat::Markdown,
        value_name = "FORMAT",
        ignore_case = true,
        help = "Output format"
    )]
    format: StatsFormat,

    #[clap(
        short,
        long,
        value_name = "FILE",
        help = "Write the statistics to FILE rather than to standard output"
    )]
    output: Option<PathBuf>,

    #[clap(
        short,
        long,
        default_value_t = false,
        help = "Descend into the subdirectories of the given directories"
    )]
    recursive: bool,

    #[clap(
        long,
        default_value_t = false,
        help = "Report each birthmark file as well as each group. With -f csv this replaces the summary table."
    )]
    per_file: bool,

    #[clap(
        short,
        long,
        value_name = "N",
        help = "Report the N most frequent elements of each group. With -f csv this replaces the summary table."
    )]
    top: Option<usize>,

    #[clap(
        index = 1,
        required = true,
        value_name = "PATHS",
        help = "Birthmark files, or directories holding them. A directory contributes the *.json directly inside it;
a file that does not read as a birthmark is skipped with a warning and counted."
    )]
    inputs: Vec<PathBuf>,
}

impl StatsOpts {
    pub fn format(&self) -> StatsFormat {
        self.format
    }

    pub fn dest(&self) -> Option<&Path> {
        self.output.as_deref()
    }

    pub fn is_recursive(&self) -> bool {
        self.recursive
    }

    pub fn is_per_file(&self) -> bool {
        self.per_file
    }

    pub fn top(&self) -> Option<usize> {
        self.top
    }

    pub fn inputs(&self) -> &[PathBuf] {
        &self.inputs
    }
}

#[derive(Debug, clap::Parser)]
pub struct CompareOpts {
    #[clap(short, long, value_parser = algorithm_parser(), default_value = "jaccard", value_name = "ALGORITHM", ignore_case = true, help = "Specify the similarity calculation algorithm.")]
    algorithm: Algorithm,

    #[clap(
        short = 'A',
        long,
        default_value = "hungarian",
        value_name = "METHOD",
        ignore_case = true,
        help = "Specify the aggregator for combining the function similarities of a pair into its similarity.
Available:
- hungarian  Use the Hungarian algorithm to find the optimal one-to-one matching between the functions of
             two birthmarks, maximizing the total similarity.
- topn:N     Take each function's best match in the other birthmark, and average the N best of those from
             each side. Matches need not be one-to-one. This can reduce noise from less relevant matches
             and focus on the most significant similarities."
    )]
    aggregator: Aggregator,

    #[clap(short, long, value_parser = strategy_parser(), default_value = "all-and-self", value_name = "STRATEGY", ignore_case = true, help = "Specify the pairing strategy for comparing files.")]
    strategy: PairingStrategy,

    #[clap(
        short,
        long,
        value_name = "DIRECTORY",
        help = "Specify the destination directory for the comparing results",
        default_value = "similarities"
    )]
    dest: PathBuf,

    #[clap(
        short = 'S',
        long,
        default_value_t = false,
        help = "Skip if the similarity file already exists for the pair of birthmarks"
    )]
    skip: bool,

    #[clap(
        index = 1,
        value_name = "JSON_FILES",
        help = "Path to the birthmark JSON files to compare"
    )]
    files: Vec<PathBuf>,
}

impl CompareOpts {
    pub fn dest(&self) -> &Path {
        &self.dest
    }

    pub fn comparator(&self) -> Comparator {
        self.algorithm.comparator()
    }

    pub fn iter(&self) -> Box<dyn Iterator<Item = (&PathBuf, &PathBuf)> + Send + '_> {
        self.strategy.pairs(&self.files)
    }

    pub fn aggregator(&self) -> &Aggregator {
        log::info!(
            "Using {:?} as the aggregator for combining function similarities",
            self.aggregator
        );
        &self.aggregator
    }

    pub fn compare_count(&self) -> usize {
        self.strategy.compare_count(&self.files)
    }

    pub fn is_skip(&self) -> bool {
        self.skip
    }
}

#[derive(Debug, clap::Parser)]
pub struct RunOpts {
    #[clap(short, long, value_name = "ANALYSIS", value_parser = AnalysisParser, default_value = "op-set-jaccard", hide_possible_values = true, help = "Analysis to run, as '{birthmark}-{algorithm}' -- for example 'op-set-jaccard' or 'op-3gram-freq-cosine'.
Run 'oinkie info' for the birthmarks and the algorithms they pair with. Any k
parses in a k-gram name, not only the ones listed.")]
    pub(crate) analysis: Analysis,

    #[clap(short, long, value_parser = strategy_parser(), default_value = "all-and-self", ignore_case = true, help = "Pairing strategy for file comparisons")]
    pub strategy: PairingStrategy,

    #[clap(
        short,
        long,
        default_value = "similarities",
        help = "Destination path for the output CSV file (default: 'similarities' directory"
    )]
    pub(crate) dest: PathBuf,

    #[clap(
        short = 'A',
        long,
        default_value = "hungarian",
        value_name = "METHOD",
        ignore_case = true,
        help = "Specify the aggregator for combining the function similarities of a pair into its similarity.
Available:
- hungarian  Use the Hungarian algorithm to find the optimal one-to-one matching between the functions of
             two birthmarks, maximizing the total similarity.
- topn:N     Take each function's best match in the other birthmark, and average the N best of those from
             each side. Matches need not be one-to-one. This can reduce noise from less relevant matches
             and focus on the most significant similarities. available topn:N or topn:all (same as topn)."
    )]
    aggregator: Aggregator,

    #[clap(
        short = 'S',
        long,
        default_value_t = false,
        help = "Skip if the similarity file already exists for the pair of birthmarks"
    )]
    pub(crate) skip: bool,

    #[clap(index = 1, help = "Path to the JSON files")]
    pub(crate) files: Vec<PathBuf>,
}

impl RunOpts {
    pub fn dest(&self) -> &Path {
        &self.dest
    }

    pub fn analysis_type(&self) -> Result<AnalysisType> {
        self.analysis.analysis_type()
    }

    pub fn iter(&self) -> Box<dyn Iterator<Item = (&PathBuf, &PathBuf)> + Send + '_> {
        self.strategy.pairs(&self.files)
    }

    pub fn compare_count(&self) -> usize {
        self.strategy.compare_count(&self.files)
    }

    pub fn is_skip(&self) -> bool {
        self.skip
    }

    pub fn aggregator(&self) -> &Aggregator {
        log::info!(
            "Using {:?} as the aggregator for combining function similarities",
            self.aggregator
        );
        &self.aggregator
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_every_log_level_maps_to_a_filter() {
        let cases = [
            (LogLevel::Trace, log::LevelFilter::Trace),
            (LogLevel::Debug, log::LevelFilter::Debug),
            (LogLevel::Info, log::LevelFilter::Info),
            (LogLevel::Warn, log::LevelFilter::Warn),
            (LogLevel::Error, log::LevelFilter::Error),
            (LogLevel::Off, log::LevelFilter::Off),
        ];
        for (level, expected) in cases {
            assert_eq!(filter_level(&level), expected, "{level:?}");
        }
    }

    /// `-j` is rejected at parse time in terms of the option rather than of
    /// the `NonZeroUsize` behind it, so both refusals have to read as
    /// something a person typed.
    #[test]
    fn test_jobs_refuses_what_is_not_a_count() {
        assert_eq!(parse_jobs("4").unwrap().get(), 4);
        assert_eq!(parse_jobs("0").unwrap_err(), "must be at least 1");
        let e = parse_jobs("many").unwrap_err();
        assert!(e.contains("invalid digit"), "{e}");
    }

    #[test]
    fn test_run_takes_skip_and_reports_it() {
        for (args, expected) in [
            (vec!["oinkie", "run", "a.json"], false),
            (vec!["oinkie", "run", "-S", "a.json"], true),
        ] {
            let OinkieCommand::Run(opts) = OinkieOpts::try_parse_from(args).unwrap().command else {
                panic!("Expected Run command");
            };
            assert_eq!(opts.is_skip(), expected);
        }
    }
}
