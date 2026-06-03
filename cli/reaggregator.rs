use std::path::Path;
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
    match load_comparison_file(cr.index, score_dir) {
        Ok(matrix) => {
            let (rows, cols) = matrix.dim();
            let size = std::cmp::max(rows, cols);
            let mut square_matrix = Array2::zeros((size, size));
            square_matrix.slice_mut(ndarray::s![0..rows, 0..cols]).assign(&matrix);
            let similarities = aggregator.aggregate(&square_matrix)?;
            let similarity = similarities.iter().sum::<f64>() / similarities.len() as f64;
            Ok((CompareResult::new(cr.index, similarity, cr.path1.clone(), cr.path2.clone(), cr.duration), cr.duration))
        },
        Err(e) => Err(e),
    }
}

fn load_results(score_dir: &Path) -> Result<Vec<CompareResult>> {
    let result_file = score_dir.join("results.csv");
    let mut reader = std::fs::File::open(result_file.clone())
        .map_err(|e| Error::Io(result_file.clone(), e))?;
    let mut results = Vec::new();
    let bufr = BufReader::new(&mut reader);
    for line in bufr.lines().map_while(|r| r.ok()) {
        if line.strip_prefix("total duration,").is_some() {
            break;
        } else {
            let cr = CompareResult::parse(&line)?;
            results.push(cr);
        }
    }
    Ok(results)
}

fn load_comparison_file(index: usize, score_dir: &Path) -> Result<Array2<f64>> {
    let file_name = format!("{index:05}.csv");
    let path = score_dir.join(file_name);
    let r = load_comparison(&path);
    log::info!("load comparison file {index} {:?}", path.display());
    r
}

fn load_comparison<P: AsRef<Path>>(path: P) -> Result<Array2<f64>> {
    let mut file = std::fs::File::open(path.as_ref())
        .map_err(|e| Error::Io(path.as_ref().to_path_buf(), e))?;
    let csv_reader = csv::ReaderBuilder::new()
        .flexible(true)
        .has_headers(false)
        .from_reader(&mut file);
    let mut iter = csv_reader.into_records();
    let _result_line = iter.next().unwrap().map_err(Error::Csv)?;
    let _left_line = iter.next().unwrap().map_err(Error::Csv)?;
    let _right_line = iter.next().unwrap().map_err(Error::Csv)?;
    let col_names_line = iter.next().unwrap().map_err(Error::Csv)?;
    let cols = col_names_line.iter().skip(2).map(|s| s.to_string()).collect::<Vec<_>>();
    let mut items = vec![];
    let mut rows = vec![];
    for line in iter {
        let line = line.map_err(Error::Csv)?;
        let row_item = line.get(1).unwrap();
        rows.push(row_item.to_string());
        log::info!("load comparison file row {:?} ({}) {:?}", row_item, line.len(), line.iter().skip(2).collect::<Vec<_>>());
        for value in line.iter().skip(2) {
            items.push(value.parse::<f64>().map_err(|e| Error::ParseFloat(value.to_string(), e))?);
        }
    }
    log::info!("reshape vector (length: {}) with {} cols and {} rows", items.len(), cols.len(), rows.len());
    let matrix = Array2::from_shape_vec((rows.len(), cols.len()), items)
        .map_err(Error::ShapeError)?;

    Ok(matrix)
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