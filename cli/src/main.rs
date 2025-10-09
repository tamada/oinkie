use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

use oinkie::birthmarks::{Birthmark, BirthmarkType};
use oinkie::{OinkieError, Result};
use oinkie::extractors::{self, Mode};
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
    Info,
}

#[derive(Parser, Debug)]
struct ExtractSourceOpts {
    #[clap(short = 't', long = "type", value_name = "BIRTHMARK_TYPE", default_value = "op-seq", help = "Birthmark type for extraction")]
    btype: BirthmarkType,

    #[clap(short = 'm', long = "mode", value_name = "EXTRACTION_MODE", default_value = "file", help = "Extraction mode")]
    mode: Mode,

    #[clap(index = 1, value_name = "IR|BC", help = "Path to the LLVM IR or BC file")]
    inputs: Vec<PathBuf>,
}

#[derive(Parser, Debug)]
struct ExtractOpts {
    #[clap(short, long, default_value = "-", value_name = "DEST", help = "Output file path (default: stdout (\"-\"))")]
    dest: String,

    #[clap(flatten)]
    source: ExtractSourceOpts,
}

fn extract_birthmarks(inputs: Vec<PathBuf>, btype: BirthmarkType, mode: &Mode) -> Result<Vec<Birthmark>> {
    let result = inputs.iter()
        .map(|p| extractors::from_path(p, &btype, mode))
        .collect::<Vec<_>>();
    OinkieError::vec_result_to_result_vec(result)
        .map(|v| v.into_iter().flatten().collect())
}

fn extract(opts: ExtractOpts) -> oinkie::Result<()> {
    let mut errs = vec![];
    let (btype, dest, inputs, mode) = (opts.source.btype, opts.dest, opts.source.inputs, opts.source.mode);

    match extract_birthmarks(inputs, btype, &mode) {
        Ok(birthmarks) => {
            let json = serde_json::to_string_pretty(&birthmarks)
                .map_err(OinkieError::Json)?;
            if dest == "-" {
                println!("{}", json);
            } else {
                std::fs::write(dest, json)
                    .map_err(OinkieError::Io)?;
            }
        },
        Err(e) => errs.push(e),
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
    extract_opts: ExtractSourceOpts,

    #[clap(flatten)]
    compare_opts: CompareAlgorithmsOpts,
}

#[derive(Parser, Debug)]
struct ExecuteOpts {
    #[clap(index = 1, default_value = "-", help = "the script file. If absent or '-', read from stdin.")]
    script: PathBuf,
}

fn read_birthmarks_from_json(paths: Vec<PathBuf>) -> Result<Vec<Birthmark>> {
    let result = paths.iter()
        .map(|p| oinkie::birthmarks::load(p))
        .collect::<Vec<_>>();
    OinkieError::vec_result_to_result_vec(result)
        .map(|v| v.into_iter().flatten().collect())
}

fn read_and_compare(opts: CompareOpts) -> oinkie::Result<()> {
    let (paths, algorithm) = (opts.birthmarks, opts.algorithm);
    let birthmarks = read_birthmarks_from_json(paths);
    compare(birthmarks, algorithm)
}

fn compare(birthmarks: Result<Vec<Birthmark>>, opts: CompareAlgorithmsOpts) -> oinkie::Result<()> {
    let comparator = oinkie::comparators::comparator(&opts.comparator);
    match birthmarks {
        Ok(birthmarks) => match calculate_similarities(birthmarks, comparator) {
            Ok(similarities) => {
                output_similarities(similarities, opts.dest)
            },
            Err(e) => Err(e),
        },
        Err(e) => Err(e),
    }
}

fn output_similarities(similarities: Vec<Similarity>, dest: String) -> oinkie::Result<()> {
    let json = serde_json::to_string_pretty(&similarities)
        .map_err(OinkieError::Json)?;
    if dest == "-" {
        println!("{}", json);
    } else {
        std::fs::write(dest, json)
            .map_err(OinkieError::Io)?;
    }
    Ok(())
}

fn calculate_similarities(birthmarks: Vec<Birthmark>, comparator: Box<dyn Comparator>) -> Result<Vec<Similarity>> {
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
                    errs.push(OinkieError::Fatal(format!("Birthmark types do not match: {:?} vs {:?}", a.info, b.info)));
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
    let (eopts, copts) = (opts.extract_opts, opts.compare_opts);
    let birthmarks = extract_birthmarks(eopts.inputs, eopts.btype, &eopts.mode);
    let (ctype, dest) = (copts.comparator, copts.dest);
    let comparator = oinkie::comparators::comparator(&ctype);
    match birthmarks {
        Ok(birthmarks) => match calculate_similarities(birthmarks, comparator) {
            Ok(similarities) => output_similarities(similarities, dest),
            Err(e) => Err(e),
        },
        Err(e) => Err(e),
    }
}

fn execute(_opts: ExecuteOpts) -> oinkie::Result<()> {
    Ok(())
}

fn info() -> oinkie::Result<()> {
    println!("======== Oinkie Info ========");
    println!("Oinkie is a tool for detecting the code theft from LLVM IR/CB codes with birthmarks.
The birthmark is a unique characteristic of a program that can be used to identify it.
Oinkie extracts birthmarks from given IR/BC codes and compares them to calculate the similarities.");
    println!("======== Birthmarks  ========");
    BirthmarkType::value_variants().iter().for_each(|b| {
        println!("- {b:9}: {}", b.to_possible_value().unwrap().get_help().unwrap());
    });
    println!("======== Comparators ========");
    ComparatorType::value_variants().iter().for_each(|c| {
        println!("- {c:9}: {}", c.to_possible_value().unwrap().get_help().unwrap());
    });

    Ok(())
}

fn perform(opts: OinkieOpts) -> oinkie::Result<()> {
    use OinkieCommand::*;
    match opts.command {
        Extract(opts) => extract(opts),
        Compare(opts) => read_and_compare(opts),
        Run(opts) => run(opts),
        Execute(opts) => execute(opts),
        Info => info(),
    }
}

fn main() -> Result<()>{
    perform(OinkieOpts::parse())
}
