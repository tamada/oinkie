use std::path::{Path, PathBuf};

pub use crate::values::Analysis;
use crate::values::{AnalysisParser, BirthmarkTypeParser};
use clap::ValueEnum;
use oinkie::prelude::*;

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
        name = "reaggregate",
        about = "Reaggregate the element-wise similarity scores and recalculate the birthmark-wise similarity score"
    )]
    Reaggregate(ReaggregateOpts),

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
///
/// Empty for now. The server speaks stdio and takes its vocabulary from the
/// library, so there is nothing to configure yet; confining which paths the
/// tools may touch is what will fill this in.
#[cfg(feature = "mcp")]
#[derive(Debug, clap::Parser)]
pub struct McpOpts {}

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

    #[clap(short = 'l', long, value_enum, default_value_t = LifterType::Ghidra, help = "Specify the lifter type")]
    lifter_type: LifterType,

    #[clap(
        short = 'H',
        long,
        value_name = "HOME",
        help = "Path to the lifter's installation directory. If not specified, the lifter's own environment variable (GHIDRA_HOME for Ghidra) is read, then the usual install locations are searched. The error names which variable to set."
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
        help = "Path to a custom lifting script, replacing the built-in one. The language is the lifter's own: Java for Ghidra. It must write {input file name}.json into its working directory."
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

    pub fn lifter_type(&self) -> LifterType {
        self.lifter_type
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
pub struct ReaggregateOpts {
    #[clap(
        short = 'A',
        long,
        default_value = "hungarian",
        value_name = "METHOD",
        ignore_case = true,
        help = "Specify the aggregator for combining element-wise similarity scores into a birthmark-wise similarity score.
Available:
- hungarian  Use the Hungarian algorithm to find the optimal matching between elements of two birthmarks,
             maximizing the total similarity score.
- topn:N     For each element in the first birthmark, consider only the top N most similar elements in the
             second birthmark when calculating the overall similarity score. This can reduce noise from less
             relevant matches and focus on the most significant similarities."
    )]
    aggregator: Aggregator,

    #[clap(
        short,
        long,
        value_name = "RESULT.CSV",
        help = "Specify the result CSV file of the comparing results to reaggregate.
The file contains the birthmark-wise similarity score list.",
        default_value = "reaggregate.csv"
    )]
    dest_file: PathBuf,

    #[clap(
        index = 1,
        value_name = "SCORE_DIRECTORY",
        help = "Path to the directory containing the element-wise similarity scores"
    )]
    score_directory: PathBuf,
}

impl ReaggregateOpts {
    pub fn aggregator(&self) -> &Aggregator {
        &self.aggregator
    }

    pub fn score_directory(&self) -> &Path {
        &self.score_directory
    }

    pub fn dest_file(&self) -> &PathBuf {
        &self.dest_file
    }
}

#[derive(Debug, clap::Parser)]
pub struct CompareOpts {
    #[clap(short, long, value_enum, default_value_t = Algorithm::Jaccard, value_name = "ALGORITHM", ignore_case = true, help = "Specify the similarity calculation algorithm.")]
    algorithm: Algorithm,

    #[clap(
        short = 'A',
        long,
        default_value = "hungarian",
        value_name = "METHOD",
        ignore_case = true,
        help = "Specify the aggregator for combining element-wise similarity scores into a birthmark-wise similarity score.
Available:
- hungarian  Use the Hungarian algorithm to find the optimal matching between elements of two birthmarks,
             maximizing the total similarity score.
- topn:N     For each element in the first birthmark, consider only the top N most similar elements in the
             second birthmark when calculating the overall similarity score. This can reduce noise from less
             relevant matches and focus on the most significant similarities."
    )]
    aggregator: Aggregator,

    #[clap(short, long, value_enum, default_value_t = PairingStrategy::AllAndSelf, value_name = "STRATEGY", ignore_case = true, help = "Specify the pairing strategy for comparing files.")]
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
            "Using {:?} as the aggregator for combining element-wise similarity scores",
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

    #[clap(short, long, value_enum, default_value_t = PairingStrategy::AllAndSelf, ignore_case = true, help = "Pairing strategy for file comparisons")]
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
        help = "Specify the aggregator for combining element-wise similarity scores into a birthmark-wise similarity score.
Available:
- hungarian  Use the Hungarian algorithm to find the optimal matching between elements of two birthmarks,
             maximizing the total similarity score.
- topn:N     For each element in the first birthmark, consider only the top N most similar elements in the
             second birthmark when calculating the overall similarity score. This can reduce noise from less
             relevant matches and focus on the most significant similarities. available topn:N or topn:all (same as topn)."
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
            "Using {:?} as the aggregator for combining element-wise similarity scores",
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
