use crate::prelude::*;
use std::{io::Write, path::Path, time::Instant};
use clap::ValueEnum;
use pathfinding::matrix::Matrix;
use rustc_hash::{FxHashMap, FxHashSet};
use itertools::Itertools;

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
}

impl PairingStrategy {
    pub fn compare_count<T>(&self, targets: &[T]) -> usize {
        match self {
            PairingStrategy::All => targets.len() * (targets.len() - 1) / 2,
            PairingStrategy::SelfCoverage => targets.len(),
            PairingStrategy::Adjacent => targets.len().saturating_sub(1),
            PairingStrategy::FirstVsOthers => targets.len().saturating_sub(1),
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
            }
        }
    }
}

pub struct Comparison<'a, S> {
    columns: &'a S,
    rows: &'a S,
    matrix: Matrix<i32>,
    duration: std::time::Duration,
    similarities: f64,
}

impl<'a, S: CsvInfo> Comparison<'a, S> {
    pub fn new(columns: &'a S, rows: &'a S, matrix: Matrix<i32>, similarities: f64, duration: std::time::Duration) -> Comparison<'a, S> {
        Comparison { columns, rows, matrix, similarities, duration }
    }

    pub fn store<P: AsRef<Path>>(&self, dest: P) -> Result<()> {
        let mut file = std::fs::File::create(dest.as_ref())
            .map_err(|e| Error::Io(dest.as_ref().to_path_buf(), e))?;
        let mut out = std::io::BufWriter::new(&mut file);
        let _ = writeln!(out, "result,{},{}", self.duration.as_nanos(), self.similarities);
        let _ = writeln!(out, "left,{}", self.rows.csv_info());
        let _ = writeln!(out, "right,{}", self.columns.csv_info());
        let _ = writeln!(out, "matrix,,{}", self.columns.names().join(","));
        let row_names = self.rows.names();
        for ((row_idx, label), values) in row_names.iter().enumerate().zip(self.matrix.iter()) {
            let _ = writeln!(out, "{row_idx},{label},{}", values.iter().join(","));
        }
        Ok(())
    }

    pub fn similarity(&self) -> f64 {
        self.similarities
    }

    pub fn duration(&self) -> std::time::Duration {
        self.duration
    }
}

trait BirthmarkComparator {
    fn compare_birthmarks<'a>(&self, b1: &'a Birthmark, b2: &'a Birthmark) -> Comparison<'a, Birthmark> {
        if !b1.comparable_with(b2) {
            return Comparison::new(b1, b2, Matrix::new(0, 0, 0), 0.0, std::time::Duration::from_millis(0));
        }
        let p1_len = b1.elements.len();
        let p2_len = b2.elements.len();
        if p1_len == 0 && p2_len == 0 {
            Comparison::new(b1, b2, Matrix::new(0, 0, 0), 1.0, std::time::Duration::from_millis(0))
        } else if p1_len == 0 || p2_len == 0 {
            Comparison::new(b1, b2, Matrix::new(0, 0, 0), 0.0, std::time::Duration::from_millis(0))
        } else {
            let start = Instant::now();
            let r = build_birthmark_matrix(b1, b2, |e1, e2| {
                self.compare_elements(e1, e2)
            });
            let (total, matches) = if p1_len > p2_len {
                let transposed = r.transposed();
                pathfinding::kuhn_munkres::kuhn_munkres(&transposed)
            } else {
                pathfinding::kuhn_munkres::kuhn_munkres(&r)
            };
            let similarity = (total as f64 / 1000.0) / matches.len() as f64;
            Comparison::new(b1, b2, r, similarity, start.elapsed())
        }
    }

    fn compare_elements(&self, e1: &Elements, e2: &Elements) -> f64;
}

