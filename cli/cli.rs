use std::path::{Path, PathBuf};

pub use crate::info::BType;
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

impl OinkieOpts {
    pub fn init(&self) -> Result<()> {
        unsafe {
            match self.level {
                LogLevel::Debug => std::env::set_var("RUST_LOG", "debug"),
                LogLevel::Info => std::env::set_var("RUST_LOG", "info"),
                LogLevel::Warn => std::env::set_var("RUST_LOG", "warn"),
                LogLevel::Error => std::env::set_var("RUST_LOG", "error"),
                LogLevel::Trace => std::env::set_var("RUST_LOG", "trace"),
                LogLevel::Off => std::env::set_var("RUST_LOG", "off"),
            }
        }
        env_logger::init();
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
    #[command(name="info", about = "Display information about the application")]
    Info,

    #[command(name="lift", about = "Lift binary files to P-code JSON files using a specified lifter")]
    Lift(LiftOpts),

    #[command(name="extract", about = "Extract birthmarks from a lifted binary file (JSON format)")]
    Extract(ExtractOpts),

    #[command(name="compare", about = "Compare birthmarks and output the similarity score")]
    Compare(CompareOpts),
    // #[command(name="execute", about = "Execute a command on the JSON files")]
    // Execute(ExecuteOpts),
    #[command(name="reaggregate", about = "Reaggregate the element-wise similarity scores and recalculate the birthmark-wise similarity score")]
    Reaggregate(ReaggregateOpts),

    #[command(name="run", about = "Extract birthmarks and compare them in one command")]
    Run(RunOpts),
}

#[derive(Debug, clap::Parser, ValueEnum, Clone)]
#[clap(rename_all = "kebab-case")]
pub enum LifterType {
    Ghidra,
    Llvm,
    BinaryNinja,
}

#[derive(Debug, clap::Parser)]
pub struct LiftOpts {
    #[clap(short, long, default_value = "pcodes", value_name = "DIRECTORY", help = "Specify the directory for putting the resultant JSON files for the lifted P-code (default: './pcodes' directory)")]
    dest: PathBuf,

    #[clap(short = 'l', long, value_enum, default_value_t = LifterType::Ghidra, help = "Specify the lifter type")]
    lifter_type: LifterType,

    #[clap(short = 'H', long, value_name = "HOME", help = "Specify the path to the home directory of the lifter (e.g., GHIDRA_HOME for Ghidra). If not specified, the environment variable (e.g., GHIDRA_HOME) or default paths are searched.")]
    home: Option<PathBuf>,

    #[clap(short = 'i', long = "intermediate", value_name = "DIRECTORY", help = "Directory to keep intermediate files like Ghidra project directories. If not specified, a temporary directory is used and deleted.")]
    intermediate_dir: Option<PathBuf>,

    #[clap(long, value_name = "SCRIPT", help = "Path to a custom lifting script. Interpretation depends on the lifter type. For Ghidra, it's the path to a Java script.")]
    script: Option<PathBuf>,

    #[clap(short = 'S', long, default_value_t = false, help = "Skip if the resultant JSON file already exists")]
    skip: bool,

    #[clap(index = 1, value_name = "FILES", help = "Path to the binary or intermediate files to lift")]
    files: Vec<PathBuf>,
}

impl LiftOpts {
    pub fn dest(&self) -> &Path {
        &self.dest
    }

    pub fn lifter_type(&self) -> &LifterType {
        &self.lifter_type
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

    pub fn iter(&self) -> impl Iterator<Item = &PathBuf> {
        self.files.iter()
    }

    pub fn len(&self) -> usize {
        self.files.len()
    }
}

#[derive(Debug, clap::Parser, ValueEnum, Clone)]
#[clap(rename_all = "kebab-case")]
pub enum BinaryType {
    Ghidra,
    Llvm,
    BinaryNinja,
}

#[derive(Debug, clap::Parser)]
pub struct ExtractOpts {
    #[clap(short, long, default_value = "birthmarks", value_name = "DIRECTORY", help = "Specify the directory for putting the resultant JSON files for the extracted birthmarks (default: './birthmarks' directory)")]
    dest: PathBuf,

