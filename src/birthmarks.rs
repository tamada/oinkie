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

impl TryFrom<&str> for BirthmarkType {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self> {
        let s = s.to_lowercase();
        match s.as_str() {
            "fc-seq" => Ok(BirthmarkType::FcSeq),
            "fc-set" => Ok(BirthmarkType::FcSet),
            "fc-freq" => Ok(BirthmarkType::FcFreq),
            "op-freq" => Ok(BirthmarkType::OpFreq),
            "op-seq" => Ok(BirthmarkType::OpSeq),
            "op-set" => Ok(BirthmarkType::OpSet),
            _ => parse_kgram(&s).ok_or_else(|| Error::BirthmarkType(s)),
        }
    }
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
        let file = std::fs::File::open(&path)
            .map_err(|e| Error::Io(path.clone(), e))?;
        serde_json::from_reader(file)
            .map_err(|e| Error::Json(path, e))
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
        format!("birthmark,{},{},{},{},{}",
            crate::compare::escape_csv_string(&self.file_name),
            crate::compare::escape_csv_string(&self.path.display().to_string()),
            self.birthmark_type, self.extracted_at.to_rfc3339(), self.duration.as_nanos())
    }

    pub fn parse(line: &str) -> Result<Self> {
        let r = csv::ReaderBuilder::new()
            .flexible(true)
            .has_headers(false)
            .from_reader(line.as_bytes());
        if let Some(result) = r.into_records().next() {
            let record = result.map_err(Error::Csv)?;
            let items = record.iter().collect::<Vec<_>>();
            if items.len() != 6 {
                return Err(Error::Parse(format!("expected 6 items in metadata, got {}", items.len())));
            }
            let file_name = items[1].to_string();
            let path = PathBuf::from(items[2]);
            let birthmark_type: BirthmarkType = items[3].try_into()?;
            let extracted_at = chrono::DateTime::parse_from_rfc3339(items[4])
                .map_err(|e| Error::Parse(format!("invalid extracted_at datetime: {}", e)))?
                .with_timezone(&chrono::Utc);
            let duration = items[5].parse::<u64>()
                .map_err(|e| Error::Parse(format!("invalid duration: {}", e)))?;
            Ok(Self {
                file_name,
                path,
                birthmark_type,
                extracted_at,
                duration: Duration::from_nanos(duration),
            })
        } else {
            Err(Error::Parse(format!("invalid metadata line: {line}")))
        }
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
    #[serde(skip)]
    pub json_path: Option<PathBuf>,
}

impl CsvInfo for Birthmark {
    fn csv_info(&self) -> String {
        let json_path = self.json_path.as_ref().map(|p| p.display().to_string()).unwrap_or_default();
        format!("{},{}", self.metadata.csv_info(), crate::compare::escape_csv_string(&json_path))
    }

    fn names(&self) -> Vec<String> {
        self.elements.iter().map(|e| e.name.clone()).collect()
    }
}

