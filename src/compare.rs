use crate::{Iterable, prelude::*};
use std::{io::Write, path::Path, time::Instant};
use clap::ValueEnum;
use rustc_hash::{FxHashMap, FxHashSet};
use itertools::Itertools;
use ndarray::Array2;

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
            PairingStrategy::All => targets.len() * (targets.len() - 1) / 2,
            PairingStrategy::SelfCoverage => targets.len(),
            PairingStrategy::Adjacent => targets.len().saturating_sub(1),
            PairingStrategy::FirstVsOthers | PairingStrategy::LastVsOthers => targets.len().saturating_sub(1),
            PairingStrategy::AllAndSelf => targets.len() * (targets.len() + 1) / 2,
        }
    }
    pub fn pairs<'a, T: std::marker::Sync>(&'a self, targets: &'a [T]) -> Box<dyn Iterator<Item = (&'a T, &'a T)> + Send + 'a> {
        match self {
            PairingStrategy::AllAndSelf => Box::new(targets.iter().combinations(2).map(|c| (c[0], c[1]))
                .chain(targets.iter().map(|f| (f, f)))),
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
            },
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

fn escape_csv_string(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

impl<'a, S: CsvInfo> Comparison<'a, S> {
    pub fn new(columns: &'a S, rows: &'a S, matrix: Array2<f64>, similarities: Vec<f64>, duration: std::time::Duration) -> Comparison<'a, S> {
        Self { columns, rows, matrix, similarities, duration }
    }

    pub fn store<P: AsRef<Path>>(&self, dest: P) -> Result<()> {
        let mut file = std::fs::File::create(dest.as_ref())
            .map_err(|e| Error::Io(dest.as_ref().to_path_buf(), e))?;
        let mut out = std::io::BufWriter::new(&mut file);
        let _ = writeln!(out, "result,{},{}", self.duration.as_nanos(), self.similarity());
        let _ = writeln!(out, "left,{}", self.rows.csv_info());
        let _ = writeln!(out, "right,{}", self.columns.csv_info());
        let b1_names = self.columns.names();
        let b2_names = self.rows.names();
        let _ = write!(out, "matrix,,{}", b1_names.iter()
            .map(|s| escape_csv_string(s)).join(","));
        for j in 0..b2_names.len() {
            let _ = write!(out, "\n{}, {}", j, escape_csv_string(&b2_names[j]));
            for i in 0..b1_names.len() {
                let value = self.matrix[[i, j]];
                let _ = write!(out, ",{value}");
            }
        }
        let _ = writeln!(out);
        Ok(())
    }

    pub fn similarity(&self) -> f64 {
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
            match n.parse::<usize>().map(Size::Num) {
                Ok(n) => {
                    log::info!("Using TopN({:?}) algorithm for aggregation", n);
                    Ok(Aggregator::TopN(n))
                },
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
            Aggregator::Hungarian => 
                hungarian_algorithm(array)
                    .map(|(sim, _matches)| sim),
            Aggregator::TopN(n) => 
                top_n_selection(array, n)
        }
    }
}

trait BirthmarkComparator {
    fn compare_birthmarks<'a>(&self, b1: &'a Birthmark, b2: &'a Birthmark, aggregator: &Aggregator) -> Result<Comparison<'a, Birthmark>> {
        if !b1.comparable_with(b2) {
            return Err(Error::Mismatch(b1.metadata.birthmark_type.clone(), b2.metadata.birthmark_type.clone()));
        }
        let p1_len = b1.elements.len();
        let p2_len = b2.elements.len();
        let size = std::cmp::max(p1_len, p2_len);
        if p1_len == 0 && p2_len == 0 {
            Ok(Comparison::new(b1, b2, Array2::<f64>::zeros((0, 0)), vec![1.0], std::time::Duration::from_millis(0)))
        } else if p1_len == 0 || p2_len == 0 {
            Ok(Comparison::new(b1, b2, Array2::<f64>::zeros((0, 0)), vec![0.0], std::time::Duration::from_millis(0)))
        } else {
            let start = Instant::now();
            let r = build_matrix(b1, b2, size, |e1, e2| {
                self.compare_elements(e1, e2)
            })?;
            aggregator.aggregate(&r)
                .map(|sim| Comparison::new(b1, b2, r, sim, start.elapsed()))
        }
    }

    fn compare_elements(&self, e1: &Elements, e2: &Elements) -> f64;
}

fn build_matrix<F, T>(p1: impl Iterable<Item = T>, p2: impl Iterable<Item = T>, size: usize, compare_func: F) -> Result<Array2<f64>>
where 
    F: Fn(&T, &T) -> f64,
{
    let mut flat_costs = vec![0.0; size * size];
    for (i, item1) in p1.iter().enumerate() {
        for (j, item2) in p2.iter().enumerate() {
            // lapjv expects a cost matrix where lower values indicate better matches.
            // so we convert similarity to cost by using (1.0 - similarity).
            flat_costs[i * size + j] = compare_func(item1, item2);
        }
    }
    Array2::from_shape_vec((size, size), flat_costs)
        .map_err(Error::ShapeError)
}

fn top_n_selection(array2d: &Array2<f64>, n: &Size) -> Result<Vec<f64>> {
    let rows = array2d.axis_iter(ndarray::Axis(0))
        .map(|col| col.fold(0.0f64, |acc, &v| acc.max(v)))
        .sorted_by(|a, b| b.total_cmp(a))
        .collect::<Vec<_>>();
    let cols = array2d.axis_iter(ndarray::Axis(1))
        .map(|col| col.fold(0.0f64, |acc, &v| acc.max(v)))
        .sorted_by(|a, b| b.total_cmp(a))
        .collect::<Vec<_>>();
    match n {
        Size::Num(k) => 
            Ok(rows.into_iter().take(*k).chain(cols.into_iter().take(*k)).collect()),
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
        },
        Err(e) => Err(Error::LapJV(e)),
    }
}

