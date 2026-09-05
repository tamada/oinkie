mod cli;
mod values;

use clap::{Parser, ValueEnum};
use indicatif::ProgressBar;
use oinkie::prelude::*;
use rayon::prelude::*;
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;
use std::{path::Path, time::Instant};

fn load<T>(path: PathBuf) -> Result<T>
where
    T: TryFrom<PathBuf, Error = Error>,
{
    let s = Instant::now();
    let item: T = path.clone().try_into()?;
    let d = s.elapsed().as_millis();
    log::info!("Loading {path:?} done in {d} msec");
    Ok(item)
}

/// Reads a lifted program, letting the file say which representation it holds
/// rather than assuming Ghidra's.
fn load_program(path: &Path) -> Result<AnyProgram> {
    let s = Instant::now();
    let program = AnyProgram::load(path)?;
    log::info!("Loading {path:?} done in {} msec", s.elapsed().as_millis());
    Ok(program)
}

fn perform_run(opts: cli::RunOpts) -> Result<Vec<Duration>> {
    let start = Instant::now();
    let atype = opts.analysis_type()?;
    let dest = opts.dest();
    let comparing_count = opts.compare_count();
    let pbar = new_progress_bar(comparing_count * 3);
    let aggregator = opts.aggregator();
    std::fs::create_dir_all(dest).map_err(|e| Error::Io(dest.to_path_buf(), e))?;

    let results = opts
        .iter()
        .enumerate()
        .par_bridge()
        .map(|(i, (path1, path2))| {
            let dest_file = dest.join(format!("{i:05}.csv"));
            if dest_file.exists() && opts.is_skip() {
                log::info!(
                    "Similarity file for {:?} and {:?} already exists. Skipping comparison.",
                    path1,
                    path2
                );
                pbar.inc(3);
                read_result_file(&dest_file, i, path1, path2)
            } else {
                pbar.set_message(format!("Loading program from {:?}", path1.display()));
                let mut p1 = load_program(path1)?;
                p1.set_json_path(path1.to_path_buf());
                pbar.inc(1);
                pbar.set_message(format!("Loading program from {:?}", path2.display()));
                let mut p2 = load_program(path2)?;
                p2.set_json_path(path2.to_path_buf());
                pbar.inc(1);
                pbar.set_message(format!(
                    "Comparing two programs ({}/{})",
                    i + 1,
                    comparing_count
                ));
                let result = atype.comparator().compare_any(&p1, &p2, aggregator)?;
                pbar.inc(1);
                result.store(&dest_file)?;
                Ok(CompareResult::new(
                    i,
                    result.similarity(),
                    p1.path().to_path_buf(),
                    p2.path().to_path_buf(),
                    result.duration(),
                ))
            }
        })
        .collect::<Vec<_>>();
    pbar.finish();
    let results = Error::vec_result_to_result_vec(results)?;
    let dest_file = dest.join("results.csv");
    store_and_get_durations(results, &dest_file, start)
}

