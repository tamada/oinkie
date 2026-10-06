use std::ffi::OsStr;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::CompareResult;
use crate::cli::{self, MinElements};
use ndarray::{Array2, Axis};
use oinkie::birthmarks::Birthmark;
use oinkie::compare::Aggregator;
use oinkie::{Error, Result};
use rayon::prelude::*;
use rustc_hash::FxHashMap;

pub(crate) fn perform(opts: cli::ReviewOpts) -> Result<Vec<Duration>> {
    let start = Instant::now();
    let review = review_all(
        opts.score_directory(),
        opts.aggregator(),
        opts.min_elements(),
        &|_| Ok(()),
    )?;
    store(review, opts.dest_file(), start)
}

/// The scores of a directory, recomputed, and the threshold they were
/// recomputed under when one was given.
pub(crate) struct Review {
    pub(crate) results: Vec<CompareResult>,
    pub(crate) min_elements: Option<(MinElements, f64)>,
}

/// Writes the summary, and below it the threshold the scores came from: scores
/// under different thresholds are not comparable with each other, so a summary
/// that did not say which one it used would be numbers without units.
pub(crate) fn store(review: Review, dest: &Path, start: Instant) -> Result<Vec<Duration>> {
    let durations = super::store_and_get_durations(review.results, dest, start)?;
    if let Some((given, threshold)) = review.min_elements {
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(dest)
            .map_err(|e| Error::Io(dest.to_path_buf(), e))?;
        writeln!(file, "min elements,{given},{threshold}")
            .map_err(|e| Error::Io(dest.to_path_buf(), e))?;
    }
    Ok(durations)
}

/// Recomputes every score in a directory, and hands them back.
///
/// Split out of [`perform`], which now only decides where to write them,
/// because the MCP tool wants the scores themselves rather than a CSV. Both
/// callers get the same numbers by construction rather than by two
/// implementations agreeing.
///
/// Given `min_elements`, each pair's functions with fewer elements than the
/// threshold are dropped before aggregating -- a row and a column of the
/// stored matrix each -- so nothing is compared again. The counts come from the
/// birthmarks the pair CSVs name, each passed to `confine` before it is read:
/// the paths come from the files rather than from whoever asked, and the MCP
/// server reads only under its roots.
pub(crate) fn review_all(
    score_dir: &Path,
    aggregator: &Aggregator,
    min_elements: Option<&MinElements>,
    confine: &(dyn Fn(&Path) -> Result<()> + Sync),
) -> Result<Review> {
    let start = Instant::now();
    let crs = load_results(score_dir)?;
    log::info!("read the previous results {:?}", start.elapsed());
    let filter = min_elements
        .map(|m| Filter::new(&crs, score_dir, m, confine))
        .transpose()?;
    let results = crs
        .iter()
        .map(|cr| review_pair(cr, score_dir, aggregator, filter.as_ref()))
        .collect::<Vec<_>>();
    let results = Error::vec_result_to_result_vec(results)?;
    Ok(Review {
        results: results.into_iter().map(|(cr, _)| cr).collect(),
        min_elements: min_elements.cloned().zip(filter.map(|f| f.threshold)),
    })
}

/// What `--min-elements` needs of every birthmark in the directory: each
/// function's name and element count, in the order the matrices use.
struct Filter {
    threshold: f64,
    birthmarks: FxHashMap<PathBuf, Vec<(String, usize)>>,
}