trait ProgramComparator<T: crate::Op> {
    fn compare_programs<'a>(&self, p1: &'a Program<T>, p2: &'a Program<T>, aggregator: &Aggregator) -> Result<Comparison<'a, Program<T>>> {
        let p1_len = p1.len();
        let p2_len = p2.len();
        let size = std::cmp::max(p1_len, p2_len);
        if p1_len == 0 && p2_len == 0 {
            Ok(Comparison::new(p1, p2, Array2::<f64>::zeros((0, 0)), vec![1.0], std::time::Duration::from_millis(0)))
        } else if p1_len == 0 || p2_len == 0 {
            Ok(Comparison::new(p1, p2, Array2::<f64>::zeros((0, 0)), vec![0.0], std::time::Duration::from_millis(0)))
        } else {
            let start = Instant::now();
            let r = build_matrix(p1, p2, size, |f1, f2| {
                self.compare_func(f1, f2)
            })?;
            aggregator.aggregate(&r)
                .map(|sim| Comparison::new(p1, p2, r, sim, start.elapsed()))
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
    pub fn comparator(&self) -> Comparator {
        self.into()
    }
}

/// This struct holds the specific algorithm instance and dispatches the comparison calls to it.
pub struct Comparator {
    inner: ComparatorImpl
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
            Algorithm::Cosine => Comparator{ inner: ComparatorImpl::Cosine(Cosine{}) },
            Algorithm::Dice => Comparator{ inner: ComparatorImpl::Dice(Dice{}) },
            Algorithm::Euclidean => Comparator{ inner: ComparatorImpl::Euclidean(Euclidean{}) },
            Algorithm::Jaccard => Comparator{ inner: ComparatorImpl::Jaccard(Jaccard{}) },
            Algorithm::Levenshtein => Comparator{ inner: ComparatorImpl::Levenshtein(Levenshtein{}) },
            Algorithm::Lcs => Comparator{ inner: ComparatorImpl::Lcs(Lcs{}) },
            Algorithm::Simpson => Comparator{ inner: ComparatorImpl::Simpson(Simpson{}) },
            Algorithm::WeightedJaccard => Comparator{ inner: ComparatorImpl::WeightedJaccard(WeightedJaccard{}) },
        }
    }
}

impl Comparator {
    pub fn compare_programs<'a, T: crate::Op>(&self, p1: &'a Program<T>, p2: &'a Program<T>, aggregator: &Aggregator) -> Result<Comparison<'a, Program<T>>> {
        match &self.inner {
            ComparatorImpl::Cosine(c) => c.compare_programs(p1, p2, aggregator),
            ComparatorImpl::Dice(d) => d.compare_programs(p1, p2, aggregator),
            ComparatorImpl::Euclidean(e) => e.compare_programs(p1, p2, aggregator),
            ComparatorImpl::Jaccard(j) => j.compare_programs(p1, p2, aggregator),
            ComparatorImpl::Levenshtein(l) => l.compare_programs(p1, p2, aggregator),
            ComparatorImpl::Lcs(lcs) => lcs.compare_programs(p1, p2, aggregator),
            ComparatorImpl::Simpson(s) => s.compare_programs(p1, p2, aggregator),
            ComparatorImpl::WeightedJaccard(wj) => wj.compare_programs(p1, p2, aggregator),
        }
    }

