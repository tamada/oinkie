//! Comparing birthmarks.
//!
//! An [`Oinkie`](crate::Oinkie) makes the [`Comparator`] for an
//! [`Algorithm`], which compares every function of
//! one [`Birthmark`] with every function of another into a matrix of function
//! similarities. An [`Aggregator`] turns the matrix into the pair's one
//! similarity, and the [`Comparison`] holds both. A [`PairingStrategy`]
//! chooses which pairs of a list to compare. An [`Observer`] passed to
//! [`Comparator::compare_birthmarks_with`] is told how far a comparison has
//! got, for a caller that shows it.
//!
//! ```
//! use std::path::Path;
//!
//! use oinkie::{Oinkie, Program};
//! use oinkie::birthmarks::BirthmarkType;
//! use oinkie::compare::{Aggregator, Algorithm};
//! use oinkie::extract::Extractor;
//!
//! # fn main() -> oinkie::Result<()> {
//! let extractor = Extractor::new(BirthmarkType::OpSet);
//! let a = extractor.extract(&Program::load(Path::new("testdata/lifted/pcodes/hello_clang.json"))?)?;
//!
//! let comparison = Oinkie::new()?
//!     .comparator(&Algorithm::Jaccard)
//!     .compare_birthmarks(&a, &a, &Aggregator::Hungarian)?;
//! assert_eq!(comparison.similarity(), 1.0, "a birthmark is identical to itself");
//! # Ok(())
//! # }
//! ```

use crate::birthmarks::{Birthmark, Data, Shape};
use crate::{Error, Result};
use itertools::Itertools;
use rayon::prelude::*;
use rustc_hash::{FxHashMap, FxHashSet};
use std::borrow::Cow;
use std::sync::Arc;
use std::time::Instant;

/// Which pairs of a list of birthmarks to compare.
#[cfg_attr(doc, katexit::katexit)]
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PairingStrategy {
    /// All possible combinations including self-comparisons ($_nC_2 + n$).
    /// Used for full matrix visualization or comprehensive heatmaps.
    AllAndSelf,

    /// Compares all possible combinations ($_nC_2$).
    /// Used for comprehensive validation of accuracy (False Positive / True Positive).
    All,

    /// Compares each file with itself ($n$).
    /// Used for sanity checks to ensure identical files yield a similarity score of 1.0.
    SelfCoverage,

    /// Compares only adjacent pairs in the list ($n-1$).
    /// Useful for comparing sequential versions (e.g., v1.0 vs v1.1, v1.1 vs v1.2).
    Adjacent,

    /// Compares a specific reference file against all other files ($n-1$).
    /// Compares first item and all other items. Useful for comparing a baseline version against multiple variants.
    FirstVsOthers,

    /// Compares a specific reference file against all other files ($n-1$).
    /// Compares the last item and all other items. Useful for comparing a baseline version against multiple variants.
    LastVsOthers,
}

impl PairingStrategy {
    /// Every strategy, in the order they are declared.
    ///
    /// Written out because nothing derives it; a test holds it to the enum
    /// with an exhaustive `match`.
    pub const ALL: &[PairingStrategy] = &[
        PairingStrategy::AllAndSelf,
        PairingStrategy::All,
        PairingStrategy::SelfCoverage,
        PairingStrategy::Adjacent,
        PairingStrategy::FirstVsOthers,
        PairingStrategy::LastVsOthers,
    ];

    /// How many pairs [`PairingStrategy::pairs`] gives for `targets`.
    pub fn compare_count<T>(&self, targets: &[T]) -> usize {
        match self {
            PairingStrategy::All => targets.len() * targets.len().saturating_sub(1) / 2,
            PairingStrategy::SelfCoverage => targets.len(),
            PairingStrategy::Adjacent => targets.len().saturating_sub(1),
            PairingStrategy::FirstVsOthers | PairingStrategy::LastVsOthers => {
                targets.len().saturating_sub(1)
            }
            PairingStrategy::AllAndSelf => targets.len() * (targets.len() + 1) / 2,
        }
    }
    /// The pairs of `targets` this strategy compares, in the order they are
    /// numbered.
    pub fn pairs<'a, T: std::marker::Sync>(
        &'a self,
        targets: &'a [T],
    ) -> Box<dyn Iterator<Item = (&'a T, &'a T)> + Send + 'a> {
        match self {
            PairingStrategy::AllAndSelf => Box::new(
                targets
                    .iter()
                    .combinations(2)
                    .map(|c| (c[0], c[1]))
                    .chain(targets.iter().map(|f| (f, f))),
            ),
            PairingStrategy::All => Box::new(targets.iter().combinations(2).map(|c| (c[0], c[1]))),
            PairingStrategy::SelfCoverage => Box::new(targets.iter().map(|f| (f, f))),
            PairingStrategy::Adjacent => Box::new(targets.windows(2).map(|w| (&w[0], &w[1]))),
            PairingStrategy::FirstVsOthers => {
                let mut it = targets.iter();
                if let Some(first) = it.next() {
                    Box::new(it.map(move |other| (first, other))) // move the refs into the closure
                } else {
                    Box::new(std::iter::empty())
                }
            }
            PairingStrategy::LastVsOthers => {
                let mut it = targets.iter().rev();
                if let Some(last) = it.next() {
                    Box::new(it.map(move |other| (last, other))) // move the refs into the closure
                } else {
                    Box::new(std::iter::empty())
                }
            }
        }
    }
}

/// Function similarities, one for each function of one side against each of
/// the other, indexed `[i, j]`.
///
/// Every cell is a finite number, which is what lets the assignment that
/// aggregates a matrix always find a best pairing.
#[derive(Debug, Clone, PartialEq)]
pub struct Matrix {
    dim: (usize, usize),
    cells: Vec<f64>,
}

impl Matrix {
    /// A matrix of `dim.0` by `dim.1` cells, `cells` laid out with the second
    /// index running fastest.
    ///
    /// `None` when `cells` is not that many long or holds a number that is not
    /// finite.
    pub fn new(dim: (usize, usize), cells: Vec<f64>) -> Option<Matrix> {
        (Some(cells.len()) == dim.0.checked_mul(dim.1) && cells.iter().all(|c| c.is_finite()))
            .then_some(Matrix { dim, cells })
    }

    /// A matrix of `dim.0` by `dim.1` zeros.
    pub fn zeros(dim: (usize, usize)) -> Matrix {
        Matrix {
            dim,
            cells: vec![0.0; dim.0 * dim.1],
        }
    }

    /// How many cells it has along each index.
    pub fn dim(&self) -> (usize, usize) {
        self.dim
    }

    /// The cells whose first index is `i`, in order of the second.
    pub fn row(&self, i: usize) -> &[f64] {
        &self.cells[i * self.dim.1..(i + 1) * self.dim.1]
    }
}

impl std::ops::Index<[usize; 2]> for Matrix {
    type Output = f64;

    fn index(&self, [i, j]: [usize; 2]) -> &f64 {
        assert!(
            i < self.dim.0 && j < self.dim.1,
            "[{i}, {j}] is outside {:?}",
            self.dim
        );
        &self.cells[i * self.dim.1 + j]
    }
}

/// Two birthmarks compared: their function similarity matrix, the pair's one
/// similarity, and how long it took.
pub struct Comparison<'a, S> {
    columns: &'a S,
    rows: &'a S,
    matrix: Matrix,
    duration: std::time::Duration,
    similarities: Vec<f64>,
}

impl<'a, S> Comparison<'a, S> {
    pub(crate) fn new(
        columns: &'a S,
        rows: &'a S,
        matrix: Matrix,
        similarities: Vec<f64>,
        duration: std::time::Duration,
    ) -> Comparison<'a, S> {
        Self {
            columns,
            rows,
            matrix,
            similarities,
            duration,
        }
    }

    /// The left-hand side: one column of [`Comparison::matrix`] per function.
    pub fn columns(&self) -> &S {
        self.columns
    }

    /// The right-hand side: one row of [`Comparison::matrix`] per function.
    pub fn rows(&self) -> &S {
        self.rows
    }

    /// The function-to-function scores, indexed `[column, row]`: one index
    /// for each function of [`Comparison::columns`] and one for each of
    /// [`Comparison::rows`].
    pub fn matrix(&self) -> &Matrix {
        &self.matrix
    }

    /// The pair's similarity, between 0 and 1: the mean of what the
    /// [`Aggregator`] kept of the matrix.
    ///
    /// Two birthmarks with no functions score 1.0, and one with none against
    /// one with some, 0.0.
    pub fn similarity(&self) -> f64 {
        if self.similarities.is_empty() {
            return 0.0;
        }
        self.similarities.iter().sum::<f64>() / self.similarities.len() as f64
    }

    /// How long the comparison took.
    pub fn duration(&self) -> std::time::Duration {
        self.duration
    }
}

/// How many of each side's best matches [`Aggregator::TopN`] averages.
#[derive(Debug, Clone)]
pub enum Size {
    /// This many from each side, at least 1.
    Num(usize),
    /// Every function's.
    All,
}

/// How a pair's function similarities become its one similarity.
///
/// Read by `from_str` from `hungarian`, `topn:N`, `topn:all`, `topn`,
/// `containment` or `matched:T`, in any case.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub enum Aggregator {
    /// Each function's best match in the other birthmark, on both sides and
    /// not necessarily one-to-one; the [`Size`] best of those from each side
    /// are averaged.
    TopN(Size),
    /// The one-to-one matching of functions that maximises the total, by the
    /// Hungarian algorithm; the matched pairs' similarities are averaged. The
    /// default.
    #[default]
    Hungarian,
    /// The matching of [`Aggregator::Hungarian`], averaged over the smaller
    /// side's functions rather than the larger's: how much of the smaller
    /// birthmark is found in the larger one. A program copied whole into a
    /// larger one scores 1.0, where under `Hungarian` it scores the share of
    /// the larger it makes up. It says the smaller is contained in the larger,
    /// not that the two are alike.
    Containment,
    /// The matching of [`Aggregator::Hungarian`], counting the matched pairs
    /// whose similarity is at least this threshold: the proportion of the
    /// larger side's functions that have a counterpart that close. Read from
    /// `matched:T`, which refuses a threshold outside (0, 1].
    Matched(f64),
}

impl std::str::FromStr for Aggregator {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        let s = s.to_lowercase();
        if s == "hungarian" {
            log::info!("Using Hungarian algorithm for aggregation");
            Ok(Aggregator::Hungarian)
        } else if s == "topn" || s == "topn:all" {
            log::info!("Using TopN(all) algorithm for aggregation");
            Ok(Aggregator::TopN(Size::All))
        } else if let Some(n) = s.strip_prefix("topn:") {
            match n.parse::<usize>() {
                Ok(0) => Err(Error::Parse(
                    "topn requires N >= 1 (use \"topn:all\" for all elements)".to_string(),
                )),
                Ok(n) => {
                    log::info!("Using TopN({n}) algorithm for aggregation");
                    Ok(Aggregator::TopN(Size::Num(n)))
                }
                Err(e) => Err(Error::ParseInt(s, e)),
            }
        } else if s == "containment" {
            log::info!("Using containment for aggregation");
            Ok(Aggregator::Containment)
        } else if let Some(t) = s.strip_prefix("matched:") {
            match t.parse::<f64>() {
                Ok(t) if t > 0.0 && t <= 1.0 => {
                    log::info!("Using matched({t}) for aggregation");
                    Ok(Aggregator::Matched(t))
                }
                _ => Err(Error::Parse(format!(
                    "{s}: matched requires a threshold above 0 and at most 1, such as \"matched:0.9\""
                ))),
            }
        } else if s == "matched" {
            Err(Error::Parse(
                "matched requires a threshold, such as \"matched:0.9\"".to_string(),
            ))
        } else {
            Err(Error::Parse(format!("{s}: Invalid aggregator")))
        }
    }
}