    #[clap(short, long, value_enum, value_name = "BIRTHMARK_TYPE", default_value_t = BType::OpSeq, hide_possible_values = true, ignore_case = true, help = "Type of birthmark to extract.
fc (Function Calls) and op (Opcode) with set, seq, and freq variants are supported.
For example, 'op-seq' extracts the sequence of operations as a birthmark,
while 'fc-freq' extracts the frequency of function calls.
The full birthmark types cann be found by running 'oinkie info'.")]
    birthmark_type: BType,

    #[clap(short = 'S', long, default_value_t = false, help = "Skip the resultant birthmark file is already exists")]
    skip: bool,

    #[clap(short = 'B', long, value_enum, value_name = "BINARY_TYPE", default_value_t = BinaryType::Ghidra, ignore_case = true, help = "Type of binary. Current version only supports Ghidra JSON format")]
    binary_type: BinaryType,

    #[clap(index = 1, value_name = "JSON_FILES", help = "Path to the JSON files to extract birthmarks from")]
    files: Vec<PathBuf>,
}

impl ExtractOpts {
    pub fn dest(&self) -> &Path {
        &self.dest
    }

    pub fn extractor(&self) -> Extractor {
        Extractor::new(self.birthmark_type.clone().into())
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
        short = 'A', long, default_value = "hungarian", value_name = "METHOD", ignore_case = true, 
        help = "Specify the aggregator for combining element-wise similarity scores into a birthmark-wise similarity score.
Available: 
- hungarian  Use the Hungarian algorithm to find the optimal matching between elements of two birthmarks,
             maximizing the total similarity score.
- topn:N     For each element in the first birthmark, consider only the top N most similar elements in the
             second birthmark when calculating the overall similarity score. This can reduce noise from less
             relevant matches and focus on the most significant similarities."
    )]
    aggregator: Aggregator,

    #[clap(short, long, value_name = "RESULT.CSV", help = "Specify the result CSV file of the comparing results to reaggregate.
The file contains the birthmark-wise similarity score list.", default_value = "reaggregate.csv")]
    dest_file: PathBuf,

    #[clap(index = 1, value_name = "SCORE_DIRECTORY", help = "Path to the directory containing the element-wise similarity scores")]
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
        short = 'A', long, default_value = "hungarian", value_name = "METHOD", ignore_case = true, 
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

    #[clap(short, long, value_name = "DIRECTORY", help = "Specify the destination directory for the comparing results", default_value = "similarities")]
    dest: PathBuf,

    #[clap(short = 'S', long, default_value_t = false, help = "Skip if the similarity file already exists for the pair of birthmarks")]
    skip: bool,

    #[clap(index = 1, value_name = "JSON_FILES", help = "Path to the birthmark JSON files to compare")]
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
        log::info!("Using {:?} as the aggregator for combining element-wise similarity scores", self.aggregator);
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
    #[clap(short, long, value_enum, default_value_t = Analysis::OpSetJaccard, ignore_case = true, help = "Similarity algorithm to use")]
    pub(crate) analysis: Analysis,

    #[clap(short, long, value_enum, default_value_t = PairingStrategy::AllAndSelf, ignore_case = true, help = "Pairing strategy for file comparisons")]
    pub strategy: PairingStrategy,

    #[clap(short, long, default_value = "similarities", help = "Destination path for the output CSV file (default: 'similarities' directory")]
    pub(crate) dest: PathBuf,

    #[clap(
        short = 'A', long, default_value = "hungarian", value_name = "METHOD", ignore_case = true,
        help = "Specify the aggregator for combining element-wise similarity scores into a birthmark-wise similarity score.
Available: 
- hungarian  Use the Hungarian algorithm to find the optimal matching between elements of two birthmarks,
             maximizing the total similarity score.
- topn:N     For each element in the first birthmark, consider only the top N most similar elements in the
             second birthmark when calculating the overall similarity score. This can reduce noise from less
             relevant matches and focus on the most significant similarities. available topn:N or topn:all (same as topn)."
    )]
    aggregator: Aggregator,

    #[clap(short = 'S', long, default_value_t = false, help = "Skip if the similarity file already exists for the pair of birthmarks")]
    pub(crate) skip: bool,

    #[clap(index = 1, help = "Path to the JSON files")]
    pub(crate) files: Vec<PathBuf>,
}

