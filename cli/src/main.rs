use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

use oinkie::birthmarks::BirthmarkType;
use oinkie::{OinkieError, Result};
use oinkie::comparators::{Comparator, Similarity, Type as ComparatorType};

#[derive(Parser, Debug)]
#[command(name = "oinkie", about = "A tool for extracting and comparing birthmarks from LLVM IR or BC files")]
struct OinkieOpts {
    #[clap(subcommand)]
    command: OinkieCommand,
}

#[derive(Subcommand, Debug)]
enum OinkieCommand {
    #[command(name = "extract", about = "Extract birthmarks from LLVM IR or BC files")]
    Extract(ExtractOpts),

    #[command(name = "compare", about = "Compare two birthmarks")]
    Compare(CompareOpts),

    #[command(name = "run", about = "Extract birthmarks and compare them in one command")]
    Run(RunOpts),

    #[command(name = "execute", about = "Execute the given WASM script for analyzing birthmarks")]
    Execute(ExecuteOpts),

    #[command(name = "info", about = "Show information about the tool")]
    Info(InfoOpts),
}

#[derive(Parser, Debug)]
struct ExtractOpts {
    #[clap(short = 't', long = "type", value_name = "BIRTHMARK_TYPE", default_value = "op-seq", help = "Birthmark type for extraction")]
    btype: BirthmarkType,

    #[clap(short, long, default_value = "-", value_name = "DEST", help = "Output file path (default: stdout (\"-\"))")]
    dest: String,

    #[clap(index = 1, value_name = "IR|BC", help = "Path to the LLVM IR or BC file")]
    inputs: Vec<PathBuf>,
}

fn extract(opts: ExtractOpts) -> oinkie::Result<()> {
    let mut errs = vec![];
    for input in opts.inputs {
        let birthmark = match oinkie::extractors::from_path(input, opts.btype.clone()) {
            Ok(b) => b,
            Err(e) => {
                errs.push(e);
                continue;
            }
        };
        match serde_json::to_string_pretty(&birthmark) {
            Ok(json) => if opts.dest == "-" {
                println!("{}", json);
            } else {
                match std::fs::write(&opts.dest, json) {
                    Ok(_) => {},
                    Err(e) => errs.push(OinkieError::Io(e)),
                }
            },
            Err(e) => errs.push(OinkieError::Json(e)),
        };
    }
    OinkieError::error_or((), errs)
}

#[derive(Parser, Debug)]
struct CompareAlgorithmsOpts {
    #[clap(short, long, default_value = "jaccard", value_name = "COMPARATOR_TYPE", help = "Specifies the comparator")]
    comparator: ComparatorType,

    #[clap(short, long, default_value = "-", value_name = "DEST", help = "Output file path (default: stdout (\"-\"))")]
    dest: String,
}

#[derive(Parser, Debug)]
struct CompareOpts {
    #[clap(flatten)]
    algorithm: CompareAlgorithmsOpts,

    #[clap(index = 1, help = "Paths of the birthmark files to compare")]
    birthmarks: Vec<PathBuf>,
}

#[derive(Parser, Debug)]
struct RunOpts {
    #[clap(flatten)]
    extract_opts: ExtractOpts,

    #[clap(flatten)]
    compare_opts: CompareAlgorithmsOpts,
}

#[derive(Parser, Debug)]
struct ExecuteOpts {
    #[clap(index = 1, default_value = "-", help = "the script file. If absent or '-', read from stdin.")]
    script: PathBuf,
}

#[derive(Parser, Debug)]
struct InfoOpts {}

fn compare(opts: CompareOpts) -> oinkie::Result<()> {
    let result = opts.birthmarks.iter()
        .map(|p| oinkie::birthmarks::Birthmark::from_path(p))
        .collect::<Vec<_>>();
    let comparator = oinkie::comparators::comparator(&opts.algorithm.comparator);
    match OinkieError::vec_result_to_result_vec(result) {
        Ok(birthmarks) => match compare_impl(birthmarks, comparator) {
            Ok(similarities) => {
                let json = serde_json::to_string_pretty(&similarities)
                    .map_err(OinkieError::Json)?;
                if opts.algorithm.dest == "-" {
                    println!("{}", json);
                } else {
                    std::fs::write(&opts.algorithm.dest, json)
                        .map_err(OinkieError::Io)?;
                }
                Ok(())
            },
            Err(e) => Err(e),
        },
        Err(e) => Err(e),
    }
}

fn compare_impl(birthmarks: Vec<oinkie::birthmarks::Birthmark>, comparator: Box<dyn Comparator>) -> oinkie::Result<Vec<Similarity>> {
    if birthmarks.len() < 2 {
        Err(OinkieError::Fatal("At least two birthmarks are required for comparison".to_string()))
    } else {
        let mut errs = vec![];
        let mut oks = vec![];
        for i in 0..birthmarks.len() {
            for j in (i+1)..birthmarks.len() {
                let a = &birthmarks[i];
                let b = &birthmarks[j];
                if !a.is_same_type(b) {
                    errs.push(OinkieError::Fatal(format!("Birthmark types do not match: {:?} vs {:?}", a.btype, b.btype)));
                }
                match comparator.compare(a, b) {
                    Ok(similarity) => oks.push(similarity),
                    Err(e) => errs.push(e),
                }
            }
        }
        OinkieError::error_or(oks, errs)
    }
}

fn run(opts: RunOpts) -> oinkie::Result<()> {
    Ok(())
}

fn execute(opts: ExecuteOpts) -> oinkie::Result<()> {
    Ok(())
}

fn info(opts: InfoOpts) -> oinkie::Result<()> {
    println!("======== Oinkie Info ========");
    println!("======== Extractors  ========");
    BirthmarkType::value_variants().iter().for_each(|b| {
        println!("- {:?}: {}", b, b.to_possible_value().unwrap().get_help().unwrap());
    });
    println!("======== Comparators ========");
    ComparatorType::value_variants().iter().for_each(|c| {
        println!("- {:?}: {}", c, c.to_possible_value().unwrap().get_help().unwrap());
    });

    Ok(())
}

fn perform(opts: OinkieOpts) -> oinkie::Result<()> {
    use OinkieCommand::*;
    match opts.command {
        Extract(opts) => extract(opts),
        Compare(opts) => compare(opts),
        Run(opts) => run(opts),
        Execute(opts) => execute(opts),
        Info(opts) => info(opts),
    }
}

fn main() -> Result<()>{
    perform(OinkieOpts::parse())
}
