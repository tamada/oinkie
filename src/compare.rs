use crate::{Iterable, prelude::*};
use clap::ValueEnum;
use itertools::Itertools;
use ndarray::Array2;
use rustc_hash::{FxHashMap, FxHashSet};
use std::{io::Write, path::Path, time::Instant};

#[cfg_attr(doc, katexit::katexit)]
#[derive(Debug, Clone, ValueEnum)]
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

pub struct Comparison<'a, S> {
    columns: &'a S,
    rows: &'a S,
    matrix: Array2<f64>,
    duration: std::time::Duration,
    similarities: Vec<f64>,
}

/// Quotes a value for embedding into a CSV field when it contains
/// characters that would otherwise break the record structure.
pub fn escape_csv_string(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

impl<'a, S: CsvInfo> Comparison<'a, S> {
    pub fn new(
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

    pub fn store<P: AsRef<Path>>(&self, dest: P) -> Result<()> {
        let dest = dest.as_ref();
        let io_err = |e| Error::Io(dest.to_path_buf(), e);
        let mut file = std::fs::File::create(dest).map_err(io_err)?;
        let mut out = std::io::BufWriter::new(&mut file);
        writeln!(
            out,
            "result,{},{}",
            self.duration.as_nanos(),
            self.similarity()
        )
        .map_err(io_err)?;
        writeln!(out, "left,{}", self.columns.csv_info()).map_err(io_err)?;
        writeln!(out, "right,{}", self.rows.csv_info()).map_err(io_err)?;
        let b1_names = self.columns.names();
        let b2_names = self.rows.names();
        write!(
            out,
            "matrix,,{}",
            b1_names.iter().map(|s| escape_csv_string(s)).join(",")
        )
        .map_err(io_err)?;
        for (j, item) in b2_names.iter().enumerate() {
            write!(out, "\n{}, {}", j, escape_csv_string(item)).map_err(io_err)?;
            for i in 0..b1_names.len() {
                let value = self.matrix[[i, j]];
                write!(out, ",{value}").map_err(io_err)?;
            }
        }
        writeln!(out).map_err(io_err)?;
        out.flush().map_err(io_err)?;
        Ok(())
    }

    pub fn similarity(&self) -> f64 {
        if self.similarities.is_empty() {
            return 0.0;
        }
        self.similarities.iter().sum::<f64>() / self.similarities.len() as f64
    }

    pub fn duration(&self) -> std::time::Duration {
        self.duration
    }
}

#[derive(Debug, Clone)]
pub enum Size {
    Num(usize),
    All,
}

#[derive(Debug, Clone, Default)]
pub enum Aggregator {
    TopN(Size),
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
        let p1_len = b1.elements.len();
        let p2_len = b2.elements.len();
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
            let r = build_matrix(b1, b2, size, |e1, e2| self.compare_elements(e1, e2))?;
            aggregator
                .aggregate(&r)
                .map(|sim| Comparison::new(b1, b2, r, sim, start.elapsed()))
        }
    }

    fn compare_elements(&self, e1: &Elements, e2: &Elements) -> f64;
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

/// A finished comparison before it is attached to the two things compared.
///
/// The programs are held by reference in a [`Comparison`], so producing one
/// fixes the type they are reported as. Returning the result on its own lets
/// the same computation be reported against a `Program<T>` or against the
/// [`AnyProgram`] a caller loaded without naming `T`.
type Scored = (Array2<f64>, Vec<f64>, std::time::Duration);

trait ProgramComparator<T: crate::Op> {
    fn score_programs(
        &self,
        p1: &Program<T>,
        p2: &Program<T>,
        aggregator: &Aggregator,
    ) -> Result<Scored> {
        let p1_len = p1.len();
        let p2_len = p2.len();
        let size = std::cmp::max(p1_len, p2_len);
        let zero = std::time::Duration::from_millis(0);
        if p1_len == 0 && p2_len == 0 {
            Ok((Array2::<f64>::zeros((0, 0)), vec![1.0], zero))
        } else if p1_len == 0 || p2_len == 0 {
            Ok((Array2::<f64>::zeros((0, 0)), vec![0.0], zero))
        } else {
            let start = Instant::now();
            let r = build_matrix(p1, p2, size, |f1, f2| self.compare_func(f1, f2))?;
            aggregator
                .aggregate(&r)
                .map(|sim| (r, sim, start.elapsed()))
        }
    }

    fn compare_func(&self, f1: &Function<T>, f2: &Function<T>) -> f64;
}

#[derive(Debug, Clone, ValueEnum)]
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
    /// The CLI spelling of this algorithm and the shape it operates on, in one
    /// table so that the parser, the validation and the error message cannot
    /// drift apart.
    ///
    /// Written out rather than derived from `ValueEnum`, whose kebab-case
    /// renders `WeightedJaccard` as `weighted-jaccard`. This is the canonical
    /// spelling inside an analysis name and the one the completion list is
    /// built from; the kebab-case is accepted there too, since it is what
    /// `compare --algorithm` takes and what `oinkie info` prints (#71).
    ///
    /// A hyphen here is no longer forbidden. It was, while an analysis name
    /// was split on its last one.
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

    /// The birthmark shape this algorithm computes over. Anything else it is
    /// handed is converted to this first, which is why a pairing that does not
    /// match is rejected rather than quietly re-encoded.
    pub fn shape(&self) -> Shape {
        self.spec().1
    }

    /// The spelling this algorithm has in an analysis name. Public because
    /// the CLI builds its completion list from it.
    pub fn cli_name(&self) -> &'static str {
        self.spec().0
    }

    pub fn comparator(&self) -> Comparator {
        self.into()
    }
}