    pub fn compare_birthmarks<'a>(&self, b1: &'a Birthmark, b2: &'a Birthmark, aggregator: &Aggregator) -> Result<Comparison<'a, Birthmark>> {
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

fn jaccard_index<T: std::fmt::Debug + std::cmp::Eq + std::hash::Hash>(s1: &FxHashSet<T>, s2: &FxHashSet<T>) -> f64 {
    if s1.is_empty() && s2.is_empty() {
        1.0
    } else if s1.is_empty() || s2.is_empty() {
        0.0
    } else {
        s1.intersection(s2).count() as f64 / s1.union(s2).count() as f64
    }
}

fn dice_index<T: std::fmt::Debug + std::cmp::Eq + std::hash::Hash>(s1: &FxHashSet<T>, s2: &FxHashSet<T>) -> f64 {
    if s1.is_empty() && s2.is_empty() {
        1.0
    } else if s1.is_empty() || s2.is_empty() {
        0.0
    } else {
        (2.0 * s1.intersection(s2).count() as f64) / (s1.len() + s2.len()) as f64
    }
}

fn simpson_index<T: std::fmt::Debug + std::cmp::Eq + std::hash::Hash>(s1: &FxHashSet<T>, s2: &FxHashSet<T>) -> f64 {
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

    if n == 0 && m == 0 { return 1.0; }
    if n == 0 || m == 0 { return 0.0; }
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

fn cosine_similarity<T: std::cmp::Eq + std::hash::Hash>(f1: &FxHashMap<T, usize>, f2: &FxHashMap<T, usize>) -> f64 {
    let keys = f1.keys().chain(f2.keys()).collect::<FxHashSet<_>>();
    let dot_product = keys.iter().map(|k| {
        let v1 = *f1.get(*k).unwrap_or(&0) as f64;
        let v2 = *f2.get(*k).unwrap_or(&0) as f64;
        v1 * v2
    }).sum::<f64>();
    let magnitude1 = f1.values().map(|v| (*v as f64).powi(2)).sum::<f64>().sqrt();
    let magnitude2 = f2.values().map(|v| (*v as f64).powi(2)).sum::<f64>().sqrt();
    if magnitude1 > 0.0 && magnitude2 > 0.0 {
        dot_product / (magnitude1 * magnitude2)
    } else {
        1.0
    }
}

fn euclidean_distance<T: std::cmp::Eq + std::hash::Hash>(f1: &FxHashMap<T, usize>, f2: &FxHashMap<T, usize>) -> f64 {
    let keys = f1.keys().chain(f2.keys()).collect::<FxHashSet<_>>();
    let sum_of_squares = keys.iter().map(|k| {
        let v1 = *f1.get(*k).unwrap_or(&0) as f64;
        let v2 = *f2.get(*k).unwrap_or(&0) as f64;
        (v1 - v2).powi(2)
    }).sum::<f64>();
    1.0 - (sum_of_squares.sqrt() / (f1.values().map(|v| (*v as f64).powi(2)).sum::<f64>().sqrt() + f2.values().map(|v| (*v as f64).powi(2)).sum::<f64>().sqrt()))
}

fn weighted_jaccard<T: std::cmp::Eq + std::hash::Hash>(f1: &FxHashMap<T, usize>, f2: &FxHashMap<T, usize>) -> f64 {
    let keys = f1.keys().chain(f2.keys()).collect::<FxHashSet<_>>();
    let min_sum = keys.iter().map(|k| {
        let v1 = *f1.get(*k).unwrap_or(&0) as f64;
        let v2 = *f2.get(*k).unwrap_or(&0) as f64;
        v1.min(v2)
    }).sum::<f64>();
    let max_sum = keys.iter().map(|k| {
        let v1 = *f1.get(*k).unwrap_or(&0) as f64;
        let v2 = *f2.get(*k).unwrap_or(&0) as f64;
        v1.max(v2)
    }).sum::<f64>();
    if max_sum > 0.0 {
        min_sum / max_sum
    } else {
        1.0
    }
}

fn longest_common_subsequence<T: PartialEq>(s1: &[T], s2: &[T]) -> f64 {
    let n = s1.len();
    let m = s2.len();
    if n == 0 || m == 0 { return 0.0; }

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
