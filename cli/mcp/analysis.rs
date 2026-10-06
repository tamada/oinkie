//! What the tools do, apart from being tools.
//!
//! Written against the library rather than through the `perform_*` drivers,
//! because those print to stdout and stdout is the JSON-RPC channel. What that
//! duplicates is the plumbing -- read the files, walk the pairs, write the
//! results -- and not the arithmetic: the comparison itself is one library
//! call in both, so the two cannot disagree about a score without the library
//! disagreeing with itself. The parity tests check that anyway.

use std::path::{Path, PathBuf};
use std::time::Instant;

use oinkie::birthmarks::{AnalysisType, Birthmark, BirthmarkType};
use oinkie::compare::{Aggregator, Comparator, PairingStrategy};
use oinkie::extract::Extractor;
use oinkie::{Error, Program, Result};
use rayon::prelude::*;
use rmcp::schemars;
use serde::Serialize;

use crate::CompareResult;

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Score {
    /// The pair's number, which is also the name of its CSV in the
    /// destination directory when one was given.
    pub index: usize,
    pub left: String,
    pub right: String,
    /// Between 0 and 1. Two birthmarks that are both empty score 1.0, so a
    /// perfect match between programs that should have nothing in common is a
    /// reason to look at the inputs.
    pub similarity: f64,
    pub duration_ms: u64,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Extracted {
    pub input: String,
    pub output: String,
    /// How many functions the birthmark covers.
    pub elements: usize,
    pub duration_ms: u64,
}

fn ms(d: std::time::Duration) -> u64 {
    d.as_millis() as u64
}

fn scores(results: &[CompareResult]) -> Vec<Score> {
    results
        .iter()
        .map(|r| Score {
            index: r.index,
            left: r.path1.display().to_string(),
            right: r.path2.display().to_string(),
            similarity: r.similarity,
            duration_ms: ms(r.duration),
        })
        .collect()
}

/// Writes the pair CSVs' directory index, so that a directory this produced
/// can be handed straight to `oinkie_review` -- or to `oinkie
/// review` -- afterwards.
fn store(results: Vec<CompareResult>, dest: &Path, start: Instant) -> Result<()> {
    crate::store_and_get_durations(results, &results_file(dest), start).map(|_| ())
}

/// Where `run` and `compare` write the function similarities of pair `index`.
pub fn pair_file(dest: &Path, index: usize) -> PathBuf {
    dest.join(format!("{index:05}.csv"))
}

/// Where `run` and `compare` write the index of the pairs' scores.
pub fn results_file(dest: &Path) -> PathBuf {
    dest.join("results.csv")
}

/// Every file `run` or `compare` writes into `dest` for `pairs` pairs, so the
/// server can confine them as it confines `dest` itself.
pub fn score_files(dest: &Path, pairs: usize) -> impl Iterator<Item = PathBuf> + '_ {
    (0..pairs)
        .map(move |i| pair_file(dest, i))
        .chain(std::iter::once(results_file(dest)))
}

/// The files `extract` writes the birthmarks of `inputs` to in `dir`, one per
/// distinct file.
pub fn extracted_files(inputs: &[PathBuf], dir: &Path) -> Result<Vec<PathBuf>> {
    let inputs = inputs.iter().collect::<Vec<_>>();
    Ok(crate::unique_destinations(&inputs, dir)?
        .into_iter()
        .map(|(_, out)| out)
        .collect())
}

fn prepare(dest: Option<&Path>) -> Result<()> {
    match dest {
        Some(d) => std::fs::create_dir_all(d).map_err(|e| Error::Io(d.to_path_buf(), e)),
        None => Ok(()),
    }
}

/// Extracts a birthmark from each lifted program, once per file written.
///
/// Two inputs that name one file -- the same path given twice, or two copies
/// of one program under one stem -- are extracted once and reported once,
/// under the first of them. Extracting both would have two workers write one
/// file at once, which can interleave them into invalid JSON.
pub fn extract(
    inputs: &[PathBuf],
    birthmark_type: &BirthmarkType,
    dest: &Path,
    skip: bool,
) -> Result<Vec<Extracted>> {
    std::fs::create_dir_all(dest).map_err(|e| Error::Io(dest.to_path_buf(), e))?;
    let extractor = Extractor::new(birthmark_type.clone());
    let inputs = inputs.iter().collect::<Vec<_>>();
    let done = crate::unique_destinations(&inputs, dest)?
        .par_iter()
        .map(|(input, out)| {
            let start = Instant::now();
            if out.exists() && skip {
                // Read back rather than reported blindly: the count is part of
                // the answer, and a file left by an earlier run is the only
                // place it can come from.
                let birthmark: Birthmark = out.clone().try_into()?;
                return Ok(Extracted {
                    input: input.display().to_string(),
                    output: out.display().to_string(),
                    elements: birthmark.len(),
                    duration_ms: ms(start.elapsed()),
                });
            }
            let program = Program::load(input)?;
            let birthmark = extractor.extract(&program)?;
            let json = serde_json::to_string_pretty(&birthmark)
                .map_err(|e| Error::Json(out.clone(), e))?;
            std::fs::write(out, json).map_err(|e| Error::Io(out.clone(), e))?;
            Ok(Extracted {
                input: input.display().to_string(),
                output: out.display().to_string(),
                elements: birthmark.len(),
                duration_ms: ms(start.elapsed()),
            })
        })
        .collect::<Vec<_>>();
    Error::vec_result_to_result_vec(done)
}

