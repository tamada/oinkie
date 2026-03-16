mod cli;
mod info;

use std::path::PathBuf;
use std::io::Write;
use std::time::Duration;
use std::{path::Path, time::Instant};
use clap::{Parser, ValueEnum};
use indicatif::ProgressBar;
use rayon::prelude::*;
use oinkie::prelude::*;
use oinkie::ghidra::Op;

fn load<T>(path: PathBuf) -> Result<T> 
where 
    T: TryFrom<PathBuf, Error = Error>
{
    let s = Instant::now();
    let item: T = path.clone().try_into()?;
    let d = s.elapsed().as_millis();
    log::info!("Loading {path:?} done in {d} msec");
    Ok(item)
}

fn perform_run(opts: cli::RunOpts) -> Result<Vec<Duration>> {
    let start = Instant::now();
    let atype = opts.analysis_type();
    let dest = opts.dest();
    let comparing_count = opts.compare_count();
    let pbar = new_progress_bar(comparing_count * 3);

    let results = opts.iter().enumerate().par_bridge()
            .map(|(i, (path1, path2))| {
        let dest_file = dest.join(format!("{i:05}.csv"));
        if dest_file.exists() && opts.is_skip() {
            log::info!("Similarity file for {:?} and {:?} already exists. Skipping comparison.", path1, path2);
            read_result_file(&dest_file, i, path1, path2)
        } else {
            pbar.set_message(format!("Loading program from {:?}", path1.display()));
            let p1: Program<Op> = load(path1.to_path_buf())?;
            pbar.inc(1);
            pbar.set_message(format!("Loading program from {:?}", path2.display()));
            let p2: Program<Op> = load(path2.to_path_buf())?;
            pbar.inc(1);
            pbar.set_message(format!("Comparing two programs ({}/{})", i + 1, comparing_count));
            let result = atype.comparator().compare_programs(&p1, &p2);
            pbar.inc(1);
            result.store(dest.join(format!("{i:05}.csv")))?;
            Ok(CompareResult::new(i, result.similarity(), path1, path2, result.duration()))
        }
    }).collect::<Vec<_>>();
    pbar.finish();
    let results = Error::vec_result_to_result_vec(results)?;
    store_and_get_durations(results, dest, start)
}

/// Read the comparison result from the given CSV file and return a CompareResult struct.
/// This function skips actual comparison and shorten the comparison time if the result file already exists.
/// The CSV file is expected to have a line starting with "result," followed by the duration in nanoseconds and the similarity value, separated by commas.
fn read_result_file<'a>(dest_path: &Path, index: usize, path1: &'a Path, path2: &'a Path) -> Result<CompareResult<'a>> {
    let content = std::fs::read_to_string(dest_path)
        .map_err(|e| Error::Io(dest_path.to_path_buf(), e))?;
    let mut lines = content.lines();
    let result_line = lines.find(|line| line.starts_with("result,"))
        .ok_or_else(|| Error::Parse(format!("Result line not found in {}", dest_path.display())))?;
    let parts: Vec<&str> = result_line.split(',').collect();
    if parts.len() < 3 {
        return Err(Error::Parse(format!("Invalid result line format in {}", dest_path.display())));
    }
    let duration_nanos: u64 = parts[1].parse()
        .map_err(|e| Error::ParseInt(e))?;
    let similarity: f64 = parts[2].parse()
        .map_err(|e| Error::Parse(format!("Failed to parse similarity value in {}: {}", dest_path.display(), e)))?;
    Ok(CompareResult::new(index, similarity, path1, path2, Duration::from_nanos(duration_nanos)))
}

fn new_progress_bar(len: usize) -> ProgressBar {
    ProgressBar::new(len as u64)
        .with_style(indicatif::ProgressStyle::with_template("[{elapsed_precise}/({per_sec})] {bar:40} {pos:>7}/{len:7} {msg}").unwrap())
}

fn perform_compare(opts: cli::CompareOpts) -> Result<Vec<Duration>> {
    let start = Instant::now();
    let comparator = opts.comparator();
    let dest = opts.dest();
    let comparing_count = opts.compare_count();
    let pbar = new_progress_bar(comparing_count * 3);
    std::fs::create_dir_all(dest).map_err(|e| Error::Io(dest.to_path_buf(), e))?;
    let r = opts.iter().enumerate().par_bridge()
        .map(|(i, (path1, path2))| compare_impl((i, (path1, path2)), &comparator, dest, &pbar, comparing_count, opts.is_skip()))
        .collect::<Vec<_>>();
    pbar.finish();
    store_and_get_durations(r, dest, start)
}

fn store_and_get_durations(results: Vec<CompareResult>, dest: &Path, start: Instant) -> Result<Vec<Duration>> {
    let destcsv = dest.join("results.csv");
    let mut file = std::fs::File::create(&destcsv)
        .map_err(|e| Error::Io(dest.to_path_buf(), e))?;
    let ritems = results.into_iter().map(|r| {
        let csv = r.to_csv();
        match writeln!(file, "{}", csv) {
            Ok(_) => Ok(r.duration),
            Err(e) => Err(Error::Io(destcsv.clone(), e)),
        }
    }).collect::<Vec<_>>();
    let duration = start.elapsed();
    let _ = writeln!(file, "total duration,{},{}", duration.as_nanos(), format_duration(duration));
    Error::vec_result_to_result_vec(ritems)
}