impl Filter {
    /// Reads the birthmarks every pair names, once each, and resolves the
    /// threshold. The mean is over every function of every distinct
    /// birthmark, as one population, so that the threshold is the same for
    /// every pair: per-birthmark means would judge a program of small
    /// functions leniently, and per-pair means would judge one birthmark
    /// differently depending on its partner.
    ///
    /// Only the headers are read here. The matrices are read one at a time
    /// afterwards, as without the option, rather than all held at once.
    fn new(
        crs: &[CompareResult],
        score_dir: &Path,
        min: &MinElements,
        confine: &(dyn Fn(&Path) -> Result<()> + Sync),
    ) -> Result<Self> {
        let mut paths = Vec::new();
        for cr in crs {
            let csv = pair_csv(cr.index, score_dir);
            let (left, right) = recorded_birthmarks(&csv)?;
            paths.push(left);
            paths.push(right);
        }
        paths.sort();
        paths.dedup();
        let loaded = paths
            .par_iter()
            .map(|p| {
                confine(p)?;
                let b: Birthmark = p.clone().try_into()?;
                let counts = b
                    .functions()
                    .iter()
                    .map(|f| (f.name().to_string(), f.len()))
                    .collect::<Vec<_>>();
                Ok((p.clone(), counts))
            })
            .collect::<Vec<_>>();
        let birthmarks = Error::vec_result_to_result_vec(loaded)?
            .into_iter()
            .collect::<FxHashMap<_, _>>();
        let threshold = match min {
            MinElements::Count(n) => *n as f64,
            MinElements::Ratio(r) => {
                let (sum, n) = birthmarks
                    .values()
                    .flatten()
                    .fold((0usize, 0usize), |(sum, n), (_, len)| (sum + len, n + 1));
                let mean = if n == 0 { 0.0 } else { sum as f64 / n as f64 };
                r * mean
            }
        };
        log::info!("min elements {min}: threshold {threshold}");
        Ok(Filter {
            threshold,
            birthmarks,
        })
    }

    /// The indices of the functions to keep, after checking that the
    /// birthmark is still the one the matrix was computed from: one extracted
    /// again since would hand its counts to another's functions.
    fn keep(&self, birthmark: &Path, names: &[String], csv: &Path) -> Result<Vec<usize>> {
        let functions = &self.birthmarks[birthmark];
        let matches =
            functions.len() == names.len() && functions.iter().zip(names).all(|((f, _), n)| f == n);
        if !matches {
            return Err(Error::Parse(format!(
                "{}: its functions are not the ones {} was computed from; \
                 the birthmark has changed since",
                birthmark.display(),
                csv.display()
            )));
        }
        Ok(functions
            .iter()
            .enumerate()
            .filter(|(_, (_, len))| *len as f64 >= self.threshold)
            .map(|(i, _)| i)
            .collect())
    }
}

fn pair_csv(index: usize, score_dir: &Path) -> PathBuf {
    score_dir.join(format!("{index:05}.csv"))
}

/// The birthmark files a pair CSV says it was computed from.
fn recorded_birthmarks(csv: &Path) -> Result<(PathBuf, PathBuf)> {
    let file = std::fs::File::open(csv).map_err(|e| Error::Io(csv.to_path_buf(), e))?;
    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .has_headers(false)
        .from_reader(file);
    let (mut left, mut right) = (None, None);
    for record in reader.records() {
        let record = record.map_err(Error::Csv)?;
        match record.get(0) {
            Some("left") => left = recorded(&record).birthmark,
            Some("right") => right = recorded(&record).birthmark,
            Some("matrix") => break,
            _ => continue,
        }
    }
    match (left, right) {
        (Some(l), Some(r)) => Ok((l, r)),
        _ => Err(Error::Parse(format!(
            "{}: does not name the birthmarks it compared, so their element counts \
             cannot be found. Directories written by compare, or by run since v0.8.0, name them",
            csv.display()
        ))),
    }
}

fn review_pair(
    cr: &CompareResult,
    score_dir: &Path,
    aggregator: &Aggregator,
    filter: Option<&Filter>,
) -> Result<(CompareResult, Duration)> {
    let start = Instant::now();
    let csv = pair_csv(cr.index, score_dir);
    log::info!("load comparison file {} {:?}", cr.index, csv.display());
    let stored = load_comparison(&csv)?;
    let matrix = match filter {
        None => stored.matrix,
        Some(f) => {
            let (Some(left), Some(right)) = (&stored.left.birthmark, &stored.right.birthmark)
            else {
                return Err(Error::Parse(format!(
                    "{}: does not name the birthmarks it compared",
                    csv.display()
                )));
            };
            let cols = f.keep(left, &stored.left.functions, &csv)?;
            let rows = f.keep(right, &stored.right.functions, &csv)?;
            stored.matrix.select(Axis(0), &rows).select(Axis(1), &cols)
        }
    };
    let similarity = score(&matrix, aggregator)?;
    Ok((
        CompareResult::new(
            cr.index,
            similarity,
            stored.left.path,
            stored.right.path,
            stored.duration,
        ),
        start.elapsed(),
    ))
}

