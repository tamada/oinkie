mod cli;
mod info;

use std::{path::Path, time::Instant};
use clap::{Parser, ValueEnum};
use rayon::prelude::*;
use oinkie2::prelude::*;
use oinkie2::ghidra::Op;

fn convert(path1: &Path, path2: &Path) -> Result<(Program<Op>, Program<Op>)> {
    let s1 = Instant::now();
    let p1: Program<Op> = path1.try_into()?;
    let d1 = s1.elapsed().as_millis();
    log::info!("Loading {path1:?} done in {d1} msec");

    let s1 = Instant::now();
    let p2: Program<Op> = path2.try_into()?;
    let d2 = s1.elapsed().as_millis();
    log::info!("Loading {path2:?} done in {d2} msec");
    Ok((p1, p2))
}

fn perform_run(opts: cli::RunOpts) -> Result<()> {
    let context = opts.build_context()?;
    let progressor = context.main_progressor();
    let comparator = context.comparator();
    let mut dest = context.dest()?;
    let start = std::time::Instant::now();
    let results = context.iter().enumerate().par_bridge()
            .map(|(i, (path1, path2))| {
        let (p1, p2) = match convert(path1, path2) {
            Ok((p1, p2)) => (p1, p2),
            Err(e) => return Err(e),
        };
        progressor.println("Comparing two programs...".to_string());
        let now = Instant::now();
        let result = comparator.compare(&p1, &p2, i, &context);
        progressor.inc();
        Ok(CompareResult::new(i, result, path1, path2, now.elapsed()))
    }).collect::<Vec<_>>();
    let duration = start.elapsed();
    for r in results {
        let r = r?;
        let csv = r.to_csv();
        let _ = writeln!(dest, "{}", csv);
    }
    let _ = writeln!(dest, "Total Duration,{}", duration.as_nanos());
    Ok(())
}

fn perform_compare(opts: cli::CompareOpts) -> Result<()> {
    todo!()
}

fn perform_extract(opts: cli::ExtractOpts) -> Result<()> {
    todo!()
}

fn perform_info() -> Result<()> {
    println!("=========== Oinkie Info ============");
    println!("Oinkie is a tool for detecting the code theft with Ghidra P-code as birthmarks.
The birthmark is a unique characteristic of a program that can be used to identify it.
Oinkie extracts birthmarks from given codes and compares them to calculate the similarities.");
    println!("============ Birthmarks =============");
    cli::BType::value_variants().iter().for_each(|b| {
        let pv = b.to_possible_value().unwrap();
        println!("- {:<20}  {}", pv.get_name(), pv.get_help().unwrap());
    });
    println!("======== Compare Algorithms ========");
    Algorithm::value_variants().iter().for_each(|c| {
        let pv = c.to_possible_value().unwrap();
        println!("- {:<20}  {}", pv.get_name(), pv.get_help().unwrap());
    });

    Ok(())
}

fn perform(opts: cli::OinkieOpts) -> Result<()> {
    opts.init()?;
    use cli::OinkieCommand::*;
    match opts.command {
        Run(opts) => perform_run(opts),
        Compare(opts) => perform_compare(opts),
        Extract(opts) => perform_extract(opts),
        Info => perform_info(),
    }
}

pub struct CompareResult<'a> {
    pub index: usize,
    pub result: f64,
    pub path1: &'a Path,
    pub path2: &'a Path,
    pub duration: std::time::Duration,
}

impl<'a> CompareResult<'a> {
    pub fn new(index: usize, result: f64, path1: &'a Path, path2: &'a Path, duration: std::time::Duration) -> Self {
        Self { index, result, path1, path2, duration }
    }

    pub fn to_csv(&self) -> String {
        format!("{},{},{},{},{}", self.index, self.result, self.path1.display(), self.path2.display(), self.duration.as_nanos())
    }
}

fn rs_main(args: Vec<String>) -> Result<()> {
    cli::OinkieOpts::try_parse_from(args)
        .map_err(Error::Clap)
        .and_then(perform)
}

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    if let Err(e) = rs_main(args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
