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

const DEFAULT_GHIDRA_SCRIPT: &str = include_str!("../lifter/scripts/HighPCodeLifter.java");

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
    let aggregator = opts.aggregator();
    std::fs::create_dir_all(dest).map_err(|e| Error::Io(dest.to_path_buf(), e))?;

    let results = opts.iter().enumerate().par_bridge()
            .map(|(i, (path1, path2))| {
        let dest_file = dest.join(format!("{i:05}.csv"));
        if dest_file.exists() && opts.is_skip() {
            log::info!("Similarity file for {:?} and {:?} already exists. Skipping comparison.", path1, path2);
            pbar.inc(3);
            read_result_file(&dest_file, i, path1, path2)
        } else {
            pbar.set_message(format!("Loading program from {:?}", path1.display()));
            let mut p1: Program<Op> = load(path1.to_path_buf())?;
            p1.set_json_path(path1.to_path_buf());
            pbar.inc(1);
            pbar.set_message(format!("Loading program from {:?}", path2.display()));
            let mut p2: Program<Op> = load(path2.to_path_buf())?;
            p2.set_json_path(path2.to_path_buf());
            pbar.inc(1);
            pbar.set_message(format!("Comparing two programs ({}/{})", i + 1, comparing_count));
            let result = atype.comparator().compare_programs(&p1, &p2, aggregator)?;
            pbar.inc(1);
            result.store(&dest_file)?;
            Ok(CompareResult::new(i, result.similarity(), p1.path().to_path_buf(), p2.path().to_path_buf(), result.duration()))
        }
    }).collect::<Vec<_>>();
    pbar.finish();
    let results = Error::vec_result_to_result_vec(results)?;
    let dest_file = dest.join("results.csv");
    store_and_get_durations(results, &dest_file, start)
}

/// Read the comparison result from the given CSV file and return a CompareResult struct.
/// This function skips actual comparison and shorten the comparison time if the result file already exists.
/// The CSV file is expected to have a line starting with "result," followed by the duration in nanoseconds and the similarity value, separated by commas.
fn read_result_file(dest_path: &Path, index: usize, _path1: &Path, _path2: &Path) -> Result<CompareResult> {
    let content = std::fs::read_to_string(dest_path)
        .map_err(|e| Error::Io(dest_path.to_path_buf(), e))?;
    let lines = content.lines();
    
    let mut original_path1 = PathBuf::new();
    let mut original_path2 = PathBuf::new();
    let mut similarity = 0.0;
    let mut duration_nanos = 0;

    for line in lines {
        if line.starts_with("result,") {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() < 3 {
                return Err(Error::Parse(format!("Invalid result line format in {}", dest_path.display())));
            }
            duration_nanos = parts[1].parse()
                .map_err(|e| Error::ParseInt(parts[1].to_string(), e))?;
            similarity = parts[2].parse()
                .map_err(|e| Error::Parse(format!("Failed to parse similarity value in {}: {}", dest_path.display(), e)))?;
        } else if line.starts_with("left,") {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 4 {
                original_path1 = PathBuf::from(parts[3]);
            }
        } else if line.starts_with("right,") {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 4 {
                original_path2 = PathBuf::from(parts[3]);
            }
        }
    }

    if original_path1.as_os_str().is_empty() || original_path2.as_os_str().is_empty() {
        return Err(Error::Parse(format!("Birthmark paths not found in {}", dest_path.display())));
    }

    Ok(CompareResult::new(index, similarity, original_path1, original_path2, Duration::from_nanos(duration_nanos)))
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
    let aggregator = opts.aggregator();
    std::fs::create_dir_all(dest).map_err(|e| Error::Io(dest.to_path_buf(), e))?;
    let r = opts.iter().enumerate().par_bridge()
        .map(|(i, (path1, path2))| compare_impl((i, (path1, path2)), &comparator, dest, &pbar, comparing_count, opts.is_skip(), aggregator))
        .collect::<Vec<_>>();
    pbar.finish();
    let result_file = dest.join("results.csv");
    Error::vec_result_to_result_vec(r)
        .and_then(|r| store_and_get_durations(r, &result_file, start))
    
}

pub(crate) fn store_and_get_durations(results: Vec<CompareResult>, destcsv: &Path, start: Instant) -> Result<Vec<Duration>> {
    let mut file = std::fs::File::create(destcsv)
        .map_err(|e| Error::Io(destcsv.to_path_buf(), e))?;
    let ritems = results.into_iter().map(|r| {
        let csv = r.to_csv();
        match writeln!(file, "{}", csv) {
            Ok(_) => Ok(r.duration),
            Err(e) => Err(Error::Io(destcsv.to_path_buf(), e)),
        }
    }).collect::<Vec<_>>();
    let duration = start.elapsed();
    let _ = writeln!(file, "total duration,{},{}", duration.as_nanos(), format_duration(duration));
    Error::vec_result_to_result_vec(ritems)
}