/// One pair's score from its function-to-function matrix, by the rule
/// `compare` uses: two sides with no functions are identical, and one side
/// with none shares nothing with the other.
fn score(matrix: &Array2<f64>, aggregator: &Aggregator) -> Result<f64> {
    let (rows, cols) = matrix.dim();
    if rows == 0 && cols == 0 {
        return Ok(1.0);
    }
    if rows == 0 || cols == 0 {
        return Ok(0.0);
    }
    let size = std::cmp::max(rows, cols);
    let mut square_matrix = Array2::zeros((size, size));
    square_matrix
        .slice_mut(ndarray::s![0..rows, 0..cols])
        .assign(matrix);
    let similarities = aggregator.aggregate(&square_matrix)?;
    if similarities.is_empty() {
        return Ok(0.0);
    }
    Ok(similarities.iter().sum::<f64>() / similarities.len() as f64)
}

fn load_results(score_dir: &Path) -> Result<Vec<CompareResult>> {
    if score_dir.join("results.csv").exists() {
        load_results_impl(score_dir)
    } else {
        scan_directory_for_results(score_dir)
    }
}

fn scan_directory_for_results(score_dir: &Path) -> Result<Vec<CompareResult>> {
    let mut results = Vec::new();
    for entry in std::fs::read_dir(score_dir).map_err(|e| Error::Io(score_dir.to_path_buf(), e))? {
        let entry = entry.map_err(|e| Error::Io(score_dir.to_path_buf(), e))?;
        let path = entry.path();
        // ASCII digits: `is_numeric` also takes any other script's digits,
        // which `parse` then refuses, so a file named in them failed the
        // whole review instead of being passed over as not a pair CSV.
        if path.extension().and_then(OsStr::to_str) == Some("csv")
            && let Some(stem) = path.file_stem().and_then(OsStr::to_str)
            && !stem.is_empty()
            && stem.bytes().all(|b| b.is_ascii_digit())
        {
            let index = stem
                .parse::<usize>()
                .map_err(|e| Error::ParseInt(stem.to_string(), e))?;
            let cr = CompareResult::new(
                index,
                0.0,
                PathBuf::new(),
                PathBuf::new(),
                Duration::from_secs(0),
            );
            results.push(cr);
        }
    }
    Ok(results)
}

fn load_results_impl(score_dir: &Path) -> Result<Vec<CompareResult>> {
    let result_file = score_dir.join("results.csv");
    let mut reader =
        std::fs::File::open(result_file.clone()).map_err(|e| Error::Io(result_file.clone(), e))?;
    let mut results = Vec::new();
    let bufr = BufReader::new(&mut reader);
    // A line that cannot be read is an error, not the end of the list. It
    // used to end it quietly, so a summary unreadable part-way through came
    // back as one with fewer pairs, and `review` rescored only those.
    for line in bufr.lines() {
        let line = line.map_err(|e| Error::Io(result_file.clone(), e))?;
        if line.to_lowercase().starts_with("total duration,") {
            break;
        }
        results.push(CompareResult::parse(&line)?);
    }
    Ok(results)
}

/// One side of a stored comparison.
#[derive(Debug)]
pub(crate) struct Recorded {
    /// The program the side was extracted from.
    pub(crate) path: PathBuf,
    /// The birthmark file, when the CSV names one.
    pub(crate) birthmark: Option<PathBuf>,
    /// Its functions, in the matrix's order.
    pub(crate) functions: Vec<String>,
}

/// A pair CSV, read back.
#[derive(Debug)]
pub(crate) struct Stored {
    /// One row per right-hand function and one column per left-hand one,
    /// unpadded.
    pub(crate) matrix: Array2<f64>,
    pub(crate) left: Recorded,
    pub(crate) right: Recorded,
    pub(crate) duration: Duration,
}

/// The field a birthmark side's line keeps its file in, counting `left` as 0.
const BIRTHMARK_FILE_FIELD: usize = 8;

