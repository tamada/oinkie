use std::path::PathBuf;

pub use crate::info::BType;
use clap::ValueEnum;
use oinkie2::prelude::*;

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
    #[command(name="compare", about = "Compare two JSON files and output the similarity score")]
    Compare(CompareOpts),
    #[command(name="extract", about = "Extract birthmarks from JSON files")]
    Extract(ExtractOpts),
    // #[command(name="execute", about = "Execute a command on the JSON files")]
    // Execute(ExecuteOpts),
    #[command(name="run", about = "Extract birthmarks and compare them in one command")]
    Run(RunOpts),
    #[command(name="info", about = "Display information about the application")]
    Info,
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
    #[clap(short, long, default_value = "-", help = "The destination JSON file for the extracted birthmarks (default: - for stdout)")]
    pub dest: PathBuf,

    #[clap(short, long, value_enum, default_value_t = BType::OpSeq, ignore_case = true, help = "Type of birthmark to extract")]
    pub birthmark_type: BType,

    #[clap(short, long, value_enum, default_value_t = BinaryType::Ghidra, ignore_case = true, help = "Type of binary. Currently only supports Ghidra JSON format")]
    pub binary_type: BinaryType,

    #[clap(index = 1, help = "Path to the JSON files to extract birthmarks from")]
    pub files: Vec<PathBuf>,
}

#[derive(Debug, clap::Parser)]
pub struct CompareOpts {
    #[clap(short, long, value_enum, default_value_t = Algorithm::Jaccard, ignore_case = true, help = "Specify the similarity calculation algorithm.")]
    pub algorithm: Algorithm,    

    #[clap(short, long, value_enum, default_value_t = PairingStrategy::AllAndSelf, ignore_case = true, help = "Specify the pairing strategy for comparing files.")]
    pub strategy: PairingStrategy,

    #[clap(short, long, help = "Destination path for the output CSV file (default: results.csv)")]
    pub dest: Option<PathBuf>,

    #[clap(flatten)]
    pub progress: ProgressOpts,

    #[clap(index = 1, help = "Path to the birthmark JSON files to compare")]
    pub files: Vec<PathBuf>,
}

#[derive(Debug, clap::Parser)]
pub struct ProgressOpts {
    #[clap(long, help = "Path to the first JSON file")]
    pub disable_progress: bool,
}

#[derive(Debug, clap::Parser)]
pub struct RunOpts {
    #[clap(flatten)]
    pub(crate) progress: ProgressOpts,

    #[clap(short, long, value_enum, default_value_t = Analysis::OpSetJaccard, ignore_case = true, help = "Similarity algorithm to use")]
    pub(crate) algorithm: Analysis,

    #[clap(short, long, value_enum, default_value_t = PairingStrategy::AllAndSelf, ignore_case = true, help = "Pairing strategy for file comparisons")]
    pub strategy: PairingStrategy,

    #[clap(short, long, help = "Destination path for the output CSV file (default: results.csv)")]
    pub(crate) dest: Option<PathBuf>,

    #[clap(index = 1, help = "Path to the JSON files")]
    pub(crate) files: Vec<PathBuf>,
}

#[derive(ValueEnum, Clone, Debug)]
#[clap(rename_all = "kebab-case")]
pub(crate) enum Analysis {
    FcFreqCosine,
    FcSetDice,
    FcFreqEuclidean,
    FcSetJaccard,
    FcSeqLevenshtein,
    FcSetSimpson,
    FcFreqWeightedjaccard,

    OpFreqCosine,
    OpSetDice,
    OpFreqEuclidean,
    OpSetJaccard,
    OpSeqLevenshtein,
    OpSetSimpson,
    OpFreqWeightedjaccard,

    Op1gramSetDice,
    Op1gramSetJaccard,
    Op1gramSetSimpson,
    Op1gramSeqLevenshtein,
    Op1gramFreqCosine,
    Op1gramFreqEuclidean,
    Op1gramFreqWeightedjaccard,

    Op2gramSetDice,
    Op2gramSetJaccard,
    Op2gramSetSimpson,
    Op2gramSeqLevenshtein,
    Op2gramFreqCosine,
    Op2gramFreqEuclidean,
    Op2gramFreqWeightedjaccard,

    Op3gramSetDice,
    Op3gramSetJaccard,
    Op3gramSetSimpson,
    Op3gramSeqLevenshtein,
    Op3gramFreqCosine,
    Op3gramFreqEuclidean,
    Op3gramFreqWeightedjaccard,

    Op4gramSetDice,
    Op4gramSetJaccard,
    Op4gramSetSimpson,
    Op4gramSeqLevenshtein,
    Op4gramFreqCosine,
    Op4gramFreqEuclidean,
    Op4gramFreqWeightedjaccard,