fn compare_impl(tuple: (usize, (&Path, &Path)), comparator: &Comparator, dest: &Path, pbar: &ProgressBar, comparing_count: usize, skip: bool, aggregator: &Aggregator) -> Result<CompareResult> {
    let (i, (path1, path2)) = tuple;
    let dest_file = dest.join(format!("{i:05}.csv"));
    log::info!("compare_impl(dest: {} (exists: {}), skip: {skip})", dest_file.display(), dest_file.exists());
    if dest_file.exists() && skip {
        log::info!("Similarity file for {:?} and {:?} already exists. Skipping comparison.", path1, path2);
        pbar.inc(3);
        read_result_file(&dest_file, i, path1, path2)
    } else {
        log::info!("Loading birthmark from {:?}", path2.display());
        pbar.set_message(format!("Loading birthmark from {:?}", path1.display()));
        let mut b1: Birthmark = load(path1.to_path_buf())?;
        b1.set_json_path(path1.to_path_buf());
        pbar.inc(1);
        log::info!("Loading birthmark from {:?}", path2.display());
        pbar.set_message(format!("Loading birthmark from {:?}", path2.display()));
        let mut b2: Birthmark = load(path2.to_path_buf())?;
        b2.set_json_path(path2.to_path_buf());
        pbar.inc(1);
        log::info!("Comparing birthmarks (len: {} x {}) progress: {}/{comparing_count}", b1.len(), b2.len(), i + 1);
        pbar.set_message(format!("Comparing birthmarks (len: {} x {}) progress: {}/{comparing_count}", b1.len(), b2.len(), i + 1));
        let result = comparator.compare_birthmarks(&b1, &b2, aggregator)?;
        pbar.inc(1);
        result.store(&dest_file)?;
        Ok(CompareResult::new(i, result.similarity(), b1.path().to_path_buf(), b2.path().to_path_buf(), result.duration()))
    }
}