trait ProgramComparator<T: crate::Op> {
    fn compare_programs<'a>(&self, p1: &'a Program<T>, p2: &'a Program<T>) -> Comparison<'a, Program<T>> {
        let p1_len = p1.len();
        let p2_len = p2.len();
        if p1_len == 0 && p2_len == 0 {
            Comparison::new(p1, p2, Matrix::new(0, 0, 0), 1.0, std::time::Duration::from_millis(0))
        } else if p1_len == 0 || p2_len == 0 {
            Comparison::new(p1, p2, Matrix::new(0, 0, 0), 0.0, std::time::Duration::from_millis(0))
        } else {
            let start = Instant::now();
            let r = build_program_matrix(p1, p2, |f1, f2| {
                self.compare_func(f1, f2)
            });
            let (total, matches) = if p1_len > p2_len {
                let transposed = r.transposed();
                pathfinding::kuhn_munkres::kuhn_munkres(&transposed)
            } else {
                pathfinding::kuhn_munkres::kuhn_munkres(&r)
            };
            let similarity = (total as f64 / 1000.0) / matches.len() as f64;
            Comparison::new(p1, p2, r, similarity, start.elapsed())
        }
    }

    fn compare_func(&self, f1: &Function<T>, f2: &Function<T>) -> f64;
}

fn build_program_matrix<F, T>(p1: &Program<T>, p2: &Program<T>, compare_func: F) -> Matrix<i32>
where
    F: Fn(&Function<T>, &Function<T>) -> f64,
{
    let m = p1.len();
    let n = p2.len();
    let mut matrix = Matrix::new(m, n, 0);

    p1.iter().enumerate().for_each(|(i, f1)| {
        p2.iter().enumerate().for_each(|(j, f2)| {
            matrix[(i, j)] = (compare_func(f1, f2) * 1000.0) as i32;
        })
    });
    matrix
}

fn build_birthmark_matrix<F>(b1: &Birthmark, b2: &Birthmark, compare_func: F) -> Matrix<i32>
where
    F: Fn(&Elements, &Elements) -> f64,
{
    let m = b1.elements.len();
    let n = b2.elements.len();
    let mut matrix = Matrix::new(m, n, 0);

    b1.elements.iter().enumerate().for_each(|(i, e1)| {
        b2.elements.iter().enumerate().for_each(|(j, e2)| {
            matrix[(i, j)] = (compare_func(e1, e2) * 1000.0) as i32;
        })
    });
    matrix
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

pub enum Comparator {
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
            Algorithm::Cosine => Comparator::Cosine(Cosine{}),
            Algorithm::Dice => Comparator::Dice(Dice{}),
            Algorithm::Euclidean => Comparator::Euclidean(Euclidean{}),
            Algorithm::Jaccard => Comparator::Jaccard(Jaccard{}),
            Algorithm::Levenshtein => Comparator::Levenshtein(Levenshtein{}),
            Algorithm::Lcs => Comparator::Lcs(Lcs{}),
            Algorithm::Simpson => Comparator::Simpson(Simpson{}),
            Algorithm::WeightedJaccard => Comparator::WeightedJaccard(WeightedJaccard{}),
        }
    }
}

impl Comparator {
    pub fn compare_programs<'a, T: crate::Op>(&self, p1: &'a Program<T>, p2: &'a Program<T>) -> Comparison<'a, Program<T>> {
        match self {
            Comparator::Cosine(c) => c.compare_programs(p1, p2),
            Comparator::Dice(d) => d.compare_programs(p1, p2),
            Comparator::Euclidean(e) => e.compare_programs(p1, p2),
            Comparator::Jaccard(j) => j.compare_programs(p1, p2),
            Comparator::Levenshtein(l) => l.compare_programs(p1, p2),
            Comparator::Lcs(lcs) => lcs.compare_programs(p1, p2),
            Comparator::Simpson(s) => s.compare_programs(p1, p2),
            Comparator::WeightedJaccard(wj) => wj.compare_programs(p1, p2),
        }
    }