/// This struct holds the specific algorithm instance and dispatches the comparison calls to it.
pub struct Comparator {
    inner: ComparatorImpl,
}

/// The implementation of Comparator, which holds the specific algorithm instance and dispatches the comparison calls to it.
/// However, the specific algorithm does not appear in the public API of Comparator, so that the internal implementation can be changed without affecting the users of Comparator.
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
    fn score_programs<T: crate::Op>(
        &self,
        p1: &Program<T>,
        p2: &Program<T>,
        aggregator: &Aggregator,
    ) -> Result<Scored> {
        match &self.inner {
            ComparatorImpl::Cosine(c) => c.score_programs(p1, p2, aggregator),
            ComparatorImpl::Dice(d) => d.score_programs(p1, p2, aggregator),
            ComparatorImpl::Euclidean(e) => e.score_programs(p1, p2, aggregator),
            ComparatorImpl::Jaccard(j) => j.score_programs(p1, p2, aggregator),
            ComparatorImpl::Levenshtein(l) => l.score_programs(p1, p2, aggregator),
            ComparatorImpl::Lcs(lcs) => lcs.score_programs(p1, p2, aggregator),
            ComparatorImpl::Simpson(s) => s.score_programs(p1, p2, aggregator),
            ComparatorImpl::WeightedJaccard(wj) => wj.score_programs(p1, p2, aggregator),
        }
    }

    pub fn compare_programs<'a, T: crate::Op>(
        &self,
        p1: &'a Program<T>,
        p2: &'a Program<T>,
        aggregator: &Aggregator,
    ) -> Result<Comparison<'a, Program<T>>> {
        let (matrix, similarities, duration) = self.score_programs(p1, p2, aggregator)?;
        Ok(Comparison::new(p1, p2, matrix, similarities, duration))
    }

    /// Compares two programs read without naming their operation type.
    ///
    /// Two representations describe the same instruction with different
    /// operations, so a comparison across them would measure the disagreement
    /// between the lifters rather than anything about the programs. That is
    /// refused here for the same reason it is refused for birthmarks in
    /// [`Self::compare_birthmarks`]. Only one representation can be read
    /// today, so nothing reachable is refused yet; the check is what keeps
    /// that true when a second one arrives.
    pub fn compare_any<'a>(
        &self,
        p1: &'a AnyProgram,
        p2: &'a AnyProgram,
        aggregator: &Aggregator,
    ) -> Result<Comparison<'a, AnyProgram>> {
        if p1.ir() != p2.ir() {
            return Err(Error::IrMismatch(p1.ir(), p2.ir()));
        }
        let (matrix, similarities, duration) = match (p1, p2) {
            (AnyProgram::GhidraPcode(a), AnyProgram::GhidraPcode(b)) => {
                self.score_programs(a, b, aggregator)?
            }
        };
        Ok(Comparison::new(p1, p2, matrix, similarities, duration))
    }

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
    fn compare_elements(&self, e1: &Elements, e2: &Elements) -> f64 {
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
    fn compare_elements(&self, e1: &Elements, e2: &Elements) -> f64 {
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
    fn compare_elements(&self, e1: &Elements, e2: &Elements) -> f64 {
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
    fn compare_elements(&self, e1: &Elements, e2: &Elements) -> f64 {
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
    fn compare_elements(&self, e1: &Elements, e2: &Elements) -> f64 {
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
    fn compare_elements(&self, e1: &Elements, e2: &Elements) -> f64 {
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
    fn compare_elements(&self, e1: &Elements, e2: &Elements) -> f64 {
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
    fn compare_elements(&self, e1: &Elements, e2: &Elements) -> f64 {
        if let (Data::Seq(s1), Data::Seq(s2)) = (&e1.data, &e2.data) {
            longest_common_subsequence(s1, s2)
        } else if let (Data::KgramSeq(k1), Data::KgramSeq(k2)) = (&e1.data, &e2.data) {
            longest_common_subsequence(k1, k2)
        } else {
            0.0
        }
    }
}

impl<T: crate::Op> ProgramComparator<T> for Jaccard {
    fn compare_func(&self, f1: &Function<T>, f2: &Function<T>) -> f64 {
        let set1: rustc_hash::FxHashSet<&str> = f1.ops().collect();
        let set2: rustc_hash::FxHashSet<&str> = f2.ops().collect();
        jaccard_index(&set1, &set2)
    }
}

impl<T: crate::Op> ProgramComparator<T> for Dice {
    fn compare_func(&self, f1: &Function<T>, f2: &Function<T>) -> f64 {
        let set1: rustc_hash::FxHashSet<&str> = f1.ops().collect();
        let set2: rustc_hash::FxHashSet<&str> = f2.ops().collect();
        dice_index(&set1, &set2)
    }
}

impl<T: crate::Op> ProgramComparator<T> for Simpson {
    fn compare_func(&self, f1: &Function<T>, f2: &Function<T>) -> f64 {
        let set1: rustc_hash::FxHashSet<&str> = f1.ops().collect();
        let set2: rustc_hash::FxHashSet<&str> = f2.ops().collect();
        simpson_index(&set1, &set2)
    }
}

impl<T: crate::Op> ProgramComparator<T> for Levenshtein {
    fn compare_func(&self, f1: &Function<T>, f2: &Function<T>) -> f64 {
        let ops1: Vec<&str> = f1.ops().collect();
        let ops2: Vec<&str> = f2.ops().collect();
        levenshtein_distance(&ops1, &ops2)
    }
}

impl<T: crate::Op> ProgramComparator<T> for Lcs {
    fn compare_func(&self, f1: &Function<T>, f2: &Function<T>) -> f64 {
        let ops1: Vec<&str> = f1.ops().collect();
        let ops2: Vec<&str> = f2.ops().collect();
        longest_common_subsequence(&ops1, &ops2)
    }
}

impl<T: crate::Op> ProgramComparator<T> for Cosine {
    fn compare_func(&self, f1: &Function<T>, f2: &Function<T>) -> f64 {
        let map1 = f1.ops_freq();
        let map2 = f2.ops_freq();
        cosine_similarity(&map1, &map2)
    }
}

impl<T: crate::Op> ProgramComparator<T> for Euclidean {
    /// Computes the Euclidean distance between two functions and converts it to a similarity score.
    /// The similarity is calculated as `exp(-distance)`, which maps a distance of 0 to a similarity of 1, and larger distances to values approaching 0.
    fn compare_func(&self, f1: &Function<T>, f2: &Function<T>) -> f64 {
        let map1 = f1.ops_freq();
        let map2 = f2.ops_freq();
        euclidean_distance(&map1, &map2)
    }
}

impl<T: crate::Op> ProgramComparator<T> for WeightedJaccard {
    fn compare_func(&self, f1: &Function<T>, f2: &Function<T>) -> f64 {
        let map1 = f1.ops_freq();
        let map2 = f2.ops_freq();
        weighted_jaccard(&map1, &map2)
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

            // 3つの操作の最小値をとる
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
    if magnitude1 > 0.0 && magnitude2 > 0.0 {
        dot_product / (magnitude1 * magnitude2)
    } else {
        1.0
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
    1.0 - (sum_of_squares.sqrt()
        / (f1.values().map(|v| (*v as f64).powi(2)).sum::<f64>().sqrt()
            + f2.values().map(|v| (*v as f64).powi(2)).sum::<f64>().sqrt()))
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
    use crate::birthmarks::Metadata;
    use std::path::PathBuf;

    /// Dispatching on the file's representation must change nothing about the
    /// answer -- only about who chose the operation type.
    #[test]
    fn test_compare_any_agrees_with_the_typed_comparison() {
        let paths = [
            "testdata/hello_world/pcodes/hello_clang.json",
            "testdata/hello_world/pcodes/hello_gcc.json",
        ];
        let typed: Vec<Program<crate::ghidra::Op>> = paths
            .iter()
            .map(|p| Path::new(p).try_into().unwrap())
            .collect();
        let any: Vec<AnyProgram> = paths
            .iter()
            .map(|p| AnyProgram::load(Path::new(p)).unwrap())
            .collect();

        let comparator = Algorithm::Jaccard.comparator();
        let aggregator = &Aggregator::Hungarian;
        let expected = comparator
            .compare_programs(&typed[0], &typed[1], aggregator)
            .unwrap()
            .similarity();
        let actual = comparator
            .compare_any(&any[0], &any[1], aggregator)
            .unwrap()
            .similarity();
        assert_eq!(actual, expected);
    }

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

    fn elements(name: &str, data: Data) -> Elements {
        Elements {
            name: name.to_string(),
            data,
        }
    }

    fn seq(items: &[&str]) -> Data {
        Data::Seq(items.iter().map(|s| s.to_string()).collect())
    }

    fn set(items: &[&str]) -> Data {
        Data::Set(items.iter().map(|s| s.to_string()).collect())
    }

    fn freq(items: &[&str]) -> Data {
        Data::Freq(seq2freq(
            &items.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
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
            let e1 = elements("f", build(&["A", "B", "C"]));
            let e2 = elements("g", build(&["A", "B", "C"]));
            assert!((Jaccard.compare_elements(&e1, &e2) - 1.0).abs() < 1e-9);
            assert!((Dice.compare_elements(&e1, &e2) - 1.0).abs() < 1e-9);
            assert!((Simpson.compare_elements(&e1, &e2) - 1.0).abs() < 1e-9);
        }
    }

    #[test]
    fn sequence_comparators_accept_seq_representations() {
        let builders: [fn(&[&str]) -> Data; 2] = [seq, kgram_seq];
        for build in builders {
            let e1 = elements("f", build(&["A", "B", "C"]));
            let e2 = elements("g", build(&["A", "B", "C"]));
            assert!((Levenshtein.compare_elements(&e1, &e2) - 1.0).abs() < 1e-9);
            assert!((Lcs.compare_elements(&e1, &e2) - 1.0).abs() < 1e-9);
        }
    }

    #[test]
    fn frequency_comparators_accept_every_supported_representation() {
        let builders: [fn(&[&str]) -> Data; 4] = [seq, freq, kgram_seq, kgram_freq];
        for build in builders {
            let e1 = elements("f", build(&["A", "B", "C"]));
            let e2 = elements("g", build(&["A", "B", "C"]));
            assert!((Cosine.compare_elements(&e1, &e2) - 1.0).abs() < 1e-9);
            assert!((WeightedJaccard.compare_elements(&e1, &e2) - 1.0).abs() < 1e-9);
            assert!((Euclidean.compare_elements(&e1, &e2) - 1.0).abs() < 1e-9);
        }
    }

    /// Mismatched or unsupported representations fall through to 0.0 rather
    /// than panicking.
    #[test]
    fn comparators_return_zero_for_unsupported_representations() {
        let s = elements("f", seq(&["A"]));
        let st = elements("g", set(&["A"]));
        // Data variants that do not pair up
        assert_eq!(Jaccard.compare_elements(&s, &st), 0.0);
        assert_eq!(Dice.compare_elements(&s, &st), 0.0);
        assert_eq!(Simpson.compare_elements(&s, &st), 0.0);
        assert_eq!(Cosine.compare_elements(&s, &st), 0.0);
        assert_eq!(Euclidean.compare_elements(&s, &st), 0.0);
        assert_eq!(WeightedJaccard.compare_elements(&s, &st), 0.0);
        // Levenshtein and LCS only support sequences at all
        assert_eq!(Levenshtein.compare_elements(&st, &st), 0.0);
        assert_eq!(Lcs.compare_elements(&st, &st), 0.0);
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
            elements: funcs.iter().map(|(n, ops)| elements(n, seq(ops))).collect(),
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
    fn comparison_store_writes_a_parsable_matrix() {
        let b1 = birthmark("left", &[("f", &["A", "B"]), ("g", &["B"])]);
        let b2 = birthmark("right", &[("h", &["A"]), ("i", &["B"])]);
        let c = Jaccard
            .compare_birthmarks(&b1, &b2, &Aggregator::Hungarian)
            .unwrap();

        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("00000.csv");
        c.store(&dest).expect("failed to store the comparison");

        let content = std::fs::read_to_string(&dest).unwrap();
        assert!(content.starts_with("result,"));
        assert!(content.contains("\nleft,birthmark,left,"));
        assert!(content.contains("\nright,birthmark,right,"));
        assert!(content.contains("\nmatrix,,"));
        // one line per row element, each prefixed with its index
        assert_eq!(
            content
                .lines()
                .filter(|l| l.starts_with('0') || l.starts_with('1'))
                .count(),
            2
        );
    }

    #[test]
    fn comparison_store_reports_io_errors() {
        let b = birthmark("a", &[("f", &["A"])]);
        let c = Jaccard
            .compare_birthmarks(&b, &b, &Aggregator::Hungarian)
            .unwrap();
        // a directory that does not exist cannot receive the result file
        let err = c.store("no/such/directory/out.csv").unwrap_err();
        assert!(matches!(err, Error::Io(..)));
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

    #[test]
    fn escape_csv_string_quotes_only_when_needed() {
        assert_eq!(escape_csv_string("plain"), "plain");
        assert_eq!(escape_csv_string("a,b"), "\"a,b\"");
        assert_eq!(escape_csv_string("a\"b"), "\"a\"\"b\"");
        assert_eq!(escape_csv_string("a\nb"), "\"a\nb\"");
    }
}