/// Where `run` writes the birthmarks of a destination.
pub fn birthmarks_dir(dest: &Path) -> PathBuf {
    dest.join("birthmarks")
}

/// The files `run` writes the birthmarks of `inputs` to under `dest`, so the
/// server can confine them as it confines `dest` itself.
pub fn birthmark_files(inputs: &[PathBuf], dest: &Path) -> Result<Vec<PathBuf>> {
    extracted_files(inputs, &birthmarks_dir(dest))
}

/// Extracts the analysis's birthmark from each lifted program once and
/// compares the birthmarks.
///
/// Both halves of the analysis are used. Comparing the programs directly
/// scored their operations whatever birthmark the analysis named (#150).
///
/// Given `dest`, the birthmarks are written into its `birthmarks/` and the
/// scores' CSVs name them, as `run` does, so the directory can be reviewed on
/// its own (#128). Without it they are held in memory only.
pub fn run(
    inputs: &[PathBuf],
    analysis: &AnalysisType,
    strategy: &PairingStrategy,
    aggregator: &Aggregator,
    dest: Option<&Path>,
) -> Result<Vec<Score>> {
    let start = Instant::now();
    prepare(dest)?;
    let extractor = Extractor::new(analysis.birthmark().clone());
    let by_input = match dest {
        Some(d) => {
            let dir = birthmarks_dir(d);
            std::fs::create_dir_all(&dir).map_err(|e| Error::Io(dir.clone(), e))?;
            crate::extract_all(inputs, &dir, &extractor, false, || ())?
                .into_iter()
                .map(|(input, e)| (input, e.birthmark))
                .collect::<rustc_hash::FxHashMap<_, _>>()
        }
        None => {
            let mut unique = inputs.iter().collect::<Vec<_>>();
            unique.sort();
            unique.dedup();
            let done = unique
                .par_iter()
                .map(|input| Ok(((*input).clone(), extractor.extract(&Program::load(input)?)?)))
                .collect::<Vec<_>>();
            Error::vec_result_to_result_vec(done)?
                .into_iter()
                .collect::<rustc_hash::FxHashMap<_, _>>()
        }
    };
    let results = strategy
        .pairs(inputs)
        .enumerate()
        .par_bridge()
        .map(|(i, (left, right))| {
            let (b1, b2) = (&by_input[left], &by_input[right]);
            let comparison = analysis
                .comparator()
                .compare_birthmarks(b1, b2, aggregator)?;
            if let Some(d) = dest {
                crate::score_csv::store(&comparison, pair_file(d, i))?;
            }
            Ok(CompareResult::new(
                i,
                comparison.similarity(),
                b1.path().to_path_buf(),
                b2.path().to_path_buf(),
                comparison.duration(),
            ))
        })
        .collect::<Vec<_>>();
    finish(results, dest, start)
}

/// Compares birthmarks that `extract` already wrote.
pub fn compare(
    inputs: &[PathBuf],
    comparator: &Comparator,
    strategy: &PairingStrategy,
    aggregator: &Aggregator,
    dest: Option<&Path>,
) -> Result<Vec<Score>> {
    let start = Instant::now();
    prepare(dest)?;
    let results = strategy
        .pairs(inputs)
        .enumerate()
        .par_bridge()
        .map(|(i, (left, right))| {
            let mut b1: Birthmark = left.clone().try_into()?;
            b1.set_json_path(left.clone());
            let mut b2: Birthmark = right.clone().try_into()?;
            b2.set_json_path(right.clone());
            let comparison = comparator.compare_birthmarks(&b1, &b2, aggregator)?;
            if let Some(d) = dest {
                crate::score_csv::store(&comparison, pair_file(d, i))?;
            }
            Ok(CompareResult::new(
                i,
                comparison.similarity(),
                b1.path().to_path_buf(),
                b2.path().to_path_buf(),
                comparison.duration(),
            ))
        })
        .collect::<Vec<_>>();
    finish(results, dest, start)
}

fn finish(
    results: Vec<Result<CompareResult>>,
    dest: Option<&Path>,
    start: Instant,
) -> Result<Vec<Score>> {
    let mut results = Error::vec_result_to_result_vec(results)?;
    // The pairs are walked in parallel, so they arrive in whatever order they
    // finished. Sorted by the index they were given, so that two runs over the
    // same inputs report them the same way round.
    results.sort_by_key(|r| r.index);
    let out = scores(&results);
    if let Some(d) = dest {
        store(results, d, start)?;
    }
    Ok(out)
}
