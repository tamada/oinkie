//! Comparing birthmarks.
//!
//! An [`Algorithm`] gives a [`Comparator`], which compares every function of
//! one [`Birthmark`] with every function of another into a matrix of function
//! similarities. An [`Aggregator`] turns the matrix into the pair's one
//! similarity, and the [`Comparison`] holds both. A [`PairingStrategy`]
//! chooses which pairs of a list to compare.
//!
//! ```
//! use std::path::Path;
//!
//! use oinkie::Program;
//! use oinkie::birthmarks::BirthmarkType;
//! use oinkie::compare::{Aggregator, Algorithm};
//! use oinkie::extract::Extractor;
//!
//! # fn main() -> oinkie::Result<()> {
//! let extractor = Extractor::new(BirthmarkType::OpSet);
//! let a = extractor.extract(&Program::load(Path::new("testdata/lifted/pcodes/hello_clang.json"))?)?;
//!
//! let comparison = Algorithm::Jaccard
//!     .comparator()
//!     .compare_birthmarks(&a, &a, &Aggregator::Hungarian)?;
//! assert_eq!(comparison.similarity(), 1.0, "a birthmark is identical to itself");
//! # Ok(())
//! # }
//! ```

use crate::Iterable;
use crate::birthmarks::{Birthmark, Data, Function, Shape};
use crate::{Error, Result};
use itertools::Itertools;
use ndarray::Array2;
use rustc_hash::{FxHashMap, FxHashSet};
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

/// Two birthmarks compared: their function similarity matrix, the pair's one
/// similarity, and how long it took.
pub struct Comparison<'a, S> {
    columns: &'a S,
    rows: &'a S,
    matrix: Array2<f64>,
    duration: std::time::Duration,
    similarities: Vec<f64>,
}

impl<'a, S> Comparison<'a, S> {
    pub(crate) fn new(
        columns: &'a S,
        rows: &'a S,
        matrix: Array2<f64>,
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

    /// The function-to-function scores, indexed `[column, row]`.
    ///
    /// Square, because the assignment that aggregates it needs it to be: as
    /// many rows and columns as the longer side has functions, with the cells
    /// past the shorter side left at 0.0. Empty when either side has no
    /// functions. The scores proper are the first `columns` x `rows` cells.
    pub fn matrix(&self) -> &Array2<f64> {
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
/// Read by `from_str` from `hungarian`, `topn:N`, `topn:all` or `topn`, in any
/// case.
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
        } else {
            Err(Error::Parse(format!("{s}: Invalid aggregator")))
        }
    }
}

impl Aggregator {
    /// The similarities this aggregator keeps of `array`, a square matrix of
    /// function similarities, which [`Comparison::similarity`] averages.
    pub fn aggregate(&self, array: &Array2<f64>) -> Result<Vec<f64>> {
        match self {
            Aggregator::Hungarian => hungarian_algorithm(array).map(|(sim, _matches)| sim),
            Aggregator::TopN(n) => top_n_selection(array, n),
        }
    }
}

trait BirthmarkComparator {
    fn compare_birthmarks<'a>(
        &self,
        b1: &'a Birthmark,
        b2: &'a Birthmark,
        aggregator: &Aggregator,
    ) -> Result<Comparison<'a, Birthmark>> {
        // Asking the birthmark rather than re-deriving the conditions here
        // keeps the reason in the error: a mismatch of representation reads
        // differently from a mismatch of type.
        b1.check_comparable_with(b2)?;
        let p1_len = b1.functions.len();
        let p2_len = b2.functions.len();
        let size = std::cmp::max(p1_len, p2_len);
        if p1_len == 0 && p2_len == 0 {
            Ok(Comparison::new(
                b1,
                b2,
                Array2::<f64>::zeros((0, 0)),
                vec![1.0],
                std::time::Duration::from_millis(0),
            ))
        } else if p1_len == 0 || p2_len == 0 {
            Ok(Comparison::new(
                b1,
                b2,
                Array2::<f64>::zeros((0, 0)),
                vec![0.0],
                std::time::Duration::from_millis(0),
            ))
        } else {
            let start = Instant::now();
            let r = build_matrix(b1, b2, size, |e1, e2| self.compare_functions(e1, e2))?;
            aggregator
                .aggregate(&r)
                .map(|sim| Comparison::new(b1, b2, r, sim, start.elapsed()))
        }
    }

    fn compare_functions(&self, e1: &Function, e2: &Function) -> f64;
}