impl Aggregator {
    /// The similarities this aggregator keeps of `matrix`, which
    /// [`Comparison::similarity`] averages.
    ///
    /// The shorter side counts as though it had as many functions as the
    /// longer, each sharing nothing with any function: a function left
    /// without a counterpart lowers the pair's similarity rather than drop
    /// out of it. [`Aggregator::Containment`] alone does not, since leaving
    /// the larger side's other functions out is what it is for.
    pub fn aggregate(&self, matrix: &Matrix) -> Result<Vec<f64>> {
        let longer = |mut kept: Vec<f64>| {
            let (n1, n2) = matrix.dim();
            kept.resize(n1.max(n2), 0.0);
            kept
        };
        match self {
            Aggregator::Hungarian => Ok(longer(paired_similarities(matrix))),
            Aggregator::TopN(n) => Ok(top_n_selection(matrix, n)),
            Aggregator::Containment => Ok(paired_similarities(matrix)),
            Aggregator::Matched(threshold) => Ok(longer(
                paired_similarities(matrix)
                    .into_iter()
                    .map(|s| if s >= *threshold { 1.0 } else { 0.0 })
                    .collect(),
            )),
        }
    }
}

/// How far a comparison has got, as [`Comparator::compare_birthmarks_with`]
/// reports it to an [`Observer`].
///
/// A pair with no functions on a side has no matrix to fill and reports
/// nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Progress {
    /// The matrix is about to be filled.
    MatrixStarted {
        /// How many rows it will have, each reported by
        /// [`Progress::RowFilled`] when done: the first of [`Matrix::dim`].
        rows: usize,
        /// How many cells each row holds: the second of [`Matrix::dim`].
        columns: usize,
    },
    /// One [`Matrix::row`] is filled. Reported once per row, in no
    /// particular order.
    RowFilled,
    /// Every row is filled and the [`Aggregator`] is running.
    Aggregating,
}

/// Told how far a comparison has got.
///
/// The rows of a matrix are filled on several threads at once, so
/// [`Observer::notify`] is called concurrently, and once per row: it should
/// return quickly. Any `Fn(Progress) + Sync` closure is one.
pub trait Observer: Sync {
    /// Called with each step of a comparison as it happens.
    fn notify(&self, progress: Progress);
}

impl<F: Fn(Progress) + Sync> Observer for F {
    fn notify(&self, progress: Progress) {
        self(progress)
    }
}

trait BirthmarkComparator: Sync {
    fn compare_birthmarks<'a>(
        &self,
        b1: &'a Birthmark,
        b2: &'a Birthmark,
        aggregator: &Aggregator,
        observer: &dyn Observer,
    ) -> Result<Comparison<'a, Birthmark>> {
        // Asking the birthmark rather than re-deriving the conditions here
        // keeps the reason in the error: a mismatch of representation reads
        // differently from a mismatch of type.
        b1.check_comparable_with(b2)?;
        let p1_len = b1.functions.len();
        let p2_len = b2.functions.len();
        if p1_len == 0 && p2_len == 0 {
            Ok(Comparison::new(
                b1,
                b2,
                Matrix::zeros((0, 0)),
                vec![1.0],
                std::time::Duration::from_millis(0),
            ))
        } else if p1_len == 0 || p2_len == 0 {
            Ok(Comparison::new(
                b1,
                b2,
                Matrix::zeros((p1_len, p2_len)),
                vec![0.0],
                std::time::Duration::from_millis(0),
            ))
        } else {
            let start = Instant::now();
            // Each function once in the shape the algorithm compares, rather
            // than once per cell: converting is what a cell would otherwise
            // spend most of its time on.
            let shape = self.shape();
            let d1: Vec<_> = b1
                .functions
                .iter()
                .map(|f| in_shape(&f.data, shape))
                .collect();
            let d2: Vec<_> = b2
                .functions
                .iter()
                .map(|f| in_shape(&f.data, shape))
                .collect();
            observer.notify(Progress::MatrixStarted {
                rows: d1.len(),
                columns: d2.len(),
            });
            let r = build_matrix(
                &d1,
                &d2,
                |e1, e2| self.compare_data(e1, e2),
                || observer.notify(Progress::RowFilled),
            )?;
            observer.notify(Progress::Aggregating);
            aggregator
                .aggregate(&r)
                .map(|sim| Comparison::new(b1, b2, r, sim, start.elapsed()))
        }
    }

    /// The shape this algorithm compares; see [`Algorithm::shape`].
    fn shape(&self) -> Shape;

    /// The similarity of two functions' elements, each already in
    /// [`BirthmarkComparator::shape`] where it can be. Elements that cannot
    /// be, such as a set handed to a sequence algorithm, score 0.0.
    fn compare_data(&self, d1: &Data, d2: &Data) -> f64;

    /// The similarity of two functions as they come, converting each first.
    #[cfg(test)]
    fn compare_functions(
        &self,
        e1: &crate::birthmarks::Function,
        e2: &crate::birthmarks::Function,
    ) -> f64 {
        let shape = self.shape();
        self.compare_data(&in_shape(&e1.data, shape), &in_shape(&e2.data, shape))
    }
}

/// Every function of `p1` against every function of `p2`, one row per function
/// of `p1`.
///
/// The rows are filled in parallel: each cell depends on its two functions
/// alone, and a large pair is where a comparison spends its time. `on_row` is
/// called as each row is done, a row being the finest step worth reporting:
/// a cell can take well under a microsecond.
fn build_matrix<T: Sync>(
    p1: &[T],
    p2: &[T],
    compare_func: impl Fn(&T, &T) -> f64 + Sync,
    on_row: impl Fn() + Sync,
) -> Result<Matrix> {
    let mut similarities = vec![0.0; p1.len() * p2.len()];
    if !p2.is_empty() {
        similarities
            .par_chunks_mut(p2.len())
            .zip(p1.par_iter())
            .for_each(|(row, item1)| {
                for (cell, item2) in row.iter_mut().zip(p2) {
                    *cell = compare_func(item1, item2);
                }
                on_row();
            });
    }
    Matrix::new((p1.len(), p2.len()), similarities).ok_or_else(|| {
        Error::Parse("a function similarity came out as a number that is not finite".to_string())
    })
}

/// Each function's best similarity, on both sides, the best `n` of each.
///
/// The shorter side's list is filled out with zeros to the longer's length,
/// as [`Aggregator::aggregate`] says.
fn top_n_selection(matrix: &Matrix, n: &Size) -> Vec<f64> {
    let (n1, n2) = matrix.dim();
    let size = n1.max(n2);
    let best = |maxima: Vec<f64>| {
        let mut maxima = maxima;
        maxima.resize(size, 0.0);
        maxima.sort_by(|a, b| b.total_cmp(a));
        match n {
            Size::Num(k) => maxima.truncate(*k),
            Size::All => {}
        }
        maxima
    };
    let firsts = (0..n1)
        .map(|i| matrix.row(i).iter().fold(0.0f64, |acc, &v| acc.max(v)))
        .collect();
    let mut seconds = vec![0.0f64; n2];
    for i in 0..n1 {
        for (best, &v) in seconds.iter_mut().zip(matrix.row(i)) {
            *best = best.max(v);
        }
    }
    let mut kept = best(firsts);
    kept.extend(best(seconds));
    kept
}

/// The similarities of the one-to-one pairing that maximises their total, by
/// the Hungarian method: one for each function of the shorter side.
fn paired_similarities(matrix: &Matrix) -> Vec<f64> {
    let (n1, n2) = matrix.dim();
    let pairing = crate::assignment::assign(n1, n2, &matrix.cells)
        .expect("a Matrix holds only finite cells, rows * cols of them");
    pairing
        .iter()
        .enumerate()
        .filter_map(|(i, j)| j.map(|j| matrix[[i, j]]))
        .collect()
}

/// How two functions' elements are compared into a function similarity,
/// between 0 and 1.
///
/// Each operates on one [`Shape`] ([`Algorithm::shape`]). Two empty functions
/// score 1.0 under every one, and an empty one against one that is not, 0.0.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Algorithm {
    /// Cosine similarity based on term frequency vectors. Available: seq and freq.
    Cosine,
    /// Dice coefficient. Available: seq, set and freq.
    Dice,
    /// Euclidean distance between term frequency vectors. Available: seq and freq.
    Euclidean,
    /// Jaccard index. Available: seq, set and freq.
    Jaccard,
    /// Jensen–Shannon divergence between the functions' element distributions,
    /// as the similarity 1 − √JSD. Available: seq and freq.
    JensenShannon,
    /// Levenshtein distance. Available: seq.
    Levenshtein,
    /// Longest Common Subsequence (LCS). Available: seq.
    Lcs,
    /// Simpson's coefficient. Available: seq, set and freq.
    Simpson,
    /// Tanimoto coefficient over term frequency vectors. Available: seq and freq.
    Tanimoto,
    /// Weighted Jaccard index based on term frequency vectors. Available: seq and freq.
    WeightedJaccard,
}

/// Reads either spelling of an algorithm's name, in any case.
impl std::str::FromStr for Algorithm {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let name = s.to_lowercase();
        Algorithm::ALL
            .iter()
            .find(|a| a.name() == name || a.hyphenated() == name)
            .cloned()
            .ok_or_else(|| Error::Parse(format!("{s}: unknown algorithm")))
    }
}

impl std::fmt::Display for Algorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Algorithm::Cosine => write!(f, "Cosine Similarity"),
            Algorithm::Dice => write!(f, "Dice Coefficient"),
            Algorithm::Euclidean => write!(f, "Euclidean Distance"),
            Algorithm::Jaccard => write!(f, "Jaccard Index"),
            Algorithm::JensenShannon => write!(f, "Jensen-Shannon Divergence"),
            Algorithm::Levenshtein => write!(f, "Levenshtein Distance"),
            Algorithm::Lcs => write!(f, "Longest Common Subsequence"),
            Algorithm::Simpson => write!(f, "Simpson's Coefficient"),
            Algorithm::Tanimoto => write!(f, "Tanimoto Coefficient"),
            Algorithm::WeightedJaccard => write!(f, "Weighted Jaccard Index"),
        }
    }
}

