use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

use crate::prelude::*;
use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::time::Duration;

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

/// How far the completion and help lists go for k-grams.
///
/// The parser has no ceiling — `op-9gram-set-dice` is a valid name and always
/// was. This bounds only what gets *suggested*, because a list of completions
/// has to be finite, and it is set where `extract --birthmark-type` already
/// stopped.
pub const MAX_ADVERTISED_K: usize = 8;

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

impl BirthmarkType {
    /// The birthmark names the command line offers: every family, with
    /// k-grams up to [`MAX_ADVERTISED_K`].
    ///
    /// This is what completion and `--help` list, not what the parser accepts
    /// — [`BirthmarkType::try_from`] takes any k. Generated rather than
    /// written out because the CLI used to hand-list these twice, at two
    /// different ceilings and in a third spelling (#25).
    pub fn advertised() -> impl Iterator<Item = BirthmarkType> {
        [
            BirthmarkType::FcSeq,
            BirthmarkType::FcFreq,
            BirthmarkType::FcSet,
            BirthmarkType::OpSeq,
            BirthmarkType::OpSet,
            BirthmarkType::OpFreq,
        ]
        .into_iter()
        .chain((1..=MAX_ADVERTISED_K).map(BirthmarkType::OpKgramSeq))
        .chain((1..=MAX_ADVERTISED_K).map(BirthmarkType::OpKgramFreq))
        .chain((1..=MAX_ADVERTISED_K).map(BirthmarkType::OpKgramSet))
    }

    /// One line saying what this birthmark holds, for `oinkie info` and for
    /// the help text of `--birthmark-type`.
    ///
    /// Phrased without a trailing full stop, as clap renders a short help.
    pub fn description(&self) -> String {
        match self {
            BirthmarkType::FcSeq => "the sequence of method calls in a program".to_string(),
            BirthmarkType::FcFreq => "the frequency of method calls in a program".to_string(),
            BirthmarkType::FcSet => "the set of method calls in a program".to_string(),
            BirthmarkType::OpSeq => "the sequence of operations in a program".to_string(),
            BirthmarkType::OpSet => "the set of operations in a program".to_string(),
            BirthmarkType::OpFreq => "the frequency of operations in a program".to_string(),
            BirthmarkType::OpKgramSeq(k) => {
                format!("the sequence of {k}-grams of operations in a program")
            }
            BirthmarkType::OpKgramFreq(k) => {
                format!("the frequency of {k}-grams of operations in a program")
            }
            BirthmarkType::OpKgramSet(k) => {
                format!("the set of {k}-grams of operations in a program")
            }
        }
    }

    pub fn shape(&self) -> Shape {
        match self {
            BirthmarkType::FcSeq | BirthmarkType::OpSeq | BirthmarkType::OpKgramSeq(_) => {
                Shape::Seq
            }
            BirthmarkType::FcSet | BirthmarkType::OpSet | BirthmarkType::OpKgramSet(_) => {
                Shape::Set
            }
            BirthmarkType::FcFreq | BirthmarkType::OpFreq | BirthmarkType::OpKgramFreq(_) => {
                Shape::Freq
            }
        }
    }

    /// Whether this birthmark's shape is the one the algorithm computes over.
    /// Anything else is converted by the comparator first, so a pairing that
    /// does not match either reproduces another pairing's numbers under a
    /// misleading name or scores nothing at all.
    pub fn pairs_with(&self, algorithm: &Algorithm) -> bool {
        self.shape() == algorithm.shape()
    }

    /// The same birthmark family in another shape, used to name the pairing a
    /// rejected analysis should have used.
    pub fn with_shape(&self, shape: Shape) -> BirthmarkType {
        match (self, shape) {
            (BirthmarkType::FcSeq | BirthmarkType::FcSet | BirthmarkType::FcFreq, s) => match s {
                Shape::Seq => BirthmarkType::FcSeq,
                Shape::Set => BirthmarkType::FcSet,
                Shape::Freq => BirthmarkType::FcFreq,
            },
            (
                BirthmarkType::OpKgramSeq(k)
                | BirthmarkType::OpKgramSet(k)
                | BirthmarkType::OpKgramFreq(k),
                s,
            ) => match s {
                Shape::Seq => BirthmarkType::OpKgramSeq(*k),
                Shape::Set => BirthmarkType::OpKgramSet(*k),
                Shape::Freq => BirthmarkType::OpKgramFreq(*k),
            },
            (_, s) => match s {
                Shape::Seq => BirthmarkType::OpSeq,
                Shape::Set => BirthmarkType::OpSet,
                Shape::Freq => BirthmarkType::OpFreq,
            },
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

    /// Reads the file whole, then parses the bytes.
    ///
    /// `from_reader` on a bare `File` looks like the obvious thing and is a
    /// trap: serde_json's `IoRead` takes one byte at a time, so an unbuffered
    /// file is one `read` syscall per byte. On a 9 MB birthmark that is 2.7 s
    /// against 15 ms here, and `compare` loads its inputs once per *pair*, so
    /// the loader runs O(n²) times (#51).
    ///
    /// `from_slice` rather than `read_to_string` + `from_str`: bytes that are
    /// not UTF-8 are the file's content being wrong, and stay a JSON error
    /// rather than becoming an IO one.
    fn try_from(path: PathBuf) -> Result<Self> {
        let bytes = std::fs::read(&path).map_err(|e| Error::Io(path.clone(), e))?;
        serde_json::from_slice(&bytes).map_err(|e| Error::Json(path, e))
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
        deserialize_with = "deserialize_duration_from_nanos"
    )]
    pub duration: std::time::Duration,
    pub birthmark_type: BirthmarkType,
    /// The representation the program was lifted to, carried over so that two
    /// birthmarks can be told apart by more than their type.
    ///
    /// Defaulted as a fallback for the same reason the field on `Program` is:
    /// every birthmark this crate wrote before the field existed came from
    /// Ghidra's P-Code, since no other lifter has existed. It is not a
    /// deduction — a file that simply omits the field reads the same way —
    /// but anything written since carries it.
    #[serde(default)]
    pub ir: crate::lift::Ir,
}

impl Metadata {
    pub fn csv_info(&self) -> String {
        format!(
            "birthmark,{},{},{},{},{},{}",
            crate::compare::escape_csv_string(&self.file_name),
            crate::compare::escape_csv_string(&self.path.display().to_string()),
            self.birthmark_type,
            self.extracted_at.to_rfc3339(),
            self.duration.as_nanos(),
            self.ir
        )
    }