fn build_matrix<F, T>(
    p1: impl Iterable<Item = T>,
    p2: impl Iterable<Item = T>,
    size: usize,
    compare_func: F,
) -> Result<Array2<f64>>
where
    F: Fn(&T, &T) -> f64,
{
    // The matrix holds similarities; the conversion to costs for lapjv
    // happens later in hungarian_algorithm. Cells beyond the shorter input
    // stay 0.0 so that the matrix is always square (size x size).
    let mut similarities = vec![0.0; size * size];
    for (i, item1) in p1.iter().enumerate() {
        for (j, item2) in p2.iter().enumerate() {
            similarities[i * size + j] = compare_func(item1, item2);
        }
    }
    Array2::from_shape_vec((size, size), similarities).map_err(Error::ShapeError)
}

fn top_n_selection(array2d: &Array2<f64>, n: &Size) -> Result<Vec<f64>> {
    let rows = array2d
        .axis_iter(ndarray::Axis(0))
        .map(|col| col.fold(0.0f64, |acc, &v| acc.max(v)))
        .sorted_by(|a, b| b.total_cmp(a))
        .collect::<Vec<_>>();
    let cols = array2d
        .axis_iter(ndarray::Axis(1))
        .map(|col| col.fold(0.0f64, |acc, &v| acc.max(v)))
        .sorted_by(|a, b| b.total_cmp(a))
        .collect::<Vec<_>>();
    match n {
        Size::Num(k) => Ok(rows
            .into_iter()
            .take(*k)
            .chain(cols.into_iter().take(*k))
            .collect()),
        Size::All => Ok(rows.into_iter().chain(cols).collect()),
    }
}

fn hungarian_algorithm(similarity_matrix: &Array2<f64>) -> Result<(Vec<f64>, Vec<usize>)> {
    let cost_matrix = 1.0 - similarity_matrix;
    match lapjv::lapjv(&cost_matrix) {
        Ok((rows, _cols)) => {
            let mut similarities = vec![];
            for (i, &j) in rows.iter().enumerate() {
                if i < cost_matrix.nrows() && j < cost_matrix.ncols() {
                    // Convert cost back to similarity by using (1.0 - cost).
                    similarities.push(1.0 - cost_matrix[(i, j)]);
                }
            }
            Ok((similarities, rows))
        }
        Err(e) => Err(Error::LapJV(e)),
    }
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
    /// Levenshtein distance. Available: seq.
    Levenshtein,
    /// Longest Common Subsequence (LCS). Available: seq.
    Lcs,
    /// Simpson's coefficient. Available: seq, set and freq.
    Simpson,
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
            Algorithm::Levenshtein => write!(f, "Levenshtein Distance"),
            Algorithm::Lcs => write!(f, "Longest Common Subsequence"),
            Algorithm::Simpson => write!(f, "Simpson's Coefficient"),
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
        Algorithm::Levenshtein,
        Algorithm::Lcs,
        Algorithm::Simpson,
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
            Algorithm::Lcs => ("lcs", Shape::Seq),
            Algorithm::Levenshtein => ("levenshtein", Shape::Seq),
            Algorithm::Simpson => ("simpson", Shape::Set),
            Algorithm::WeightedJaccard => ("weightedjaccard", Shape::Freq),
        }
    }

    /// The same name with a hyphen between its words.
    ///
    /// It differs from [`Algorithm::name`] for `weightedjaccard` alone;
    /// the other seven are single words and spell the same either way.
    fn hyphenated(&self) -> &'static str {
        match self {
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

    /// The comparator that runs this algorithm.
    pub fn comparator(&self) -> Comparator {
        self.into()
    }
}