impl Algorithm {
    /// Every algorithm, in the order they are declared.
    ///
    /// Written out because nothing derives it; a test holds it to the enum
    /// with an exhaustive `match`.
    pub const ALL: &[Algorithm] = &[
        Algorithm::Cosine,
        Algorithm::Dice,
        Algorithm::Euclidean,
        Algorithm::Jaccard,
        Algorithm::JensenShannon,
        Algorithm::Levenshtein,
        Algorithm::Lcs,
        Algorithm::Simpson,
        Algorithm::Tanimoto,
        Algorithm::WeightedJaccard,
    ];

    /// The spelling of this algorithm and the shape it operates on, in one
    /// table so that the parser, the validation and the error message cannot
    /// drift apart.
    ///
    /// This is the canonical spelling inside an analysis name. A name with a
    /// hyphen between its words is accepted as well (see
    /// [`Algorithm::hyphenated`]), so that a name built from either spelling
    /// parses.
    fn spec(&self) -> (&'static str, Shape) {
        match self {
            Algorithm::Cosine => ("cosine", Shape::Freq),
            Algorithm::Dice => ("dice", Shape::Set),
            Algorithm::Euclidean => ("euclidean", Shape::Freq),
            Algorithm::Jaccard => ("jaccard", Shape::Set),
            Algorithm::JensenShannon => ("jensenshannon", Shape::Freq),
            Algorithm::Lcs => ("lcs", Shape::Seq),
            Algorithm::Levenshtein => ("levenshtein", Shape::Seq),
            Algorithm::Simpson => ("simpson", Shape::Set),
            Algorithm::Tanimoto => ("tanimoto", Shape::Freq),
            Algorithm::WeightedJaccard => ("weightedjaccard", Shape::Freq),
        }
    }

    /// The same name with a hyphen between its words.
    ///
    /// It differs from [`Algorithm::name`] for `jensenshannon` and
    /// `weightedjaccard`; the others are single words and spell the same
    /// either way.
    fn hyphenated(&self) -> &'static str {
        match self {
            Algorithm::JensenShannon => "jensen-shannon",
            Algorithm::WeightedJaccard => "weighted-jaccard",
            other => other.name(),
        }
    }

    /// The birthmark shape this algorithm computes over. Anything else it is
    /// handed is converted to this first, which is why a pairing that does not
    /// match is rejected rather than quietly re-encoded.
    pub fn shape(&self) -> Shape {
        self.spec().1
    }

    /// The spelling this algorithm has in an analysis name.
    pub fn name(&self) -> &'static str {
        self.spec().0
    }
}

/// Compares birthmarks under one [`Algorithm`], on the threads of the
/// [`Oinkie`](crate::Oinkie) that made it with
/// [`Oinkie::comparator`](crate::Oinkie::comparator).
pub struct Comparator {
    inner: ComparatorImpl,
    pool: Arc<rayon::ThreadPool>,
}

/// The algorithm behind a [`Comparator`], kept out of its public API so that
/// the algorithms can change without changing it.
enum ComparatorImpl {
    Cosine(Cosine),
    Dice(Dice),
    Euclidean(Euclidean),
    Jaccard(Jaccard),
    JensenShannon(JensenShannon),
    Levenshtein(Levenshtein),
    Lcs(Lcs),
    Simpson(Simpson),
    Tanimoto(Tanimoto),
    WeightedJaccard(WeightedJaccard),
}

impl Comparator {
    pub(crate) fn new(algorithm: &Algorithm, pool: Arc<rayon::ThreadPool>) -> Comparator {
        let inner = match algorithm {
            Algorithm::Cosine => ComparatorImpl::Cosine(Cosine),
            Algorithm::Dice => ComparatorImpl::Dice(Dice),
            Algorithm::Euclidean => ComparatorImpl::Euclidean(Euclidean),
            Algorithm::Jaccard => ComparatorImpl::Jaccard(Jaccard),
            Algorithm::JensenShannon => ComparatorImpl::JensenShannon(JensenShannon),
            Algorithm::Levenshtein => ComparatorImpl::Levenshtein(Levenshtein),
            Algorithm::Lcs => ComparatorImpl::Lcs(Lcs),
            Algorithm::Simpson => ComparatorImpl::Simpson(Simpson),
            Algorithm::Tanimoto => ComparatorImpl::Tanimoto(Tanimoto),
            Algorithm::WeightedJaccard => ComparatorImpl::WeightedJaccard(WeightedJaccard),
        };
        Comparator { inner, pool }
    }

    /// Compares every function of `b1`, the matrix's columns, with every
    /// function of `b2`, its rows, and aggregates the matrix with
    /// `aggregator`.
    ///
    /// Fails with [`Error::IrMismatch`] or [`Error::Mismatch`] unless both
    /// were lifted to the same representation and are of the same type: a
    /// birthmark is comparable only with one made the same way.
    pub fn compare_birthmarks<'a>(
        &self,
        b1: &'a Birthmark,
        b2: &'a Birthmark,
        aggregator: &Aggregator,
    ) -> Result<Comparison<'a, Birthmark>> {
        self.compare_birthmarks_with(b1, b2, aggregator, &|_: Progress| {})
    }

    /// [`Comparator::compare_birthmarks`], telling `observer` how far it has
    /// got as it goes. The comparison is the same whatever `observer` does.
    pub fn compare_birthmarks_with<'a>(
        &self,
        b1: &'a Birthmark,
        b2: &'a Birthmark,
        aggregator: &Aggregator,
        observer: &dyn Observer,
    ) -> Result<Comparison<'a, Birthmark>> {
        let (a, o) = (aggregator, observer);
        self.pool.install(|| match &self.inner {
            ComparatorImpl::Cosine(c) => c.compare_birthmarks(b1, b2, a, o),
            ComparatorImpl::Dice(d) => d.compare_birthmarks(b1, b2, a, o),
            ComparatorImpl::Euclidean(e) => e.compare_birthmarks(b1, b2, a, o),
            ComparatorImpl::Jaccard(j) => j.compare_birthmarks(b1, b2, a, o),
            ComparatorImpl::JensenShannon(js) => js.compare_birthmarks(b1, b2, a, o),
            ComparatorImpl::Levenshtein(l) => l.compare_birthmarks(b1, b2, a, o),
            ComparatorImpl::Lcs(lcs) => lcs.compare_birthmarks(b1, b2, a, o),
            ComparatorImpl::Simpson(s) => s.compare_birthmarks(b1, b2, a, o),
            ComparatorImpl::Tanimoto(t) => t.compare_birthmarks(b1, b2, a, o),
            ComparatorImpl::WeightedJaccard(wj) => wj.compare_birthmarks(b1, b2, a, o),
        })
    }
}

struct Jaccard;
struct Dice;
struct Simpson;
struct Levenshtein;
struct Cosine;
struct Euclidean;
struct WeightedJaccard;
struct Lcs;
struct JensenShannon;
struct Tanimoto;

impl BirthmarkComparator for Jaccard {
    fn shape(&self) -> Shape {
        Shape::Set
    }

    fn compare_data(&self, d1: &Data, d2: &Data) -> f64 {
        match (d1, d2) {
            (Data::Set(s1), Data::Set(s2)) => jaccard_index(s1, s2),
            (Data::KgramSet(k1), Data::KgramSet(k2)) => jaccard_index(k1, k2),
            _ => 0.0,
        }
    }
}

impl BirthmarkComparator for Dice {
    fn shape(&self) -> Shape {
        Shape::Set
    }

    fn compare_data(&self, d1: &Data, d2: &Data) -> f64 {
        match (d1, d2) {
            (Data::Set(s1), Data::Set(s2)) => dice_index(s1, s2),
            (Data::KgramSet(k1), Data::KgramSet(k2)) => dice_index(k1, k2),
            _ => 0.0,
        }
    }
}

impl BirthmarkComparator for Simpson {
    fn shape(&self) -> Shape {
        Shape::Set
    }

    fn compare_data(&self, d1: &Data, d2: &Data) -> f64 {
        match (d1, d2) {
            (Data::Set(s1), Data::Set(s2)) => simpson_index(s1, s2),
            (Data::KgramSet(k1), Data::KgramSet(k2)) => simpson_index(k1, k2),
            _ => 0.0,
        }
    }
}

impl BirthmarkComparator for Levenshtein {
    fn shape(&self) -> Shape {
        Shape::Seq
    }

    fn compare_data(&self, d1: &Data, d2: &Data) -> f64 {
        match (d1, d2) {
            (Data::Seq(s1), Data::Seq(s2)) => levenshtein_distance(s1, s2),
            (Data::KgramSeq(k1), Data::KgramSeq(k2)) => levenshtein_distance(k1, k2),
            _ => 0.0,
        }
    }
}

impl BirthmarkComparator for Cosine {
    fn shape(&self) -> Shape {
        Shape::Freq
    }

    fn compare_data(&self, d1: &Data, d2: &Data) -> f64 {
        match (d1, d2) {
            (Data::Freq(f1), Data::Freq(f2)) => cosine_similarity(f1, f2),
            (Data::KgramFreq(k1), Data::KgramFreq(k2)) => cosine_similarity(k1, k2),
            _ => 0.0,
        }
    }
}

impl BirthmarkComparator for Euclidean {
    fn shape(&self) -> Shape {
        Shape::Freq
    }

    fn compare_data(&self, d1: &Data, d2: &Data) -> f64 {
        match (d1, d2) {
            (Data::Freq(f1), Data::Freq(f2)) => euclidean_distance(f1, f2),
            (Data::KgramFreq(k1), Data::KgramFreq(k2)) => euclidean_distance(k1, k2),
            _ => 0.0,
        }
    }
}

impl BirthmarkComparator for WeightedJaccard {
    fn shape(&self) -> Shape {
        Shape::Freq
    }

    fn compare_data(&self, d1: &Data, d2: &Data) -> f64 {
        match (d1, d2) {
            (Data::Freq(f1), Data::Freq(f2)) => weighted_jaccard(f1, f2),
            (Data::KgramFreq(k1), Data::KgramFreq(k2)) => weighted_jaccard(k1, k2),
            _ => 0.0,
        }
    }
}

impl BirthmarkComparator for JensenShannon {
    fn shape(&self) -> Shape {
        Shape::Freq
    }

    fn compare_data(&self, d1: &Data, d2: &Data) -> f64 {
        match (d1, d2) {
            (Data::Freq(f1), Data::Freq(f2)) => jensen_shannon(f1, f2),
            (Data::KgramFreq(k1), Data::KgramFreq(k2)) => jensen_shannon(k1, k2),
            _ => 0.0,
        }
    }
}

impl BirthmarkComparator for Tanimoto {
    fn shape(&self) -> Shape {
        Shape::Freq
    }

    fn compare_data(&self, d1: &Data, d2: &Data) -> f64 {
        match (d1, d2) {
            (Data::Freq(f1), Data::Freq(f2)) => tanimoto(f1, f2),
            (Data::KgramFreq(k1), Data::KgramFreq(k2)) => tanimoto(k1, k2),
            _ => 0.0,
        }
    }
}

impl BirthmarkComparator for Lcs {
    fn shape(&self) -> Shape {
        Shape::Seq
    }