impl Birthmark {
    pub fn set_json_path(&mut self, path: PathBuf) {
        self.json_path = Some(path);
    }

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Data {
    Freq(FxHashMap<String, usize>),
    Seq(Vec<String>),
    Set(FxHashSet<String>),
    KgramSeq(Vec<Kgram>),
    KgramFreq(FxHashMap<Kgram, usize>),
    KgramSet(FxHashSet<Kgram>),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
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
    let name = if let Some(strip_op) = name.strip_prefix("op-") {
        strip_op.to_string()
    } else {
        name.to_string()
    };
    if let Some(k) = name.strip_suffix("gram-seq") {
        parse_k_value(k).map(BirthmarkType::OpKgramSeq)
    } else if let Some(k) = name.strip_suffix("gram-set") {
        parse_k_value(k).map(BirthmarkType::OpKgramSet)
    } else if let Some(k) = name.strip_suffix("gram-freq") {
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
    use super::*;

    #[test]
    pub fn test_metadata_csv_info_roundtrip_with_comma_in_path() {
        let metadata = Metadata {
            file_name: "app, v1".to_string(),
            path: PathBuf::from("dir,with,commas/app"),
            extracted_at: chrono::Utc::now(),
            duration: Duration::from_nanos(42),
            birthmark_type: BirthmarkType::OpSeq,
        };
        let parsed = Metadata::parse(&metadata.csv_info())
            .expect("failed to parse escaped metadata");
        assert_eq!(parsed.file_name, metadata.file_name);
        assert_eq!(parsed.path, metadata.path);
        assert_eq!(parsed.birthmark_type, metadata.birthmark_type);
    }

    #[test]
    pub fn test_parse_metadata() {
        let metadata = Metadata::parse("birthmark,bzip2-1.0.2,${HOME}/oinkie/bzip2-1.0.2,op-seq,2026-04-17T04:57:55.385904+00:00,4462500")
            .expect("failed to parse metadata");
        assert_eq!(metadata.file_name, "bzip2-1.0.2");
        assert_eq!(metadata.path, PathBuf::from("${HOME}/oinkie/bzip2-1.0.2"));
        assert_eq!(metadata.birthmark_type, BirthmarkType::OpSeq);
        assert_eq!(metadata.extracted_at, chrono::DateTime::parse_from_rfc3339("2026-04-17T04:57:55.385904+00:00")
            .expect("failed to parse extracted_at")
            .with_timezone(&chrono::Utc));
        assert_eq!(metadata.duration, Duration::from_nanos(4462500));
    }

    #[test]
    fn test_parse_metadata_rejects_wrong_field_count() {
        let err = Metadata::parse("birthmark,name,path").unwrap_err();
        assert!(matches!(err, Error::Parse(msg) if msg.contains("expected 6 items")));
    }

    #[test]
    fn test_parse_metadata_rejects_empty_line() {
        assert!(Metadata::parse("").is_err());
    }

    #[test]
    fn test_parse_metadata_rejects_bad_datetime_and_duration() {
        assert!(Metadata::parse("birthmark,n,p,op-seq,not-a-date,1").is_err());
        assert!(Metadata::parse("birthmark,n,p,op-seq,2026-04-17T04:57:55+00:00,x").is_err());
    }

    #[test]
    fn test_birthmark_type_try_from_str() {
        let cases = [
            ("fc-seq", BirthmarkType::FcSeq),
            ("fc-set", BirthmarkType::FcSet),
            ("fc-freq", BirthmarkType::FcFreq),
            ("op-freq", BirthmarkType::OpFreq),
            ("op-seq", BirthmarkType::OpSeq),
            ("op-set", BirthmarkType::OpSet),
            ("op-3gram-seq", BirthmarkType::OpKgramSeq(3)),
            ("op-4gram-set", BirthmarkType::OpKgramSet(4)),
            ("op-5gram-freq", BirthmarkType::OpKgramFreq(5)),
        ];
        for (input, expected) in cases {
            assert_eq!(BirthmarkType::try_from(input).unwrap(), expected, "input: {input}");
            // parsing must be case insensitive
            assert_eq!(BirthmarkType::try_from(input.to_uppercase().as_str()).unwrap(), expected);
        }
    }

    #[test]
    fn test_birthmark_type_try_from_str_rejects_unknown() {
        for input in ["", "op-unknown", "op-xgram-seq", "op-3gram-unknown"] {
            assert!(BirthmarkType::try_from(input).is_err(), "input: {input}");
        }
    }

    #[test]
    fn test_birthmark_type_display_roundtrips() {
        let types = [
            BirthmarkType::FcSeq, BirthmarkType::FcSet, BirthmarkType::FcFreq,
            BirthmarkType::OpFreq, BirthmarkType::OpSeq, BirthmarkType::OpSet,
            BirthmarkType::OpKgramSeq(2), BirthmarkType::OpKgramFreq(3), BirthmarkType::OpKgramSet(4),
        ];
        for bt in types {
            let rendered = bt.to_string();
            assert_eq!(BirthmarkType::try_from(rendered.as_str()).unwrap(), bt, "rendered: {rendered}");
        }
    }

    #[test]
    fn test_analysis_type_try_from_named_combinations() {
        let names = [
            "op-freq-cosine", "op-set-dice", "op-freq-euclidean", "op-set-jaccard",
            "op-seq-levenshtein", "op-set-simpson", "op-freq-weightedjaccard",
        ];
        for name in names {
            assert!(AnalysisType::try_from(name).is_ok(), "name: {name}");
            // the String and &str impls must agree
            assert!(AnalysisType::try_from(name.to_string()).is_ok(), "name: {name}");
        }
    }

    #[test]
    fn test_analysis_type_try_from_kgram_combinations() {
        let cases = [
            ("op-2gram-set-jaccard", BirthmarkType::OpKgramSet(2)),
            ("op-3gram-set-dice", BirthmarkType::OpKgramSet(3)),
            ("op-3gram-set-simpson", BirthmarkType::OpKgramSet(3)),
            ("op-4gram-seq-levenshtein", BirthmarkType::OpKgramSeq(4)),
            ("op-5gram-freq-cosine", BirthmarkType::OpKgramFreq(5)),
            ("op-5gram-freq-euclidean", BirthmarkType::OpKgramFreq(5)),
            ("op-6gram-freq-weightedjaccard", BirthmarkType::OpKgramFreq(6)),
        ];
        for (name, expected) in cases {
            let at = AnalysisType::try_from(name)
                .unwrap_or_else(|e| panic!("{name}: {e}"));
            assert_eq!(at.birthmark, expected, "name: {name}");
        }
    }

    #[test]
    fn test_analysis_type_try_from_rejects_unknown() {
        for name in ["", "unknown", "op-2gram-set-unknown", "fc-set-jaccard-extra"] {
            assert!(AnalysisType::try_from(name).is_err(), "name: {name}");
        }
    }

    #[test]
    fn test_data_len_and_iter_for_every_variant() {
        let kgram = Kgram::new(vec!["A".to_string(), "B".to_string()]);
        let variants = [
            Data::Seq(vec!["A".to_string(), "B".to_string()]),
            Data::Set(["A".to_string(), "B".to_string()].into_iter().collect()),
            Data::Freq([("A".to_string(), 1), ("B".to_string(), 2)].into_iter().collect()),
            Data::KgramSeq(vec![kgram.clone()]),
            Data::KgramSet([kgram.clone()].into_iter().collect()),
            Data::KgramFreq([(kgram, 1)].into_iter().collect()),
        ];
        for data in variants {
            let elements = Elements { name: "f".to_string(), data };
            // every variant above carries two mnemonics in total
            assert_eq!(elements.ops().count(), 2);
            assert!(!elements.is_empty());
            assert_eq!(elements.name(), "f");
        }
    }

    #[test]
    fn test_elements_is_empty_on_empty_data() {
        let elements = Elements { name: "f".to_string(), data: Data::Seq(vec![]) };
        assert!(elements.is_empty());
        assert_eq!(elements.len(), 0);
        assert_eq!(elements.ops().count(), 0);
    }

    fn sample_birthmark() -> Birthmark {
        Birthmark {
            metadata: Metadata {
                file_name: "sample".to_string(),
                path: PathBuf::from("/tmp/sample"),
                extracted_at: chrono::Utc::now(),
                duration: Duration::from_nanos(7),
                birthmark_type: BirthmarkType::OpSeq,
            },
            elements: vec![Elements {
                name: "main".to_string(),
                data: Data::Seq(vec!["COPY".to_string()]),
            }],
            json_path: None,
        }
    }

    #[test]
    fn test_birthmark_accessors() {
        let mut b = sample_birthmark();
        assert_eq!(b.name(), "sample");
        assert_eq!(b.path(), Path::new("/tmp/sample"));
        assert_eq!(b.duration(), Duration::from_nanos(7));
        assert_eq!(b.birthmark_type(), &BirthmarkType::OpSeq);
        assert_eq!(b.len(), 1);
        assert!(!b.is_empty());
        assert!(b.extracted_at() <= chrono::Utc::now());

        // csv_info leaves the json path empty until it is set
        assert!(b.csv_info().ends_with(','));
        b.set_json_path(PathBuf::from("/tmp/sample.json"));
        assert!(b.csv_info().ends_with("/tmp/sample.json"));
        assert_eq!(b.names(), vec!["main".to_string()]);
    }

    #[test]
    fn test_birthmark_comparable_with() {
        let b1 = sample_birthmark();
        let mut b2 = sample_birthmark();
        assert!(b1.comparable_with(&b2));
        b2.metadata.birthmark_type = BirthmarkType::OpSet;
        assert!(!b1.comparable_with(&b2));
    }

    #[test]
    fn test_birthmark_iterable_for_value_and_reference() {
        // Iterable is implemented for both Birthmark and &Birthmark; going
        // through a generic function exercises each impl distinctly.
        fn count_elements<I: crate::Iterable>(iterable: I) -> usize {
            iterable.iter().count()
        }
        let b = sample_birthmark();
        assert_eq!(count_elements(&b), 1);
        assert_eq!(count_elements(b), 1);
    }

    #[test]
    fn test_birthmark_try_from_path_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("b.json");
        let b = sample_birthmark();
        std::fs::write(&path, serde_json::to_string(&b).unwrap()).unwrap();

        // both the PathBuf and the &Path impls must work
        let loaded = Birthmark::try_from(path.as_path()).expect("failed to load birthmark");
        assert_eq!(loaded.name(), b.name());
        assert_eq!(loaded.len(), b.len());
        assert!(Birthmark::try_from(path.clone()).is_ok());
    }

    #[test]
    fn test_birthmark_try_from_path_reports_errors() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("missing.json");
        assert!(matches!(Birthmark::try_from(missing).unwrap_err(), Error::Io(..)));

        let broken = dir.path().join("broken.json");
        std::fs::write(&broken, b"{ not json").unwrap();
        assert!(matches!(Birthmark::try_from(broken).unwrap_err(), Error::Json(..)));
    }
}