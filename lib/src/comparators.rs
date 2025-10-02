use std::collections::{HashMap, HashSet};
use std::iter::zip;
use std::path::PathBuf;
use clap::{Parser, ValueEnum};
use serde::{Serialize, Deserialize};

use crate::birthmarks::{Birthmark, BirthmarkType, Element};
use crate::Result;

#[derive(Parser, Debug, Clone, PartialEq, Eq, Hash, ValueEnum)]
pub enum Type {
    #[clap(help = "Simpson's coefficient")]
    Simpson,
    #[clap(help = "Jaccard index")]
    Jaccard,
    #[clap(help = "Dice's coefficient")]
    Dice,
    #[clap(help = "Cosine similarity")]
    Cosine,
    #[clap(help = "Longest common subsequence")]
    LCS,
    #[clap(help = "Levenshtein distance (Edit distance)")]
    Levenshtein,
}

pub fn comparator(t: &Type) -> Box<dyn Comparator> {
    match t {
        Type::Simpson => Box::new(Simpson{}),
        Type::Jaccard => Box::new(Jaccard{}),
        Type::Dice => Box::new(Dice{}),
        Type::Cosine => Box::new(Cosine{}),
        Type::LCS => Box::new(LCS{}),
        Type::Levenshtein => Box::new(Levenshtein{}),
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Similarity {
    pub btype: BirthmarkType,
    pub a_path: PathBuf,
    pub b_path: PathBuf,
    pub score: f64,
}

pub trait Comparator {
    fn name(&self) -> &str;

    fn compare(&self, a: &Birthmark, b: &Birthmark) -> Result<Similarity> {
        let s = match (a.len(), b.len()) {
            (0, 0) => Ok(1.0),
            (0, _) | (_, 0) => Ok(0.0),
            _ => self.compare_impl(a, b),
        };
        s.map(|score| Similarity {
            btype: a.btype.clone(),
            a_path: a.path.clone(),
            b_path: b.path.clone(),
            score,
        })
    }

    fn compare_impl(&self, a: &Birthmark, b: &Birthmark) -> Result<f64>;
}

struct Simpson {
}

struct Jaccard {
}

struct Dice {
}

struct Cosine {
}

struct LCS {
}

struct Levenshtein {
}

impl Comparator for Simpson {
    fn name(&self) -> &str {
        "Simpson"
    }

    fn compare_impl(&self, a: &Birthmark, b: &Birthmark) -> Result<f64> {
        Ok(zip(a.iter(), b.iter())
                .filter(|(a, b)| a.is_same(b))
                .count() as f64
                / a.len().min(b.len()) as f64)
    }
}

impl Comparator for Jaccard {
    fn name(&self) -> &str {
        "Jaccard"
    }

    fn compare_impl(&self, a: &Birthmark, b: &Birthmark) -> Result<f64> {
        Ok(zip(a.iter(), b.iter())
                .filter(|(a, b)| a.is_same(b))
                .count() as f64
                / (a.len() + b.len() - zip(a.iter(), b.iter())
                    .filter(|(a, b)| a.is_same(b))
                    .count()) as f64)
    }
}

impl Comparator for Dice{
    fn name(&self) -> &str {
        "Dice"
    }

    fn compare_impl(&self, a: &Birthmark, b: &Birthmark) -> Result<f64> {
        Ok(2.0 * zip(a.iter(), b.iter())
                .filter(|(a, b)| a.is_same(b))
                .count() as f64
                / (a.len() + b.len()) as f64)
    }   
}

impl Comparator for Cosine {
    fn name(&self) -> &str {
        "Cosine"
    }

    fn compare_impl(&self, a: &Birthmark, b: &Birthmark) -> Result<f64> {
        let m1 = a.freq();
        let m2 = b.freq();
        let keys = merge_keys(&m1, &m2);

        let dot_product = keys.iter()
            .map(|k| m1.get(k).unwrap_or(&0) * m2.get(k).unwrap_or(&0))
            .sum::<usize>() as f64;
        let magnitude_a = (m1.values().map(|v| v * v).sum::<usize>() as f64).sqrt();
        let magnitude_b = (m2.values().map(|v| v * v).sum::<usize>() as f64).sqrt();
        Ok(dot_product / (magnitude_a * magnitude_b))
    }
}

fn merge_keys<'a>(m1: &'a HashMap<&'a Element, usize>, m2: &'a HashMap<&'a Element, usize>) -> HashSet<&'a Element> {
    let mut keys = HashSet::new();
    for k in m1.keys() {
        keys.insert(*k);
    }
    for k in m2.keys() {
        keys.insert(*k);
    }
    keys
}

impl Comparator for LCS {
    fn name(&self) -> &str {
        "LCS"
    }

    fn compare_impl(&self, a: &Birthmark, b: &Birthmark) -> Result<f64> {
        let len_a = a.len();
        let len_b = b.len();
        let lcs_len = {
            let mut dp = vec![vec![0; len_b + 1]; len_a + 1];
            for i in 1..=len_a {
                for j in 1..=len_b {
                    if a.iter().nth(i - 1).unwrap().is_same(b.iter().nth(j - 1).unwrap()) {
                        dp[i][j] = dp[i - 1][j - 1] + 1;
                    } else {
                        dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
                    }
                }
            }
            dp[len_a][len_b]
        };
        Ok(lcs_len as f64 / len_a.max(len_b) as f64)
    }
}

impl Comparator for Levenshtein {
    fn name(&self) -> &str {
        "Levenshtein"
    }

    fn compare_impl(&self, a: &Birthmark, b: &Birthmark) -> Result<f64> {
        let len_a = a.len();
        let len_b = b.len();
        let dist = edit_distance(a, b);
        Ok(1.0 - dist as f64 / len_a.max(len_b) as f64)
    }
}

fn edit_distance(a: &Birthmark, b: &Birthmark) -> usize {
    let len_a = a.len();
    let len_b = b.len();
    let mut dp = vec![vec![0; len_b + 1]; len_a + 1];

    for i in 0..=len_a {
        dp[i][0] = i;
    }
    for j in 0..=len_b {
        dp[0][j] = j;
    }

    for i in 1..=len_a {
        for j in 1..=len_b {
            let cost = if a.iter().nth(i - 1).unwrap().is_same(b.iter().nth(j - 1).unwrap()) {
                0
            } else {
                1
            };
            dp[i][j] = *[
                dp[i - 1][j] + 1,     // Deletion
                dp[i][j - 1] + 1,     // Insertion
                dp[i - 1][j - 1] + cost, // Substitution
            ]
            .iter()
            .min()
            .unwrap();
        }
    }

    dp[len_a][len_b]
}