impl RunOpts {
    pub fn dest(&self) -> &Path {
        &self.dest
    }

    pub fn analysis_type(&self) -> AnalysisType {
        (&self.analysis).into()
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
        log::info!("Using {:?} as the aggregator for combining element-wise similarity scores", self.aggregator);
        &self.aggregator
    }
}

#[derive(ValueEnum, Clone, Debug)]
#[clap(rename_all = "kebab-case")]
pub(crate) enum Analysis {
    FcFreqCosine,
    FcSetDice,
    FcFreqEuclidean,
    FcSetJaccard,
    FcSeqLevenshtein,
    FcSeqLcs,
    FcSetSimpson,
    FcFreqWeightedjaccard,

    OpFreqCosine,
    OpSetDice,
    OpFreqEuclidean,
    OpSetJaccard,
    OpSeqLevenshtein,
    OpSeqLcs,
    OpSetSimpson,
    OpFreqWeightedjaccard,

    Op1gramSetDice,
    Op1gramSetJaccard,
    Op1gramSetSimpson,
    Op1gramSeqLevenshtein,
    Op1gramSeqLcs,
    Op1gramFreqCosine,
    Op1gramFreqEuclidean,
    Op1gramFreqWeightedjaccard,

    Op2gramSetDice,
    Op2gramSetJaccard,
    Op2gramSetSimpson,
    Op2gramSeqLevenshtein,
    Op2gramSeqLcs,
    Op2gramFreqCosine,
    Op2gramFreqEuclidean,
    Op2gramFreqWeightedjaccard,

    Op3gramSetDice,
    Op3gramSetJaccard,
    Op3gramSetSimpson,
    Op3gramSeqLevenshtein,
    Op3gramSeqLcs,
    Op3gramFreqCosine,
    Op3gramFreqEuclidean,
    Op3gramFreqWeightedjaccard,

    Op4gramSetDice,
    Op4gramSetJaccard,
    Op4gramSetSimpson,
    Op4gramSeqLevenshtein,
    Op4gramSeqLcs,
    Op4gramFreqCosine,
    Op4gramFreqEuclidean,
    Op4gramFreqWeightedjaccard,

    Op5gramSetDice,
    Op5gramSetJaccard,
    Op5gramSetSimpson,
    Op5gramSeqLevenshtein,
    Op5gramSeqLcs,
    Op5gramFreqCosine,
    Op5gramFreqEuclidean,
    Op5gramFreqWeightedjaccard,

    Op6gramSetDice,
    Op6gramSetJaccard,
    Op6gramSetSimpson,
    Op6gramSeqLevenshtein,
    Op6gramSeqLcs,
    Op6gramFreqCosine,
    Op6gramFreqEuclidean,
    Op6gramFreqWeightedjaccard,
}

impl From<Analysis> for AnalysisType {
    fn from(v: Analysis) -> Self {
        AnalysisType::from(&v)
    }
}

