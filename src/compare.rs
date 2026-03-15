use crate::prelude::*;
use std::{io::Write, time::Instant};
use clap::ValueEnum;
use pathfinding::matrix::Matrix;
use rustc_hash::FxHashSet;
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

trait ComparatorTrait<T: crate::Op> {
    fn compare(&self, p1: &Program<T>, p2: &Program<T>, index: usize, context: &Context) -> f64 {
        let p1_len = p1.len();
        let p2_len = p2.len();
        if p1_len == 0 && p2_len == 0 {
            1.0
        } else if p1_len == 0 || p2_len == 0 {
            0.0
        } else {
            let (pp1, pp2) = if p1_len > p2_len {
                (p2, p1)
            } else {
                (p1, p2)
            };
            let start = Instant::now();
            let progress = context.sub_progressor("Comparing...", (p1_len * p2_len) as u64);
            let r = build_matrix(pp1, pp2, |f1, f2| {
                progress.inc();
                self.compare_func(f1, f2)
            });
            let duration = start.elapsed();
            output_matrix(&r, pp1, pp2, index, context, duration);
            let (total, matches) = pathfinding::kuhn_munkres::kuhn_munkres(&r);
            context.remove_progress(progress);
            total as f64 / matches.len() as f64
        }
    }

    fn compare_func(&self, f1: &Function<T>, f2: &Function<T>) -> f64;
}

fn output_matrix<T: crate::Op>(matrix: &Matrix<i32>, p1: &Program<T>, p2: &Program<T>, index: usize, context: &Context, duration: std::time::Duration) {
    if let Ok(mut file) = context.open_dest(index) {
        let mut out = std::io::BufWriter::new(&mut file);
        for col_idx in 0..matrix.columns {
            let _ = write!(out, ",{col_idx}");
        }
        let _ = writeln!(out);
        for f in p1.iter() {
            let _ = write!(out, ",{}", f.name());
        }
        let _ = writeln!(out);
        for f in p1.iter() {
            let _ = write!(out, ",{}", f.len());
        }
        let _ = writeln!(out);
        let row_labels = p2.iter().map(|f| f.name()).collect::<Vec<_>>();
        let col_labels = p1.iter().map(|f| f.name()).collect::<Vec<_>>();
        for (row_idx, row) in matrix.iter().enumerate() {
            let _ = write!(out, "{row_idx},{},{}", row_labels[row_idx], col_labels[row_idx]);
            for col in row.iter() {
                let _ = write!(out, ",{}", *col);
            }
            let _ = writeln!(out);
        }
        let _ = writeln!(out, "Duration,{}", duration.as_nanos());
    }
}