fn recorded(record: &csv::StringRecord) -> Recorded {
    let birthmark = (record.get(1) == Some("birthmark"))
        .then(|| record.get(BIRTHMARK_FILE_FIELD))
        .flatten()
        .filter(|s| !s.is_empty())
        .map(PathBuf::from);
    Recorded {
        path: record.get(3).map(PathBuf::from).unwrap_or_default(),
        birthmark,
        functions: Vec::new(),
    }
}

/// A row's function name. Files before v0.8.0 wrote a space after the row's
/// number, which kept the reader from seeing a quoted name as quoted; such a
/// name is unquoted here. One with a comma in it was split by the reader and
/// cannot be recovered.
fn row_name(field: &str) -> String {
    match field.strip_prefix(' ') {
        Some(old) if old.len() >= 2 && old.starts_with('"') && old.ends_with('"') => {
            old[1..old.len() - 1].replace("\"\"", "\"")
        }
        Some(old) => old.to_string(),
        None => field.to_string(),
    }
}

pub(crate) fn load_comparison<P: AsRef<Path>>(path: P) -> Result<Stored> {
    let path = path.as_ref();
    let mut file = std::fs::File::open(path).map_err(|e| Error::Io(path.to_path_buf(), e))?;
    let mut csv_reader = csv::ReaderBuilder::new()
        .flexible(true)
        .has_headers(false)
        .from_reader(&mut file);
    // None until the matrix line: a file without one is not a comparison,
    // while a matrix line naming no functions is one of an empty birthmark.
    let mut col_names: Option<Vec<String>> = None;
    let mut rows = Vec::new();
    let mut items = Vec::new();
    let mut left = None;
    let mut right = None;
    let mut duration = Duration::from_nanos(0);

    for result in csv_reader.records() {
        let record = result.map_err(Error::Csv)?;
        let prefix = record.get(0).unwrap_or("");
        match prefix {
            "result" => {
                if let Some(d_str) = record.get(1) {
                    let nanos = d_str.parse::<u64>().unwrap_or(0);
                    duration = Duration::from_nanos(nanos);
                }
            }
            "left" => left = Some(recorded(&record)),
            "right" => right = Some(recorded(&record)),
            "matrix" => {
                let names = record
                    .iter()
                    .skip(2)
                    .map(str::to_string)
                    .collect::<Vec<_>>();
                // `matrix,,` is how a side with no functions is written
                col_names = Some(if names == [""] { Vec::new() } else { names });
            }
            _ if !prefix.is_empty() && prefix.bytes().all(|b| b.is_ascii_digit()) => {
                rows.push(row_name(record.get(1).unwrap_or("")));
                for value in record.iter().skip(2) {
                    items.push(
                        value
                            .parse::<f64>()
                            .map_err(|e| Error::ParseFloat(value.to_string(), e))?,
                    );
                }
            }
            _ => continue,
        }
    }
    let Some(col_names) = col_names else {
        return Err(Error::Parse(format!(
            "Valid matrix data not found in {}",
            path.display()
        )));
    };
    if items.len() != rows.len() * col_names.len() {
        return Err(Error::Parse(format!(
            "{}: the matrix has {} values, not one for each of {} rows and {} columns",
            path.display(),
            items.len(),
            rows.len(),
            col_names.len()
        )));
    }
    let matrix =
        Array2::from_shape_vec((rows.len(), col_names.len()), items).map_err(Error::ShapeError)?;
    let mut left = left.unwrap_or_else(|| recorded(&csv::StringRecord::new()));
    let mut right = right.unwrap_or_else(|| recorded(&csv::StringRecord::new()));
    left.functions = col_names;
    right.functions = rows;
    Ok(Stored {
        matrix,
        left,
        right,
        duration,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn dir_with(files: &[(&str, &str)]) -> TempDir {
        let d = tempfile::tempdir().unwrap();
        for (name, body) in files {
            std::fs::write(d.path().join(name), body).unwrap();
        }
        d
    }

    /// A similarity CSV as `compare` writes one: the score, the pair, then
    /// the matrix as a header row of column names and one numbered row per
    /// element.
    const A_COMPARISON: &str = "result,28083,0.75\n\
        left,program,hello_clang,bin/hello_clang,1,1,pcodes/hello_clang.json\n\
        right,program,hello_gcc,bin/hello_gcc,1,1,pcodes/hello_gcc.json\n\
        matrix,,entry,second\n\
        0,entry,0.75,0.25\n";

    #[test]
    fn test_a_comparison_is_read_into_a_matrix_and_its_pair() {
        let d = dir_with(&[("00000.csv", A_COMPARISON)]);
        let stored = load_comparison(d.path().join("00000.csv")).unwrap();
        assert_eq!(stored.matrix.dim(), (1, 2));
        assert_eq!(stored.matrix[[0, 1]], 0.25);
        assert_eq!(stored.left.path, PathBuf::from("bin/hello_clang"));
        assert_eq!(stored.right.path, PathBuf::from("bin/hello_gcc"));
        assert_eq!(stored.left.functions, ["entry", "second"]);
        assert_eq!(stored.right.functions, ["entry"]);
        // a program's line has no birthmark file in it
        assert_eq!(stored.left.birthmark, None);
        assert_eq!(stored.duration, Duration::from_nanos(28083));
    }

    /// A record whose first field is none of the known prefixes is skipped
    /// rather than refused, so a file gaining a row does not stop an older
    /// directory being reviewed.
    #[test]
    fn test_a_record_it_does_not_know_is_skipped() {
        let with_extra = format!("something,else\n{A_COMPARISON}");
        let d = dir_with(&[("00000.csv", with_extra.as_str())]);
        let stored = load_comparison(d.path().join("00000.csv")).unwrap();
        assert_eq!(stored.matrix.dim(), (1, 2));
    }

    #[test]
    fn test_min_elements_is_a_count_or_a_ratio_with_its_x() {
        assert_eq!("5".parse(), Ok(MinElements::Count(5)));
        assert_eq!("0.3x".parse(), Ok(MinElements::Ratio(0.3)));
        assert_eq!(" 0x ".parse(), Ok(MinElements::Ratio(0.0)));
        assert_eq!(MinElements::Ratio(0.3).to_string(), "0.3x");
        assert_eq!(MinElements::Count(5).to_string(), "5");
        // a bare fraction is neither, and the refusal says how to write it
        let e = "0.3".parse::<MinElements>().unwrap_err();
        assert!(e.contains("write 0.3x"), "{e}");
        for bad in ["-1", "-1x", "NaNx", "infx", "x", "abc", ""] {
            assert!(bad.parse::<MinElements>().is_err(), "{bad:?} was accepted");
        }
    }

    /// The rule `compare` scores an empty side by, so that dropping every
    /// function of a side means the same thing as a side that had none.
    #[test]
    fn test_an_empty_side_scores_as_compare_scores_it() {
        let h = Aggregator::Hungarian;
        assert_eq!(score(&Array2::zeros((0, 0)), &h).unwrap(), 1.0);
        assert_eq!(score(&Array2::zeros((0, 3)), &h).unwrap(), 0.0);
        assert_eq!(score(&Array2::zeros((3, 0)), &h).unwrap(), 0.0);
        let one = Array2::from_elem((1, 1), 0.5);
        assert_eq!(score(&one, &h).unwrap(), 0.5);
    }

    /// `compare` writes a side with no functions as `matrix,,` and rows with
    /// no values, or no rows at all. Both used to fail to load, so a
    /// directory holding one empty birthmark could not be reviewed.
    #[test]
    fn test_a_comparison_with_an_empty_side_loads() {
        let head = "result,0,0\nleft,program,a,a,1,1,\nright,program,b,b,1,1,\n";
        for (body, dim) in [
            ("matrix,,\n0,entry\n1,main\n", (2, 0)),
            ("matrix,,entry,main\n", (0, 2)),
            ("matrix,,\n", (0, 0)),
        ] {
            let d = dir_with(&[("00000.csv", &format!("{head}{body}"))]);
            let stored = load_comparison(d.path().join("00000.csv"))
                .unwrap_or_else(|e| panic!("{body:?}: {e}"));
            assert_eq!(stored.matrix.dim(), dim, "{body:?}");
        }
    }

    /// Before v0.8.0 a row's name followed a space, which hid its quotes
    /// from the reader.
    #[test]
    fn test_a_row_name_is_read_in_either_format() {
        assert_eq!(row_name("entry"), "entry");
        assert_eq!(row_name(" entry"), "entry");
        assert_eq!(row_name(" \"operator\"\"\"\"__km\""), "operator\"\"__km");
        assert_eq!(row_name("operator\"\"__km"), "operator\"\"__km");
    }

    #[test]
    fn test_a_matrix_short_of_values_is_refused_saying_so() {
        let d = dir_with(&[("00000.csv", "matrix,,a,b\n0,x,0.5\n")]);
        let e = load_comparison(d.path().join("00000.csv")).unwrap_err();
        assert!(e.to_string().contains("1 values"), "{e}");
    }

    /// A birthmark extracted again since the comparison would lend its
    /// counts to another's functions, so it is refused rather than used.
    #[test]
    fn test_a_birthmark_that_no_longer_matches_its_matrix_is_refused() {
        let b = PathBuf::from("b.json");
        let mut birthmarks = FxHashMap::default();
        birthmarks.insert(
            b.clone(),
            vec![("entry".to_string(), 3), ("main".to_string(), 1)],
        );
        let f = Filter {
            threshold: 2.0,
            birthmarks,
        };
        let names = |n: &[&str]| n.iter().map(ToString::to_string).collect::<Vec<_>>();
        let csv = Path::new("00000.csv");
        assert_eq!(f.keep(&b, &names(&["entry", "main"]), csv).unwrap(), [0]);
        for other in [&["main", "entry"][..], &["entry"], &["entry", "main", "x"]] {
            assert!(f.keep(&b, &names(other), csv).is_err(), "{other:?}");
        }
    }

    #[test]
    fn test_a_comparison_with_no_matrix_in_it_is_refused() {
        let d = dir_with(&[("00000.csv", "result,28083,0.75\nleft,,,x\nright,,,y\n")]);
        let e = load_comparison(d.path().join("00000.csv")).unwrap_err();
        assert!(e.to_string().contains("Valid matrix data not found"), "{e}");
    }

    #[test]
    fn test_a_cell_that_is_not_a_number_is_refused() {
        let broken = "matrix,,entry\n0,entry,notafloat\n";
        let d = dir_with(&[("00000.csv", broken)]);
        let e = load_comparison(d.path().join("00000.csv")).unwrap_err();
        assert!(e.to_string().contains("Parse float error"), "{e}");
    }

    #[test]
    fn test_a_comparison_that_is_not_there_is_an_io_error() {
        let d = tempfile::tempdir().unwrap();
        let e = load_comparison(d.path().join("00000.csv")).unwrap_err();
        assert!(e.to_string().starts_with("IO error for"), "{e}");
    }

    /// `review` passes a load failure through rather than scoring the
    /// pair as zero, which would be indistinguishable from two programs with
    /// nothing in common.
    #[test]
    fn test_review_passes_a_load_failure_on() {
        let d = tempfile::tempdir().unwrap();
        let cr = CompareResult::new(
            0,
            0.0,
            PathBuf::new(),
            PathBuf::new(),
            Duration::from_secs(0),
        );
        let Err(e) = review_pair(&cr, d.path(), &Aggregator::Hungarian, None) else {
            panic!("a missing comparison file should not load");
        };
        assert!(e.to_string().starts_with("IO error for"), "{e}");
    }

    #[test]
    fn test_review_rescores_from_the_stored_matrix() {
        let d = dir_with(&[("00000.csv", A_COMPARISON)]);
        let cr = CompareResult::new(
            0,
            0.0,
            PathBuf::new(),
            PathBuf::new(),
            Duration::from_secs(0),
        );
        let (rescored, _) = review_pair(&cr, d.path(), &Aggregator::Hungarian, None).unwrap();
        assert_eq!(rescored.index, 0);
        assert_eq!(rescored.path1, PathBuf::from("bin/hello_clang"));
        assert!(rescored.similarity > 0.0);
    }

    /// Without a `results.csv` the directory is scanned for the numbered
    /// files instead, so a run that was interrupted before writing the
    /// summary can still be reviewed.
    #[test]
    fn test_a_directory_without_a_summary_is_scanned_for_numbered_files() {
        let d = dir_with(&[
            ("00000.csv", A_COMPARISON),
            ("00002.csv", A_COMPARISON),
            // neither of these is a comparison: one is not numbered, the
            // other is not a CSV
            ("summary.csv", A_COMPARISON),
            ("00001.txt", A_COMPARISON),
        ]);
        let mut found = load_results(d.path())
            .unwrap()
            .iter()
            .map(|cr| cr.index)
            .collect::<Vec<_>>();
        found.sort_unstable();
        assert_eq!(found, vec![0, 2]);
    }

    #[test]
    fn test_a_summary_is_read_rather_than_the_directory_scanned() {
        let d = dir_with(&[
            (
                "results.csv",
                "0,0.75,bin/a,bin/b,28083\ntotal duration,830292,00:00:000\n",
            ),
            ("00000.csv", A_COMPARISON),
            ("00001.csv", A_COMPARISON),
        ]);
        let results = load_results(d.path()).unwrap();
        // the scan would have found two; the summary names one, and stops at
        // the total-duration line rather than trying to parse it as a result
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].similarity, 0.75);
    }

    /// A summary unreadable part-way through is an error. It used to end the
    /// list there, so `review` rescored the pairs before the bad line and
    /// reported nothing about the rest.
    #[test]
    fn test_a_summary_that_cannot_be_read_to_the_end_is_refused() {
        let d = dir_with(&[("00000.csv", A_COMPARISON), ("00001.csv", A_COMPARISON)]);
        let mut summary = b"0,0.75,bin/a,bin/b,28083\n".to_vec();
        summary.extend_from_slice(b"\xff\xfe not text\n");
        summary.extend_from_slice(b"1,0.5,bin/a,bin/c,28083\ntotal duration,1,00:00:000\n");
        std::fs::write(d.path().join("results.csv"), summary).unwrap();
        let Err(e) = load_results(d.path()) else {
            panic!("a summary with an unreadable line was read as a shorter one");
        };
        assert!(e.to_string().starts_with("IO error for"), "{e}");
    }

    /// Only ASCII digits name a pair. A stem in another script's digits used
    /// to pass the check and fail the parse, which failed the whole review
    /// over a file that is simply not a pair CSV.
    #[test]
    fn test_a_file_named_in_other_digits_is_not_a_pair() {
        let d = dir_with(&[("00000.csv", A_COMPARISON), ("\u{0663}.csv", A_COMPARISON)]);
        let found = load_results(d.path())
            .unwrap()
            .iter()
            .map(|cr| cr.index)
            .collect::<Vec<_>>();
        assert_eq!(found, vec![0]);
    }

    /// The same inside a pair CSV: a record starting with other digits is not
    /// a matrix row, and is skipped like any record it does not know.
    #[test]
    fn test_a_record_starting_with_other_digits_is_not_a_row() {
        let with_extra = format!("{A_COMPARISON}\u{0663},x,notanumber\n");
        let d = dir_with(&[("00000.csv", with_extra.as_str())]);
        let stored = load_comparison(d.path().join("00000.csv")).unwrap();
        assert_eq!(stored.matrix.dim(), (1, 2));
    }

    /// The stem is all digits by the time it is parsed, so the only way this
    /// fails is a number too large for a `usize` -- which is why it is an
    /// error rather than an `unwrap`.
    #[test]
    fn test_a_number_too_large_to_be_an_index_is_refused() {
        let d = dir_with(&[("999999999999999999999999999999.csv", A_COMPARISON)]);
        let Err(e) = load_results(d.path()) else {
            panic!("an index that does not fit a usize should be refused");
        };
        assert!(e.to_string().contains("Parse int error"), "{e}");
    }

    #[test]
    fn test_a_score_directory_that_is_not_there_is_an_io_error() {
        let d = tempfile::tempdir().unwrap();
        let Err(e) = load_results(&d.path().join("nope")) else {
            panic!("a directory that is not there should not scan");
        };
        assert!(e.to_string().starts_with("IO error for"), "{e}");
    }
}