    pub fn compare_birthmarks<'a>(&self, b1: &'a Birthmark, b2: &'a Birthmark) -> Comparison<'a, Birthmark> {
        match self {
            Comparator::Cosine(c) => c.compare_birthmarks(b1, b2),
            Comparator::Dice(d) => d.compare_birthmarks(b1, b2),
            Comparator::Euclidean(e) => e.compare_birthmarks(b1, b2),
            Comparator::Jaccard(j) => j.compare_birthmarks(b1, b2),
            Comparator::Levenshtein(l) => l.compare_birthmarks(b1, b2),
            Comparator::Lcs(lcs) => lcs.compare_birthmarks(b1, b2),
            Comparator::Simpson(s) => s.compare_birthmarks(b1, b2),
            Comparator::WeightedJaccard(wj) => wj.compare_birthmarks(b1, b2),
        }
    }
}

pub struct Jaccard;
pub struct Dice;
pub struct Simpson;
pub struct Levenshtein;
pub struct Cosine;
pub struct Euclidean;
pub struct WeightedJaccard;
pub struct Lcs;

impl BirthmarkComparator for Jaccard {
    fn compare_elements(&self, e1: &Elements, e2: &Elements) -> f64 {
        if let (Data::Set(s1), Data::Set(s2)) = (&e1.data, &e2.data) {
            jaccard_index(s1, s2)
        } else if let (Data::KgramSet(k1), Data::KgramSet(k2)) = (&e1.data, &e2.data) {
            jaccard_index(k1, k2)
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

fn jaccard_index<T: std::cmp::Eq + std::hash::Hash>(s1: &FxHashSet<T>, s2: &FxHashSet<T>) -> f64 {
    if !s1.is_empty() && !s2.is_empty() {
        s1.intersection(s2).count() as f64 / s1.union(s2).count() as f64
    } else {
        1.0
    }
}

fn dice_index<T: std::cmp::Eq + std::hash::Hash>(s1: &FxHashSet<T>, s2: &FxHashSet<T>) -> f64 {
    if !s1.is_empty() && !s2.is_empty() {
        (2.0 * s1.intersection(s2).count() as f64) / (s1.len() + s2.len()) as f64
    } else {
        1.0
    }
}

fn simpson_index<T: std::cmp::Eq + std::hash::Hash>(s1: &FxHashSet<T>, s2: &FxHashSet<T>) -> f64 {
    if !s1.is_empty() && !s2.is_empty() {
        s1.intersection(s2).count() as f64 / s1.len().min(s2.len()) as f64
    } else {
        1.0
    }
}

fn levenshtein_distance<T: PartialEq>(s1: &[T], s2: &[T]) -> f64 {
    let mut matrix = pathfinding::matrix::Matrix::new(s1.len() + 1, s2.len() + 1, 0);
    for i in 0..=s1.len() {
        matrix[(i, 0)] = i as i32;
    }
    for j in 0..=s2.len() {
        matrix[(0, j)] = j as i32;
    }
    for i in 1..=s1.len() {
        for j in 1..=s2.len() {
            let cost = if s1[i - 1] == s2[j - 1] { 0 } else { 1 };
            matrix[(i, j)] = (matrix[(i - 1, j)] + 1)
                .min(matrix[(i, j - 1)] + 1)
                .min(matrix[(i - 1, j - 1)] + cost);
        }
    }
    1.0 - (matrix[(s1.len(), s2.len())] as f64 / (s1.len().max(s2.len()) as f64))
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
    let mut matrix = pathfinding::matrix::Matrix::new(s1.len() + 1, s2.len() + 1, 0);
    for i in 1..=s1.len() {
        for j in 1..=s2.len() {
            if s1[i - 1] == s2[j - 1] {
                matrix[(i, j)] = matrix[(i - 1, j - 1)] + 1;
            } else {
                matrix[(i, j)] = matrix[(i - 1, j)].max(matrix[(i, j - 1)]);
            }
        }
    }
    (matrix[(s1.len(), s2.len())] as f64) / (s1.len().max(s2.len()) as f64)
}