    pub fn parse(line: &str) -> Result<Self> {
        let r = csv::ReaderBuilder::new()
            .flexible(true)
            .has_headers(false)
            .from_reader(line.as_bytes());
        if let Some(result) = r.into_records().next() {
            let record = result.map_err(Error::Csv)?;
            let items = record.iter().collect::<Vec<_>>();
            // Six is a record written before the ir field existed; seven is
            // one written since. Both are read, for the same reason the JSON
            // field is defaulted.
            if items.len() != 6 && items.len() != 7 {
                return Err(Error::Parse(format!(
                    "expected 6 or 7 items in metadata, got {}",
                    items.len()
                )));
            }
            let file_name = items[1].to_string();
            let path = PathBuf::from(items[2]);
            let birthmark_type: BirthmarkType = items[3].try_into()?;
            let extracted_at = chrono::DateTime::parse_from_rfc3339(items[4])
                .map_err(|e| Error::Parse(format!("invalid extracted_at datetime: {}", e)))?
                .with_timezone(&chrono::Utc);
            let duration = items[5]
                .parse::<u64>()
                .map_err(|e| Error::Parse(format!("invalid duration: {}", e)))?;
            // Ir::from_str already reports what was wrong and in this
            // Error type, so wrapping it again would only prefix a second
            // "Parse error:" onto the same sentence.
            let ir = match items.get(6) {
                Some(text) => text.parse()?,
                None => crate::lift::Ir::default(),
            };
            Ok(Self {
                file_name,
                path,
                birthmark_type,
                extracted_at,
                duration: Duration::from_nanos(duration),
                ir,
            })
        } else {
            Err(Error::Parse(format!("invalid metadata line: {line}")))
        }
    }
}

fn serialize_duration_as_nanos<S>(
    duration: &std::time::Duration,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error>
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
        let json_path = self
            .json_path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_default();
        format!(
            "{},{}",
            self.metadata.csv_info(),
            crate::compare::escape_csv_string(&json_path)
        )
    }

    fn names(&self) -> Vec<String> {
        self.elements.iter().map(|e| e.name.clone()).collect()
    }
}

impl Birthmark {
    pub fn set_json_path(&mut self, path: PathBuf) {
        self.json_path = Some(path);
    }

    /// Explains why these two cannot be compared, when they cannot.
    ///
    /// A birthmark is only meaningful against another built the same way. The
    /// type has always had to match; the representation now does too, because
    /// two lifters describe the same instruction with different operations,
    /// and comparing across them measures the disagreement between the tools
    /// rather than anything about the programs.
    ///
    /// The `fc-*` family is the one that could eventually cross this line: it
    /// holds symbol names read from the binary rather than operations read
    /// from the IR, so two tools should recover much the same set. It is
    /// refused all the same, because the spellings still differ between tools
    /// (`_printf` against `printf`) and nothing normalises them yet. Relaxing
    /// this is worth doing once that normalisation exists and a second lifter
    /// can be used to measure whether the result is worth trusting.
    pub fn check_comparable_with(&self, other: &Birthmark) -> Result<()> {
        if self.metadata.ir != other.metadata.ir {
            return Err(Error::IrMismatch(self.metadata.ir, other.metadata.ir));
        }
        if self.metadata.birthmark_type != other.metadata.birthmark_type {
            return Err(Error::Mismatch(
                self.metadata.birthmark_type.clone(),
                other.metadata.birthmark_type.clone(),
            ));
        }
        Ok(())
    }