/// Compares birthmarks under one [`Algorithm`]. [`Algorithm::comparator`] and
/// [`AnalysisType::comparator`](crate::birthmarks::AnalysisType::comparator)
/// make one.
pub struct Comparator {
    inner: ComparatorImpl,
}

/// The algorithm behind a [`Comparator`], kept out of its public API so that
/// the algorithms can change without changing it.
enum ComparatorImpl {
    Cosine(Cosine),
    Dice(Dice),
    Euclidean(Euclidean),
    Jaccard(Jaccard),
    Levenshtein(Levenshtein),
    Lcs(Lcs),
    Simpson(Simpson),
    WeightedJaccard(WeightedJaccard),
}

impl From<&Algorithm> for Comparator {
    fn from(algorithm: &Algorithm) -> Self {
        match algorithm {
            Algorithm::Cosine => Comparator {
                inner: ComparatorImpl::Cosine(Cosine {}),
            },
            Algorithm::Dice => Comparator {
                inner: ComparatorImpl::Dice(Dice {}),
            },
            Algorithm::Euclidean => Comparator {
                inner: ComparatorImpl::Euclidean(Euclidean {}),
            },
            Algorithm::Jaccard => Comparator {
                inner: ComparatorImpl::Jaccard(Jaccard {}),
            },
            Algorithm::Levenshtein => Comparator {
                inner: ComparatorImpl::Levenshtein(Levenshtein {}),
            },
            Algorithm::Lcs => Comparator {
                inner: ComparatorImpl::Lcs(Lcs {}),
            },
            Algorithm::Simpson => Comparator {
                inner: ComparatorImpl::Simpson(Simpson {}),
            },
            Algorithm::WeightedJaccard => Comparator {
                inner: ComparatorImpl::WeightedJaccard(WeightedJaccard {}),
            },
        }
    }
}