    Op5gramSetDice,
    Op5gramSetJaccard,
    Op5gramSetSimpson,
    Op5gramSeqLevenshtein,
    Op5gramFreqCosine,
    Op5gramFreqEuclidean,
    Op5gramFreqWeightedjaccard,

    Op6gramSetDice,
    Op6gramSetJaccard,
    Op6gramSetSimpson,
    Op6gramSeqLevenshtein,
    Op6gramFreqCosine,
    Op6gramFreqEuclidean,
    Op6gramFreqWeightedjaccard,
}

impl TryFrom<Analysis> for AnalysisType {
    type Error = Error;

    fn try_from(value: Analysis) -> Result<Self> {
        match value {
            Analysis::FcFreqCosine => Ok(AnalysisType::new(BirthmarkType::FcFreq, Algorithm::Cosine)),
            Analysis::FcSetDice => Ok(AnalysisType::new(BirthmarkType::FcSet, Algorithm::Dice)),
            Analysis::FcFreqEuclidean => Ok(AnalysisType::new(BirthmarkType::FcFreq, Algorithm::Euclidean)),
            Analysis::FcSetJaccard => Ok(AnalysisType::new(BirthmarkType::FcSet, Algorithm::Jaccard)),
            Analysis::FcSeqLevenshtein => Ok(AnalysisType::new(BirthmarkType::FcSeq, Algorithm::Levenshtein)),
            Analysis::FcSetSimpson => Ok(AnalysisType::new(BirthmarkType::FcSet, Algorithm::Simpson)),
            Analysis::FcFreqWeightedjaccard => Ok(AnalysisType::new(BirthmarkType::FcFreq, Algorithm::WeightedJaccard)),

            Analysis::OpFreqCosine => Ok(AnalysisType::new(BirthmarkType::OpFreq, Algorithm::Cosine)),
            Analysis::OpSetDice => Ok(AnalysisType::new(BirthmarkType::OpSet, Algorithm::Dice)),
            Analysis::OpFreqEuclidean => Ok(AnalysisType::new(BirthmarkType::OpFreq, Algorithm::Euclidean)),
            Analysis::OpSetJaccard => Ok(AnalysisType::new(BirthmarkType::OpSet, Algorithm::Jaccard)),
            Analysis::OpSeqLevenshtein => Ok(AnalysisType::new(BirthmarkType::OpSeq, Algorithm::Levenshtein)),
            Analysis::OpSetSimpson => Ok(AnalysisType::new(BirthmarkType::OpSet, Algorithm::Simpson)),
            Analysis::OpFreqWeightedjaccard => Ok(AnalysisType::new(BirthmarkType::OpFreq, Algorithm::WeightedJaccard)),

            Analysis::Op1gramSetDice => Ok(AnalysisType::new(BirthmarkType::OpKgramSet(1), Algorithm::Dice)),
            Analysis::Op1gramSetJaccard => Ok(AnalysisType::new(BirthmarkType::OpKgramSet(1), Algorithm::Jaccard)),
            Analysis::Op1gramSetSimpson => Ok(AnalysisType::new(BirthmarkType::OpKgramSet(1), Algorithm::Simpson)),
            Analysis::Op1gramSeqLevenshtein => Ok(AnalysisType::new(BirthmarkType::OpKgramSeq(1), Algorithm::Levenshtein)),
            Analysis::Op1gramFreqCosine => Ok(AnalysisType::new(BirthmarkType::OpKgramFreq(1), Algorithm::Cosine)),
            Analysis::Op1gramFreqEuclidean => Ok(AnalysisType::new(BirthmarkType::OpKgramFreq(1), Algorithm::Euclidean)),
            Analysis::Op1gramFreqWeightedjaccard => Ok(AnalysisType::new(BirthmarkType::OpKgramFreq(1), Algorithm::WeightedJaccard)),

            Analysis::Op2gramSetDice => Ok(AnalysisType::new(BirthmarkType::OpKgramSet(2), Algorithm::Dice)),
            Analysis::Op2gramSetJaccard => Ok(AnalysisType::new(BirthmarkType::OpKgramSet(2), Algorithm::Jaccard)),
            Analysis::Op2gramSetSimpson => Ok(AnalysisType::new(BirthmarkType::OpKgramSet(2), Algorithm::Simpson)),
            Analysis::Op2gramSeqLevenshtein => Ok(AnalysisType::new(BirthmarkType::OpKgramSeq(2), Algorithm::Levenshtein)),
            Analysis::Op2gramFreqCosine => Ok(AnalysisType::new(BirthmarkType::OpKgramFreq(2), Algorithm::Cosine)),
            Analysis::Op2gramFreqEuclidean => Ok(AnalysisType::new(BirthmarkType::OpKgramFreq(2), Algorithm::Euclidean)),
            Analysis::Op2gramFreqWeightedjaccard => Ok(AnalysisType::new(BirthmarkType::OpKgramFreq(2), Algorithm::WeightedJaccard)),

            Analysis::Op3gramSetDice => Ok(AnalysisType::new(BirthmarkType::OpKgramSet(3), Algorithm::Dice)),
            Analysis::Op3gramSetJaccard => Ok(AnalysisType::new(BirthmarkType::OpKgramSet(3), Algorithm::Jaccard)),
            Analysis::Op3gramSetSimpson => Ok(AnalysisType::new(BirthmarkType::OpKgramSet(3), Algorithm::Simpson)),
            Analysis::Op3gramSeqLevenshtein => Ok(AnalysisType::new(BirthmarkType::OpKgramSeq(3), Algorithm::Levenshtein)),
            Analysis::Op3gramFreqCosine => Ok(AnalysisType::new(BirthmarkType::OpKgramFreq(3), Algorithm::Cosine)),
            Analysis::Op3gramFreqEuclidean => Ok(AnalysisType::new(BirthmarkType::OpKgramFreq(3), Algorithm::Euclidean)),
            Analysis::Op3gramFreqWeightedjaccard => Ok(AnalysisType::new(BirthmarkType::OpKgramFreq(3), Algorithm::WeightedJaccard)),

            Analysis::Op4gramSetDice => Ok(AnalysisType::new(BirthmarkType::OpKgramSet(4), Algorithm::Dice)),
            Analysis::Op4gramSetJaccard => Ok(AnalysisType::new(BirthmarkType::OpKgramSet(4), Algorithm::Jaccard)),
            Analysis::Op4gramSetSimpson => Ok(AnalysisType::new(BirthmarkType::OpKgramSet(4), Algorithm::Simpson)),
            Analysis::Op4gramSeqLevenshtein => Ok(AnalysisType::new(BirthmarkType::OpKgramSeq(4), Algorithm::Levenshtein)),
            Analysis::Op4gramFreqCosine => Ok(AnalysisType::new(BirthmarkType::OpKgramFreq(4), Algorithm::Cosine)),
            Analysis::Op4gramFreqEuclidean => Ok(AnalysisType::new(BirthmarkType::OpKgramFreq(4), Algorithm::Euclidean)),
            Analysis::Op4gramFreqWeightedjaccard => Ok(AnalysisType::new(BirthmarkType::OpKgramFreq(4), Algorithm::WeightedJaccard)),

            Analysis::Op5gramSetDice => Ok(AnalysisType::new(BirthmarkType::OpKgramSet(5), Algorithm::Dice)),
            Analysis::Op5gramSetJaccard => Ok(AnalysisType::new(BirthmarkType::OpKgramSet(5), Algorithm::Jaccard)),
            Analysis::Op5gramSetSimpson => Ok(AnalysisType::new(BirthmarkType::OpKgramSet(5), Algorithm::Simpson)),
            Analysis::Op5gramSeqLevenshtein => Ok(AnalysisType::new(BirthmarkType::OpKgramSeq(5), Algorithm::Levenshtein)),
            Analysis::Op5gramFreqCosine => Ok(AnalysisType::new(BirthmarkType::OpKgramFreq(5), Algorithm::Cosine)),
            Analysis::Op5gramFreqEuclidean => Ok(AnalysisType::new(BirthmarkType::OpKgramFreq(5), Algorithm::Euclidean)),
            Analysis::Op5gramFreqWeightedjaccard => Ok(AnalysisType::new(BirthmarkType::OpKgramFreq(5), Algorithm::WeightedJaccard)),

            Analysis::Op6gramSetDice => Ok(AnalysisType::new(BirthmarkType::OpKgramSet(6), Algorithm::Dice)),
            Analysis::Op6gramSetJaccard => Ok(AnalysisType::new(BirthmarkType::OpKgramSet(6), Algorithm::Jaccard)),
            Analysis::Op6gramSetSimpson => Ok(AnalysisType::new(BirthmarkType::OpKgramSet(6), Algorithm::Simpson)),
            Analysis::Op6gramSeqLevenshtein => Ok(AnalysisType::new(BirthmarkType::OpKgramSeq(6), Algorithm::Levenshtein)),
            Analysis::Op6gramFreqCosine => Ok(AnalysisType::new(BirthmarkType::OpKgramFreq(6), Algorithm::Cosine)),
            Analysis::Op6gramFreqEuclidean => Ok(AnalysisType::new(BirthmarkType::OpKgramFreq(6), Algorithm::Euclidean)),
            Analysis::Op6gramFreqWeightedjaccard => Ok(AnalysisType::new(BirthmarkType::OpKgramFreq(6), Algorithm::WeightedJaccard)),

        }
    }
}

impl RunOpts {
    pub fn build_context(self) -> oinkie2::Result<Context> {
        let analysis_type = self.algorithm.try_into()?;
        Ok(Context::new_with(
            self.strategy,
            analysis_type,
            self.files,
            self.dest,
            !self.progress.disable_progress,
        ))
    }
}