    pub fn comparable_with(&self, other: &Birthmark) -> bool {
        self.check_comparable_with(other).is_ok()
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
    /// An operation named more than once is refused, for the reason
    /// [`Data::KgramFreq`] gives: a count the file states twice is ambiguous,
    /// and serde's derived map deserializer takes the last one without saying
    /// so (#66).
    Freq(
        #[serde(serialize_with = "sorted_freq", deserialize_with = "no_repeated_key")]
        FxHashMap<String, usize>,
    ),
    Seq(Vec<String>),
    Set(#[serde(serialize_with = "sorted_set")] FxHashSet<String>),
    KgramSeq(Vec<Kgram>),
    /// Written as a list of `[kgram, count]` pairs rather than as a JSON
    /// object.
    ///
    /// A JSON object's keys are strings, and a k-gram is a list of
    /// operations. `serde_json` refuses it — "key must be a string" — so
    /// every non-empty `op-*gram-freq` birthmark failed at write time.
    /// `op-1gram-freq` through `op-8gram-freq` — eight of the thirty
    /// birthmark types the CLI advertises, and one of the three shapes a
    /// k-gram comes in — could not produce a file at all (#59). It looked k-dependent only because a program with fewer than k
    /// operations yields an empty map, and an empty map has no key to refuse.
    ///
    /// A list of pairs rather than a stringified key, because the alternative
    /// is to make `Kgram` serialize as a string, and `KgramSeq` and
    /// `KgramSet` already write it as a list — those two families work, and
    /// their files would stop reading. It also needs no separator, so no
    /// mnemonic can ever contain one.
    KgramFreq(#[serde(with = "kgram_freq")] FxHashMap<Kgram, usize>),
    KgramSet(#[serde(serialize_with = "sorted_kgram_set")] FxHashSet<Kgram>),
}

/// A map and a set have no order of their own, so the one they are written in
/// has to come from somewhere. Hash order comes from the insertion history and
/// the hasher, which means two equal birthmarks can be written differently and
/// a `rustc-hash` bump relays every file for no reason. Sorted, the bytes are
/// a function of what the birthmark holds (#68).
///
/// `Seq` and `KgramSeq` are not here on purpose. Their order is the program's,
/// and sorting them would not canonicalise the file — it would destroy the
/// birthmark.
fn sorted_freq<S>(map: &FxHashMap<String, usize>, s: S) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut pairs = map.iter().collect::<Vec<_>>();
    pairs.sort_unstable_by_key(|(name, _)| *name);
    s.collect_map(pairs)
}

/// See [`sorted_freq`].
fn sorted_set<S>(set: &FxHashSet<String>, s: S) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut items = set.iter().collect::<Vec<_>>();
    items.sort_unstable();
    s.collect_seq(items)
}

/// See [`sorted_freq`].
fn sorted_kgram_set<S>(set: &FxHashSet<Kgram>, s: S) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut items = set.iter().collect::<Vec<_>>();
    items.sort_unstable_by(|a, b| a.0.cmp(&b.0));
    s.collect_seq(items)
}

/// Reads a frequency map, refusing a key that appears twice.
///
/// serde's derived deserializer inserts in a loop, so `{"COPY": 3, "COPY": 5}`
/// loads as 5 and nothing is said — a similarity computed from a count the
/// file contradicts itself about. Nothing this crate writes can produce one,
/// since it serializes from a map, so refusing costs no real file anything
/// (#66).
///
/// Only the frequency shapes need this. A repeated element in a `Set` denotes
/// the same set, and a repeated element in a `Seq` is what a sequence is for
/// — neither is ambiguous. The problem is the contradiction, not the
/// repetition.
fn no_repeated_key<'de, D>(d: D) -> std::result::Result<FxHashMap<String, usize>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::{Error as _, MapAccess, Visitor};

    struct Frequencies;

    impl<'de> Visitor<'de> for Frequencies {
        type Value = FxHashMap<String, usize>;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("a map of operations to how often each occurs")
        }

        fn visit_map<M>(self, mut entries: M) -> std::result::Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut map = FxHashMap::default();
            while let Some((name, count)) = entries.next_entry::<String, usize>()? {
                if map.contains_key(&name) {
                    return Err(M::Error::custom(format!(
                        "{name}: listed more than once, so its frequency is ambiguous"
                    )));
                }
                map.insert(name, count);
            }
            Ok(map)
        }
    }

    d.deserialize_map(Frequencies)
}

/// `KgramFreq` as a list of pairs. See the variant for why.
mod kgram_freq {
    use super::Kgram;
    use rustc_hash::FxHashMap;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub(super) fn serialize<S>(map: &FxHashMap<Kgram, usize>, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Sorted, so that extracting the same program twice writes the same
        // bytes. A hash map's order is not stable across runs, and a
        // birthmark file is something people diff and cache.
        let mut pairs = map.iter().collect::<Vec<_>>();
        pairs.sort_unstable_by(|(a, _), (b, _)| a.0.cmp(&b.0));
        pairs.serialize(s)
    }

    /// A repeated k-gram is refused rather than resolved.
    ///
    /// The list is a map on disk, and collecting it would let the last pair
    /// win silently — a file saying a k-gram occurred 3 times and again 5
    /// times would load as 5, with a similarity computed from it and nothing
    /// said. Nothing this crate writes can produce a duplicate, since it
    /// serializes from a map, so a file holding one has been hand-edited or
    /// corrupted, and guessing which count was meant is worse than stopping.
    pub(super) fn deserialize<'de, D>(d: D) -> Result<FxHashMap<Kgram, usize>, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error as _;
        let mut map = FxHashMap::default();
        for (kgram, count) in Vec::<(Kgram, usize)>::deserialize(d)? {
            if map.contains_key(&kgram) {
                return Err(D::Error::custom(format!(
                    "[{}]: listed more than once, so its frequency is ambiguous",
                    kgram.0.join(", ")
                )));
            }
            map.insert(kgram, count);
        }
        Ok(map)
    }
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
            Data::KgramSeq(seq) => {
                Box::new(seq.iter().flat_map(|k| k.0.iter().map(|s| s.as_str())))
            }
            Data::KgramFreq(freq) => {
                Box::new(freq.keys().flat_map(|k| k.0.iter().map(|s| s.as_str())))
            }
            Data::KgramSet(set) => {
                Box::new(set.iter().flat_map(|k| k.0.iter().map(|s| s.as_str())))
            }
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
    /// Fails when the algorithm does not operate on the birthmark's shape.
    /// Validating here rather than only in `try_from` means the check cannot
    /// be walked around by constructing the pair directly.
    pub fn new(bt: BirthmarkType, algorithm: Algorithm) -> Result<Self> {
        if !bt.pairs_with(&algorithm) {
            return Err(Error::IncompatibleAnalysis(bt, algorithm));
        }
        Ok(Self {
            birthmark: bt,
            comparator: algorithm.comparator(),
        })
    }

    pub fn comparator(&self) -> &Comparator {
        &self.comparator
    }

    /// Every canonical `{birthmark}-{algorithm}` name, for completion and
    /// `--help`.
    ///
    /// Generated by pairing each advertised birthmark with the algorithms
    /// that operate on its shape, so the list cannot offer a pairing that
    /// [`AnalysisType::new`] would reject — which is what a hand-written list
    /// could, and what made the CLI's copy of it worth deleting (#25).
    ///
    /// This is not the set of names that parse: `try_from` accepts any k, and
    /// this stops at [`MAX_ADVERTISED_K`].
    pub fn advertised_names() -> impl Iterator<Item = String> {
        use clap::ValueEnum;
        BirthmarkType::advertised().flat_map(|bt| {
            Algorithm::value_variants()
                .iter()
                // One closure rather than `filter` then `map`, because both
                // would have to borrow `bt`, which does not outlive this
                // iterator. Written as an `if` rather than `bool::then`,
                // which clippy refuses inside `filter_map`.
                .filter_map(move |a| {
                    if bt.pairs_with(a) {
                        Some(format!("{bt}-{}", a.cli_name()))
                    } else {
                        None
                    }
                })
        })
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

    /// Parses `{birthmark}-{algorithm}`, for example `op-3gram-set-dice`.
    ///
    /// Only the algorithm name is read here; everything before it is handed to
    /// [`BirthmarkType::try_from`], which is the parser that owns that half.
    /// Splitting on the last hyphen is safe because no algorithm name contains
    /// one — `weightedjaccard` is deliberately a single word.
    fn try_from(name: String) -> Result<Self> {
        let lowered = name.to_lowercase();
        let (birthmark, algorithm) = lowered
            .rsplit_once('-')
            .ok_or_else(|| Error::BirthmarkType(name.clone()))?;
        let birthmark = BirthmarkType::try_from(birthmark)?;
        let algorithm =
            parse_algorithm(algorithm).ok_or_else(|| Error::BirthmarkType(name.clone()))?;
        AnalysisType::new(birthmark, algorithm)
    }
}