/// Read the comparison result from the given CSV file and return a CompareResult struct.
/// This function skips actual comparison and shorten the comparison time if the result file already exists.
/// The CSV file is expected to have a line starting with "result," followed by the duration in nanoseconds and the similarity value, separated by commas.
///
/// "Expected" is enforced. Without the line there is no score to report, and
/// returning one anyway meant reporting 0.0 -- a real answer, and the wrong
/// one -- for a file an interrupted run left half written.
fn read_result_file(
    dest_path: &Path,
    index: usize,
    _path1: &Path,
    _path2: &Path,
) -> Result<CompareResult> {
    let file = std::fs::File::open(dest_path).map_err(|e| Error::Io(dest_path.to_path_buf(), e))?;
    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .has_headers(false)
        .from_reader(file);

    let mut original_path1 = PathBuf::new();
    let mut original_path2 = PathBuf::new();
    // An Option rather than a zeroed pair, so that "the file never said" is
    // not spelled the same way as "the score was zero". It used to be the
    // latter: a file holding the pair but no result line came back as a
    // successful comparison scoring 0.0, which is what an interrupted run
    // leaves behind and what --skip then reads.
    let mut scored: Option<(u64, f64)> = None;

    for result in reader.records() {
        let record = result.map_err(Error::Csv)?;
        match record.get(0) {
            Some("result") => {
                if record.len() < 3 {
                    return Err(Error::Parse(format!(
                        "Invalid result line format in {}",
                        dest_path.display()
                    )));
                }
                let duration_nanos = record[1]
                    .parse()
                    .map_err(|e| Error::ParseInt(record[1].to_string(), e))?;
                let similarity = record[2].parse().map_err(|e| {
                    Error::Parse(format!(
                        "Failed to parse similarity value in {}: {}",
                        dest_path.display(),
                        e
                    ))
                })?;
                scored = Some((duration_nanos, similarity));
            }
            Some("left") => {
                if let Some(s) = record.get(3) {
                    original_path1 = PathBuf::from(s);
                }
            }
            Some("right") => {
                if let Some(s) = record.get(3) {
                    original_path2 = PathBuf::from(s);
                }
            }
            _ => {}
        }
    }

    let Some((duration_nanos, similarity)) = scored else {
        return Err(Error::Parse(format!(
            "Result line not found in {}",
            dest_path.display()
        )));
    };
    if original_path1.as_os_str().is_empty() || original_path2.as_os_str().is_empty() {
        return Err(Error::Parse(format!(
            "Birthmark paths not found in {}",
            dest_path.display()
        )));
    }

    Ok(CompareResult::new(
        index,
        similarity,
        original_path1,
        original_path2,
        Duration::from_nanos(duration_nanos),
    ))
}

fn new_progress_bar(len: usize) -> ProgressBar {
    ProgressBar::new(len as u64).with_style(
        indicatif::ProgressStyle::with_template(
            "[{elapsed_precise}/({per_sec})] {bar:40} {pos:>7}/{len:7} {msg}",
        )
        .unwrap(),
    )
}

fn perform_compare(opts: cli::CompareOpts) -> Result<Vec<Duration>> {
    let start = Instant::now();
    let comparator = opts.comparator();
    let dest = opts.dest();
    let comparing_count = opts.compare_count();
    let pbar = new_progress_bar(comparing_count * 3);
    let aggregator = opts.aggregator();
    std::fs::create_dir_all(dest).map_err(|e| Error::Io(dest.to_path_buf(), e))?;
    let r = opts
        .iter()
        .enumerate()
        .par_bridge()
        .map(|(i, (path1, path2))| {
            compare_impl(
                (i, (path1, path2)),
                &comparator,
                dest,
                &pbar,
                comparing_count,
                opts.is_skip(),
                aggregator,
            )
        })
        .collect::<Vec<_>>();
    pbar.finish();
    let result_file = dest.join("results.csv");
    Error::vec_result_to_result_vec(r).and_then(|r| store_and_get_durations(r, &result_file, start))
}

pub(crate) fn store_and_get_durations(
    results: Vec<CompareResult>,
    destcsv: &Path,
    start: Instant,
) -> Result<Vec<Duration>> {
    let mut file =
        std::fs::File::create(destcsv).map_err(|e| Error::Io(destcsv.to_path_buf(), e))?;
    let ritems = results
        .into_iter()
        .map(|r| {
            let csv = r.to_csv();
            match writeln!(file, "{}", csv) {
                Ok(_) => Ok(r.duration),
                Err(e) => Err(Error::Io(destcsv.to_path_buf(), e)),
            }
        })
        .collect::<Vec<_>>();
    let duration = start.elapsed();
    let _ = writeln!(
        file,
        "total duration,{},{}",
        duration.as_nanos(),
        format_duration(duration)
    );
    Error::vec_result_to_result_vec(ritems)
}

