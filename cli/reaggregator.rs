use std::path::{Path, PathBuf};
use std::time::Duration;
use std::io::{BufRead, BufReader};

use crate::{CompareResult, cli};
use ndarray::Array2;
use oinkie::prelude::*;

pub(crate) fn perform(opts: cli::ReaggregateOpts) -> Result<Vec<Duration>> {
    let start = std::time::Instant::now();
    let score_dir = opts.score_directory();
    let crs = load_results(score_dir)?;
    log::info!("read the previous results {:?}", start.elapsed());
    let results = crs.iter()
        .map(|cr| reaggregate(cr, score_dir, opts.aggregator()))
        .collect::<Vec<_>>();
    let results = Error::vec_result_to_result_vec(results)?;
    let cresults = results.into_iter()
        .map(|(cr, _)| cr).collect::<Vec<_>>();
    super::store_and_get_durations(cresults, opts.dest_file(), start)
}

fn reaggregate(cr: &CompareResult, score_dir: &Path, aggregator: &Aggregator) -> Result<(CompareResult, Duration)> {
    let start = std::time::Instant::now();
    match load_comparison_file(cr.index, score_dir) {
        Ok((matrix, p1, p2, duration)) => {
            let (rows, cols) = matrix.dim();
            let size = std::cmp::max(rows, cols);
            let mut square_matrix = Array2::zeros((size, size));
            square_matrix.slice_mut(ndarray::s![0..rows, 0..cols]).assign(&matrix);
            let similarities = aggregator.aggregate(&square_matrix)?;
            let similarity = similarities.iter().sum::<f64>() / similarities.len() as f64;
            Ok((CompareResult::new(cr.index, similarity, p1, p2, duration), start.elapsed()))
        },
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
    for entry in std::fs::read_dir(score_dir)
                .map_err(|e| Error::Io(score_dir.to_path_buf(), e))? {
        let entry = entry.map_err(|e| Error::Io(score_dir.to_path_buf(), e))?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("csv") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                if stem.chars().all(|c| c.is_numeric()) {
                    let index = stem.parse::<usize>().map_err(|e| Error::ParseInt(stem.to_string(), e))?;
                    let cr = CompareResult::new(index, 0.0, PathBuf::new(), PathBuf::new(), Duration::from_secs(0));
                    results.push(cr);
                }
            }
        }
    }
    Ok(results)
}

fn load_results_impl(score_dir: &Path) -> Result<Vec<CompareResult>> {
    let result_file = score_dir.join("results.csv");
    let mut reader = std::fs::File::open(result_file.clone())
        .map_err(|e| Error::Io(result_file.clone(), e))?;
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

fn load_comparison_file(index: usize, score_dir: &Path) -> Result<(Array2<f64>, PathBuf, PathBuf, Duration)> {
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

    for results in csv_reader.records() {
        let record = results.map_err(Error::Csv)?;
        let prefix = record.get(0).unwrap_or("");
        match prefix {
            "result" => if let Some(d_str) = record.get(1) {
                let nanos = d_str.parse::<u64>().unwrap_or(0);
                duration = Duration::from_nanos(nanos);
            },
            "left" => if let Some(s) = record.get(3) {
                path1 = PathBuf::from(s);
            },
            "right" => if let Some(s) = record.get(3) {
                path2 = PathBuf::from(s);
            },
            "matrix" => col_names = record.iter().skip(2).map(|s| s.to_string()).collect(),
            _ if prefix.chars().all(|c| c.is_numeric()) && !prefix.is_empty() => {
                rows.push(record.get(1).unwrap_or("").to_string());
                for value in record.iter().skip(2) {
                    items.push(value.parse::<f64>().map_err(|e| Error::ParseFloat(value.to_string(), e))?);
                }
            },
            _ => continue,
        }
    }
    if col_names.is_empty() || rows.is_empty() {
        Err(Error::Parse(format!("Valid matrix data not found in {}", path.as_ref().display())))
    } else {
        let matrix = Array2::from_shape_vec((rows.len(), col_names.len()), items)
            .map_err(Error::ShapeError)?;
        Ok((matrix, path1, path2, duration))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_comparison() {
        let _ = load_comparison_file(1, &Path::new("testdata/comparison_data"))
            .expect("failed to load comparison file");
        
    }
}