    fn compare_data(&self, d1: &Data, d2: &Data) -> f64 {
        match (d1, d2) {
            (Data::Seq(s1), Data::Seq(s2)) => longest_common_subsequence(s1, s2),
            (Data::KgramSeq(k1), Data::KgramSeq(k2)) => longest_common_subsequence(k1, k2),
            _ => 0.0,
        }
    }
}

/// `data` in `shape`, converted only when it is in another: a sequence
/// becomes a set or a frequency map, and a frequency map a set. A shape that
/// cannot be reached -- a sequence from a set, say -- leaves `data` as it is.
fn in_shape(data: &Data, shape: Shape) -> Cow<'_, Data> {
    match (data, shape) {
        (Data::Seq(s), Shape::Set) => Cow::Owned(Data::Set(seq2set(s))),
        (Data::Seq(s), Shape::Freq) => Cow::Owned(Data::Freq(seq2freq(s))),
        (Data::Freq(f), Shape::Set) => Cow::Owned(Data::Set(freq2set(f))),
        (Data::KgramSeq(k), Shape::Set) => Cow::Owned(Data::KgramSet(seq2set(k))),
        (Data::KgramSeq(k), Shape::Freq) => Cow::Owned(Data::KgramFreq(seq2freq(k))),
        (Data::KgramFreq(k), Shape::Set) => Cow::Owned(Data::KgramSet(freq2set(k))),
        _ => Cow::Borrowed(data),
    }
}

fn seq2set<T>(seq: &[T]) -> FxHashSet<T>
where
    T: std::hash::Hash + Eq + Clone,
{
    seq.iter().cloned().collect()
}

fn freq2set<T>(freq: &FxHashMap<T, usize>) -> FxHashSet<T>
where
    T: std::hash::Hash + Eq + Clone,
{
    freq.keys().cloned().collect()
}

fn seq2freq<T>(seq: &[T]) -> FxHashMap<T, usize>
where
    T: std::hash::Hash + Eq + Clone,
{
    let mut freq = FxHashMap::default();
    for item in seq {
        *freq.entry(item.clone()).or_insert(0) += 1;
    }
    freq
}

fn jaccard_index<T: std::fmt::Debug + std::cmp::Eq + std::hash::Hash>(
    s1: &FxHashSet<T>,
    s2: &FxHashSet<T>,
) -> f64 {
    if s1.is_empty() && s2.is_empty() {
        1.0
    } else if s1.is_empty() || s2.is_empty() {
        0.0
    } else {
        s1.intersection(s2).count() as f64 / s1.union(s2).count() as f64
    }
}

fn dice_index<T: std::fmt::Debug + std::cmp::Eq + std::hash::Hash>(
    s1: &FxHashSet<T>,
    s2: &FxHashSet<T>,
) -> f64 {
    if s1.is_empty() && s2.is_empty() {
        1.0
    } else if s1.is_empty() || s2.is_empty() {
        0.0
    } else {
        (2.0 * s1.intersection(s2).count() as f64) / (s1.len() + s2.len()) as f64
    }
}

fn simpson_index<T: std::fmt::Debug + std::cmp::Eq + std::hash::Hash>(
    s1: &FxHashSet<T>,
    s2: &FxHashSet<T>,
) -> f64 {
    if s1.is_empty() && s2.is_empty() {
        1.0
    } else if s1.is_empty() || s2.is_empty() {
        0.0
    } else {
        s1.intersection(s2).count() as f64 / s1.len().min(s2.len()) as f64
    }
}

fn levenshtein_distance<T: PartialEq>(s1: &[T], s2: &[T]) -> f64 {
    let n = s1.len();
    let m = s2.len();

    if n == 0 && m == 0 {
        return 1.0;
    }
    if n == 0 || m == 0 {
        return 0.0;
    }
    let max_len = n.max(m);

    // shorter sequence is s2, longer sequence is s1 for minimizing memory usage.
    let (s1, s2) = if n < m { (s2, s1) } else { (s1, s2) };
    let m = s2.len();

    // allocate two rows of data (previous and current) to save memory
    let mut prev = (0..=m).collect::<Vec<usize>>();
    let mut curr = vec![0usize; m + 1];

    for i in 1..=s1.len() {
        curr[0] = i;
        for j in 1..=m {
            let cost = if s1[i - 1] == s2[j - 1] { 0 } else { 1 };

            // the cheapest of substitution, insertion and deletion
            let substitution = prev[j - 1] + cost;
            let insertion = curr[j - 1] + 1;
            let deletion = prev[j] + 1;

            curr[j] = substitution.min(insertion).min(deletion);
        }
        // swap prev and curr for the next iteration
        std::mem::swap(&mut prev, &mut curr);
    }

    let distance = prev[m];

    // change distance to similarity in the range of 0.0 to 1.0
    1.0 - (distance as f64 / max_len as f64)
}

/// The counts the two functions share: for each element in both, its count
/// in each.
///
/// The smaller map is walked and the other looked up, so nothing is
/// allocated. An element in one function only contributes nothing to the
/// sums below that need this -- its count in the other is 0 -- so the shared
/// ones are all they need.
fn shared<'a, T: std::cmp::Eq + std::hash::Hash>(
    f1: &'a FxHashMap<T, usize>,
    f2: &'a FxHashMap<T, usize>,
) -> impl Iterator<Item = (usize, usize)> + 'a {
    let swapped = f1.len() > f2.len();
    let (small, large) = if swapped { (f2, f1) } else { (f1, f2) };
    small
        .iter()
        .filter_map(move |(k, &a)| large.get(k).map(|&b| if swapped { (b, a) } else { (a, b) }))
}

// The sums below are of counts, which are whole numbers, and are taken in
// integers. Each is then the exact value that summing the same terms in f64
// gives in any order, as long as it stays below 2^53, so that rearranging a
// sum over every element into one over the shared ones changes no score.

fn cosine_similarity<T: std::cmp::Eq + std::hash::Hash>(
    f1: &FxHashMap<T, usize>,
    f2: &FxHashMap<T, usize>,
) -> f64 {
    let dot_product = shared(f1, f2)
        .map(|(a, b)| a as u128 * b as u128)
        .sum::<u128>() as f64;
    let magnitude1 = f1.values().map(|v| (*v as f64).powi(2)).sum::<f64>().sqrt();
    let magnitude2 = f2.values().map(|v| (*v as f64).powi(2)).sum::<f64>().sqrt();
    // A zero magnitude is an empty function. Two of them agree, as under every
    // other algorithm; one against a function that is not empty does not, or
    // an empty function would match anything.
    match (magnitude1 > 0.0, magnitude2 > 0.0) {
        (true, true) => dot_product / (magnitude1 * magnitude2),
        (false, false) => 1.0,
        _ => 0.0,
    }
}

fn euclidean_distance<T: std::cmp::Eq + std::hash::Hash>(
    f1: &FxHashMap<T, usize>,
    f2: &FxHashMap<T, usize>,
) -> f64 {
    let squares =
        |f: &FxHashMap<T, usize>| f.values().map(|&v| v as u128 * v as u128).sum::<u128>();
    let dot_product = shared(f1, f2)
        .map(|(a, b)| a as u128 * b as u128)
        .sum::<u128>();
    // Σ(a - b)² over every element, as Σa² + Σb² - 2Σab.
    let sum_of_squares = (squares(f1) + squares(f2) - 2 * dot_product) as f64;
    let scale = f1.values().map(|v| (*v as f64).powi(2)).sum::<f64>().sqrt()
        + f2.values().map(|v| (*v as f64).powi(2)).sum::<f64>().sqrt();
    // Zero only when both functions are empty, where the quotient below would
    // be 0/0: a NaN that would reach the assignment and the pair's
    // similarity. Two empty functions agree, as under every other algorithm.
    if scale == 0.0 {
        return 1.0;
    }
    1.0 - (sum_of_squares.sqrt() / scale)
}

fn weighted_jaccard<T: std::cmp::Eq + std::hash::Hash>(
    f1: &FxHashMap<T, usize>,
    f2: &FxHashMap<T, usize>,
) -> f64 {
    let total = |f: &FxHashMap<T, usize>| f.values().map(|&v| v as u128).sum::<u128>();
    let min_sum = shared(f1, f2).map(|(a, b)| a.min(b) as u128).sum::<u128>();
    // Σmax(a, b) over every element, as Σa + Σb - Σmin(a, b).
    let max_sum = total(f1) + total(f2) - min_sum;
    if max_sum > 0 {
        min_sum as f64 / max_sum as f64
    } else {
        1.0
    }
}

/// a·b / (|a|² + |b|² − a·b) over the counts: Jaccard's index on sets,
/// extended to multiplicities. In integers, as the sums above.
fn tanimoto<T: std::cmp::Eq + std::hash::Hash>(
    f1: &FxHashMap<T, usize>,
    f2: &FxHashMap<T, usize>,
) -> f64 {
    let squares =
        |f: &FxHashMap<T, usize>| f.values().map(|&v| v as u128 * v as u128).sum::<u128>();
    let dot_product = shared(f1, f2)
        .map(|(a, b)| a as u128 * b as u128)
        .sum::<u128>();
    // Zero only when both functions are empty, which agree.
    let denominator = squares(f1) + squares(f2) - dot_product;
    if denominator == 0 {
        return 1.0;
    }
    dot_product as f64 / denominator as f64
}

/// 1 − √JSD, the Jensen–Shannon divergence in bits between the two functions'
/// elements as distributions: each count over its function's total.
///
/// The square root because √JSD is a metric, and because it grows with the
/// differences between the distributions where JSD grows with their squares,
/// which would leave quite different functions scoring close to 1.
fn jensen_shannon<T: std::cmp::Eq + std::hash::Hash>(
    f1: &FxHashMap<T, usize>,
    f2: &FxHashMap<T, usize>,
) -> f64 {
    let total = |f: &FxHashMap<T, usize>| f.values().map(|&v| v as u128).sum::<u128>();
    let (total1, total2) = (total(f1), total(f2));
    // An empty function has no distribution. Two of them agree, and one
    // against a function that is not empty does not, as under every other
    // algorithm.
    match (total1 > 0, total2 > 0) {
        (true, true) => {}
        (false, false) => return 1.0,
        _ => return 0.0,
    }
    // An element in one function only contributes its whole probability to
    // the divergence, halved, so those terms come from the mass the shared
    // elements leave, which is counted in integers so that identical
    // functions leave exactly none.
    let (mut shared1, mut shared2, mut terms) = (0u128, 0u128, 0.0f64);
    for (a, b) in shared(f1, f2) {
        shared1 += a as u128;
        shared2 += b as u128;
        let p = a as f64 / total1 as f64;
        let q = b as f64 / total2 as f64;
        let m = p + q;
        terms += p * (2.0 * p / m).log2() + q * (2.0 * q / m).log2();
    }
    let unshared =
        (total1 - shared1) as f64 / total1 as f64 + (total2 - shared2) as f64 / total2 as f64;
    // Rounding can take a divergence of zero a hair below it, where the root
    // is NaN, or one of 1 a hair above.
    let divergence = ((unshared + terms) / 2.0).clamp(0.0, 1.0);
    1.0 - divergence.sqrt()
}