fn compare_impl(
    tuple: (usize, (&Path, &Path)),
    comparator: &Comparator,
    dest: &Path,
    pbar: &ProgressBar,
    comparing_count: usize,
    skip: bool,
    aggregator: &Aggregator,
) -> Result<CompareResult> {
    let (i, (path1, path2)) = tuple;
    let dest_file = dest.join(format!("{i:05}.csv"));
    log::info!(
        "compare_impl(dest: {} (exists: {}), skip: {skip})",
        dest_file.display(),
        dest_file.exists()
    );
    if dest_file.exists() && skip {
        log::info!(
            "Similarity file for {:?} and {:?} already exists. Skipping comparison.",
            path1,
            path2
        );
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
        log::info!(
            "Comparing birthmarks (len: {} x {}) progress: {}/{comparing_count}",
            b1.len(),
            b2.len(),
            i + 1
        );
        pbar.set_message(format!(
            "Comparing birthmarks (len: {} x {}) progress: {}/{comparing_count}",
            b1.len(),
            b2.len(),
            i + 1
        ));
        let result = comparator.compare_birthmarks(&b1, &b2, aggregator)?;
        pbar.inc(1);
        result.store(&dest_file)?;
        Ok(CompareResult::new(
            i,
            result.similarity(),
            b1.path().to_path_buf(),
            b2.path().to_path_buf(),
            result.duration(),
        ))
    }
}

fn perform_extract(opts: cli::ExtractOpts) -> Result<Vec<Duration>> {
    let dest = opts.dest();
    let pb = ProgressBar::new(opts.len() as u64)
        .with_style(
            indicatif::ProgressStyle::with_template(
                "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
            )
            .unwrap(),
        )
        .with_message("Extracting birthmarks...");
    std::fs::create_dir_all(dest).map_err(|e| Error::Io(dest.to_path_buf(), e))?;
    let extractor = opts.extractor();
    let start = Instant::now();
    let r = opts
        .iter()
        .par_bridge()
        .map(|path| {
            let e1 = Instant::now();
            let r = match extract_impl(path, dest, &extractor, opts.is_skip()) {
                Ok(_) => Ok(e1.elapsed()),
                Err(e) => Err(e),
            };
            pb.inc(1);
            r
        })
        .collect::<Result<Vec<_>>>();
    let duration = start.elapsed();
    // Only on success. This used to print unconditionally and then return the
    // error, so a failed run announced that it had completed and the failure
    // came immediately below it -- which is the same thing #83 is about, one
    // layer up.
    if r.is_ok() {
        println!(
            "Extraction completed in {} nsec ({})",
            duration.as_nanos(),
            format_duration(duration)
        );
    }
    r
}

