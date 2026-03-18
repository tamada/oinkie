use std::{fmt::Display, path::{Path, PathBuf}};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::time::Duration;
use rustc_hash::{FxHashMap, FxHashSet};
use crate::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
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

impl Display for BirthmarkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BirthmarkType::FcSeq => write!(f, "fc-seq"),
            BirthmarkType::FcSet => write!(f, "fc-set"),
            BirthmarkType::FcFreq => write!(f, "fc-freq"),
            BirthmarkType::OpFreq => write!(f, "op-freq"),
            BirthmarkType::OpSeq => write!(f, "op-seq"),
            BirthmarkType::OpSet => write!(f, "op-set"),
            BirthmarkType::OpKgramSeq(k) => write!(f, "op-{k}gram-seq"),
            BirthmarkType::OpKgramFreq(k) => write!(f, "op-{k}gram-freq"),
            BirthmarkType::OpKgramSet(k) => write!(f, "op-{k}gram-set"),
        }
    }
}

impl TryFrom<PathBuf> for Birthmark {
    type Error = Error;

    fn try_from(path: PathBuf) -> Result<Self> {
        let file = std::fs::File::open(&path).map_err(|e| Error::Io(path, e))?;
        serde_json::from_reader(file).map_err(Error::Json)
    }
}

impl TryFrom<&Path> for Birthmark {
    type Error = Error;

    fn try_from(path: &Path) -> Result<Self> {
        Self::try_from(path.to_path_buf())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Metadata {
    pub file_name: String,
    pub path: PathBuf,
    pub extracted_at: chrono::DateTime<chrono::Utc>,
    /// Represents the time taken to extract the birthmark, measured in nanoseconds for precision.
    #[serde(
        serialize_with = "serialize_duration_as_nanos",
        deserialize_with = "deserialize_duration_from_nanos",
    )]
    pub duration: std::time::Duration,
    pub birthmark_type: BirthmarkType,
}

impl Metadata {
    pub fn csv_info(&self) -> String {
        format!("birthmark,{},{},{},{},{}", self.file_name, self.path.display(), self.birthmark_type, self.extracted_at.to_rfc3339(), self.duration.as_nanos())
    }
}

fn serialize_duration_as_nanos<S>(duration: &std::time::Duration, serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let nanos = duration.as_nanos();
    serializer.serialize_u64(nanos as u64)
}

fn deserialize_duration_from_nanos<'de, D>(d: D) -> std::result::Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let nanos = u64::deserialize(d)?;
    Ok(Duration::from_nanos(nanos))
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Birthmark {
    pub metadata: Metadata,
    pub elements: Vec<Elements>,
}

impl CsvInfo for Birthmark {
    fn csv_info(&self) -> String {
        self.metadata.csv_info()
    }

    fn names(&self) -> Vec<String> {
        self.elements.iter().map(|e| e.name.clone()).collect()
    }
}

impl Birthmark {
    pub fn comparable_with(&self, other: &Birthmark) -> bool {
        self.metadata.birthmark_type == other.metadata.birthmark_type
    }

    pub fn name(&self) -> &str {
        &self.metadata.file_name
    }

    pub fn path(&self) -> &Path {
        &self.metadata.path
    }

    pub fn extracted_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.metadata.extracted_at
    }

    pub fn duration(&self) -> std::time::Duration {
        self.metadata.duration
    }

    pub fn birthmark_type(&self) -> &BirthmarkType {
        &self.metadata.birthmark_type
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }

    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    // pub fn iter(&self) -> impl Iterator<Item = &Elements> {
    //     self.elements.iter()
    // }
}

impl crate::Iterable for &Birthmark {
    type Item = Elements;
    fn iter(&self) -> Box<dyn Iterator<Item = &Self::Item> + '_> {
        Box::new(self.elements.iter())
    }
}

impl crate::Iterable for Birthmark {
    type Item = Elements;
    fn iter(&self) -> Box<dyn Iterator<Item = &Self::Item> + '_> {
        Box::new(self.elements.iter())
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

    pub fn is_empty(&self) -> bool {
        self.data.len() == 0
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Data {
    Freq(FxHashMap<String, usize>),
    Seq(Vec<String>),
    Set(FxHashSet<String>),
    KgramSeq(Vec<Kgram>),
    KgramFreq(FxHashMap<Kgram, usize>),
    KgramSet(FxHashSet<Kgram>),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub struct Kgram(Vec<String>);

impl Kgram {
    pub fn new(seq: Vec<String>) -> Self {
        Self(seq)
    }
}

impl Data {
    fn iter(&self) -> Box<dyn Iterator<Item = &str> + '_> {
        match self {
            Data::Freq(freq) => Box::new(freq.keys().map(|s| s.as_str())),
            Data::Seq(seq) => Box::new(seq.iter().map(|s| s.as_str())),
            Data::Set(set) => Box::new(set.iter().map(|s| s.as_str())),
            Data::KgramSeq(seq) => Box::new(seq.iter().flat_map(|k| k.0.iter().map(|s| s.as_str()))),
            Data::KgramFreq(freq) => Box::new(freq.keys().flat_map(|k| k.0.iter().map(|s| s.as_str()))),
            Data::KgramSet(set) => Box::new(set.iter().flat_map(|k| k.0.iter().map(|s| s.as_str()))),
        }
    }

    fn len(&self) -> usize {
        match self {
            Data::Freq(freq) => freq.len(),
            Data::Seq(seq) => seq.len(),
            Data::Set(set) => set.len(),
            Data::KgramSeq(seq) => seq.len(),
            Data::KgramFreq(freq) => freq.len(),
            Data::KgramSet(set) => set.len(),
        }
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