fn build_matrix<F, T>(p1: &Program<T>, p2: &Program<T>, compare_func: F) -> Matrix<i32>
where
    F: Fn(&Function<T>, &Function<T>) -> f64,
{
    let m = p1.len();
    let n = p2.len();
    let mut matrix = Matrix::new(m, n, 0);

    p1.iter().enumerate().for_each(|(i, f1)| {
        p2.iter().enumerate().for_each(|(j, f2)| {
            matrix[(i, j)] = (compare_func(f1, f2) * 100.0) as i32;
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

impl Comparator {
    pub fn compare<T: crate::Op>(&self, p1: &Program<T>, p2: &Program<T>, index: usize, context: &Context) -> f64 {
        match self {
            Comparator::Cosine(c) => c.compare(p1, p2, index, context),
            Comparator::Dice(d) => d.compare(p1, p2, index, context),
            Comparator::Euclidean(e) => e.compare(p1, p2, index, context),
            Comparator::Jaccard(j) => j.compare(p1, p2, index, context),
            Comparator::Levenshtein(l) => l.compare(p1, p2, index, context),
            Comparator::Lcs(lcs) => lcs.compare(p1, p2, index, context),
            Comparator::Simpson(s) => s.compare(p1, p2, index, context),
            Comparator::WeightedJaccard(wj) => wj.compare(p1, p2, index, context),
        }
    }
}

impl Algorithm {
    pub fn comparator(&self) -> Comparator {
        match self {
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

pub struct Jaccard;
pub struct Dice;
pub struct Simpson;
pub struct Levenshtein;
pub struct Cosine;
pub struct Euclidean;
pub struct WeightedJaccard;
pub struct Lcs;

impl<T: crate::Op> ComparatorTrait<T> for Jaccard {
    fn compare_func(&self, f1: &Function<T>, f2: &Function<T>) -> f64 {
        let set1: rustc_hash::FxHashSet<&str> = f1.ops().collect();
        let set2: rustc_hash::FxHashSet<&str> = f2.ops().collect();
        let intersection = set1.intersection(&set2).count() as f64;
        let union = set1.union(&set2).count() as f64;
        if union == 0.0 {
            1.0
        } else {
            intersection / union
        }
    }
}

impl<T: crate::Op> ComparatorTrait<T> for Dice {
    fn compare_func(&self, f1: &Function<T>, f2: &Function<T>) -> f64 {
        let set1: rustc_hash::FxHashSet<&str> = f1.ops().collect();
        let set2: rustc_hash::FxHashSet<&str> = f2.ops().collect();
        let intersection = set1.intersection(&set2).count() as f64;
        let total = (set1.len() + set2.len()) as f64;
        if total == 0.0 {
            1.0
        } else {
            (2.0 * intersection) / total
        }
    }
}

impl<T: crate::Op> ComparatorTrait<T> for Simpson {
    fn compare_func(&self, f1: &Function<T>, f2: &Function<T>) -> f64 {
        let set1: rustc_hash::FxHashSet<&str> = f1.ops().collect();
        let set2: rustc_hash::FxHashSet<&str> = f2.ops().collect();
        let intersection = set1.intersection(&set2).count() as f64;
        let smaller = set1.len().min(set2.len()) as f64;
        if smaller == 0.0 {
            1.0
        } else {
            intersection / smaller
        }
    }
}

impl<T: crate::Op> ComparatorTrait<T> for Levenshtein {
    fn compare_func(&self, f1: &Function<T>, f2: &Function<T>) -> f64 {
        let ops1: Vec<&str> = f1.ops().collect();
        let ops2: Vec<&str> = f2.ops().collect();
        let mut matrix = pathfinding::matrix::Matrix::new(ops1.len() + 1, ops2.len() + 1, 0);
        for i in 0..=ops1.len() {
            matrix[(i, 0)] = i as i32;
        }
        for j in 0..=ops2.len() {
            matrix[(0, j)] = j as i32;
        }
        for i in 1..=ops1.len() {
            for j in 1..=ops2.len() {
                let cost = if ops1[i - 1] == ops2[j - 1] { 0 } else { 1 };
                matrix[(i, j)] = (matrix[(i - 1, j)] + 1)
                    .min(matrix[(i, j - 1)] + 1)
                    .min(matrix[(i - 1, j - 1)] + cost);
            }
        }
        1.0 - (matrix[(ops1.len(), ops2.len())] as f64 / (ops1.len().max(ops2.len()) as f64))
    }
}

impl<T: crate::Op> ComparatorTrait<T> for Lcs {
    fn compare_func(&self, f1: &Function<T>, f2: &Function<T>) -> f64 {
        let ops1: Vec<&str> = f1.ops().collect();
        let ops2: Vec<&str> = f2.ops().collect();
        let mut matrix = pathfinding::matrix::Matrix::new(ops1.len() + 1, ops2.len() + 1, 0);
        for i in 1..=ops1.len() {
            for j in 1..=ops2.len() {
                if ops1[i - 1] == ops2[j - 1] {
                    matrix[(i, j)] = matrix[(i - 1, j - 1)] + 1;
                } else {
                    matrix[(i, j)] = matrix[(i - 1, j)].max(matrix[(i, j - 1)]);
                }
            }
        }
        (matrix[(ops1.len(), ops2.len())] as f64) / (ops1.len().max(ops2.len()) as f64)
    }
}

impl<T: crate::Op> ComparatorTrait<T> for Cosine {
    fn compare_func(&self, f1: &Function<T>, f2: &Function<T>) -> f64 {
        let map1 = f1.ops_freq();
        let map2 = f2.ops_freq();
        let keys = map1.keys().chain(map2.keys()).collect::<FxHashSet<_>>();
        let dot_product = keys.iter().map(|k| {
            let v1 = *map1.get(*k).unwrap_or(&0) as f64;
            let v2 = *map2.get(*k).unwrap_or(&0) as f64;
            v1 * v2
        }).sum::<f64>();
        let magnitude1 = map1.values().map(|v| (*v as f64).powi(2)).sum::<f64>().sqrt();
        let magnitude2 = map2.values().map(|v| (*v as f64).powi(2)).sum::<f64>().sqrt();
        if magnitude1 == 0.0 || magnitude2 == 0.0 {
            1.0
        } else {
            dot_product / (magnitude1 * magnitude2)
        }
    }
}

impl<T: crate::Op> ComparatorTrait<T> for Euclidean {
    /// Computes the Euclidean distance between two functions and converts it to a similarity score.
    /// The similarity is calculated as `exp(-distance)`, which maps a distance of 0 to a similarity of 1, and larger distances to values approaching 0.
    fn compare_func(&self, f1: &Function<T>, f2: &Function<T>) -> f64 {
        let map1 = f1.ops_freq();
        let map2 = f2.ops_freq();
        let keys = map1.keys().chain(map2.keys()).collect::<FxHashSet<_>>();
        let distance = keys.iter().map(|k| {
            let v1 = *map1.get(*k).unwrap_or(&0) as f64;
            let v2 = *map2.get(*k).unwrap_or(&0) as f64;
            (v1 - v2).powi(2)
        }).sum::<f64>().sqrt();
        (-distance).exp()
    }
}

impl<T: crate::Op> ComparatorTrait<T> for WeightedJaccard {
    fn compare_func(&self, f1: &Function<T>, f2: &Function<T>) -> f64 {
        let map1 = f1.ops_freq();
        let map2 = f2.ops_freq();
        let keys = map1.keys().chain(map2.keys()).collect::<FxHashSet<_>>();
        let intersection = keys.iter().map(|k| {
            let v1 = *map1.get(*k).unwrap_or(&0) as f64;
            let v2 = *map2.get(*k).unwrap_or(&0) as f64;
            v1.min(v2)
        }).sum::<f64>();
        let union = keys.iter().map(|k| {
            let v1 = *map1.get(*k).unwrap_or(&0) as f64;
            let v2 = *map2.get(*k).unwrap_or(&0) as f64;
            v1.max(v2)
        }).sum::<f64>();
        if union == 0.0 {
            1.0
        } else {
            intersection / union
        }
    }
}