fn extract_impl(path: &Path, dest: &Path, extractor: &Extractor, skip: bool) -> Result<()> {
    let file_name = oinkie::extractor::dest_file_name(path)?;
    let dest_path = dest.join(file_name);
    if dest_path.exists() && skip {
        log::info!(
            "Birthmark for {:?} already exists. Skipping extraction.",
            path
        );
        return Ok(());
    }
    let p = load_program(path)?;
    let birthmarks = extractor.extract_any(&p)?;
    let json =
        serde_json::to_string_pretty(&birthmarks).map_err(|e| Error::Json(dest_path.clone(), e))?;
    std::fs::write(&dest_path, json).map_err(|e| Error::Io(dest_path.clone(), e))?;
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
    println!(
        "Oinkie is a tool for detecting the code theft with Ghidra P-code as birthmarks.
The birthmark is a unique characteristic of a program that can be used to identify it.
Oinkie extracts birthmarks from given codes and compares them to calculate the similarities."
    );
    println!("============ Birthmarks =============");
    BirthmarkType::advertised().for_each(|b| {
        println!("- {:<20}  {}", b.to_string(), b.description());
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
        return Err(Error::Parse(
            "No valid files to extract birthmarks from.".to_string(),
        ));
    }
    Ok(opts)
}

mod reaggregator;

#[cfg(feature = "mcp")]
mod mcp;

fn perform(opts: cli::OinkieOpts) -> Result<Vec<Duration>> {
    opts.init()?;
    use cli::OinkieCommand::*;
    match opts.command {
        Run(opts) => perform_run(opts),
        Compare(opts) => perform_compare(opts),
        Extract(opts) => validate_extract_opts(opts).and_then(perform_extract),
        Reaggregate(opts) => reaggregator::perform(opts),
        Lift(opts) => perform_lift(opts),
        Info => perform_info(),
        #[cfg(feature = "mcp")]
        Mcp(opts) => mcp::perform(&opts),
    }
}

/// How many lifts will actually run at once.
///
/// Asking for more jobs than there are files does not produce more
/// concurrency, so the smaller of the two is the real answer. Deriving it once
/// keeps the notice and the branch below from disagreeing about whether
/// anything is running in parallel.
fn effective_jobs(requested: usize, files: usize) -> usize {
    requested.min(files.max(1))
}

/// What to say before lifting several files at once, or `None` when only one
/// lift will run at a time.
///
/// Ghidra compiles its SLEIGH language definitions on first use and caches
/// them inside its own installation, so concurrent lifts against an
/// installation nobody has used yet race to write the same file. The loser
/// reads a half-written one -- and `analyzeHeadless` exits successfully
/// regardless, so it arrives as a missing output rather than as an error.
/// Once the cache is built it cannot happen again, which is why this is a
/// notice rather than a refusal.
///
/// Broken across lines rather than left as one: CI logs and narrow terminals
/// truncate, and the sentence that says what to do about it is at the end.
fn parallel_lift_notice(jobs: usize) -> Option<String> {
    (jobs > 1).then(|| {
        format!(
            "notice: lifting up to {jobs} files at a time.\n\
             Ghidra builds its language cache on first use, inside its own installation,\n\
             and parallel lifts can corrupt a cache that has not been built yet --\n\
             reported as a missing output, not as an error.\n\
             If Ghidra was installed recently, lift one file without -j first."
        )
    })
}

fn perform_lift(opts: cli::LiftOpts) -> Result<Vec<Duration>> {
    let dest = opts.dest();
    let pb = ProgressBar::new(opts.len() as u64)
        .with_style(
            indicatif::ProgressStyle::with_template(
                "[{elapsed_precise}] {bar:40.magenta/blue} {pos:>7}/{len:7} {msg}",
            )
            .unwrap(),
        )
        .with_message("Lifting binaries...");
    std::fs::create_dir_all(dest).map_err(|e| Error::Io(dest.to_path_buf(), e))?;

    let lifter: Box<dyn Lifter + Sync> = LifterBuilder::new(opts.lifter_type())
        .home(opts.home().map(|p| p.to_path_buf()))
        .script(opts.script().map(|p| p.to_path_buf()))
        .intermediate_dir(opts.intermediate_dir().map(|p| p.to_path_buf()))
        .build()?;

    let lift_one = |path: &PathBuf| {
        let e1 = Instant::now();
        let dest_file = dest.join(format!(
            "{}.json",
            path.file_name().unwrap().to_str().unwrap()
        ));
        if dest_file.exists() && opts.is_skip() {
            log::info!(
                "Lifted JSON for {:?} already exists. Skipping lifting.",
                path
            );
        } else {
            lifter.lift(path, &dest_file)?;
        }
        // Counted whether it was lifted or skipped: the bar measures inputs
        // dealt with, and a --skip run that left it short of the total looked
        // like it had stopped early.
        pb.inc(1);
        Ok(e1.elapsed())
    };

    let jobs = effective_jobs(opts.jobs(), opts.len());
    if let Some(notice) = parallel_lift_notice(jobs) {
        eprintln!("{notice}");
    }

    let start = Instant::now();
    let r = if jobs == 1 {
        opts.iter().map(lift_one).collect::<Result<Vec<_>>>()
    } else {
        // A pool of our own rather than rayon's global one, whose size is the
        // machine's core count. --jobs has to bound decompiler processes, not
        // rayon tasks.
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(jobs)
            .build()
            .map_err(|e| Error::Parse(format!("could not start {jobs} lift jobs: {e}")))?;
        pool.install(|| {
            opts.iter()
                .par_bridge()
                .map(lift_one)
                .collect::<Result<Vec<_>>>()
        })
    };
    let duration = start.elapsed();
    // Only on success. This used to print unconditionally and then return the
    // error, so a failed run announced that it had completed and the failure
    // came immediately below it -- which is the same thing #83 is about, one
    // layer up.
    if r.is_ok() {
        println!(
            "Lifting completed in {} nsec ({})",
            duration.as_nanos(),
            format_duration(duration)
        );
    }
    r
}

pub struct CompareResult {
    pub index: usize,
    pub similarity: f64,
    pub path1: PathBuf,
    pub path2: PathBuf,
    pub duration: std::time::Duration,
}

impl CompareResult {
    pub fn new(
        index: usize,
        similarity: f64,
        path1: PathBuf,
        path2: PathBuf,
        duration: std::time::Duration,
    ) -> Self {
        Self {
            index,
            similarity,
            path1,
            path2,
            duration,
        }
    }

    pub fn to_csv(&self) -> String {
        format!(
            "{},{},{},{},{}",
            self.index,
            self.similarity,
            escape_csv_string(&self.path1.display().to_string()),
            escape_csv_string(&self.path2.display().to_string()),
            self.duration.as_nanos()
        )
    }

    pub fn parse(line: &str) -> Result<Self> {
        let mut reader = csv::ReaderBuilder::new()
            .flexible(true)
            .has_headers(false)
            .from_reader(line.as_bytes());
        let record = match reader.records().next() {
            Some(r) => {
                r.map_err(|e| Error::Parse(format!("Invalid compare result line format: {e}")))?
            }
            None => {
                return Err(Error::Parse(format!(
                    "Invalid compare result line format: {}",
                    line
                )));
            }
        };
        if record.len() < 5 {
            return Err(Error::Parse(format!(
                "Invalid compare result line format: {}",
                line
            )));
        }
        let index: usize = record[0]
            .parse()
            .map_err(|e| Error::ParseInt(record[0].to_string(), e))?;
        let similarity: f64 = record[1]
            .parse()
            .map_err(|e| Error::Parse(format!("Failed to parse similarity value: {}", e)))?;
        let path1 = PathBuf::from(&record[2]);
        let path2 = PathBuf::from(&record[3]);
        let duration_nanos: u64 = record[4]
            .parse()
            .map_err(|e| Error::ParseInt(record[4].to_string(), e))?;
        Ok(Self::new(
            index,
            similarity,
            path1,
            path2,
            Duration::from_nanos(duration_nanos),
        ))
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
            if e.kind() == clap::error::ErrorKind::DisplayHelp
                || e.kind() == clap::error::ErrorKind::DisplayVersion
            {
                println!("{}", e.render().ansi());
                0
            } else {
                // No "Error: " in front: clap's own message already begins
                // "error: ", and prefixing it gave "Error: error: ..." (#62).
                // The other arm keeps its prefix, since oinkie's own errors
                // carry none.
                //
                // Display rather than `render().ansi()`, which would put
                // colour codes into a redirected stderr.
                eprintln!("{e}");
                1
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            2
        }
        Ok(_) => 0,
    };
    std::process::exit(status_code);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_ghidra_home_from_opt() {
        let opt = PathBuf::from("/custom/path");
        let result = LifterType::Ghidra.find_home(Some(&opt)).unwrap();
        assert_eq!(result, opt);
    }

    /// The three backends that are not implemented still have to say what to
    /// set, since a message naming GHIDRA_HOME for Binary Ninja is worse than
    /// no message at all.
    #[test]
    fn test_each_backend_names_its_own_environment_variable() {
        for (lifter, env) in [
            (LifterType::Ghidra, "GHIDRA_HOME"),
            (LifterType::IDAPro, "IDA_HOME"),
            (LifterType::BinaryNinja, "BINARY_NINJA_HOME"),
        ] {
            let spec = lifter.home_spec().expect("this backend has an install dir");
            assert_eq!(spec.env, env, "{}", lifter.name());
        }
        // angr is imported rather than installed, so --home means nothing for
        // it and the error says so instead of naming a variable.
        assert!(LifterType::Angr.home_spec().is_none());
        let err = LifterType::Angr.find_home(None).unwrap_err().to_string();
        assert!(err.contains("angr"), "unhelpful message: {err}");
    }

    /// A similarity CSV as `compare` writes one, written to a temp file so
    /// that `read_result_file` can be exercised directly.
    ///
    /// Through the CLI it is only reachable with `--skip` over a directory
    /// left by an earlier run, which is why every one of its error paths was
    /// uncovered (#28): the end-to-end tests never take the branch, and none
    /// of them arranges a *malformed* leftover.
    fn read_result(body: &str) -> Result<CompareResult> {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("00000.csv");
        std::fs::write(&path, body).unwrap();
        read_result_file(&path, 7, Path::new("a.json"), Path::new("b.json"))
    }

    const A_WHOLE_RESULT: &str = "result,28083,0.75\n\
        left,program,hello_clang,bin/hello_clang,1,1,pcodes/hello_clang.json\n\
        right,program,hello_gcc,bin/hello_gcc,1,1,pcodes/hello_gcc.json\n\
        matrix,,entry\n\
        0,entry,0.75\n";

    #[test]
    fn test_a_stored_result_is_read_back_whole() {
        let cr = read_result(A_WHOLE_RESULT).unwrap();
        // the index is the caller's, not the file's: the file is named for it
        assert_eq!(cr.index, 7);
        assert_eq!(cr.similarity, 0.75);
        assert_eq!(cr.path1, PathBuf::from("bin/hello_clang"));
        assert_eq!(cr.path2, PathBuf::from("bin/hello_gcc"));
        assert_eq!(cr.duration, Duration::from_nanos(28083));
    }

    /// The paths come from the `left` and `right` records rather than from
    /// the arguments, which is what makes a `--skip` rerun agree with a fresh
    /// one about which file was on which side.
    #[test]
    fn test_the_pair_is_read_from_the_file_not_from_the_arguments() {
        let cr = read_result(A_WHOLE_RESULT).unwrap();
        assert_ne!(cr.path1, PathBuf::from("a.json"));
        assert_ne!(cr.path2, PathBuf::from("b.json"));
    }

    #[test]
    fn test_a_malformed_stored_result_says_what_is_wrong_with_it() {
        let cases = [
            (
                "result,28083\nleft,,,x\nright,,,y\n",
                "Invalid result line format",
            ),
            (
                "result,notanumber,0.75\nleft,,,x\nright,,,y\n",
                "Parse int error",
            ),
            (
                "result,28083,notafloat\nleft,,,x\nright,,,y\n",
                "Failed to parse similarity value",
            ),
            (
                "result,28083,0.75\nright,,,y\n",
                "Birthmark paths not found",
            ),
            ("result,28083,0.75\nleft,,,x\n", "Birthmark paths not found"),
            // A file with the pair but no score. Reachable: a run interrupted
            // part-way through writing leaves one, and --skip reads it.
            ("left,,,x\nright,,,y\n", "Result line not found"),
            ("", "Result line not found"),
        ];
        for (body, expected) in cases {
            let Err(e) = read_result(body) else {
                panic!("should have been refused: {body:?}");
            };
            assert!(
                e.to_string().contains(expected),
                "{body:?} gave {e}, expected something containing {expected:?}"
            );
        }
    }

    #[test]
    fn test_a_result_file_that_is_not_there_is_an_io_error() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("00000.csv");
        let Err(e) = read_result_file(&missing, 0, Path::new("a"), Path::new("b")) else {
            panic!("a missing file should not read");
        };
        assert!(e.to_string().starts_with("IO error for"), "{e}");
    }

    /// `oinkie info` prints each algorithm's clap help with two `unwrap()`s,
    /// so an algorithm added without a doc comment panics the command rather
    /// than printing a blank line. Calling it is the guard.
    #[test]
    fn test_info_prints_every_algorithm_without_panicking() {
        assert!(perform_info().is_ok());
    }

    fn run_analysis(name: &str) -> Result<AnalysisType> {
        let opts = cli::OinkieOpts::try_parse_from(vec!["oinkie", "run", "-a", name, "x.json"])
            .map_err(Error::Clap)?;
        let cli::OinkieCommand::Run(run_opts) = opts.command else {
            panic!("Expected Run command");
        };
        run_opts.analysis_type()
    }

    fn extract_birthmark(name: &str) -> Result<()> {
        cli::OinkieOpts::try_parse_from(vec!["oinkie", "extract", "-b", name, "x.json"])
            .map_err(Error::Clap)?;
        Ok(())
    }

    /// The CLI used to spell these its own way -- `op3gram-set-dice` for an
    /// analysis and `op-tri-gram-set` for a birthmark -- because each was a
    /// hand-written `ValueEnum` whose clap names were derived from Rust
    /// identifiers. Neither spelling is what the library parses, and the
    /// library is the parser now (#25).
    #[test]
    fn test_a_kgram_is_named_the_way_the_library_names_it() {
        let at = run_analysis("op-3gram-set-dice").expect("the library spelling should parse");
        assert_eq!(at.birthmark, BirthmarkType::OpKgramSet(3));
        assert!(extract_birthmark("op-3gram-set").is_ok());
    }

    #[test]
    fn test_the_spellings_the_cli_used_to_invent_are_gone() {
        assert!(
            run_analysis("op3gram-set-dice").is_err(),
            "the old --analysis spelling still parses"
        );
        assert!(
            extract_birthmark("op-tri-gram-set").is_err(),
            "the old --birthmark-type spelling still parses"
        );
    }

    /// The k ceiling was a property of the hand-written list, not of the
    /// grammar. The list still stops -- a completion list has to -- but the
    /// option does not.
    #[test]
    fn test_a_k_past_the_advertised_list_is_still_accepted() {
        let beyond = MAX_ADVERTISED_K + 1;
        let at = run_analysis(&format!("op-{beyond}gram-freq-cosine")).unwrap();
        assert_eq!(at.birthmark, BirthmarkType::OpKgramFreq(beyond));
        assert!(extract_birthmark(&format!("op-{beyond}gram-freq")).is_ok());
    }

    /// Parsing at the option rather than after the run starts is what keeps a
    /// refused pairing a usage error, and the message is the library's, which
    /// names the pairing that was meant.
    #[test]
    fn test_a_refused_pairing_is_reported_while_parsing() {
        let Err(e) = run_analysis("op-seq-euclidean") else {
            panic!("op-seq-euclidean should be refused");
        };
        let err = e.to_string();
        assert!(
            err.contains("op-freq-euclidean"),
            "does not name the canonical pairing: {err}"
        );
    }

    #[test]
    fn test_the_defaults_are_names_the_library_parses() {
        let at = run_analysis("op-set-jaccard").unwrap();
        assert_eq!(at.birthmark, BirthmarkType::OpSet);
        let opts = cli::OinkieOpts::try_parse_from(vec!["oinkie", "run", "x.json"]).unwrap();
        let cli::OinkieCommand::Run(run_opts) = opts.command else {
            panic!("Expected Run command");
        };
        assert_eq!(run_opts.analysis_type().unwrap().birthmark, at.birthmark);
        assert!(cli::OinkieOpts::try_parse_from(vec!["oinkie", "extract", "x.json"]).is_ok());
    }

    /// Lifting is serial unless asked otherwise: it runs a whole decompiler
    /// process per file, and several of them against a Ghidra installation
    /// whose language cache has not been built yet corrupt it (#54).
    #[test]
    fn test_lift_is_serial_by_default() {
        let opts = cli::OinkieOpts::try_parse_from(vec!["oinkie", "lift", "bin1"]).unwrap();
        let cli::OinkieCommand::Lift(lift_opts) = opts.command else {
            panic!("Expected Lift command");
        };
        assert_eq!(lift_opts.jobs(), 1);
        assert_eq!(
            parallel_lift_notice(effective_jobs(lift_opts.jobs(), 8)),
            None
        );
    }

    #[test]
    fn test_lift_jobs_is_taken_and_announced() {
        let opts =
            cli::OinkieOpts::try_parse_from(vec!["oinkie", "lift", "-j", "4", "bin1"]).unwrap();
        let cli::OinkieCommand::Lift(lift_opts) = opts.command else {
            panic!("Expected Lift command");
        };
        assert_eq!(lift_opts.jobs(), 4);
        let notice = parallel_lift_notice(effective_jobs(lift_opts.jobs(), 8))
            .expect("asking for 4 must say why");
        assert!(notice.contains("4 files at a time"), "{notice}");
        assert!(notice.contains("language cache"), "{notice}");
        assert!(
            notice.lines().count() > 1,
            "the notice must survive a narrow terminal: {notice}"
        );
    }

    /// Asking for more jobs than there are files buys nothing, so it is not
    /// claimed in the notice and does not build a pool below.
    #[test]
    fn test_jobs_are_capped_by_the_number_of_files() {
        assert_eq!(effective_jobs(4, 2), 2);
        assert_eq!(effective_jobs(4, 1), 1);
        assert_eq!(effective_jobs(1, 8), 1);
        // Nothing to lift is still one pass over an empty list, not zero jobs.
        assert_eq!(effective_jobs(4, 0), 1);

        assert_eq!(parallel_lift_notice(effective_jobs(4, 1)), None);
        let notice = parallel_lift_notice(effective_jobs(4, 2)).unwrap();
        assert!(notice.contains("2 files at a time"), "{notice}");
    }

    /// Zero jobs lifts nothing, so it is rejected at parse time rather than
    /// silently doing nothing -- and in terms of the option, not of the type
    /// behind it.
    #[test]
    fn test_lift_jobs_rejects_zero() {
        let err = cli::OinkieOpts::try_parse_from(vec!["oinkie", "lift", "-j", "0", "bin1"])
            .expect_err("zero jobs must not parse")
            .to_string();
        assert!(err.contains("must be at least 1"), "{err}");
    }

    #[test]
    fn test_parse_lift_opts() {
        let args = vec![
            "oinkie", "lift", "--home", "/ghidra", "--dest", "out", "--skip", "bin1", "bin2",
        ];
        let opts = cli::OinkieOpts::try_parse_from(args).unwrap();
        if let cli::OinkieCommand::Lift(lift_opts) = opts.command {
            assert_eq!(lift_opts.home(), Some(Path::new("/ghidra")));
            assert_eq!(lift_opts.dest(), Path::new("out"));
            assert!(lift_opts.is_skip());
            let files: Vec<_> = lift_opts.iter().collect();
            assert_eq!(files.len(), 2);
            assert_eq!(files[0], &PathBuf::from("bin1"));
            assert_eq!(files[1], &PathBuf::from("bin2"));
        } else {
            panic!("Expected Lift command");
        }
    }

    #[test]
    fn test_compare_result_roundtrip_with_comma_in_path() {
        let original = CompareResult::new(
            3,
            0.5,
            PathBuf::from("dir,with,commas/a.json"),
            PathBuf::from("dir/\"quoted\"/b.json"),
            Duration::from_nanos(123),
        );
        let line = original.to_csv();
        let parsed = CompareResult::parse(&line).expect("Failed to parse escaped compare result");
        assert_eq!(parsed.index, original.index);
        assert_eq!(parsed.path1, original.path1);
        assert_eq!(parsed.path2, original.path2);
        assert_eq!(parsed.duration, original.duration);
    }

    #[test]
    fn test_parse_compare_result() {
        let line = "21,0.9920060729565723,birthmarks/experiment0/bzip2-1.0.4_a83e5e24.json,birthmarks/experiment0/bzip2-1.0.8_981e34d2.json,515602167";
        let cr = CompareResult::parse(line).expect("Failed to parse compare result");
        assert_eq!(cr.index, 21);
        assert!((cr.similarity - 0.9920060729565723).abs() < 1e-10);
        assert_eq!(
            cr.path1,
            PathBuf::from("birthmarks/experiment0/bzip2-1.0.4_a83e5e24.json")
        );
        assert_eq!(
            cr.path2,
            PathBuf::from("birthmarks/experiment0/bzip2-1.0.8_981e34d2.json")
        );
        assert_eq!(cr.duration.as_nanos(), 515602167);
    }
}