/// The representation a birthmark takes. Every algorithm operates on exactly
/// one of these, converting anything else it is handed — which is why pairing
/// an algorithm with a different shape silently produces the same numbers as
/// the canonical pairing, or none at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    Seq,
    Set,
    Freq,
}

impl Shape {
    pub fn description(&self) -> &'static str {
        match self {
            Shape::Seq => "sequences",
            Shape::Set => "sets",
            Shape::Freq => "frequency vectors",
        }
    }
}

fn parse_algorithm(name: &str) -> Option<Algorithm> {
    use clap::ValueEnum;
    Algorithm::value_variants()
        .iter()
        .find(|a| a.cli_name() == name)
        .cloned()
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
    k.parse().ok()
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
            ir: crate::lift::Ir::GhidraPcode,
        };
        let parsed =
            Metadata::parse(&metadata.csv_info()).expect("failed to parse escaped metadata");
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
        assert_eq!(
            metadata.extracted_at,
            chrono::DateTime::parse_from_rfc3339("2026-04-17T04:57:55.385904+00:00")
                .expect("failed to parse extracted_at")
                .with_timezone(&chrono::Utc)
        );
        assert_eq!(metadata.duration, Duration::from_nanos(4462500));
    }

    #[test]
    fn test_parse_metadata_rejects_wrong_field_count() {
        let err = Metadata::parse("birthmark,name,path").unwrap_err();
        assert!(matches!(err, Error::Parse(msg) if msg.contains("expected 6 or 7 items")));
    }

    /// A record written before the ir field existed still parses, and reads as
    /// Ghidra's P-Code — the only representation any of them can hold.
    #[test]
    fn test_parse_metadata_without_ir() {
        let metadata = Metadata::parse(
            "birthmark,bzip2,/tmp/bzip2,op-seq,2026-04-17T04:57:55.385904+00:00,4462500",
        )
        .expect("a six-field record must still parse");
        assert_eq!(metadata.ir, crate::lift::Ir::GhidraPcode);
    }

    #[test]
    fn test_parse_metadata_rejects_an_unknown_ir() {
        assert!(
            Metadata::parse(
                "birthmark,b,/tmp/b,op-seq,2026-04-17T04:57:55+00:00,1,not-a-representation"
            )
            .is_err()
        );
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
            assert_eq!(
                BirthmarkType::try_from(input).unwrap(),
                expected,
                "input: {input}"
            );
            // parsing must be case insensitive
            assert_eq!(
                BirthmarkType::try_from(input.to_uppercase().as_str()).unwrap(),
                expected
            );
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
            BirthmarkType::FcSeq,
            BirthmarkType::FcSet,
            BirthmarkType::FcFreq,
            BirthmarkType::OpFreq,
            BirthmarkType::OpSeq,
            BirthmarkType::OpSet,
            BirthmarkType::OpKgramSeq(2),
            BirthmarkType::OpKgramFreq(3),
            BirthmarkType::OpKgramSet(4),
        ];
        for bt in types {
            let rendered = bt.to_string();
            assert_eq!(
                BirthmarkType::try_from(rendered.as_str()).unwrap(),
                bt,
                "rendered: {rendered}"
            );
        }
    }

    /// The advertised list and the parser are two views of one vocabulary, and
    /// generating the list is what makes it impossible for them to disagree.
    /// Every name offered has to parse back to what it was made from.
    #[test]
    fn test_every_advertised_birthmark_parses_back_to_itself() {
        for bt in BirthmarkType::advertised() {
            let name = bt.to_string();
            assert_eq!(
                BirthmarkType::try_from(name.as_str()).unwrap(),
                bt,
                "name: {name}"
            );
        }
    }

    #[test]
    fn test_every_advertised_analysis_name_parses() {
        let mut count = 0;
        for name in AnalysisType::advertised_names() {
            AnalysisType::try_from(name.as_str()).unwrap_or_else(|e| panic!("{name}: {e}"));
            count += 1;
        }
        // Two families that have no k, plus MAX_ADVERTISED_K that do, each in
        // three shapes; and each shape pairs with the algorithms that operate
        // on it -- two for seq, three each for set and freq.
        assert_eq!(count, (2 + 3 + 3) * (2 + MAX_ADVERTISED_K));
    }

    /// The list is what gets suggested, not what is accepted. A k past the
    /// ceiling still parses, which is the whole reason the CLI stopped
    /// hand-listing these.
    #[test]
    fn test_the_advertised_ceiling_does_not_bound_the_parser() {
        let name = format!("op-{}gram-set-dice", MAX_ADVERTISED_K + 1);
        assert!(
            !AnalysisType::advertised_names().any(|n| n == name),
            "{name} should be past the end of the list"
        );
        assert!(
            AnalysisType::try_from(name.as_str()).is_ok(),
            "name: {name}"
        );
    }

    /// A generated list would offer pairings that `new` refuses if it crossed
    /// every algorithm with every birthmark. It filters by shape, and this is
    /// what says it filters rather than merely happening to come out valid.
    #[test]
    fn test_the_advertised_names_omit_the_pairings_that_are_refused() {
        for name in ["op-seq-euclidean", "op-set-levenshtein", "op-3gram-set-lcs"] {
            assert!(
                AnalysisType::try_from(name).is_err(),
                "{name} should be refused"
            );
            assert!(
                !AnalysisType::advertised_names().any(|n| n == name),
                "{name} is refused but still advertised"
            );
        }
    }

    #[test]
    fn test_every_advertised_birthmark_describes_itself() {
        for bt in BirthmarkType::advertised() {
            let name = bt.to_string();
            let described = bt.description();
            assert!(!described.is_empty(), "{name} has no description");
            assert!(
                !described.ends_with('.'),
                "{name}: clap renders a short help without a full stop: {described}"
            );
            if let BirthmarkType::OpKgramSeq(k)
            | BirthmarkType::OpKgramFreq(k)
            | BirthmarkType::OpKgramSet(k) = &bt
            {
                assert!(
                    described.contains(&format!("{k}-gram")),
                    "{name} does not say which k: {described}"
                );
            }
        }
    }

    #[test]
    fn test_analysis_type_try_from_named_combinations() {
        let names = [
            "op-freq-cosine",
            "op-set-dice",
            "op-freq-euclidean",
            "op-set-jaccard",
            "op-seq-levenshtein",
            "op-seq-lcs",
            "op-set-simpson",
            "op-freq-weightedjaccard",
            // the fc- family reaches BirthmarkType::try_from by the same route
            "fc-freq-cosine",
            "fc-set-dice",
            "fc-freq-euclidean",
            "fc-set-jaccard",
            "fc-seq-levenshtein",
            "fc-seq-lcs",
            "fc-set-simpson",
            "fc-freq-weightedjaccard",
        ];
        for name in names {
            assert!(AnalysisType::try_from(name).is_ok(), "name: {name}");
            // the String and &str impls must agree
            assert!(
                AnalysisType::try_from(name.to_string()).is_ok(),
                "name: {name}"
            );
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
            (
                "op-6gram-freq-weightedjaccard",
                BirthmarkType::OpKgramFreq(6),
            ),
            ("op-3gram-seq-lcs", BirthmarkType::OpKgramSeq(3)),
            // no ceiling: the CLI used to stop at k = 6 because it
            // hand-listed the names (#25)
            ("op-9gram-freq-cosine", BirthmarkType::OpKgramFreq(9)),
            ("op-12gram-set-jaccard", BirthmarkType::OpKgramSet(12)),
        ];
        for (name, expected) in cases {
            let at = AnalysisType::try_from(name).unwrap_or_else(|e| panic!("{name}: {e}"));
            assert_eq!(at.birthmark, expected, "name: {name}");
        }
    }

    #[test]
    fn test_analysis_type_try_from_rejects_unknown() {
        for name in [
            "",
            "unknown",
            "op-2gram-set-unknown",
            "fc-set-jaccard-extra",
        ] {
            assert!(AnalysisType::try_from(name).is_err(), "name: {name}");
        }
    }

    /// The algorithm name is the only part read here; everything before the
    /// last hyphen must come back exactly as BirthmarkType::try_from resolves
    /// it on its own.
    #[test]
    fn test_analysis_type_delegates_the_birthmark_half() {
        // each birthmark is paired with an algorithm of its own shape, so that
        // this test observes the delegation rather than the validation
        for (birthmark, algorithm) in [
            ("op-seq", "lcs"),
            ("op-set", "jaccard"),
            ("op-freq", "cosine"),
            ("fc-seq", "levenshtein"),
            ("fc-set", "dice"),
            ("fc-freq", "euclidean"),
            ("op-4gram-seq", "lcs"),
        ] {
            let expected =
                BirthmarkType::try_from(birthmark).unwrap_or_else(|e| panic!("{birthmark}: {e}"));
            let name = format!("{birthmark}-{algorithm}");
            let at = AnalysisType::try_from(name.clone()).unwrap_or_else(|e| panic!("{name}: {e}"));
            assert_eq!(at.birthmark, expected, "name: {name}");
        }
    }

    /// A birthmark half that BirthmarkType rejects must fail the whole parse,
    /// rather than being reported as an unknown analysis name.
    #[test]
    fn test_analysis_type_reports_a_bad_birthmark_half() {
        for name in [
            "op-nonsense-jaccard",
            "xx-set-jaccard",
            "op-0.5gram-set-dice",
        ] {
            assert!(AnalysisType::try_from(name).is_err(), "name: {name}");
        }
    }

    /// Each algorithm converts whatever shape it is handed into the one it
    /// operates on, so a non-canonical pairing either reproduces the canonical
    /// one's numbers under a misleading name or scores nothing at all. Both
    /// are rejected, and the message names the pairing that was meant.
    #[test]
    fn test_analysis_type_rejects_non_canonical_pairings() {
        let cases = [
            ("op-seq-euclidean", "use op-freq-euclidean"),
            ("op-set-levenshtein", "use op-seq-levenshtein"),
            ("op-freq-lcs", "use op-seq-lcs"),
            ("fc-seq-jaccard", "use fc-set-jaccard"),
            // the suggestion keeps the k of the birthmark it was given
            ("op-3gram-seq-euclidean", "use op-3gram-freq-euclidean"),
        ];
        for (name, expected) in cases {
            match AnalysisType::try_from(name) {
                Err(e @ Error::IncompatibleAnalysis(..)) => {
                    let rendered = e.to_string();
                    assert!(
                        rendered.contains(expected),
                        "name: {name}, rendered: {rendered}"
                    );
                }
                Err(e) => panic!("{name}: unexpected error: {e}"),
                Ok(_) => panic!("{name}: expected the pairing to be rejected"),
            }
        }
    }

    /// The canonical pairing of every algorithm must survive validation, in
    /// each birthmark family.
    #[test]
    fn test_analysis_type_accepts_every_canonical_pairing() {
        use clap::ValueEnum;
        for prefix in ["op", "fc", "op-4gram"] {
            for algorithm in Algorithm::value_variants() {
                let (algorithm_name, shape) = (algorithm.cli_name(), algorithm.shape());
                let shape_name = match shape {
                    Shape::Seq => "seq",
                    Shape::Set => "set",
                    Shape::Freq => "freq",
                };
                let name = format!("{prefix}-{shape_name}-{algorithm_name}");
                assert!(AnalysisType::try_from(name.clone()).is_ok(), "name: {name}");
            }
        }
    }

    /// pairs_with is the check new and try_from share, so it must agree with
    /// both: the shapes that match are exactly the pairings they accept.
    #[test]
    fn test_pairs_with_agrees_with_construction() {
        use clap::ValueEnum;
        let birthmarks = [
            BirthmarkType::OpSeq,
            BirthmarkType::OpSet,
            BirthmarkType::OpFreq,
            BirthmarkType::FcSeq,
            BirthmarkType::FcSet,
            BirthmarkType::FcFreq,
            BirthmarkType::OpKgramSeq(3),
            BirthmarkType::OpKgramSet(3),
            BirthmarkType::OpKgramFreq(3),
        ];
        for birthmark in birthmarks {
            for algorithm in Algorithm::value_variants() {
                let paired = birthmark.pairs_with(algorithm);
                assert_eq!(
                    paired,
                    birthmark.shape() == algorithm.shape(),
                    "{birthmark}/{}",
                    algorithm.cli_name()
                );
                assert_eq!(
                    AnalysisType::new(birthmark.clone(), algorithm.clone()).is_ok(),
                    paired,
                    "new disagrees with pairs_with for {birthmark}/{}",
                    algorithm.cli_name()
                );
                let name = format!("{birthmark}-{}", algorithm.cli_name());
                assert_eq!(
                    AnalysisType::try_from(name.clone()).is_ok(),
                    paired,
                    "try_from disagrees with pairs_with for {name}"
                );
            }
        }
    }

    #[test]
    fn test_data_len_and_iter_for_every_variant() {
        let kgram = Kgram::new(vec!["A".to_string(), "B".to_string()]);
        let variants = [
            Data::Seq(vec!["A".to_string(), "B".to_string()]),
            Data::Set(["A".to_string(), "B".to_string()].into_iter().collect()),
            Data::Freq(
                [("A".to_string(), 1), ("B".to_string(), 2)]
                    .into_iter()
                    .collect(),
            ),
            Data::KgramSeq(vec![kgram.clone()]),
            Data::KgramSet([kgram.clone()].into_iter().collect()),
            Data::KgramFreq([(kgram, 1)].into_iter().collect()),
        ];
        for data in variants {
            let elements = Elements {
                name: "f".to_string(),
                data,
            };
            // every variant above carries two mnemonics in total
            assert_eq!(elements.ops().count(), 2);
            assert!(!elements.is_empty());
            assert_eq!(elements.name(), "f");
        }
    }

    #[test]
    fn test_elements_is_empty_on_empty_data() {
        let elements = Elements {
            name: "f".to_string(),
            data: Data::Seq(vec![]),
        };
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
                ir: crate::lift::Ir::GhidraPcode,
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

    /// A representation mismatch must be reported as such. Before this, the
    /// two were indistinguishable: any refusal came back as a type mismatch,
    /// and a comparison across representations was not refused at all.
    #[test]
    fn test_refuses_a_comparison_across_representations() {
        let b1 = sample_birthmark();
        let mut b2 = sample_birthmark();
        b2.metadata.ir = crate::lift::Ir::IdaMicrocode;

        match b1.check_comparable_with(&b2) {
            Err(Error::IrMismatch(..)) => {}
            Err(e) => panic!("expected an IrMismatch, got: {e}"),
            Ok(()) => panic!("a comparison across representations must be refused"),
        }
        assert!(!b1.comparable_with(&b2));
    }

    /// The type check still reports a type mismatch rather than being
    /// swallowed by the new one.
    #[test]
    fn test_still_refuses_a_mismatched_type() {
        let b1 = sample_birthmark();
        let mut b2 = sample_birthmark();
        b2.metadata.birthmark_type = BirthmarkType::OpSet;

        match b1.check_comparable_with(&b2) {
            Err(Error::Mismatch(..)) => {}
            Err(e) => panic!("expected a Mismatch, got: {e}"),
            Ok(()) => panic!("a mismatched type must be refused"),
        }
    }

    #[test]
    fn test_allows_the_same_representation_and_type() {
        let b = sample_birthmark();
        assert!(b.check_comparable_with(&b).is_ok());
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

    fn kgram(ops: &[&str]) -> Kgram {
        Kgram::new(ops.iter().map(|s| s.to_string()).collect())
    }

    fn kgram_freq_birthmark() -> Birthmark {
        let mut freq = FxHashMap::default();
        freq.insert(kgram(&["COPY", "RETURN"]), 1);
        freq.insert(kgram(&["CALL", "COPY"]), 3);
        freq.insert(kgram(&["INT_ADD", "COPY"]), 2);
        Birthmark {
            metadata: Metadata {
                birthmark_type: BirthmarkType::OpKgramFreq(2),
                ..sample_birthmark().metadata
            },
            elements: vec![Elements {
                name: "main".to_string(),
                data: Data::KgramFreq(freq),
            }],
            json_path: None,
        }
    }

    /// A k-gram is a list of operations, and a JSON object's keys are
    /// strings, so `serde_json` refused the map outright — every non-empty
    /// `op-*gram-freq` birthmark failed at write time, so eight of the thirty
    /// birthmark types the CLI advertises could not produce a file (#59).
    ///
    /// A non-empty map is what makes this a test. An empty one has no key to
    /// refuse, which is why the bug looked as though it only affected small
    /// k: the fixture yields nothing for k >= 5.
    #[test]
    fn test_a_kgram_frequency_birthmark_survives_a_round_trip() {
        let b = kgram_freq_birthmark();
        let json = serde_json::to_string(&b).expect("a k-gram frequency must be writable");
        let back: Birthmark = serde_json::from_str(&json).expect("and readable again");

        let (Data::KgramFreq(before), Data::KgramFreq(after)) =
            (&b.elements[0].data, &back.elements[0].data)
        else {
            panic!("the shape changed");
        };
        assert_eq!(before, after);
        assert_eq!(after[&kgram(&["CALL", "COPY"])], 3);
    }

    /// Extracting the same program twice should write the same bytes. A hash
    /// map's iteration order is not stable across runs, so the pairs are
    /// sorted on the way out.
    #[test]
    fn test_a_kgram_frequency_is_written_in_a_stable_order() {
        // Enough keys, inserted both ways round. Three would not discriminate:
        // FxHash is not randomised, so a small map's iteration order is
        // whatever it is and happens to come out sorted. At fifty the order
        // is neither sorted nor the same for both insertion orders, so this
        // fails if the sort is removed.
        let build = |reverse: bool| {
            let mut ops = (0..50).map(|i| format!("OP_{i:02}")).collect::<Vec<_>>();
            if reverse {
                ops.reverse();
            }
            let mut freq = FxHashMap::default();
            for op in ops {
                // The count comes from the key, not from when it was
                // inserted, or the two maps would not hold the same thing.
                let count = op.len();
                freq.insert(kgram(&[op.as_str(), "COPY"]), count);
            }
            serde_json::to_string(&Data::KgramFreq(freq)).unwrap()
        };
        assert_eq!(build(false), build(true));

        // The data alone: `sample_birthmark` stamps `Utc::now()` into the
        // metadata, which is not what this is about.
        let once = serde_json::to_string(&kgram_freq_birthmark().elements[0].data).unwrap();
        for _ in 0..8 {
            let again = serde_json::to_string(&kgram_freq_birthmark().elements[0].data).unwrap();
            assert_eq!(again, once);
        }
        let data = serde_json::to_value(&kgram_freq_birthmark().elements[0].data).unwrap();
        assert_eq!(
            data["KgramFreq"],
            serde_json::json!([
                [["CALL", "COPY"], 3],
                [["COPY", "RETURN"], 1],
                [["INT_ADD", "COPY"], 2],
            ])
        );
    }

    /// A map and a set have no order of their own, so what gets written has
    /// to be decided. Hash order decides it by insertion history and by the
    /// hasher, which means two equal birthmarks can be written differently
    /// and a `rustc-hash` bump relays every file (#68).
    ///
    /// Built twice from the same elements in opposite orders: the bytes have
    /// to match. Fifty elements rather than a handful, because `FxHash` is
    /// not randomised and a small container's order can happen to be the
    /// sorted one either way.
    #[test]
    fn test_what_has_no_order_is_written_in_one() {
        let ops = (0..50).map(|i| format!("OP_{i:02}")).collect::<Vec<_>>();
        let both_ways = |build: &dyn Fn(Vec<String>) -> Data| {
            let forward = serde_json::to_string(&build(ops.clone())).unwrap();
            let mut reversed = ops.clone();
            reversed.reverse();
            (forward, serde_json::to_string(&build(reversed)).unwrap())
        };

        let (a, b) = both_ways(&|v| {
            let mut m = FxHashMap::default();
            for op in v {
                let count = op.len();
                m.insert(op, count);
            }
            Data::Freq(m)
        });
        assert_eq!(a, b, "Freq");

        let (a, b) = both_ways(&|v| Data::Set(v.into_iter().collect()));
        assert_eq!(a, b, "Set");

        let (a, b) = both_ways(&|v| {
            Data::KgramSet(
                v.into_iter()
                    .map(|op| kgram(&[op.as_str(), "COPY"]))
                    .collect(),
            )
        });
        assert_eq!(a, b, "KgramSet");

        let (a, b) = both_ways(&|v| {
            let mut m = FxHashMap::default();
            for op in v {
                let count = op.len();
                m.insert(kgram(&[op.as_str(), "COPY"]), count);
            }
            Data::KgramFreq(m)
        });
        assert_eq!(a, b, "KgramFreq");
    }

    /// A sequence is ordered data: its order is the program's, and sorting it
    /// would not canonicalise the file but destroy the birthmark.
    #[test]
    fn test_a_sequence_keeps_the_order_it_was_extracted_in() {
        let ops = vec!["RETURN".to_string(), "CALL".to_string(), "COPY".to_string()];
        let json = serde_json::to_value(Data::Seq(ops.clone())).unwrap();
        assert_eq!(json["Seq"], serde_json::json!(["RETURN", "CALL", "COPY"]));

        let kgrams = vec![kgram(&["RETURN", "CALL"]), kgram(&["CALL", "COPY"])];
        let json = serde_json::to_value(Data::KgramSeq(kgrams)).unwrap();
        assert_eq!(
            json["KgramSeq"],
            serde_json::json!([["RETURN", "CALL"], ["CALL", "COPY"]])
        );
    }

    /// The two frequency shapes agree about a contradiction, and the other
    /// four correctly do not care.
    ///
    /// `Freq` used serde's derived map deserializer, which inserts in a loop,
    /// so `{"COPY": 3, "COPY": 5}` loaded as 5 with nothing said — the same
    /// hazard #59 removed from `KgramFreq`, in the family that already worked
    /// (#66).
    ///
    /// The line is ambiguity, not repetition. A `Set` repeating an element
    /// denotes the same set, so collapsing loses nothing; a `Seq` repeating
    /// one is a sequence doing its job.
    #[test]
    fn test_only_a_contradicted_frequency_is_refused() {
        let refused = [
            (r#"{"Freq":{"COPY":3,"COPY":5}}"#, "COPY"),
            (
                r#"{"KgramFreq":[[["CALL","COPY"],3],[["CALL","COPY"],5]]}"#,
                "CALL",
            ),
        ];
        for (json, named) in refused {
            let msg = serde_json::from_str::<Data>(json)
                .expect_err("a contradicted frequency must not pick one")
                .to_string();
            assert!(msg.contains("ambiguous"), "{json}: {msg}");
            assert!(msg.contains(named), "does not say which one: {msg}");
        }

        // Nothing is contradicted here, so nothing is refused.
        let accepted = [
            (r#"{"Set":["COPY","COPY"]}"#, 1),
            (r#"{"KgramSet":[["CALL","COPY"],["CALL","COPY"]]}"#, 1),
            (r#"{"Seq":["COPY","COPY"]}"#, 2),
            (r#"{"KgramSeq":[["CALL","COPY"],["CALL","COPY"]]}"#, 2),
        ];
        for (json, len) in accepted {
            let d: Data = serde_json::from_str(json).unwrap_or_else(|e| panic!("{json}: {e}"));
            assert_eq!(d.len(), len, "{json}");
        }
    }

    /// A frequency map that says each thing once is still read, and read
    /// whole. Refusing a repeat must not turn into refusing a second key.
    #[test]
    fn test_a_frequency_map_with_distinct_keys_is_read_whole() {
        let Data::Freq(m) =
            serde_json::from_str::<Data>(r#"{"Freq":{"COPY":3,"CALL":5,"RETURN":1}}"#).unwrap()
        else {
            panic!("the shape changed");
        };
        assert_eq!(m.len(), 3);
        assert_eq!(m["COPY"], 3);
        assert_eq!(m["CALL"], 5);
        assert_eq!(m["RETURN"], 1);
    }

    /// The custom visitor has to say what it wanted, or a file with the wrong
    /// shape reports "invalid type: sequence, expected " and stops there.
    #[test]
    fn test_a_frequency_that_is_not_a_map_says_what_was_expected() {
        for json in [r#"{"Freq":[]}"#, r#"{"Freq":"COPY"}"#] {
            let msg = serde_json::from_str::<Data>(json)
                .expect_err("this is not a frequency map")
                .to_string();
            assert!(msg.contains("invalid type"), "{json}: {msg}");
            assert!(
                msg.contains("expected a map of operations to how often each occurs"),
                "{json}: {msg}"
            );
        }
    }

    /// A list is a weaker container than the map it stands for: it can say
    /// the same thing twice. Collecting would take the last one, so a file
    /// claiming a k-gram occurred 3 times and again 5 times would load as 5
    /// and be scored, with nothing said about it.
    ///
    /// Nothing this crate writes can produce one — the pairs come from a map
    /// — so refusing costs no real file anything.
    #[test]
    fn test_a_kgram_listed_twice_is_refused_rather_than_resolved() {
        let once: Data =
            serde_json::from_str(r#"{"KgramFreq":[[["CALL","COPY"],3]]}"#).expect("one is fine");
        let Data::KgramFreq(m) = once else {
            panic!("the shape changed");
        };
        assert_eq!(m[&kgram(&["CALL", "COPY"])], 3);

        let twice = serde_json::from_str::<Data>(
            r#"{"KgramFreq":[[["CALL","COPY"],3],[["CALL","COPY"],5]]}"#,
        );
        let Err(e) = twice else {
            panic!("a k-gram listed twice must not silently pick one");
        };
        let msg = e.to_string();
        assert!(msg.contains("listed more than once"), "{msg}");
        assert!(msg.contains("CALL"), "does not say which one: {msg}");
    }

    /// Only `KgramFreq` changed. The fix could have been to make `Kgram`
    /// itself serialize as a string, and that would have rewritten the two
    /// k-gram families that already work — whose files are readable today.
    #[test]
    fn test_the_other_kgram_families_still_write_a_plain_list() {
        let k = kgram(&["CALL", "COPY"]);
        let expected = serde_json::json!(["CALL", "COPY"]);

        let seq = serde_json::to_value(Data::KgramSeq(vec![k.clone()])).unwrap();
        assert_eq!(seq["KgramSeq"][0], expected);

        let set = serde_json::to_value(Data::KgramSet(FxHashSet::from_iter([k]))).unwrap();
        assert_eq!(set["KgramSet"][0], expected);
    }

    #[test]
    fn test_birthmark_try_from_path_reports_errors() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("missing.json");
        assert!(matches!(
            Birthmark::try_from(missing).unwrap_err(),
            Error::Io(..)
        ));

        let broken = dir.path().join("broken.json");
        std::fs::write(&broken, b"{ not json").unwrap();
        assert!(matches!(
            Birthmark::try_from(broken).unwrap_err(),
            Error::Json(..)
        ));

        // Bytes that are not UTF-8 are the file's content being wrong, not
        // the read failing, so they come back as a JSON error. This is the
        // difference between reading to bytes and reading to a String:
        // `read_to_string` would report `Error::Io` here (#51).
        let not_utf8 = dir.path().join("not-utf8.json");
        std::fs::write(&not_utf8, b"{\"name\": \"\xff\xfe\"}").unwrap();
        assert!(matches!(
            Birthmark::try_from(not_utf8).unwrap_err(),
            Error::Json(..)
        ));
    }
}