fn perform_extract(opts: cli::ExtractOpts) -> Result<Vec<Duration>> {
    // let (dest, btype, _bin_type, files) = (opts.dest, opts.birthmark_type, opts.binary_type, opts.files);
    let dest = opts.dest();
    let pb = ProgressBar::new(opts.len() as u64)
            .with_style(indicatif::ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}").unwrap())
            .with_message("Extracting birthmarks...");
    std::fs::create_dir_all(dest)
        .map_err(|e| Error::Io(dest.to_path_buf(), e))?;
    let extractor = opts.extractor();
    let start = Instant::now();
    let r = opts.iter().par_bridge().map(|path| {
        let e1 = Instant::now();
        let r = match extract_impl(path, dest, &extractor, opts.is_skip()) {
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
        .map_err(|e| Error::Json(dest_path.clone(), e))?;
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

fn validate_extract_opts(opts: cli::ExtractOpts) -> Result<cli::ExtractOpts> {
    if opts.is_empty() {
        return Err(Error::Parse("No valid files to extract birthmarks from.".to_string()));
    }
    Ok(opts)
}

mod reaggregator;

fn perform(opts: cli::OinkieOpts) -> Result<Vec<Duration>> {
    opts.init()?;
    use cli::OinkieCommand::*;
    match opts.command {
        Run(opts) => perform_run(opts),
        Compare(opts) => perform_compare(opts),
        Extract(opts) => validate_extract_opts(opts)
            .and_then(perform_extract),
        Reaggregate(opts) => reaggregator::perform(opts),
        Lift(opts) => perform_lift(opts),
        Info => perform_info(),
    }
}

fn perform_lift(opts: cli::LiftOpts) -> Result<Vec<Duration>> {
    let dest = opts.dest();
    let pb = ProgressBar::new(opts.len() as u64)
            .with_style(indicatif::ProgressStyle::with_template("[{elapsed_precise}] {bar:40.magenta/blue} {pos:>7}/{len:7} {msg}").unwrap())
            .with_message("Lifting binaries...");
    std::fs::create_dir_all(dest)
        .map_err(|e| Error::Io(dest.to_path_buf(), e))?;

    let start = Instant::now();
    let r = opts.iter().par_bridge().map(|path| {
        let e1 = Instant::now();
        let r = match opts.lifter_type() {
            cli::LifterType::Ghidra => lift_ghidra_impl(path, &opts),
            cli::LifterType::Llvm => Err(Error::Parse("LLVM lifter is not yet implemented.".to_string())),
            cli::LifterType::BinaryNinja => Err(Error::Parse("Binary Ninja lifter is not yet implemented.".to_string())),
        };
        pb.inc(1);
        r.map(|_| e1.elapsed())
    }).collect::<Result<Vec<_>>>();
    let duration = start.elapsed();
    println!("Lifting completed in {} nsec ({})", duration.as_nanos(), format_duration(duration));
    r
}

fn lift_ghidra_impl(path: &Path, opts: &cli::LiftOpts) -> Result<()> {
    let ghidra_home = find_ghidra_home(opts.home())?;
    let analyze_headless = ghidra_home.join("support/analyzeHeadless");
    if !analyze_headless.exists() {
        return Err(Error::Parse(format!("Ghidra headless analyzer not found at {:?}", analyze_headless)));
    }

    let dest_file = opts.dest().join(format!("{}.json", path.file_name().unwrap().to_str().unwrap()));
    if dest_file.exists() && opts.is_skip() {
        log::info!("Lifted JSON for {:?} already exists. Skipping lifting.", path);
        return Ok(());
    }

    let (script_path, _temp_dir) = if let Some(s) = opts.script() {
        (s.to_path_buf(), None)
    } else {
        let temp_dir = tempfile::Builder::new().prefix("oinkie_script").tempdir().map_err(|e| Error::Io(PathBuf::from("temp"), e))?;
        let script_file = temp_dir.path().join("HighPCodeLifter.java");
        std::fs::write(&script_file, DEFAULT_GHIDRA_SCRIPT)
            .map_err(|e| Error::Io(script_file.clone(), e))?;
        (script_file, Some(temp_dir))
    };

    let (proj_dir, _temp_proj_dir) = if let Some(i) = opts.intermediate_dir() {
        (i.to_path_buf(), None)
    } else {
        let temp_proj = tempfile::Builder::new().prefix("oinkie_proj").tempdir().map_err(|e| Error::Io(PathBuf::from("temp"), e))?;
        (temp_proj.path().to_path_buf(), Some(temp_proj))
    };

    let proj_name = path.file_name().unwrap().to_str().unwrap();
    
    let mut command = std::process::Command::new(&analyze_headless);
    command.arg(&proj_dir)
           .arg(proj_name)
           .arg("-import").arg(path)
           .arg("-scriptPath").arg(script_path.parent().unwrap())
           .arg("-postScript").arg(script_path.file_name().unwrap());

    log::info!("Executing Ghidra: {:?}", command);
    let output = command.output().map_err(|e| Error::Io(analyze_headless, e))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(Error::Parse(format!("Ghidra failed with status {}.\nSTDOUT: {}\nSTDERR: {}", output.status, stdout, stderr)));
    }

    // Move the generated JSON to the destination
    let generated_json = std::env::current_dir().unwrap().join(format!("{}.json", proj_name));
    if generated_json.exists() {
        std::fs::rename(&generated_json, &dest_file)
            .map_err(|e| Error::Io(dest_file, e))?;
    } else {
        return Err(Error::Parse(format!("Expected Ghidra to generate {:?}, but it was not found.", generated_json)));
    }

    Ok(())
}

fn find_ghidra_home(home_opt: Option<&Path>) -> Result<PathBuf> {
    if let Some(h) = home_opt {
        return Ok(h.to_path_buf());
    }
    if let Ok(h) = std::env::var("GHIDRA_HOME") {
        return Ok(PathBuf::from(h));
    }

    let candidates = [
        "/opt/homebrew/opt/ghidra/libexec",
        "/usr/local/opt/ghidra/libexec",
        "/opt/ghidra/libexec",
    ];

    for c in candidates {
        let p = PathBuf::from(c);
        if p.exists() {
            return Ok(p);
        }
    }

    Err(Error::Parse("GHIDRA_HOME not found. Please specify it via --home option or GHIDRA_HOME environment variable.".to_string()))
}

pub struct CompareResult {
    pub index: usize,
    pub similarity: f64,
    pub path1: PathBuf,
    pub path2: PathBuf,
    pub duration: std::time::Duration,
}

impl CompareResult {
    pub fn new(index: usize, similarity: f64, path1: PathBuf, path2: PathBuf, duration: std::time::Duration) -> Self {
        Self { index, similarity, path1, path2, duration }
    }

    pub fn to_csv(&self) -> String {
        format!("{},{},{},{},{}", self.index, self.similarity, self.path1.display(), self.path2.display(), self.duration.as_nanos())
    }

    pub fn parse(line: &str) -> Result<Self> {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() < 5 {
            return Err(Error::Parse(format!("Invalid compare result line format: {}", line)));
        }
        let index: usize = parts[0].parse()
            .map_err(|e| Error::ParseInt(parts[0].to_string(), e))?;
        let similarity: f64 = parts[1].parse()
            .map_err(|e| Error::Parse(format!("Failed to parse similarity value: {}", e)))?;
        let path1 = PathBuf::from(parts[2]);
        let path2 = PathBuf::from(parts[3]);
        let duration_nanos: u64 = parts[4].parse()
            .map_err(|e| Error::ParseInt(parts[4].to_string(), e))?;
        Ok(Self::new(index, similarity, path1, path2, Duration::from_nanos(duration_nanos)))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_compare_result() {
        let line = "21,0.9920060729565723,birthmarks/experiment0/bzip2-1.0.4_a83e5e24.json,birthmarks/experiment0/bzip2-1.0.8_981e34d2.json,515602167";
        let cr = CompareResult::parse(line)
            .expect("Failed to parse compare result");
        assert_eq!(cr.index, 21);
        assert!((cr.similarity - 0.9920060729565723).abs() < 1e-10);
        assert_eq!(cr.path1, PathBuf::from("birthmarks/experiment0/bzip2-1.0.4_a83e5e24.json"));
        assert_eq!(cr.path2, PathBuf::from("birthmarks/experiment0/bzip2-1.0.8_981e34d2.json"));
        assert_eq!(cr.duration.as_nanos(), 515602167);
    }
}