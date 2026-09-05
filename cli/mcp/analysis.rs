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

use oinkie::prelude::*;
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
/// can be handed straight to `oinkie_reaggregate` -- or to `oinkie
/// reaggregate` -- afterwards.
fn store(results: Vec<CompareResult>, dest: &Path, start: Instant) -> Result<()> {
    crate::store_and_get_durations(results, &dest.join("results.csv"), start).map(|_| ())
}

fn prepare(dest: Option<&Path>) -> Result<()> {
    match dest {
        Some(d) => std::fs::create_dir_all(d).map_err(|e| Error::Io(d.to_path_buf(), e)),
        None => Ok(()),
    }
}

/// Extracts a birthmark from each lifted program.
pub fn extract(
    inputs: &[PathBuf],
    birthmark_type: &BirthmarkType,
    dest: &Path,
    skip: bool,
) -> Result<Vec<Extracted>> {
    std::fs::create_dir_all(dest).map_err(|e| Error::Io(dest.to_path_buf(), e))?;
    let extractor = Extractor::new(birthmark_type.clone());
    let done = inputs
        .par_iter()
        .map(|input| {
            let start = Instant::now();
            let out = dest.join(oinkie::extractor::dest_file_name(input)?);
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
            let program = AnyProgram::load(input)?;
            let birthmark = extractor.extract_any(&program)?;
            let json = serde_json::to_string_pretty(&birthmark)
                .map_err(|e| Error::Json(out.clone(), e))?;
            std::fs::write(&out, json).map_err(|e| Error::Io(out.clone(), e))?;
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

/// Compares lifted programs directly, without writing birthmarks first.
pub fn run(
    inputs: &[PathBuf],
    analysis: &AnalysisType,
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
            let mut p1 = AnyProgram::load(left)?;
            p1.set_json_path(left.clone());
            let mut p2 = AnyProgram::load(right)?;
            p2.set_json_path(right.clone());
            let comparison = analysis.comparator().compare_any(&p1, &p2, aggregator)?;
            if let Some(d) = dest {
                comparison.store(d.join(format!("{i:05}.csv")))?;
            }
            Ok(CompareResult::new(
                i,
                comparison.similarity(),
                p1.path().to_path_buf(),
                p2.path().to_path_buf(),
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
                comparison.store(d.join(format!("{i:05}.csv")))?;
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