fn compare_impl<'a>(tuple: (usize, (&'a Path, &'a Path)), comparator: &Comparator, dest: &Path, pbar: &ProgressBar, comparing_count: usize, skip: bool) -> CompareResult<'a> {
    let (i, (path1, path2)) = tuple;
    let dest = dest.join(format!("{i:05}.csv"));
    if dest.exists() && skip {
        log::info!("Similarity file for {:?} and {:?} already exists. Skipping comparison.", path1, path2);
        read_result_file(&dest, i, path1, path2).unwrap()
    } else {
        pbar.set_message(format!("Loading birthmark from {:?}", path1.display()));
        let b1 = load(path1.to_path_buf()).unwrap();
        pbar.inc(1);
        pbar.set_message(format!("Loading birthmark from {:?}", path2.display()));
        let b2 = load(path2.to_path_buf()).unwrap();
        pbar.inc(1);
        pbar.set_message(format!("Comparing two birthmarks ({}/{})", i + 1, comparing_count));
        let result = comparator.compare_birthmarks(&b1, &b2);
        pbar.inc(1);
        let _ = result.store(dest.join(format!("{i:05}.csv")));
        CompareResult::new(i, result.similarity(), path1, path2, result.duration())
    }
}

fn perform_extract(opts: cli::ExtractOpts) -> Result<Vec<Duration>> {
    // let (dest, btype, _bin_type, files) = (opts.dest, opts.birthmark_type, opts.binary_type, opts.files);
    let dest = opts.dest();
    let pb = ProgressBar::new(opts.len() as u64)
            .with_style(indicatif::ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}").unwrap())
            .with_message("Extracting birthmarks...");
    std::fs::create_dir_all(&dest)
        .map_err(|e| Error::Io(dest.to_path_buf(), e))?;
    let extractor = opts.extractor();
    let start = Instant::now();
    let r = opts.iter().par_bridge().map(|path| {
        let e1 = Instant::now();
        let r = match extract_impl(&path, &dest, &extractor, opts.is_skip()) {
            Ok(_) => Ok(e1.elapsed()),
            Err(e) => Err(e),
        };
        pb.inc(1);
        r
    }).collect::<Result<Vec<_>>>();
    let duration = start.elapsed();
    println!("Extraction completed in {} nsec ({})", duration.as_nanos(), format_duration(duration));
    r
}

fn extract_impl(path: &Path, dest: &Path, extractor: &Extractor, skip: bool) -> Result<()> {
    let file_name = oinkie::extractor::dest_file_name(path)?;
    let dest_path = dest.join(file_name);
    if dest_path.exists() && skip {
        log::info!("Birthmark for {:?} already exists. Skipping extraction.", path);
        return Ok(());
    }
    let p: Program<Op> = path.try_into()?;
    let birthmarks = extractor.extract_each(&p)?;
    let json = serde_json::to_string_pretty(&birthmarks)
        .map_err(Error::Json)?;
    std::fs::write(&dest_path, json)
        .map_err(|e| Error::Io(dest_path.clone(), e))?;
    Ok(())
}

fn format_duration(dur: Duration) -> String {
    let total_ms = dur.as_millis();
    let mm = total_ms / 60000;
    let ss = (total_ms % 60000) / 1000;
    let sss = total_ms % 1000;
    
    format!("{:02}:{:02}:{:03}", mm, ss, sss)
}

fn perform_info() -> Result<Vec<Duration>> {
    let now = Instant::now();
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
    Ok(vec![now.elapsed()])
}

fn perform(opts: cli::OinkieOpts) -> Result<Vec<Duration>> {
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
    pub similarity: f64,
    pub path1: &'a Path,
    pub path2: &'a Path,
    pub duration: std::time::Duration,
}

impl<'a> CompareResult<'a> {
    pub fn new(index: usize, similarity: f64, path1: &'a Path, path2: &'a Path, duration: std::time::Duration) -> Self {
        Self { index, similarity, path1, path2, duration }
    }

    pub fn to_csv(&self) -> String {
        format!("{},{},{},{},{}", self.index, self.similarity, self.path1.display(), self.path2.display(), self.duration.as_nanos())
    }
}

fn rs_main(args: Vec<String>) -> Result<Vec<Duration>> {
    cli::OinkieOpts::try_parse_from(args)
        .map_err(Error::Clap)
        .and_then(perform)
}

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let status_code = match rs_main(args) {
        Err(Error::Clap(e)) => {
            if e.kind() == clap::error::ErrorKind::DisplayHelp || e.kind() == clap::error::ErrorKind::DisplayVersion {
                println!("{}", e.render().ansi());
                0
            } else {
                eprintln!("Error: {}", e);
                1
            }
        },
        Err(e) => {
            eprintln!("Error: {}", e);
            2
        },
        Ok(_) => 0,
    };
    std::process::exit(status_code);
}
