use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::{CompareResult, cli};
use ndarray::Array2;
use oinkie::prelude::*;

pub(crate) fn perform(opts: cli::ReaggregateOpts) -> Result<Vec<Duration>> {
    let start = std::time::Instant::now();
    let results = reaggregate_all(opts.score_directory(), opts.aggregator())?;
    super::store_and_get_durations(results, opts.dest_file(), start)
}

/// Recomputes every score in a directory, and hands them back.
///
/// Split out of [`perform`], which now only decides where to write them,
/// because the MCP tool wants the scores themselves rather than a CSV. Both
/// callers get the same numbers by construction rather than by two
/// implementations agreeing.
pub(crate) fn reaggregate_all(
    score_dir: &Path,
    aggregator: &Aggregator,
) -> Result<Vec<CompareResult>> {
    let start = std::time::Instant::now();
    let crs = load_results(score_dir)?;
    log::info!("read the previous results {:?}", start.elapsed());
    let results = crs
        .iter()
        .map(|cr| reaggregate(cr, score_dir, aggregator))
        .collect::<Vec<_>>();
    let results = Error::vec_result_to_result_vec(results)?;
    Ok(results.into_iter().map(|(cr, _)| cr).collect())
}

fn reaggregate(
    cr: &CompareResult,
    score_dir: &Path,
    aggregator: &Aggregator,
) -> Result<(CompareResult, Duration)> {
    let start = std::time::Instant::now();
    match load_comparison_file(cr.index, score_dir) {
        Ok((matrix, p1, p2, duration)) => {
            let (rows, cols) = matrix.dim();
            let size = std::cmp::max(rows, cols);
            let mut square_matrix = Array2::zeros((size, size));
            square_matrix
                .slice_mut(ndarray::s![0..rows, 0..cols])
                .assign(&matrix);
            let similarities = aggregator.aggregate(&square_matrix)?;
            let similarity = similarities.iter().sum::<f64>() / similarities.len() as f64;
            Ok((
                CompareResult::new(cr.index, similarity, p1, p2, duration),
                start.elapsed(),
            ))
        }
        Err(e) => Err(e),
    }
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
        if path.extension().and_then(|s| s.to_str()) == Some("csv")
            && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
            && stem.chars().all(|c| c.is_numeric())
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
    for line in bufr.lines().map_while(|r| r.ok()) {
        let lower = line.to_lowercase();
        if lower.strip_prefix("total duration,").is_some() {
            break;
        } else {
            let cr = CompareResult::parse(&line)?;
            results.push(cr);
        }
    }
    Ok(results)
}

fn load_comparison_file(
    index: usize,
    score_dir: &Path,
) -> Result<(Array2<f64>, PathBuf, PathBuf, Duration)> {
    let file_name = format!("{index:05}.csv");
    let path = score_dir.join(file_name);
    let r = load_comparison(&path);
    log::info!("load comparison file {index} {:?}", path.display());
    r
}

fn load_comparison<P: AsRef<Path>>(path: P) -> Result<(Array2<f64>, PathBuf, PathBuf, Duration)> {
    let mut file = std::fs::File::open(path.as_ref())
        .map_err(|e| Error::Io(path.as_ref().to_path_buf(), e))?;
    let mut csv_reader = csv::ReaderBuilder::new()
        .flexible(true)
        .has_headers(false)
        .from_reader(&mut file);
    let mut col_names = Vec::new();
    let mut rows = Vec::new();
    let mut items = Vec::new();
    let mut path1 = PathBuf::new();
    let mut path2 = PathBuf::new();
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
            "left" => {
                if let Some(s) = record.get(3) {
                    path1 = PathBuf::from(s);
                }
            }
            "right" => {
                if let Some(s) = record.get(3) {
                    path2 = PathBuf::from(s);
                }
            }
            "matrix" => col_names = record.iter().skip(2).map(|s| s.to_string()).collect(),
            _ if prefix.chars().all(|c| c.is_numeric()) && !prefix.is_empty() => {
                rows.push(record.get(1).unwrap_or("").to_string());
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
    if col_names.is_empty() || rows.is_empty() {
        Err(Error::Parse(format!(
            "Valid matrix data not found in {}",
            path.as_ref().display()
        )))
    } else {
        let matrix = Array2::from_shape_vec((rows.len(), col_names.len()), items)
            .map_err(Error::ShapeError)?;
        Ok((matrix, path1, path2, duration))
    }
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
        let (matrix, p1, p2, duration) = load_comparison(d.path().join("00000.csv")).unwrap();
        assert_eq!(matrix.dim(), (1, 2));
        assert_eq!(matrix[[0, 1]], 0.25);
        assert_eq!(p1, PathBuf::from("bin/hello_clang"));
        assert_eq!(p2, PathBuf::from("bin/hello_gcc"));
        assert_eq!(duration, Duration::from_nanos(28083));
    }

    /// A record whose first field is none of the known prefixes is skipped
    /// rather than refused, so a file gaining a row does not stop an older
    /// directory being reaggregated.
    #[test]
    fn test_a_record_it_does_not_know_is_skipped() {
        let with_extra = format!("something,else\n{A_COMPARISON}");
        let d = dir_with(&[("00000.csv", with_extra.as_str())]);
        let (matrix, _, _, _) = load_comparison(d.path().join("00000.csv")).unwrap();
        assert_eq!(matrix.dim(), (1, 2));
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

    /// `reaggregate` passes a load failure through rather than scoring the
    /// pair as zero, which would be indistinguishable from two programs with
    /// nothing in common.
    #[test]
    fn test_reaggregate_passes_a_load_failure_on() {
        let d = tempfile::tempdir().unwrap();
        let cr = CompareResult::new(
            0,
            0.0,
            PathBuf::new(),
            PathBuf::new(),
            Duration::from_secs(0),
        );
        let Err(e) = reaggregate(&cr, d.path(), &Aggregator::Hungarian) else {
            panic!("a missing comparison file should not load");
        };
        assert!(e.to_string().starts_with("IO error for"), "{e}");
    }

    #[test]
    fn test_reaggregate_rescores_from_the_stored_matrix() {
        let d = dir_with(&[("00000.csv", A_COMPARISON)]);
        let cr = CompareResult::new(
            0,
            0.0,
            PathBuf::new(),
            PathBuf::new(),
            Duration::from_secs(0),
        );
        let (rescored, _) = reaggregate(&cr, d.path(), &Aggregator::Hungarian).unwrap();
        assert_eq!(rescored.index, 0);
        assert_eq!(rescored.path1, PathBuf::from("bin/hello_clang"));
        assert!(rescored.similarity > 0.0);
    }

    /// Without a `results.csv` the directory is scanned for the numbered
    /// files instead, so a run that was interrupted before writing the
    /// summary can still be reaggregated.
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