impl From<&Analysis> for AnalysisType {
    fn from(value: &Analysis) -> Self {
        match value {
            Analysis::FcFreqCosine => AnalysisType::new(BirthmarkType::FcFreq, Algorithm::Cosine),
            Analysis::FcSetDice => AnalysisType::new(BirthmarkType::FcSet, Algorithm::Dice),
            Analysis::FcFreqEuclidean => AnalysisType::new(BirthmarkType::FcFreq, Algorithm::Euclidean),
            Analysis::FcSetJaccard => AnalysisType::new(BirthmarkType::FcSet, Algorithm::Jaccard),
            Analysis::FcSeqLevenshtein => AnalysisType::new(BirthmarkType::FcSeq, Algorithm::Levenshtein),
            Analysis::FcSeqLcs => AnalysisType::new(BirthmarkType::FcSeq, Algorithm::Lcs),
            Analysis::FcSetSimpson => AnalysisType::new(BirthmarkType::FcSet, Algorithm::Simpson),
            Analysis::FcFreqWeightedjaccard => AnalysisType::new(BirthmarkType::FcFreq, Algorithm::WeightedJaccard),

            Analysis::OpFreqCosine => AnalysisType::new(BirthmarkType::OpFreq, Algorithm::Cosine),
            Analysis::OpSetDice => AnalysisType::new(BirthmarkType::OpSet, Algorithm::Dice),
            Analysis::OpFreqEuclidean => AnalysisType::new(BirthmarkType::OpFreq, Algorithm::Euclidean),
            Analysis::OpSetJaccard => AnalysisType::new(BirthmarkType::OpSet, Algorithm::Jaccard),
            Analysis::OpSeqLevenshtein => AnalysisType::new(BirthmarkType::OpSeq, Algorithm::Levenshtein),
            Analysis::OpSeqLcs => AnalysisType::new(BirthmarkType::OpSeq, Algorithm::Lcs),
            Analysis::OpSetSimpson => AnalysisType::new(BirthmarkType::OpSet, Algorithm::Simpson),
            Analysis::OpFreqWeightedjaccard => AnalysisType::new(BirthmarkType::OpFreq, Algorithm::WeightedJaccard),

            Analysis::Op1gramSetDice => AnalysisType::new(BirthmarkType::OpKgramSet(1), Algorithm::Dice),
            Analysis::Op1gramSetJaccard => AnalysisType::new(BirthmarkType::OpKgramSet(1), Algorithm::Jaccard),
            Analysis::Op1gramSetSimpson => AnalysisType::new(BirthmarkType::OpKgramSet(1), Algorithm::Simpson),
            Analysis::Op1gramSeqLevenshtein => AnalysisType::new(BirthmarkType::OpKgramSeq(1), Algorithm::Levenshtein),
            Analysis::Op1gramSeqLcs => AnalysisType::new(BirthmarkType::OpKgramSeq(1), Algorithm::Lcs),
            Analysis::Op1gramFreqCosine => AnalysisType::new(BirthmarkType::OpKgramFreq(1), Algorithm::Cosine),
            Analysis::Op1gramFreqEuclidean => AnalysisType::new(BirthmarkType::OpKgramFreq(1), Algorithm::Euclidean),
            Analysis::Op1gramFreqWeightedjaccard => AnalysisType::new(BirthmarkType::OpKgramFreq(1), Algorithm::WeightedJaccard),

            Analysis::Op2gramSetDice => AnalysisType::new(BirthmarkType::OpKgramSet(2), Algorithm::Dice),
            Analysis::Op2gramSetJaccard => AnalysisType::new(BirthmarkType::OpKgramSet(2), Algorithm::Jaccard),
            Analysis::Op2gramSetSimpson => AnalysisType::new(BirthmarkType::OpKgramSet(2), Algorithm::Simpson),
            Analysis::Op2gramSeqLevenshtein => AnalysisType::new(BirthmarkType::OpKgramSeq(2), Algorithm::Levenshtein),
            Analysis::Op2gramSeqLcs => AnalysisType::new(BirthmarkType::OpKgramSeq(2), Algorithm::Lcs),
            Analysis::Op2gramFreqCosine => AnalysisType::new(BirthmarkType::OpKgramFreq(2), Algorithm::Cosine),
            Analysis::Op2gramFreqEuclidean => AnalysisType::new(BirthmarkType::OpKgramFreq(2), Algorithm::Euclidean),
            Analysis::Op2gramFreqWeightedjaccard => AnalysisType::new(BirthmarkType::OpKgramFreq(2), Algorithm::WeightedJaccard),

            Analysis::Op3gramSetDice => AnalysisType::new(BirthmarkType::OpKgramSet(3), Algorithm::Dice),
            Analysis::Op3gramSetJaccard => AnalysisType::new(BirthmarkType::OpKgramSet(3), Algorithm::Jaccard),
            Analysis::Op3gramSetSimpson => AnalysisType::new(BirthmarkType::OpKgramSet(3), Algorithm::Simpson),
            Analysis::Op3gramSeqLevenshtein => AnalysisType::new(BirthmarkType::OpKgramSeq(3), Algorithm::Levenshtein),
            Analysis::Op3gramSeqLcs => AnalysisType::new(BirthmarkType::OpKgramSeq(3), Algorithm::Lcs),
            Analysis::Op3gramFreqCosine => AnalysisType::new(BirthmarkType::OpKgramFreq(3), Algorithm::Cosine),
            Analysis::Op3gramFreqEuclidean => AnalysisType::new(BirthmarkType::OpKgramFreq(3), Algorithm::Euclidean),
            Analysis::Op3gramFreqWeightedjaccard => AnalysisType::new(BirthmarkType::OpKgramFreq(3), Algorithm::WeightedJaccard),

            Analysis::Op4gramSetDice => AnalysisType::new(BirthmarkType::OpKgramSet(4), Algorithm::Dice),
            Analysis::Op4gramSetJaccard => AnalysisType::new(BirthmarkType::OpKgramSet(4), Algorithm::Jaccard),
            Analysis::Op4gramSetSimpson => AnalysisType::new(BirthmarkType::OpKgramSet(4), Algorithm::Simpson),
            Analysis::Op4gramSeqLevenshtein => AnalysisType::new(BirthmarkType::OpKgramSeq(4), Algorithm::Levenshtein),
            Analysis::Op4gramSeqLcs => AnalysisType::new(BirthmarkType::OpKgramSeq(4), Algorithm::Lcs),
            Analysis::Op4gramFreqCosine => AnalysisType::new(BirthmarkType::OpKgramFreq(4), Algorithm::Cosine),
            Analysis::Op4gramFreqEuclidean => AnalysisType::new(BirthmarkType::OpKgramFreq(4), Algorithm::Euclidean),
            Analysis::Op4gramFreqWeightedjaccard => AnalysisType::new(BirthmarkType::OpKgramFreq(4), Algorithm::WeightedJaccard),

            Analysis::Op5gramSetDice => AnalysisType::new(BirthmarkType::OpKgramSet(5), Algorithm::Dice),
            Analysis::Op5gramSetJaccard => AnalysisType::new(BirthmarkType::OpKgramSet(5), Algorithm::Jaccard),
            Analysis::Op5gramSetSimpson => AnalysisType::new(BirthmarkType::OpKgramSet(5), Algorithm::Simpson),
            Analysis::Op5gramSeqLevenshtein => AnalysisType::new(BirthmarkType::OpKgramSeq(5), Algorithm::Levenshtein),
            Analysis::Op5gramSeqLcs => AnalysisType::new(BirthmarkType::OpKgramSeq(5), Algorithm::Lcs),
            Analysis::Op5gramFreqCosine => AnalysisType::new(BirthmarkType::OpKgramFreq(5), Algorithm::Cosine),
            Analysis::Op5gramFreqEuclidean => AnalysisType::new(BirthmarkType::OpKgramFreq(5), Algorithm::Euclidean),
            Analysis::Op5gramFreqWeightedjaccard => AnalysisType::new(BirthmarkType::OpKgramFreq(5), Algorithm::WeightedJaccard),

            Analysis::Op6gramSetDice => AnalysisType::new(BirthmarkType::OpKgramSet(6), Algorithm::Dice),
            Analysis::Op6gramSetJaccard => AnalysisType::new(BirthmarkType::OpKgramSet(6), Algorithm::Jaccard),
            Analysis::Op6gramSetSimpson => AnalysisType::new(BirthmarkType::OpKgramSet(6), Algorithm::Simpson),
            Analysis::Op6gramSeqLevenshtein => AnalysisType::new(BirthmarkType::OpKgramSeq(6), Algorithm::Levenshtein),
            Analysis::Op6gramSeqLcs => AnalysisType::new(BirthmarkType::OpKgramSeq(6), Algorithm::Lcs),
            Analysis::Op6gramFreqCosine => AnalysisType::new(BirthmarkType::OpKgramFreq(6), Algorithm::Cosine),
            Analysis::Op6gramFreqEuclidean => AnalysisType::new(BirthmarkType::OpKgramFreq(6), Algorithm::Euclidean),
            Analysis::Op6gramFreqWeightedjaccard => AnalysisType::new(BirthmarkType::OpKgramFreq(6), Algorithm::WeightedJaccard),

        }
    }
}