impl Comparator {
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
        match &self.inner {
            ComparatorImpl::Cosine(c) => c.compare_birthmarks(b1, b2, aggregator),
            ComparatorImpl::Dice(d) => d.compare_birthmarks(b1, b2, aggregator),
            ComparatorImpl::Euclidean(e) => e.compare_birthmarks(b1, b2, aggregator),
            ComparatorImpl::Jaccard(j) => j.compare_birthmarks(b1, b2, aggregator),
            ComparatorImpl::Levenshtein(l) => l.compare_birthmarks(b1, b2, aggregator),
            ComparatorImpl::Lcs(lcs) => lcs.compare_birthmarks(b1, b2, aggregator),
            ComparatorImpl::Simpson(s) => s.compare_birthmarks(b1, b2, aggregator),
            ComparatorImpl::WeightedJaccard(wj) => wj.compare_birthmarks(b1, b2, aggregator),
        }
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

impl BirthmarkComparator for Jaccard {
    fn compare_functions(&self, e1: &Function, e2: &Function) -> f64 {
        if let (Data::Set(s1), Data::Set(s2)) = (&e1.data, &e2.data) {
            jaccard_index(s1, s2)
        } else if let (Data::KgramSet(k1), Data::KgramSet(k2)) = (&e1.data, &e2.data) {
            jaccard_index(k1, k2)
        } else if let (Data::Seq(s1), Data::Seq(s2)) = (&e1.data, &e2.data) {
            jaccard_index(&seq2set(s1), &seq2set(s2))
        } else if let (Data::KgramSeq(k1), Data::KgramSeq(k2)) = (&e1.data, &e2.data) {
            jaccard_index(&seq2set(k1), &seq2set(k2))
        } else if let (Data::Freq(s1), Data::Freq(s2)) = (&e1.data, &e2.data) {
            jaccard_index(&freq2set(s1), &freq2set(s2))
        } else if let (Data::KgramFreq(k1), Data::KgramFreq(k2)) = (&e1.data, &e2.data) {
            jaccard_index(&freq2set(k1), &freq2set(k2))
        } else {
            0.0
        }
    }
}

impl BirthmarkComparator for Dice {
    fn compare_functions(&self, e1: &Function, e2: &Function) -> f64 {
        if let (Data::Set(s1), Data::Set(s2)) = (&e1.data, &e2.data) {
            dice_index(s1, s2)
        } else if let (Data::KgramSet(k1), Data::KgramSet(k2)) = (&e1.data, &e2.data) {
            dice_index(k1, k2)
        } else if let (Data::Seq(k1), Data::Seq(k2)) = (&e1.data, &e2.data) {
            dice_index(&seq2set(k1), &seq2set(k2))
        } else if let (Data::KgramSeq(k1), Data::KgramSeq(k2)) = (&e1.data, &e2.data) {
            dice_index(&seq2set(k1), &seq2set(k2))
        } else if let (Data::Freq(k1), Data::Freq(k2)) = (&e1.data, &e2.data) {
            dice_index(&freq2set(k1), &freq2set(k2))
        } else if let (Data::KgramFreq(k1), Data::KgramFreq(k2)) = (&e1.data, &e2.data) {
            dice_index(&freq2set(k1), &freq2set(k2))
        } else {
            0.0
        }
    }
}

impl BirthmarkComparator for Simpson {
    fn compare_functions(&self, e1: &Function, e2: &Function) -> f64 {
        if let (Data::Set(s1), Data::Set(s2)) = (&e1.data, &e2.data) {
            simpson_index(s1, s2)
        } else if let (Data::KgramSet(k1), Data::KgramSet(k2)) = (&e1.data, &e2.data) {
            simpson_index(k1, k2)
        } else if let (Data::Seq(s1), Data::Seq(s2)) = (&e1.data, &e2.data) {
            simpson_index(&seq2set(s1), &seq2set(s2))
        } else if let (Data::KgramSeq(k1), Data::KgramSeq(k2)) = (&e1.data, &e2.data) {
            simpson_index(&seq2set(k1), &seq2set(k2))
        } else if let (Data::Freq(s1), Data::Freq(s2)) = (&e1.data, &e2.data) {
            simpson_index(&freq2set(s1), &freq2set(s2))
        } else if let (Data::KgramFreq(k1), Data::KgramFreq(k2)) = (&e1.data, &e2.data) {
            simpson_index(&freq2set(k1), &freq2set(k2))
        } else {
            0.0
        }
    }
}

impl BirthmarkComparator for Levenshtein {
    fn compare_functions(&self, e1: &Function, e2: &Function) -> f64 {
        if let (Data::Seq(s1), Data::Seq(s2)) = (&e1.data, &e2.data) {
            levenshtein_distance(s1, s2)
        } else if let (Data::KgramSeq(k1), Data::KgramSeq(k2)) = (&e1.data, &e2.data) {
            levenshtein_distance(k1, k2)
        } else {
            0.0
        }
    }
}

impl BirthmarkComparator for Cosine {
    fn compare_functions(&self, e1: &Function, e2: &Function) -> f64 {
        if let (Data::Freq(f1), Data::Freq(f2)) = (&e1.data, &e2.data) {
            cosine_similarity(f1, f2)
        } else if let (Data::KgramFreq(k1), Data::KgramFreq(k2)) = (&e1.data, &e2.data) {
            cosine_similarity(k1, k2)
        } else if let (Data::Seq(s1), Data::Seq(s2)) = (&e1.data, &e2.data) {
            cosine_similarity(&seq2freq(s1), &seq2freq(s2))
        } else if let (Data::KgramSeq(s1), Data::KgramSeq(s2)) = (&e1.data, &e2.data) {
            cosine_similarity(&seq2freq(s1), &seq2freq(s2))
        } else {
            0.0
        }
    }
}

impl BirthmarkComparator for Euclidean {
    fn compare_functions(&self, e1: &Function, e2: &Function) -> f64 {
        if let (Data::Freq(f1), Data::Freq(f2)) = (&e1.data, &e2.data) {
            euclidean_distance(f1, f2)
        } else if let (Data::KgramFreq(k1), Data::KgramFreq(k2)) = (&e1.data, &e2.data) {
            euclidean_distance(k1, k2)
        } else if let (Data::Seq(f1), Data::Seq(f2)) = (&e1.data, &e2.data) {
            euclidean_distance(&seq2freq(f1), &seq2freq(f2))
        } else if let (Data::KgramSeq(k1), Data::KgramSeq(k2)) = (&e1.data, &e2.data) {
            euclidean_distance(&seq2freq(k1), &seq2freq(k2))
        } else {
            0.0
        }
    }
}

impl BirthmarkComparator for WeightedJaccard {
    fn compare_functions(&self, e1: &Function, e2: &Function) -> f64 {
        if let (Data::Freq(f1), Data::Freq(f2)) = (&e1.data, &e2.data) {
            weighted_jaccard(f1, f2)
        } else if let (Data::KgramFreq(k1), Data::KgramFreq(k2)) = (&e1.data, &e2.data) {
            weighted_jaccard(k1, k2)
        } else if let (Data::Seq(f1), Data::Seq(f2)) = (&e1.data, &e2.data) {
            weighted_jaccard(&seq2freq(f1), &seq2freq(f2))
        } else if let (Data::KgramSeq(k1), Data::KgramSeq(k2)) = (&e1.data, &e2.data) {
            weighted_jaccard(&seq2freq(k1), &seq2freq(k2))
        } else {
            0.0
        }
    }
}

impl BirthmarkComparator for Lcs {
    fn compare_functions(&self, e1: &Function, e2: &Function) -> f64 {
        if let (Data::Seq(s1), Data::Seq(s2)) = (&e1.data, &e2.data) {
            longest_common_subsequence(s1, s2)
        } else if let (Data::KgramSeq(k1), Data::KgramSeq(k2)) = (&e1.data, &e2.data) {
            longest_common_subsequence(k1, k2)
        } else {
            0.0
        }
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

#[allow(dead_code)]
fn levenshtein_distance_full_memory<T: PartialEq>(s1: &[T], s2: &[T]) -> f64 {
    let mut dp = Array2::zeros((s1.len() + 1, s2.len() + 1));
    for i in 0..=s1.len() {
        dp[[i, 0]] = i;
    }
    for j in 0..=s2.len() {
        dp[[0, j]] = j;
    }
    for i in 1..=s1.len() {
        for j in 1..=s2.len() {
            let cost = if s1[i - 1] == s2[j - 1] { 0 } else { 1 };
            let substitution = dp[[i - 1, j - 1]] + cost;
            let insertion = dp[[i, j - 1]] + 1;
            let deletion = dp[[i - 1, j]] + 1;
            dp[[i, j]] = substitution.min(insertion).min(deletion);
        }
    }
    1.0 - (dp[[s1.len(), s2.len()]] as f64 / (s1.len().max(s2.len()) as f64))
}

fn cosine_similarity<T: std::cmp::Eq + std::hash::Hash>(
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

fn euclidean_distance<T: std::cmp::Eq + std::hash::Hash>(
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

fn weighted_jaccard<T: std::cmp::Eq + std::hash::Hash>(
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

#[allow(dead_code)]
fn longest_common_subsequence_full_memory<T: PartialEq>(s1: &[T], s2: &[T]) -> f64 {
    let mut dp = Array2::<usize>::zeros((s1.len() + 1, s2.len() + 1));
    for i in 1..=s1.len() {
        for j in 1..=s2.len() {
            if s1[i - 1] == s2[j - 1] {
                dp[[i, j]] = dp[[i - 1, j - 1]] + 1;
            } else {
                dp[[i, j]] = dp[[i - 1, j]].max(dp[[i, j - 1]]);
            }
        }
    }
    let lcs_length = dp[[s1.len(), s2.len()]];
    2.0 * lcs_length as f64 / (s1.len() + s2.len()) as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::birthmarks::{BirthmarkType, Kgram};

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
                Algorithm::Levenshtein => 4,
                Algorithm::Lcs => 5,
                Algorithm::Simpson => 6,
                Algorithm::WeightedJaccard => 7,
            }
        }
        let mut seen = Algorithm::ALL.iter().map(index).collect::<Vec<_>>();
        seen.sort_unstable();
        assert_eq!(seen, (0..8).collect::<Vec<_>>());
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
            (Algorithm::Levenshtein, "Levenshtein Distance"),
            (Algorithm::Lcs, "Longest Common Subsequence"),
            (Algorithm::Simpson, "Simpson's Coefficient"),
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
                    Algorithm::Levenshtein => (a.clone(), Box::new(Levenshtein), seqs.clone()),
                    Algorithm::Lcs => (a.clone(), Box::new(Lcs), seqs.clone()),
                    Algorithm::Simpson => (a.clone(), Box::new(Simpson), sets.clone()),
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

    /// Mismatched or unsupported representations fall through to 0.0 rather
    /// than panicking.
    #[test]
    fn comparators_return_zero_for_unsupported_representations() {
        let s = function("f", seq(&["A"]));
        let st = function("g", set(&["A"]));
        // Data variants that do not pair up
        assert_eq!(Jaccard.compare_functions(&s, &st), 0.0);
        assert_eq!(Dice.compare_functions(&s, &st), 0.0);
        assert_eq!(Simpson.compare_functions(&s, &st), 0.0);
        assert_eq!(Cosine.compare_functions(&s, &st), 0.0);
        assert_eq!(Euclidean.compare_functions(&s, &st), 0.0);
        assert_eq!(WeightedJaccard.compare_functions(&s, &st), 0.0);
        // Levenshtein and LCS only support sequences at all
        assert_eq!(Levenshtein.compare_functions(&st, &st), 0.0);
        assert_eq!(Lcs.compare_functions(&st, &st), 0.0);
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
        match Jaccard.compare_birthmarks(&b1, &b2, &Aggregator::Hungarian) {
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
            .compare_birthmarks(&empty, &empty, &Aggregator::Hungarian)
            .unwrap();
        assert_eq!(both.similarity(), 1.0);

        // exactly one empty means nothing in common
        let one = Jaccard
            .compare_birthmarks(&empty, &filled, &Aggregator::Hungarian)
            .unwrap();
        assert_eq!(one.similarity(), 0.0);
        let other = Jaccard
            .compare_birthmarks(&filled, &empty, &Aggregator::Hungarian)
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
            let c = Jaccard.compare_birthmarks(&b, &b, &aggregator).unwrap();
            assert!(
                (c.similarity() - 1.0).abs() < 1e-9,
                "aggregator: {aggregator:?}"
            );
        }
    }

    #[test]
    fn top_n_selection_limits_the_number_of_scores() {
        let matrix =
            Array2::from_shape_vec((3, 3), vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0])
                .unwrap();
        // Size::Num(k) keeps k row maxima plus k column maxima
        let picked = top_n_selection(&matrix, &Size::Num(2)).unwrap();
        assert_eq!(picked.len(), 4);
        assert!(picked.iter().all(|v| (*v - 1.0).abs() < 1e-9));
        // Size::All keeps every row and column maximum
        let all = top_n_selection(&matrix, &Size::All).unwrap();
        assert_eq!(all.len(), 6);
    }

    #[test]
    fn comparison_similarity_is_zero_when_no_scores_were_aggregated() {
        let b = birthmark("a", &[("f", &["A"])]);
        let c = Comparison::new(
            &b,
            &b,
            Array2::zeros((0, 0)),
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

    #[test]
    fn comparator_dispatches_every_algorithm() {
        let b = birthmark("a", &[("f", &["A", "B"])]);
        for algorithm in [
            Algorithm::Cosine,
            Algorithm::Dice,
            Algorithm::Euclidean,
            Algorithm::Jaccard,
            Algorithm::Levenshtein,
            Algorithm::Lcs,
            Algorithm::Simpson,
            Algorithm::WeightedJaccard,
        ] {
            let comparator = algorithm.comparator();
            let c = comparator
                .compare_birthmarks(&b, &b, &Aggregator::Hungarian)
                .unwrap_or_else(|e| panic!("{algorithm}: {e}"));
            assert!(
                (c.similarity() - 1.0).abs() < 1e-9,
                "algorithm: {algorithm}"
            );
        }
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
    }
}