fn longest_common_subsequence<T: PartialEq>(s1: &[T], s2: &[T]) -> f64 {
    let n = s1.len();
    let m = s2.len();
    // Two empty functions agree, as under every other algorithm.
    if n == 0 && m == 0 {
        return 1.0;
    }
    if n == 0 || m == 0 {
        return 0.0;
    }

    // shorter sequence is s2, longer sequence is s1 for minimizing memory usage.
    let (s1, s2) = if n < m { (s2, s1) } else { (s1, s2) };
    let n = s1.len();
    let m = s2.len();

    // allocate two rows of data (previous and current) to save memory
    let mut prev = vec![0usize; m + 1];
    let mut curr = vec![0usize; m + 1];

    for i in 1..=n {
        for j in 1..=m {
            if s1[i - 1] == s2[j - 1] {
                curr[j] = prev[j - 1] + 1;
            } else {
                curr[j] = prev[j].max(curr[j - 1]);
            }
        }
        // swap prev and curr for the next iteration
        std::mem::swap(&mut prev, &mut curr);
    }

    // the previous row contains the final results since the last swap
    let lcs_length = prev[m];
    2.0 * lcs_length as f64 / (n + m) as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::birthmarks::{BirthmarkType, Function, Kgram};

    /// `Algorithm::ALL` is written by hand, so it is held to the enum: the
    /// `match` stops compiling when a variant is added, and the assertion
    /// fails when it is added there but not to the list.
    #[test]
    fn test_all_lists_every_algorithm_exactly_once() {
        fn index(a: &Algorithm) -> usize {
            match a {
                Algorithm::Cosine => 0,
                Algorithm::Dice => 1,
                Algorithm::Euclidean => 2,
                Algorithm::Jaccard => 3,
                Algorithm::JensenShannon => 4,
                Algorithm::Levenshtein => 5,
                Algorithm::Lcs => 6,
                Algorithm::Simpson => 7,
                Algorithm::Tanimoto => 8,
                Algorithm::WeightedJaccard => 9,
            }
        }
        let mut seen = Algorithm::ALL.iter().map(index).collect::<Vec<_>>();
        seen.sort_unstable();
        assert_eq!(seen, (0..10).collect::<Vec<_>>());
    }

    /// The same for `PairingStrategy::ALL`.
    #[test]
    fn test_all_lists_every_pairing_strategy_exactly_once() {
        fn index(p: &PairingStrategy) -> usize {
            match p {
                PairingStrategy::AllAndSelf => 0,
                PairingStrategy::All => 1,
                PairingStrategy::SelfCoverage => 2,
                PairingStrategy::Adjacent => 3,
                PairingStrategy::FirstVsOthers => 4,
                PairingStrategy::LastVsOthers => 5,
            }
        }
        let mut seen = PairingStrategy::ALL.iter().map(index).collect::<Vec<_>>();
        seen.sort_unstable();
        assert_eq!(seen, (0..6).collect::<Vec<_>>());
    }

    /// Both spellings of an algorithm's name read back to it, in any case; a
    /// name that is not one is refused naming what was given.
    #[test]
    fn test_an_algorithm_is_read_from_either_spelling() {
        use std::str::FromStr;
        for a in Algorithm::ALL {
            assert_eq!(&Algorithm::from_str(a.name()).unwrap(), a);
            assert_eq!(&Algorithm::from_str(&a.name().to_uppercase()).unwrap(), a);
        }
        assert_eq!(
            Algorithm::from_str("weighted-jaccard").unwrap(),
            Algorithm::WeightedJaccard
        );
        assert_eq!(
            Algorithm::from_str("Jensen-Shannon").unwrap(),
            Algorithm::JensenShannon
        );
        let err = Algorithm::from_str("nonsense").unwrap_err().to_string();
        assert!(err.contains("nonsense"), "{err}");
    }
    use crate::birthmarks::Metadata;
    use std::path::PathBuf;

    #[test]
    fn compare_count_does_not_panic_on_empty_targets() {
        let empty: Vec<i32> = vec![];
        assert_eq!(PairingStrategy::All.compare_count(&empty), 0);
        assert_eq!(PairingStrategy::AllAndSelf.compare_count(&empty), 0);
        assert_eq!(PairingStrategy::SelfCoverage.compare_count(&empty), 0);
        assert_eq!(PairingStrategy::Adjacent.compare_count(&empty), 0);
        assert_eq!(PairingStrategy::FirstVsOthers.compare_count(&empty), 0);
        assert_eq!(PairingStrategy::LastVsOthers.compare_count(&empty), 0);
    }

    #[test]
    fn aggregator_rejects_topn_zero() {
        assert!("topn:0".parse::<Aggregator>().is_err());
        assert!(matches!(
            "topn:3".parse::<Aggregator>(),
            Ok(Aggregator::TopN(Size::Num(3)))
        ));
        assert!(matches!(
            "topn".parse::<Aggregator>(),
            Ok(Aggregator::TopN(Size::All))
        ));
        assert!(matches!(
            "hungarian".parse::<Aggregator>(),
            Ok(Aggregator::Hungarian)
        ));
    }

    #[test]
    fn aggregator_rejects_malformed_input() {
        // a non-numeric N and an entirely unknown name take separate error paths
        assert!(matches!(
            "topn:abc".parse::<Aggregator>(),
            Err(Error::ParseInt(..))
        ));
        assert!(matches!(
            "nonsense".parse::<Aggregator>(),
            Err(Error::Parse(_))
        ));
        // parsing is case insensitive
        assert!(matches!(
            "HUNGARIAN".parse::<Aggregator>(),
            Ok(Aggregator::Hungarian)
        ));
        assert!(matches!(
            "TopN:All".parse::<Aggregator>(),
            Ok(Aggregator::TopN(Size::All))
        ));
    }

    /// `containment` takes nothing, and `matched` a threshold in (0, 1]
    /// that it cannot do without.
    #[test]
    fn containment_and_matched_are_read_with_their_threshold() {
        assert!(matches!(
            "Containment".parse::<Aggregator>(),
            Ok(Aggregator::Containment)
        ));
        assert!(matches!(
            "matched:0.9".parse::<Aggregator>(),
            Ok(Aggregator::Matched(t)) if t == 0.9
        ));
        assert!(matches!(
            "MATCHED:1".parse::<Aggregator>(),
            Ok(Aggregator::Matched(t)) if t == 1.0
        ));
        for refused in [
            "matched",
            "matched:",
            "matched:0",
            "matched:1.5",
            "matched:-0.2",
            "matched:x",
            "matched:NaN",
        ] {
            let err = refused.parse::<Aggregator>().unwrap_err().to_string();
            assert!(err.contains("threshold"), "{refused}: {err}");
        }
    }

    #[test]
    fn pairs_on_empty_targets_yield_nothing() {
        let empty: Vec<i32> = vec![];
        for strategy in [
            PairingStrategy::FirstVsOthers,
            PairingStrategy::LastVsOthers,
            PairingStrategy::All,
            PairingStrategy::AllAndSelf,
            PairingStrategy::Adjacent,
            PairingStrategy::SelfCoverage,
        ] {
            assert_eq!(strategy.pairs(&empty).count(), 0, "strategy: {strategy:?}");
        }
    }

    #[test]
    fn pairs_match_compare_count() {
        let targets = vec![1, 2, 3, 4];
        for strategy in [
            PairingStrategy::All,
            PairingStrategy::AllAndSelf,
            PairingStrategy::Adjacent,
            PairingStrategy::SelfCoverage,
            PairingStrategy::FirstVsOthers,
            PairingStrategy::LastVsOthers,
        ] {
            assert_eq!(
                strategy.pairs(&targets).count(),
                strategy.compare_count(&targets),
                "strategy: {strategy:?}"
            );
        }
    }

    #[test]
    fn algorithm_display_is_defined_for_every_variant() {
        let cases = [
            (Algorithm::Cosine, "Cosine Similarity"),
            (Algorithm::Dice, "Dice Coefficient"),
            (Algorithm::Euclidean, "Euclidean Distance"),
            (Algorithm::Jaccard, "Jaccard Index"),
            (Algorithm::JensenShannon, "Jensen-Shannon Divergence"),
            (Algorithm::Levenshtein, "Levenshtein Distance"),
            (Algorithm::Lcs, "Longest Common Subsequence"),
            (Algorithm::Simpson, "Simpson's Coefficient"),
            (Algorithm::Tanimoto, "Tanimoto Coefficient"),
            (Algorithm::WeightedJaccard, "Weighted Jaccard Index"),
        ];
        for (algorithm, expected) in cases {
            assert_eq!(algorithm.to_string(), expected);
        }
    }

    fn function(name: &str, data: Data) -> Function {
        Function {
            name: name.to_string(),
            data,
        }
    }

    fn seq(items: &[&str]) -> Data {
        Data::Seq(items.iter().map(ToString::to_string).collect())
    }

    fn set(items: &[&str]) -> Data {
        Data::Set(items.iter().map(ToString::to_string).collect())
    }

    fn freq(items: &[&str]) -> Data {
        Data::Freq(seq2freq(
            &items.iter().map(ToString::to_string).collect::<Vec<_>>(),
        ))
    }

    fn kgram_seq(items: &[&str]) -> Data {
        Data::KgramSeq(
            items
                .iter()
                .map(|s| Kgram::new(vec![s.to_string()]))
                .collect(),
        )
    }

    fn kgram_set(items: &[&str]) -> Data {
        Data::KgramSet(
            items
                .iter()
                .map(|s| Kgram::new(vec![s.to_string()]))
                .collect(),
        )
    }

    fn kgram_freq(items: &[&str]) -> Data {
        Data::KgramFreq(seq2freq(
            &items
                .iter()
                .map(|s| Kgram::new(vec![s.to_string()]))
                .collect::<Vec<_>>(),
        ))
    }

    /// Every set-like comparator must report 1.0 for identical inputs across
    /// each Data representation it claims to support.
    #[test]
    fn set_like_comparators_accept_every_supported_representation() {
        let builders: [fn(&[&str]) -> Data; 6] = [seq, set, freq, kgram_seq, kgram_set, kgram_freq];
        for build in builders {
            let e1 = function("f", build(&["A", "B", "C"]));
            let e2 = function("g", build(&["A", "B", "C"]));
            assert!((Jaccard.compare_functions(&e1, &e2) - 1.0).abs() < 1e-9);
            assert!((Dice.compare_functions(&e1, &e2) - 1.0).abs() < 1e-9);
            assert!((Simpson.compare_functions(&e1, &e2) - 1.0).abs() < 1e-9);
        }
    }

    #[test]
    fn sequence_comparators_accept_seq_representations() {
        let builders: [fn(&[&str]) -> Data; 2] = [seq, kgram_seq];
        for build in builders {
            let e1 = function("f", build(&["A", "B", "C"]));
            let e2 = function("g", build(&["A", "B", "C"]));
            assert!((Levenshtein.compare_functions(&e1, &e2) - 1.0).abs() < 1e-9);
            assert!((Lcs.compare_functions(&e1, &e2) - 1.0).abs() < 1e-9);
        }
    }

    #[test]
    fn frequency_comparators_accept_every_supported_representation() {
        let builders: [fn(&[&str]) -> Data; 4] = [seq, freq, kgram_seq, kgram_freq];
        for build in builders {
            let e1 = function("f", build(&["A", "B", "C"]));
            let e2 = function("g", build(&["A", "B", "C"]));
            assert!((Cosine.compare_functions(&e1, &e2) - 1.0).abs() < 1e-9);
            assert!((WeightedJaccard.compare_functions(&e1, &e2) - 1.0).abs() < 1e-9);
            assert!((Euclidean.compare_functions(&e1, &e2) - 1.0).abs() < 1e-9);
            assert_eq!(JensenShannon.compare_functions(&e1, &e2), 1.0);
            assert_eq!(Tanimoto.compare_functions(&e1, &e2), 1.0);
        }
    }

    /// Builds one function's data from its elements, in one representation.
    type Build = fn(&[&str]) -> Data;

    /// An algorithm, the comparator behind it, and the representations it
    /// accepts.
    type Case = (Algorithm, Box<dyn BirthmarkComparator>, Vec<Build>);

    /// Every algorithm, with its comparator and the representations it
    /// accepts. An exhaustive `match`, so that a new algorithm does not
    /// compile until it says which it takes -- and is then held to the rule
    /// below.
    fn each_algorithm() -> Vec<Case> {
        let sets: Vec<Build> = vec![seq, set, freq, kgram_seq, kgram_set, kgram_freq];
        let seqs: Vec<Build> = vec![seq, kgram_seq];
        let freqs: Vec<Build> = vec![seq, freq, kgram_seq, kgram_freq];
        Algorithm::ALL
            .iter()
            .map(|a| -> Case {
                match a {
                    Algorithm::Cosine => (a.clone(), Box::new(Cosine), freqs.clone()),
                    Algorithm::Dice => (a.clone(), Box::new(Dice), sets.clone()),
                    Algorithm::Euclidean => (a.clone(), Box::new(Euclidean), freqs.clone()),
                    Algorithm::Jaccard => (a.clone(), Box::new(Jaccard), sets.clone()),
                    Algorithm::JensenShannon => (a.clone(), Box::new(JensenShannon), freqs.clone()),
                    Algorithm::Levenshtein => (a.clone(), Box::new(Levenshtein), seqs.clone()),
                    Algorithm::Lcs => (a.clone(), Box::new(Lcs), seqs.clone()),
                    Algorithm::Simpson => (a.clone(), Box::new(Simpson), sets.clone()),
                    Algorithm::Tanimoto => (a.clone(), Box::new(Tanimoto), freqs.clone()),
                    Algorithm::WeightedJaccard => {
                        (a.clone(), Box::new(WeightedJaccard), freqs.clone())
                    }
                }
            })
            .collect()
    }

    /// The rule the birthmark-level comparison follows, for every algorithm
    /// and every representation it takes: two empty functions agree (1.0), and
    /// an empty one against one that is not does not (0.0), in either order.
    #[test]
    fn every_algorithm_scores_empty_functions_by_the_same_rule() {
        for (algorithm, comparator, builders) in each_algorithm() {
            for build in builders {
                let empty = function("e", build(&[]));
                let other = function("o", build(&[]));
                let full = function("f", build(&["A", "B"]));
                let label = format!("{algorithm:?} over {:?}", build(&["A"]));
                assert_eq!(
                    comparator.compare_functions(&empty, &other),
                    1.0,
                    "{label}: both empty"
                );
                assert_eq!(
                    comparator.compare_functions(&empty, &full),
                    0.0,
                    "{label}: empty against not"
                );
                assert_eq!(
                    comparator.compare_functions(&full, &empty),
                    0.0,
                    "{label}: not against empty"
                );
            }
        }
    }

    /// A representation that cannot be brought to the algorithm's shape
    /// scores 0.0 rather than panicking: a set has lost the order a sequence
    /// algorithm needs and the counts a frequency algorithm needs.
    #[test]
    fn comparators_return_zero_for_unsupported_representations() {
        let st = function("g", set(&["A"]));
        let fr = function("h", freq(&["A"]));
        assert_eq!(Cosine.compare_functions(&st, &st), 0.0);
        assert_eq!(Euclidean.compare_functions(&st, &st), 0.0);
        assert_eq!(WeightedJaccard.compare_functions(&st, &st), 0.0);
        assert_eq!(JensenShannon.compare_functions(&st, &st), 0.0);
        assert_eq!(Tanimoto.compare_functions(&st, &st), 0.0);
        for unordered in [&st, &fr] {
            assert_eq!(Levenshtein.compare_functions(unordered, unordered), 0.0);
            assert_eq!(Lcs.compare_functions(unordered, unordered), 0.0);
        }
    }

    /// Converting each function once before the matrix is built scores every
    /// cell exactly as converting both functions for that cell would.
    #[test]
    fn a_comparison_scores_every_cell_as_the_two_functions_alone_score() {
        let elements: [&[&str]; 5] = [
            &["A", "B", "A", "C"],
            &["B", "C"],
            &[],
            &["A", "A", "A"],
            &["C", "B", "A", "D"],
        ];
        for (algorithm, comparator, builders) in each_algorithm() {
            for build in builders {
                let functions = |range: std::ops::Range<usize>| -> Vec<Function> {
                    range
                        .map(|i| function(&format!("f{i}"), build(elements[i % 5])))
                        .collect()
                };
                let mut b1 = birthmark("a", &[]);
                let mut b2 = birthmark("b", &[]);
                b1.functions = functions(0..4);
                b2.functions = functions(1..6);
                let c = comparator
                    .compare_birthmarks(&b1, &b2, &Aggregator::Hungarian, &|_: Progress| {})
                    .unwrap();
                for (i, f1) in b1.functions.iter().enumerate() {
                    for (j, f2) in b2.functions.iter().enumerate() {
                        assert_eq!(
                            c.matrix()[[i, j]].to_bits(),
                            comparator.compare_functions(f1, f2).to_bits(),
                            "{algorithm:?} over {:?}: [{i}, {j}]",
                            build(&["A"])
                        );
                    }
                }
            }
        }
    }

    fn birthmark(name: &str, funcs: &[(&str, &[&str])]) -> Birthmark {
        Birthmark {
            metadata: Metadata {
                file_name: name.to_string(),
                path: PathBuf::from(format!("/tmp/{name}")),
                extracted_at: chrono::Utc::now(),
                duration: std::time::Duration::from_nanos(1),
                birthmark_type: BirthmarkType::OpSeq,
                ir: crate::lift::Ir::GhidraPcode,
            },
            functions: funcs.iter().map(|(n, ops)| function(n, seq(ops))).collect(),
            json_path: None,
        }
    }

    #[test]
    fn compare_birthmarks_rejects_mismatched_types() {
        let b1 = birthmark("a", &[("main", &["A"])]);
        let mut b2 = birthmark("b", &[("main", &["A"])]);
        b2.metadata.birthmark_type = BirthmarkType::OpSet;
        match Jaccard.compare_birthmarks(&b1, &b2, &Aggregator::Hungarian, &|_: Progress| {}) {
            Err(Error::Mismatch(..)) => {}
            Err(e) => panic!("unexpected error: {e}"),
            Ok(_) => panic!("expected a mismatch error"),
        }
    }

    #[test]
    fn compare_birthmarks_handles_empty_operands() {
        let empty = birthmark("empty", &[]);
        let filled = birthmark("filled", &[("main", &["A"])]);

        // both empty means identical
        let both = Jaccard
            .compare_birthmarks(&empty, &empty, &Aggregator::Hungarian, &|_: Progress| {})
            .unwrap();
        assert_eq!(both.similarity(), 1.0);

        // exactly one empty means nothing in common
        let one = Jaccard
            .compare_birthmarks(&empty, &filled, &Aggregator::Hungarian, &|_: Progress| {})
            .unwrap();
        assert_eq!(one.similarity(), 0.0);
        let other = Jaccard
            .compare_birthmarks(&filled, &empty, &Aggregator::Hungarian, &|_: Progress| {})
            .unwrap();
        assert_eq!(other.similarity(), 0.0);
    }

    #[test]
    fn compare_birthmarks_of_identical_inputs_scores_one() {
        let b = birthmark("a", &[("f", &["A", "B"]), ("g", &["C"])]);
        for aggregator in [
            Aggregator::Hungarian,
            Aggregator::TopN(Size::All),
            Aggregator::TopN(Size::Num(1)),
        ] {
            let c = Jaccard
                .compare_birthmarks(&b, &b, &aggregator, &|_: Progress| {})
                .unwrap();
            assert!(
                (c.similarity() - 1.0).abs() < 1e-9,
                "aggregator: {aggregator:?}"
            );
        }
    }

    #[test]
    fn top_n_selection_limits_the_number_of_scores() {
        let matrix =
            Matrix::new((3, 3), vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]).unwrap();
        // Size::Num(k) keeps k row maxima plus k column maxima
        let picked = top_n_selection(&matrix, &Size::Num(2));
        assert_eq!(picked.len(), 4);
        assert!(picked.iter().all(|v| (*v - 1.0).abs() < 1e-9));
        // Size::All keeps every row and column maximum
        let all = top_n_selection(&matrix, &Size::All);
        assert_eq!(all.len(), 6);
    }

    /// A matrix that is not square scores as it would padded with zeros to a
    /// square: the shorter side's missing functions share nothing.
    #[test]
    fn a_matrix_that_is_not_square_scores_as_if_padded_with_zeros() {
        let cells = vec![0.2, 0.9, 0.4, 0.7, 0.1, 0.3];
        let wide = Matrix::new((2, 3), cells.clone()).unwrap();
        let mut padded = cells;
        padded.extend([0.0; 3]);
        let square = Matrix::new((3, 3), padded).unwrap();
        let mean = |v: Vec<f64>| v.iter().sum::<f64>() / v.len() as f64;
        for aggregator in [
            Aggregator::Hungarian,
            Aggregator::TopN(Size::All),
            Aggregator::TopN(Size::Num(1)),
            Aggregator::TopN(Size::Num(3)),
            Aggregator::Matched(0.5),
        ] {
            let (got, want) = (
                aggregator.aggregate(&wide).unwrap(),
                aggregator.aggregate(&square).unwrap(),
            );
            assert_eq!(got.len(), want.len(), "{aggregator:?}");
            assert!((mean(got) - mean(want)).abs() < 1e-12, "{aggregator:?}");
        }
        // the best pairing is 0.9 + 0.7, and the third column goes unpaired
        let paired = Aggregator::Hungarian.aggregate(&wide).unwrap();
        assert!((mean(paired) - 1.6 / 3.0).abs() < 1e-12);
    }

    /// The worked example in the documentation: A₁ and A₂ against B₁ to B₃.
    #[test]
    fn containment_and_matched_score_the_documented_example() {
        let m = Matrix::new((2, 3), vec![0.9, 0.2, 0.1, 0.3, 0.8, 0.0]).unwrap();
        let mean = |v: Vec<f64>| v.iter().sum::<f64>() / v.len() as f64;
        let score = |a: Aggregator| mean(a.aggregate(&m).unwrap());
        assert!((score(Aggregator::Hungarian) - 1.7 / 3.0).abs() < 1e-12);
        // the same pairing, over A's two functions rather than B's three
        assert!((score(Aggregator::Containment) - 0.85).abs() < 1e-12);
        // 0.9 and 0.8 are matched; B₃ is not
        assert!((score(Aggregator::Matched(0.8)) - 2.0 / 3.0).abs() < 1e-12);
        assert!((score(Aggregator::Matched(0.85)) - 1.0 / 3.0).abs() < 1e-12);
        assert_eq!(score(Aggregator::Matched(0.95)), 0.0);
    }

    /// A birthmark copied whole into a larger one is contained in it, which
    /// `containment` says and `hungarian` does not.
    #[test]
    fn a_birthmark_inside_a_larger_one_is_contained() {
        let small = birthmark("s", &[("f", &["A", "B"]), ("g", &["C"])]);
        let large = birthmark(
            "l",
            &[
                ("f", &["A", "B"]),
                ("x", &["X", "Y"]),
                ("g", &["C"]),
                ("y", &["Z"]),
            ],
        );
        let comparator = crate::Oinkie::new()
            .unwrap()
            .comparator(&Algorithm::Jaccard);
        let score = |a: Aggregator| {
            comparator
                .compare_birthmarks(&small, &large, &a)
                .unwrap()
                .similarity()
        };
        assert_eq!(score(Aggregator::Containment), 1.0);
        assert_eq!(score(Aggregator::Hungarian), 0.5);
        assert_eq!(score(Aggregator::Matched(1.0)), 0.5);
    }

    /// The empty-birthmark rule holds for both: two empty birthmarks agree,
    /// and an empty one against one that is not does not.
    #[test]
    fn containment_and_matched_keep_the_rule_for_empty_birthmarks() {
        let empty = birthmark("e", &[]);
        let filled = birthmark("f", &[("f", &["A"])]);
        let comparator = crate::Oinkie::new()
            .unwrap()
            .comparator(&Algorithm::Jaccard);
        for aggregator in [Aggregator::Containment, Aggregator::Matched(0.5)] {
            let score = |b1, b2| {
                comparator
                    .compare_birthmarks(b1, b2, &aggregator)
                    .unwrap()
                    .similarity()
            };
            assert_eq!(score(&empty, &empty), 1.0, "{aggregator:?}");
            assert_eq!(score(&empty, &filled), 0.0, "{aggregator:?}");
            assert_eq!(score(&filled, &empty), 0.0, "{aggregator:?}");
        }
    }

    #[test]
    fn a_matrix_holds_only_finite_cells_and_as_many_as_its_shape() {
        assert!(Matrix::new((1, 2), vec![0.5, f64::NAN]).is_none());
        assert!(Matrix::new((1, 2), vec![0.5, f64::INFINITY]).is_none());
        assert!(Matrix::new((2, 2), vec![0.5; 3]).is_none());
        let m = Matrix::new((2, 3), vec![0.0, 0.1, 0.2, 1.0, 1.1, 1.2]).unwrap();
        assert_eq!(m.dim(), (2, 3));
        assert_eq!(m[[1, 2]], 1.2);
        assert_eq!(m.row(1), &[1.0, 1.1, 1.2]);
    }

    #[test]
    fn comparison_similarity_is_zero_when_no_scores_were_aggregated() {
        let b = birthmark("a", &[("f", &["A"])]);
        let c = Comparison::new(
            &b,
            &b,
            Matrix::zeros((0, 0)),
            vec![],
            std::time::Duration::from_nanos(0),
        );
        assert_eq!(
            c.similarity(),
            0.0,
            "an empty aggregation must not produce NaN"
        );
        assert_eq!(c.duration(), std::time::Duration::from_nanos(0));
    }

    /// The number of threads changes when a cell is computed, never what it
    /// holds.
    #[test]
    fn a_comparison_is_the_same_on_any_number_of_threads() {
        let funcs: Vec<(String, Vec<&str>)> = (0..40)
            .map(|i| {
                let ops = ["A", "B", "C", "D", "E"];
                let f = (0..(i % 7)).map(|k| ops[(i * 3 + k) % 5]).collect();
                (format!("f{i}"), f)
            })
            .collect();
        let as_slices: Vec<(&str, &[&str])> = funcs
            .iter()
            .map(|(n, f)| (n.as_str(), f.as_slice()))
            .collect();
        let (b1, b2) = (
            birthmark("a", &as_slices[..25]),
            birthmark("b", &as_slices[10..]),
        );
        let on = |n: usize| {
            crate::Oinkie::builder()
                .threads(std::num::NonZeroUsize::new(n).unwrap())
                .build()
                .unwrap()
        };
        for algorithm in Algorithm::ALL {
            let (one, four) = (
                on(1)
                    .comparator(algorithm)
                    .compare_birthmarks(&b1, &b2, &Aggregator::Hungarian),
                on(4)
                    .comparator(algorithm)
                    .compare_birthmarks(&b1, &b2, &Aggregator::Hungarian),
            );
            let (one, four) = (one.unwrap(), four.unwrap());
            assert_eq!(one.matrix(), four.matrix(), "{algorithm}");
            assert_eq!(
                one.similarity().to_bits(),
                four.similarity().to_bits(),
                "{algorithm}"
            );
        }
    }

    /// An observer hears the matrix start with its shape, one row filled per
    /// row, and then the aggregation, in that order.
    #[test]
    fn an_observer_is_told_each_row_between_the_start_and_the_aggregation() {
        let b1 = birthmark("a", &[("f", &["A"]), ("g", &["B"]), ("h", &["C"])]);
        let b2 = birthmark("b", &[("f", &["A"]), ("g", &["B"])]);
        let heard = std::sync::Mutex::new(Vec::new());
        crate::Oinkie::new()
            .unwrap()
            .comparator(&Algorithm::Jaccard)
            .compare_birthmarks_with(&b1, &b2, &Aggregator::Hungarian, &|p| {
                heard.lock().unwrap().push(p)
            })
            .unwrap();
        assert_eq!(
            heard.into_inner().unwrap(),
            [
                Progress::MatrixStarted {
                    rows: 3,
                    columns: 2
                },
                Progress::RowFilled,
                Progress::RowFilled,
                Progress::RowFilled,
                Progress::Aggregating,
            ]
        );
    }

    /// Watching a comparison does not change it.
    #[test]
    fn a_comparison_observed_scores_as_one_not_observed() {
        let b1 = birthmark("a", &[("f", &["A", "B"]), ("g", &["B", "C"])]);
        let b2 = birthmark("b", &[("f", &["A", "C"]), ("g", &["B"]), ("h", &[])]);
        let oinkie = crate::Oinkie::new().unwrap();
        for algorithm in Algorithm::ALL {
            let comparator = oinkie.comparator(algorithm);
            let plain = comparator
                .compare_birthmarks(&b1, &b2, &Aggregator::Hungarian)
                .unwrap();
            let observed = comparator
                .compare_birthmarks_with(&b1, &b2, &Aggregator::Hungarian, &|_| {})
                .unwrap();
            assert_eq!(plain.matrix(), observed.matrix(), "{algorithm}");
            assert_eq!(
                plain.similarity().to_bits(),
                observed.similarity().to_bits(),
                "{algorithm}"
            );
        }
    }

    /// A pair with an empty side has no matrix to fill, so there is nothing
    /// to report.
    #[test]
    fn a_pair_with_an_empty_side_reports_nothing() {
        let empty = birthmark("a", &[]);
        let filled = birthmark("b", &[("f", &["A"])]);
        let comparator = crate::Oinkie::new()
            .unwrap()
            .comparator(&Algorithm::Jaccard);
        for (b1, b2) in [(&empty, &empty), (&empty, &filled), (&filled, &empty)] {
            comparator
                .compare_birthmarks_with(b1, b2, &Aggregator::Hungarian, &|p| {
                    panic!("reported {p:?}")
                })
                .unwrap();
        }
    }

    #[test]
    fn comparator_dispatches_every_algorithm() {
        let b = birthmark("a", &[("f", &["A", "B"])]);
        for algorithm in Algorithm::ALL {
            let comparator = crate::Oinkie::new().unwrap().comparator(algorithm);
            let c = comparator
                .compare_birthmarks(&b, &b, &Aggregator::Hungarian)
                .unwrap_or_else(|e| panic!("{algorithm}: {e}"));
            assert!(
                (c.similarity() - 1.0).abs() < 1e-9,
                "algorithm: {algorithm}"
            );
        }
    }

    // The frequency algorithms as they sum over every element of either
    // function, which the sums over the shared elements must agree with bit
    // for bit.

    fn cosine_similarity_by_union<T: std::cmp::Eq + std::hash::Hash>(
        f1: &FxHashMap<T, usize>,
        f2: &FxHashMap<T, usize>,
    ) -> f64 {
        let keys = f1.keys().chain(f2.keys()).collect::<FxHashSet<_>>();
        let dot_product = keys
            .iter()
            .map(|k| {
                let v1 = *f1.get(*k).unwrap_or(&0) as f64;
                let v2 = *f2.get(*k).unwrap_or(&0) as f64;
                v1 * v2
            })
            .sum::<f64>();
        let magnitude1 = f1.values().map(|v| (*v as f64).powi(2)).sum::<f64>().sqrt();
        let magnitude2 = f2.values().map(|v| (*v as f64).powi(2)).sum::<f64>().sqrt();
        // A zero magnitude is an empty function. Two of them agree, as under every
        // other algorithm; one against a function that is not empty does not, or
        // an empty function would match anything.
        match (magnitude1 > 0.0, magnitude2 > 0.0) {
            (true, true) => dot_product / (magnitude1 * magnitude2),
            (false, false) => 1.0,
            _ => 0.0,
        }
    }

    fn euclidean_distance_by_union<T: std::cmp::Eq + std::hash::Hash>(
        f1: &FxHashMap<T, usize>,
        f2: &FxHashMap<T, usize>,
    ) -> f64 {
        let keys = f1.keys().chain(f2.keys()).collect::<FxHashSet<_>>();
        let sum_of_squares = keys
            .iter()
            .map(|k| {
                let v1 = *f1.get(*k).unwrap_or(&0) as f64;
                let v2 = *f2.get(*k).unwrap_or(&0) as f64;
                (v1 - v2).powi(2)
            })
            .sum::<f64>();
        let scale = f1.values().map(|v| (*v as f64).powi(2)).sum::<f64>().sqrt()
            + f2.values().map(|v| (*v as f64).powi(2)).sum::<f64>().sqrt();
        // Zero only when both functions are empty, where the quotient below would
        // be 0/0: a NaN that would reach the assignment and the pair's
        // similarity. Two empty functions agree, as under every other algorithm.
        if scale == 0.0 {
            return 1.0;
        }
        1.0 - (sum_of_squares.sqrt() / scale)
    }

    fn weighted_jaccard_by_union<T: std::cmp::Eq + std::hash::Hash>(
        f1: &FxHashMap<T, usize>,
        f2: &FxHashMap<T, usize>,
    ) -> f64 {
        let keys = f1.keys().chain(f2.keys()).collect::<FxHashSet<_>>();
        let min_sum = keys
            .iter()
            .map(|k| {
                let v1 = *f1.get(*k).unwrap_or(&0) as f64;
                let v2 = *f2.get(*k).unwrap_or(&0) as f64;
                v1.min(v2)
            })
            .sum::<f64>();
        let max_sum = keys
            .iter()
            .map(|k| {
                let v1 = *f1.get(*k).unwrap_or(&0) as f64;
                let v2 = *f2.get(*k).unwrap_or(&0) as f64;
                v1.max(v2)
            })
            .sum::<f64>();
        if max_sum > 0.0 {
            min_sum / max_sum
        } else {
            1.0
        }
    }

    fn tanimoto_by_union<T: std::cmp::Eq + std::hash::Hash>(
        f1: &FxHashMap<T, usize>,
        f2: &FxHashMap<T, usize>,
    ) -> f64 {
        let keys = f1.keys().chain(f2.keys()).collect::<FxHashSet<_>>();
        let (mut dot, mut squares1, mut squares2) = (0.0, 0.0, 0.0);
        for k in &keys {
            let v1 = *f1.get(*k).unwrap_or(&0) as f64;
            let v2 = *f2.get(*k).unwrap_or(&0) as f64;
            dot += v1 * v2;
            squares1 += v1 * v1;
            squares2 += v2 * v2;
        }
        let denominator = squares1 + squares2 - dot;
        if denominator == 0.0 {
            1.0
        } else {
            dot / denominator
        }
    }

    /// The textbook definition: the mean of each distribution's
    /// Kullback–Leibler divergence from their mixture, over every element.
    fn jensen_shannon_by_union<T: std::cmp::Eq + std::hash::Hash>(
        f1: &FxHashMap<T, usize>,
        f2: &FxHashMap<T, usize>,
    ) -> f64 {
        let total1 = f1.values().sum::<usize>() as f64;
        let total2 = f2.values().sum::<usize>() as f64;
        if total1 == 0.0 || total2 == 0.0 {
            return if total1 == total2 { 1.0 } else { 0.0 };
        }
        let keys = f1.keys().chain(f2.keys()).collect::<FxHashSet<_>>();
        let kl = |p: f64, m: f64| if p > 0.0 { p * (p / m).log2() } else { 0.0 };
        let divergence = keys
            .iter()
            .map(|k| {
                let p = *f1.get(*k).unwrap_or(&0) as f64 / total1;
                let q = *f2.get(*k).unwrap_or(&0) as f64 / total2;
                let m = (p + q) / 2.0;
                (kl(p, m) + kl(q, m)) / 2.0
            })
            .sum::<f64>();
        1.0 - divergence.max(0.0).sqrt()
    }

    /// Every pair of a set of frequency maps -- with repeated counts, counts
    /// in one only, disjoint maps, a large count and the empty map -- scores
    /// the same bits summed over the shared elements as over all of them.
    #[test]
    fn frequency_algorithms_agree_with_their_sums_over_every_element() {
        let maps: Vec<FxHashMap<&str, usize>> = vec![
            FxHashMap::default(),
            [("A", 1)].into_iter().collect(),
            [("A", 3), ("B", 1), ("C", 2)].into_iter().collect(),
            [("B", 5), ("C", 2), ("D", 7)].into_iter().collect(),
            [("E", 1), ("F", 1)].into_iter().collect(),
            [("A", 1_000_000), ("B", 999_999), ("Z", 3)]
                .into_iter()
                .collect(),
            (0..200)
                .map(|i| (["A", "B", "C", "D", "E", "F", "G"][i % 7], i))
                .collect(),
        ];
        // and maps drawn at random over a small vocabulary, so that most
        // pairs share some elements and not others
        let mut state = 0x9e37_79b9_7f4a_7c15u64;
        let mut next = move || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state
        };
        let vocabulary = ["A", "B", "C", "D", "E", "F", "G", "H", "I", "J"];
        let mut maps = maps;
        for _ in 0..60 {
            let mut map = FxHashMap::default();
            for &k in &vocabulary {
                if next() % 2 == 0 {
                    map.insert(k, (next() % 50) as usize + 1);
                }
            }
            maps.push(map);
        }
        for f1 in &maps {
            for f2 in &maps {
                assert_eq!(
                    cosine_similarity(f1, f2).to_bits(),
                    cosine_similarity_by_union(f1, f2).to_bits(),
                    "cosine {f1:?} {f2:?}"
                );
                assert_eq!(
                    euclidean_distance(f1, f2).to_bits(),
                    euclidean_distance_by_union(f1, f2).to_bits(),
                    "euclidean {f1:?} {f2:?}"
                );
                assert_eq!(
                    weighted_jaccard(f1, f2).to_bits(),
                    weighted_jaccard_by_union(f1, f2).to_bits(),
                    "weighted jaccard {f1:?} {f2:?}"
                );
                // Over whole numbers, as the three above.
                assert_eq!(
                    tanimoto(f1, f2).to_bits(),
                    tanimoto_by_union(f1, f2).to_bits(),
                    "tanimoto {f1:?} {f2:?}"
                );
                // Over logarithms, which the rearrangement rounds
                // differently: close, not equal. The root magnifies a
                // difference near a divergence of zero, hence the margin.
                let (js, by_union) = (jensen_shannon(f1, f2), jensen_shannon_by_union(f1, f2));
                assert!(
                    (js - by_union).abs() < 1e-7,
                    "jensen-shannon {js} against {by_union}: {f1:?} {f2:?}"
                );
            }
        }
    }

    /// The textbook dynamic programme over the whole table, which the
    /// two-row versions must agree with.
    fn levenshtein_distance_full_memory<T: PartialEq>(s1: &[T], s2: &[T]) -> f64 {
        let mut dp = vec![vec![0usize; s2.len() + 1]; s1.len() + 1];
        for (i, row) in dp.iter_mut().enumerate() {
            row[0] = i;
        }
        for (j, cell) in dp[0].iter_mut().enumerate() {
            *cell = j;
        }
        for i in 1..=s1.len() {
            for j in 1..=s2.len() {
                let cost = if s1[i - 1] == s2[j - 1] { 0 } else { 1 };
                let substitution = dp[i - 1][j - 1] + cost;
                let insertion = dp[i][j - 1] + 1;
                let deletion = dp[i - 1][j] + 1;
                dp[i][j] = substitution.min(insertion).min(deletion);
            }
        }
        1.0 - (dp[s1.len()][s2.len()] as f64 / (s1.len().max(s2.len()) as f64))
    }

    fn longest_common_subsequence_full_memory<T: PartialEq>(s1: &[T], s2: &[T]) -> f64 {
        let mut dp = vec![vec![0usize; s2.len() + 1]; s1.len() + 1];
        for i in 1..=s1.len() {
            for j in 1..=s2.len() {
                dp[i][j] = if s1[i - 1] == s2[j - 1] {
                    dp[i - 1][j - 1] + 1
                } else {
                    dp[i - 1][j].max(dp[i][j - 1])
                };
            }
        }
        2.0 * dp[s1.len()][s2.len()] as f64 / (s1.len() + s2.len()) as f64
    }

    #[test]
    fn similarity_functions_agree_with_their_full_memory_variants() {
        let a = ["A", "B", "C", "D"];
        let b = ["A", "C", "D", "E"];
        assert!(
            (longest_common_subsequence(&a, &b) - longest_common_subsequence_full_memory(&a, &b))
                .abs()
                < 1e-9
        );
        assert!(
            (levenshtein_distance(&a, &b) - levenshtein_distance_full_memory(&a, &b)).abs() < 1e-9
        );
        // the row/column swap for the shorter sequence must not change the score
        let short = ["A", "B"];
        assert!(
            (longest_common_subsequence(&a, &short) - longest_common_subsequence(&short, &a)).abs()
                < 1e-9
        );
        assert!((levenshtein_distance(&a, &short) - levenshtein_distance(&short, &a)).abs() < 1e-9);
    }

    #[test]
    fn similarity_functions_handle_empty_sequences() {
        let empty: [&str; 0] = [];
        let one = ["A"];
        assert_eq!(levenshtein_distance(&empty, &empty), 1.0);
        assert_eq!(levenshtein_distance(&empty, &one), 0.0);
        assert_eq!(levenshtein_distance(&one, &empty), 0.0);
        assert_eq!(longest_common_subsequence(&empty, &one), 0.0);
        assert_eq!(longest_common_subsequence(&one, &empty), 0.0);
    }

    #[test]
    fn set_indices_handle_empty_inputs() {
        let empty: FxHashSet<&str> = FxHashSet::default();
        let one: FxHashSet<&str> = ["A"].into_iter().collect();
        for f in [jaccard_index, dice_index, simpson_index] {
            assert_eq!(f(&empty, &empty), 1.0);
            assert_eq!(f(&empty, &one), 0.0);
            assert_eq!(f(&one, &empty), 0.0);
        }
    }

    #[test]
    fn frequency_metrics_handle_empty_inputs() {
        let empty: FxHashMap<&str, usize> = FxHashMap::default();
        // an empty vector has no direction, so both are treated as identical
        assert_eq!(cosine_similarity(&empty, &empty), 1.0);
        assert_eq!(weighted_jaccard(&empty, &empty), 1.0);
        assert_eq!(tanimoto(&empty, &empty), 1.0);
        assert_eq!(jensen_shannon(&empty, &empty), 1.0);
    }

    /// Identical distributions score exactly 1.0 and disjoint ones exactly
    /// 0.0, whatever the functions' sizes: the divergence is of proportions.
    #[test]
    fn jensen_shannon_compares_proportions() {
        let small: FxHashMap<&str, usize> = [("A", 1), ("B", 3)].into_iter().collect();
        let large: FxHashMap<&str, usize> = [("A", 250), ("B", 750)].into_iter().collect();
        let other: FxHashMap<&str, usize> = [("C", 5)].into_iter().collect();
        assert_eq!(jensen_shannon(&small, &small), 1.0);
        assert_eq!(jensen_shannon(&small, &large), 1.0, "same proportions");
        assert_eq!(jensen_shannon(&small, &other), 0.0, "nothing shared");
        assert!(jensen_shannon(&small, &large) > jensen_shannon(&small, &other));
        // The counts of `small` and `large` differ, so the others do not
        // score them as identical.
        assert!(tanimoto(&small, &large) < 1.0);
        assert!(weighted_jaccard(&small, &large) < 1.0);
    }

    /// On counts of one each -- a set -- Tanimoto is Jaccard's index, which
    /// is why it is not offered on sets.
    #[test]
    fn tanimoto_on_ones_is_jaccard() {
        let f1: FxHashMap<&str, usize> = [("A", 1), ("B", 1), ("C", 1)].into_iter().collect();
        let f2: FxHashMap<&str, usize> = [("B", 1), ("C", 1), ("D", 1)].into_iter().collect();
        let s1 = freq2set(&f1);
        let s2 = freq2set(&f2);
        assert_eq!(tanimoto(&f1, &f2), jaccard_index(&s1, &s2));
    }
}
