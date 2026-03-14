use std::path::{Path, PathBuf};

use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};
use crate::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum BirthmarkType {
    FcSeq,
    FcSet,
    FcFreq,
    OpFreq,
    OpSeq,
    OpSet,
    OpKgramSeq(usize),
    OpKgramFreq(usize),
    OpKgramSet(usize),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Birthmark {
    pub name: String,
    pub path: PathBuf,
    pub birthmark_type: BirthmarkType,
    pub elements: Vec<Elements>,
}

impl Birthmark {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Elements> {
        self.elements.iter()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Elements {
    pub name: String,
    pub data: Data,
}

impl Elements {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn ops(&self) -> impl Iterator<Item = &str> {
        self.data.iter()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }
}

pub struct AnalysisType {
    pub birthmark: BirthmarkType,
    pub comparator: Comparator,
}

impl AnalysisType {
    pub fn new(bt: BirthmarkType, algorithm: Algorithm) -> Self {
        Self {
            birthmark: bt,
            comparator: algorithm.comparator(),
        }
    }

    pub fn comparator(&self) -> &Comparator {
        &self.comparator
    }
}

impl TryFrom<&str> for AnalysisType {
    type Error = Error;

    fn try_from(name: &str) -> Result<Self> {
        Self::try_from(name.to_string())
    }
}

impl TryFrom<String> for AnalysisType {
    type Error = Error;

    fn try_from(name: String) -> Result<Self> {
        let s = name.to_lowercase();
        if s == "op-freq-cosine" {
            Ok(AnalysisType::new(BirthmarkType::OpFreq, Algorithm::Cosine))
        } else if s == "op-set-dice" {
            Ok(AnalysisType::new(BirthmarkType::OpSet, Algorithm::Dice))
        } else if s == "op-freq-euclidean" {
            Ok(AnalysisType::new(BirthmarkType::OpFreq, Algorithm::Euclidean))
        } else if s == "op-set-jaccard" {
            Ok(AnalysisType::new(BirthmarkType::OpSet, Algorithm::Jaccard))
        } else if s == "op-seq-levenshtein" {
            Ok(AnalysisType::new(BirthmarkType::OpSeq, Algorithm::Levenshtein))
        } else if s == "op-set-simpson" {
            Ok(AnalysisType::new(BirthmarkType::OpSet, Algorithm::Simpson))
        } else if s == "op-freq-weightedjaccard" {
            Ok(AnalysisType::new(BirthmarkType::OpFreq, Algorithm::WeightedJaccard))
        } else if let Some((bt, c)) = parse_kgram_and_algorithm(s) {
            Ok(AnalysisType::new(bt, c))
        } else {
            Err(Error::BirthmarkType(name))
        }

    }
}

fn parse_kgram_and_algorithm(name: String) -> Option<(BirthmarkType, Algorithm)> {
    if let Some(s) = name.strip_prefix("op-") {
        let s = s.to_string();
        if let Some(s) = s.strip_suffix("-cosine") {
            parse_kgram(s).map(|bt| (bt, Algorithm::Cosine))
        } else if let Some(s) = s.strip_suffix("-dice") {
            parse_kgram(s).map(|bt| (bt, Algorithm::Dice))
        } else if let Some(s) = s.strip_suffix("-euclidean") {
            parse_kgram(s).map(|bt| (bt, Algorithm::Euclidean))
        } else if let Some(s) = s.strip_suffix("-jaccard") {
            parse_kgram(s).map(|bt| (bt, Algorithm::Jaccard))
        } else if let Some(s) = s.strip_suffix("-levenshtein") {
            parse_kgram(s).map(|bt| (bt, Algorithm::Levenshtein))
        } else if let Some(s) = s.strip_suffix("-simpson") {
            parse_kgram(s).map(|bt| (bt, Algorithm::Simpson))
        } else if let Some(s) = s.strip_suffix("-weightedjaccard") {
            parse_kgram(s).map(|bt| (bt, Algorithm::WeightedJaccard))
        } else {
            None
        }
    } else {
        None
    }
}

fn parse_kgram(name: &str) -> Option<BirthmarkType> {
    if let Some(k) = name.strip_suffix("gramseq") {
        parse_k_value(k).map(BirthmarkType::OpKgramSeq)
    } else if let Some(k) = name.strip_suffix("gramset") {
        parse_k_value(k).map(BirthmarkType::OpKgramSet)
    } else if let Some(k) = name.strip_suffix("gramfreq") {
        parse_k_value(k).map(BirthmarkType::OpKgramFreq)
    } else {
        None
    }
}

fn parse_k_value(k: &str) -> Option<usize> {
    k.parse()
        .ok()
}

#[cfg(test)]
mod tests {
    #[test]
    pub fn test_parse_analysis_type() {

    }
}