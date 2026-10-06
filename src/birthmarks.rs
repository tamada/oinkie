//! What a birthmark is, and what it holds.
//!
//! A [`Birthmark`] is one program's: one [`Function`] for each of its
//! functions, each made of elements -- operations, the names of called
//! functions, or [`Kgram`]s of operations -- held as [`Data`] in the
//! [`Shape`] its [`BirthmarkType`] names. An [`AnalysisType`] pairs a
//! birthmark type with the algorithm that compares it.
//!
//! The terms are defined in the
//! [glossary](https://tamada.github.io/oinkie/glossary/).
//!
//! ```
//! use oinkie::birthmarks::{AnalysisType, BirthmarkType, Shape};
//!
//! # fn main() -> oinkie::Result<()> {
//! let bt = BirthmarkType::try_from("op-3gram-set")?;
//! assert_eq!(bt, BirthmarkType::OpKgramSet(3));
//! assert_eq!(bt.shape(), Shape::Set);
//!
//! let analysis = AnalysisType::try_from("op-3gram-set-jaccard")?;
//! assert_eq!(analysis.birthmark(), &bt);
//!
//! // An algorithm pairs only with the shape it operates on.
//! assert!(AnalysisType::try_from("op-3gram-set-levenshtein").is_err());
//! # Ok(())
//! # }
//! ```

use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

use crate::compare::{Algorithm, Comparator};
use crate::{Error, Result};
use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::time::Duration;

/// Which birthmark to extract: the kind of element and the [`Shape`] it is
/// kept in, named `{element}-{shape}` -- `op-seq`, `fc-set`, `op-3gram-freq`.
///
/// Read from that name by [`BirthmarkType::try_from`], in any case, and
/// written back in it by `Display`.
///
/// `fc` elements are the names of the functions a function calls, as the
/// lifted program's symbol table resolves them; a call that does not resolve
/// to a name is left out. `op` elements are the function's operations in the
/// lifted representation.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum BirthmarkType {
    /// The functions each function calls, in the order of the calls
    /// (`fc-seq`).
    FcSeq,
    /// The functions each function calls, each once (`fc-set`).
    FcSet,
    /// The functions each function calls, with how often each is called
    /// (`fc-freq`).
    FcFreq,
    /// The operations of each function, with how often each occurs
    /// (`op-freq`).
    OpFreq,
    /// The operations of each function, in order (`op-seq`).
    OpSeq,
    /// The operations each function uses, each once (`op-set`).
    OpSet,
    /// Every run of `k` consecutive operations, in order
    /// (`op-{k}gram-seq`). `k` is at least 1.
    OpKgramSeq(usize),
    /// Every distinct run of `k` consecutive operations, with how often each
    /// occurs (`op-{k}gram-freq`). `k` is at least 1.
    OpKgramFreq(usize),
    /// Every distinct run of `k` consecutive operations, each once
    /// (`op-{k}gram-set`). `k` is at least 1.
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
            _ => parse_kgram(&s).ok_or(Error::BirthmarkType(s)),
        }
    }
}

impl BirthmarkType {
    /// Every birthmark family, with k-grams for each k from 1 up to `max_k`.
    ///
    /// The set of birthmarks is infinite -- [`BirthmarkType::try_from`] takes
    /// any k, and `op-9gram-set-dice` is a valid name -- so a list of them has
    /// to stop somewhere, and where is the caller's choice.
    pub fn all(max_k: usize) -> Vec<BirthmarkType> {
        [
            BirthmarkType::FcSeq,
            BirthmarkType::FcFreq,
            BirthmarkType::FcSet,
            BirthmarkType::OpSeq,
            BirthmarkType::OpSet,
            BirthmarkType::OpFreq,
        ]
        .into_iter()
        .chain((1..=max_k).map(BirthmarkType::OpKgramSeq))
        .chain((1..=max_k).map(BirthmarkType::OpKgramFreq))
        .chain((1..=max_k).map(BirthmarkType::OpKgramSet))
        .collect()
    }

    /// The shape its elements are kept in.
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
    pub(crate) fn with_shape(&self, shape: Shape) -> BirthmarkType {
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
    /// Not `from_reader` on the `File`: serde_json reads a reader one byte at
    /// a time, so an unbuffered file costs a system call per byte -- seconds
    /// for a large birthmark, against milliseconds read whole -- and a
    /// comparison loads each birthmark once per pair it is in.
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
pub(crate) struct Metadata {
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
    /// A file without the field reads as Ghidra's P-Code, as a lifted program
    /// without one does: that is what every such file was written from. It is
    /// a fallback rather than a deduction, since a file that simply omits the
    /// field reads the same way; this crate always writes it.
    #[serde(default)]
    pub ir: crate::lift::Ir,
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

/// The birthmark of one program: one [`Function`] per function it holds,
/// and what it was extracted from and how.
#[derive(Serialize, Deserialize, Debug)]
pub struct Birthmark {
    pub(crate) metadata: Metadata,
    /// Written under the key `elements`, the key every birthmark file uses,
    /// so that every one of them keeps reading.
    #[serde(rename = "elements")]
    pub(crate) functions: Vec<Function>,
    #[serde(skip)]
    pub(crate) json_path: Option<PathBuf>,
}

impl Birthmark {
    /// Records the file this birthmark was read from or written to, which
    /// [`Birthmark::json_path`] then returns. Reading one does not record it
    /// by itself.
    pub fn set_json_path(&mut self, path: PathBuf) {
        self.json_path = Some(path);
    }

    /// Explains why these two cannot be compared, when they cannot.
    ///
    /// A birthmark is only meaningful against another built the same way: of
    /// the same type, and from the same representation, because two lifters
    /// describe the same instruction with different operations, and comparing
    /// across them measures the disagreement between the tools rather than
    /// anything about the programs.
    ///
    /// The `fc-*` family is the one that could eventually cross this line: it
    /// holds symbol names read from the binary rather than operations read
    /// from the IR, so two tools should recover much the same set. It is
    /// refused all the same, because the spellings still differ between tools
    /// (`_printf` against `printf`) and nothing normalises them yet. Relaxing
    /// this is worth doing once that normalisation exists and a second lifter
    /// can be used to measure whether the result is worth trusting.
    pub(crate) fn check_comparable_with(&self, other: &Birthmark) -> Result<()> {
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

    /// The program's name, as its lifted file recorded it: the binary's file
    /// name.
    pub fn name(&self) -> &str {
        &self.metadata.file_name
    }

    /// The path of the binary the program was lifted from.
    pub fn path(&self) -> &Path {
        &self.metadata.path
    }

    /// When it was extracted.
    pub fn extracted_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.metadata.extracted_at
    }

    /// How long the extraction took.
    pub fn duration(&self) -> std::time::Duration {
        self.metadata.duration
    }

    /// Which birthmark this is.
    pub fn birthmark_type(&self) -> &BirthmarkType {
        &self.metadata.birthmark_type
    }

    /// The representation the program was lifted to.
    pub fn ir(&self) -> crate::lift::Ir {
        self.metadata.ir
    }

    /// Where this birthmark was read from, once [`Birthmark::set_json_path`]
    /// has said so.
    pub fn json_path(&self) -> Option<&Path> {
        self.json_path.as_deref()
    }

    /// The birthmark of each function, in the order they were extracted.
    pub fn functions(&self) -> &[Function] {
        &self.functions
    }

    /// How many functions it holds.
    pub fn len(&self) -> usize {
        self.functions.len()
    }

    /// Whether it holds no functions. Two such birthmarks compare as
    /// identical, a similarity of 1.0.
    pub fn is_empty(&self) -> bool {
        self.functions.is_empty()
    }
}

impl crate::Iterable for &Birthmark {
    type Item = Function;
    fn iter(&self) -> Box<dyn Iterator<Item = &Self::Item> + '_> {
        Box::new(self.functions.iter())
    }
}

impl crate::Iterable for Birthmark {
    type Item = Function;
    fn iter(&self) -> Box<dyn Iterator<Item = &Self::Item> + '_> {
        Box::new(self.functions.iter())
    }
}

/// The birthmark of one function: its name, and the elements it is made of.
///
/// Not the function itself -- what was lifted from the binary is not public --
/// but what extraction kept of it, which is why it lives in `birthmarks`.
#[derive(Serialize, Deserialize, Debug)]
pub struct Function {
    pub(crate) name: String,
    pub(crate) data: Data,
}

impl Function {
    /// The function's name, as the lifter recorded it.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The elements this function's birthmark is made of.
    pub fn data(&self) -> &Data {
        &self.data
    }

    /// Every operation or called function's name in it, with k-grams taken
    /// apart into their operations. In order for a sequence; in no particular
    /// order for a set or a frequency map, whose keys each appear once.
    pub fn ops(&self) -> impl Iterator<Item = &str> {
        self.data.iter()
    }

    /// How many elements it has: its birthmark length. Distinct elements for
    /// a set, and distinct keys for a frequency map.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Whether it has no elements.
    pub fn is_empty(&self) -> bool {
        self.data.len() == 0
    }
}

/// The elements one function's birthmark is made of.
///
/// An element is one operation or call name (`String`) or one k-gram of them
/// ([`Kgram`]), and the elements come as a sequence, a set or a frequency map,
/// which is the birthmark's [`Shape`]. The number of elements is
/// [`Function::len`]; the number of functions is [`Birthmark::len`].
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Data {
    /// Each element with how often it occurs (`freq`).
    ///
    /// A file naming an operation more than once is refused: a birthmark
    /// names each once, and taking the last of a repeated key would score a
    /// count the file contradicts itself about.
    Freq(
        #[serde(serialize_with = "sorted_freq", deserialize_with = "no_repeated_key")]
        FxHashMap<String, usize>,
    ),
    /// The elements in order, repeats included (`seq`).
    Seq(Vec<String>),
    /// Each distinct element once, in no order (`set`).
    Set(#[serde(serialize_with = "sorted_set")] FxHashSet<String>),
    /// The k-grams in order, repeats included (`op-{k}gram-seq`).
    KgramSeq(Vec<Kgram>),
    /// Each k-gram with how often it occurs (`op-{k}gram-freq`).
    ///
    /// Written as a list of `[kgram, count]` pairs rather than as a JSON
    /// object, whose keys can only be strings: a k-gram is a list of
    /// operations. A list of pairs keeps the k-gram written as the list
    /// `KgramSeq` and `KgramSet` write it as, and needs no separator that a
    /// mnemonic might contain.
    KgramFreq(#[serde(with = "kgram_freq")] FxHashMap<Kgram, usize>),
    /// Each distinct k-gram once, in no order (`op-{k}gram-set`).
    KgramSet(#[serde(serialize_with = "sorted_kgram_set")] FxHashSet<Kgram>),
}

/// A map and a set have no order of their own, so the one they are written in
/// has to come from somewhere. Hash order comes from the insertion history and
/// the hasher, so two equal birthmarks could be written differently. Sorted,
/// the bytes are a function of what the birthmark holds, and a file can be
/// diffed and cached.
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

/// Reads a frequency map, refusing a key that appears more than once.
///
/// serde's derived deserializer inserts in a loop, so `{"COPY": 3, "COPY": 5}`
/// would load as 5 and nothing be said -- a similarity computed from a count
/// the file contradicts itself about.
///
/// Two decisions here, on different axes, and they are easy to run together.
///
/// *Which shapes need a check at all* is decided by ambiguity: a repeated
/// element in a `Set` denotes the same set, and a repeated element in a `Seq`
/// is what a sequence is for, so neither can mean two things. A repeated
/// count can, which is why only the frequency shapes have this.
///
/// *Which repeats are refused* is not: every one is, without comparing the
/// values, so `{"COPY": 3, "COPY": 3}` is refused as well. A repeated key is
/// a malformed object however the values fall, and "a duplicate that happens
/// to agree with itself" is not a category worth carving out — reading it
/// would mean accepting a broken file whenever it was broken consistently.
///
/// Nothing this crate writes can produce a repeat, since it serializes from a
/// map, so refusing costs no real file anything.
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
                        "{name}: listed more than once; a birthmark names each operation once"
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

    /// A repeated k-gram is refused rather than resolved, on the same terms
    /// as [`no_repeated_key`](super::no_repeated_key): every repeat, without comparing the counts.
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
                    "[{}]: listed more than once; a birthmark names each k-gram once",
                    kgram.0.join(", ")
                )));
            }
            map.insert(kgram, count);
        }
        Ok(map)
    }
}

/// `k` consecutive operations, taken as one element.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
pub struct Kgram(Vec<String>);

impl Kgram {
    /// A k-gram of these operations, in this order.
    pub fn new(seq: Vec<String>) -> Self {
        Self(seq)
    }

    /// The operations, in the order the k-gram holds them.
    pub fn ops(&self) -> &[String] {
        &self.0
    }
}

impl Data {
    fn iter(&self) -> Box<dyn Iterator<Item = &str> + '_> {
        match self {
            Data::Freq(freq) => Box::new(freq.keys().map(String::as_str)),
            Data::Seq(seq) => Box::new(seq.iter().map(String::as_str)),
            Data::Set(set) => Box::new(set.iter().map(String::as_str)),
            Data::KgramSeq(seq) => {
                Box::new(seq.iter().flat_map(|k| k.0.iter().map(String::as_str)))
            }
            Data::KgramFreq(freq) => {
                Box::new(freq.keys().flat_map(|k| k.0.iter().map(String::as_str)))
            }
            Data::KgramSet(set) => {
                Box::new(set.iter().flat_map(|k| k.0.iter().map(String::as_str)))
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

/// A birthmark type and the algorithm that compares it, named
/// `{birthmark type}-{algorithm}` -- `op-set-jaccard`,
/// `op-3gram-seq-levenshtein`.
///
/// Only the pairings in which the algorithm operates on the birthmark's
/// [`Shape`] exist: [`AnalysisType::new`] and `try_from` refuse the rest with
/// [`Error::IncompatibleAnalysis`], whose message names the pairing to use
/// instead.
pub struct AnalysisType {
    pub(crate) birthmark: BirthmarkType,
    pub(crate) comparator: Comparator,
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

    /// The birthmark type this analysis compares.
    pub fn birthmark(&self) -> &BirthmarkType {
        &self.birthmark
    }

    /// The comparator that runs its algorithm.
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

    /// Parses `{birthmark}-{algorithm}`, for example `op-3gram-set-dice`.
    ///
    /// The algorithm's name is read here; everything before it is handed to
    /// [`BirthmarkType::try_from`], the parser that owns that half.
    ///
    /// Where the two halves meet is found rather than assumed. Each hyphen is
    /// tried as the split, rightmost first, and the first one whose tail names
    /// an algorithm and whose head is a birthmark type wins:
    ///
    /// ```text
    /// op-freq-weighted-jaccard
    ///         ^ "jaccard" is an algorithm, but "op-freq-weighted" is not a
    ///           birthmark -- keep looking
    ///    ^ "weighted-jaccard" is an algorithm and "op-freq" is a birthmark
    /// ```
    ///
    /// Searching, rather than splitting on the last hyphen, lets an
    /// algorithm's name have a hyphen of its own -- `weighted-jaccard` -- at
    /// the cost of a few string comparisons on a name a person typed.
    fn try_from(name: String) -> Result<Self> {
        let lowered = name.to_lowercase();
        // The rightmost split that got as far as naming an algorithm, so that
        // a bad birthmark half is still reported as itself rather than the
        // whole string being blamed.
        let mut birthmark_error = None;
        for (at, _) in lowered.rmatch_indices('-') {
            let (birthmark, algorithm) = (&lowered[..at], &lowered[at + 1..]);
            let Some(algorithm) = parse_algorithm(algorithm) else {
                continue;
            };
            match BirthmarkType::try_from(birthmark) {
                Ok(birthmark) => return AnalysisType::new(birthmark, algorithm),
                Err(e) => birthmark_error = birthmark_error.or(Some(e)),
            }
        }
        Err(birthmark_error.unwrap_or(Error::BirthmarkType(name)))
    }
}

/// The representation a birthmark takes. Every algorithm operates on exactly
/// one of these, converting anything else it is handed — which is why pairing
/// an algorithm with a different shape silently produces the same numbers as
/// the canonical pairing, or none at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    /// In order, repeats included.
    Seq,
    /// Each distinct element once, in no order.
    Set,
    /// Each distinct element once, with how often it occurs.
    Freq,
}

impl Shape {
    /// The shape in words, plural, as messages use it: `sequences`, `sets`,
    /// `frequency vectors`.
    pub fn description(&self) -> &'static str {
        match self {
            Shape::Seq => "sequences",
            Shape::Set => "sets",
            Shape::Freq => "frequency vectors",
        }
    }
}

/// An algorithm by either of the names it goes by: the one an analysis name
/// uses, and the same with a hyphen between its words.
///
/// They differ for `weightedjaccard` / `weighted-jaccard` alone; the other
/// seven are single words and spell the same either way.
fn parse_algorithm(name: &str) -> Option<Algorithm> {
    name.parse().ok()
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

/// The k of a k-gram: a positive count. Zero parses as a number but names no
/// birthmark -- a k-gram of nothing -- so it is refused here, where the name
/// is read, instead of reaching an extraction that cannot take it.
fn parse_k_value(k: &str) -> Option<usize> {
    k.parse().ok().filter(|&k| k > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

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

    /// A k-gram of nothing is not a birthmark: zero is refused where the name
    /// is read, in a birthmark and in an analysis, rather than reaching an
    /// extraction that cannot take it.
    #[test]
    fn test_a_zero_gram_is_refused_by_name() {
        for name in ["op-0gram-seq", "op-0gram-set", "op-0gram-freq"] {
            assert!(
                BirthmarkType::try_from(name).is_err(),
                "{name} was accepted"
            );
        }
        let err = AnalysisType::try_from("op-0gram-set-jaccard")
            .err()
            .expect("op-0gram-set-jaccard was accepted");
        assert!(err.to_string().contains("op-0gram-set"), "{err}");
        assert!(BirthmarkType::try_from("op-1gram-set").is_ok());
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

    /// `BirthmarkType::all` is written by hand, so it is held to the enum: the
    /// `match` stops compiling when a family is added, and the assertion fails
    /// when it is added there but not to the list. Callers rely on `all` to
    /// see every family -- the command line's descriptions are checked
    /// against it -- since `#[non_exhaustive]` stops them matching the enum
    /// exhaustively themselves.
    #[test]
    fn test_all_lists_every_birthmark_family() {
        fn family(bt: &BirthmarkType) -> usize {
            match bt {
                BirthmarkType::FcSeq => 0,
                BirthmarkType::FcSet => 1,
                BirthmarkType::FcFreq => 2,
                BirthmarkType::OpFreq => 3,
                BirthmarkType::OpSeq => 4,
                BirthmarkType::OpSet => 5,
                BirthmarkType::OpKgramSeq(_) => 6,
                BirthmarkType::OpKgramFreq(_) => 7,
                BirthmarkType::OpKgramSet(_) => 8,
            }
        }
        let mut seen = BirthmarkType::all(1).iter().map(family).collect::<Vec<_>>();
        seen.sort_unstable();
        assert_eq!(seen, (0..9).collect::<Vec<_>>());
    }

    /// A list of birthmarks and the parser are two views of one vocabulary.
    /// Every name in the list has to parse back to what it was made from.
    #[test]
    fn test_every_listed_birthmark_parses_back_to_itself() {
        for bt in BirthmarkType::all(8) {
            let name = bt.to_string();
            assert_eq!(
                BirthmarkType::try_from(name.as_str()).unwrap(),
                bt,
                "name: {name}"
            );
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
            // no ceiling: the grammar accepts any k, whatever list of
            // names a caller chooses to offer
            ("op-9gram-freq-cosine", BirthmarkType::OpKgramFreq(9)),
            ("op-12gram-set-jaccard", BirthmarkType::OpKgramSet(12)),
        ];
        for (name, expected) in cases {
            let at = AnalysisType::try_from(name).unwrap_or_else(|e| panic!("{name}: {e}"));
            assert_eq!(at.birthmark, expected, "name: {name}");
        }
    }

    /// An analysis name accepts an algorithm spelled with a hyphen between its
    /// words as well as without, so that a name built from either parses.
    #[test]
    fn test_an_algorithm_answers_to_both_of_its_spellings() {
        for algorithm in Algorithm::ALL {
            assert_eq!(parse_algorithm(algorithm.name()).as_ref(), Some(algorithm));
        }
        // `weighted-jaccard` is the only one that is spelled two ways.
        assert_eq!(
            parse_algorithm("weighted-jaccard"),
            Some(Algorithm::WeightedJaccard)
        );
        assert_eq!(
            "WEIGHTED-JACCARD".parse::<Algorithm>().unwrap(),
            Algorithm::WeightedJaccard
        );
    }

    /// Where the halves meet is searched for, rightmost hyphen first, rather
    /// than assumed to be the last one. Both spellings therefore mean the
    /// same analysis.
    #[test]
    fn test_the_split_is_found_rather_than_assumed() {
        for name in ["op-freq-weighted-jaccard", "op-freq-weightedjaccard"] {
            let at = AnalysisType::try_from(name).unwrap_or_else(|e| panic!("{name}: {e}"));
            assert_eq!(at.birthmark, BirthmarkType::OpFreq, "{name}");
        }
        let at = AnalysisType::try_from("op-3gram-freq-weighted-jaccard").unwrap();
        assert_eq!(at.birthmark, BirthmarkType::OpKgramFreq(3));
    }

    /// Searching must not cost the message. A split that names an algorithm
    /// but not a birthmark reports the birthmark half, not the whole string.
    #[test]
    fn test_a_bad_birthmark_half_is_still_reported_as_itself() {
        let Err(e) = AnalysisType::try_from("op-nonsense-jaccard") else {
            panic!("op-nonsense is not a birthmark type");
        };
        assert_eq!(e.to_string(), "op-nonsense: unknown birthmark type");

        // nothing names an algorithm here, so there is no half to blame
        let Err(e) = AnalysisType::try_from("fc-set-jaccard-extra") else {
            panic!("nothing in this names an algorithm");
        };
        assert_eq!(
            e.to_string(),
            "fc-set-jaccard-extra: unknown birthmark type"
        );
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
        for prefix in ["op", "fc", "op-4gram"] {
            for algorithm in Algorithm::ALL {
                let (algorithm_name, shape) = (algorithm.name(), algorithm.shape());
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
            for algorithm in Algorithm::ALL {
                let paired = birthmark.pairs_with(algorithm);
                assert_eq!(
                    paired,
                    birthmark.shape() == algorithm.shape(),
                    "{birthmark}/{}",
                    algorithm.name()
                );
                assert_eq!(
                    AnalysisType::new(birthmark.clone(), algorithm.clone()).is_ok(),
                    paired,
                    "new disagrees with pairs_with for {birthmark}/{}",
                    algorithm.name()
                );
                let name = format!("{birthmark}-{}", algorithm.name());
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
            let function = Function {
                name: "f".to_string(),
                data,
            };
            // every variant above carries two mnemonics in total
            assert_eq!(function.ops().count(), 2);
            assert!(!function.is_empty());
            assert_eq!(function.name(), "f");
        }
    }

    #[test]
    fn test_elements_is_empty_on_empty_data() {
        let function = Function {
            name: "f".to_string(),
            data: Data::Seq(vec![]),
        };
        assert!(function.is_empty());
        assert_eq!(function.len(), 0);
        assert_eq!(function.ops().count(), 0);
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
            functions: vec![Function {
                name: "main".to_string(),
                data: Data::Seq(vec!["COPY".to_string()]),
            }],
            json_path: None,
        }
    }

    /// Birthmark files keep the key `elements` for the list of functions,
    /// read and written. The fixtures were written by an `oinkie extract` built
    /// independently of this code, so they cannot agree with it by
    /// construction.
    #[test]
    fn test_a_birthmark_written_before_the_rename_still_loads() {
        for (fixture, functions) in [
            ("testdata/birthmarks/hello_clang_op-seq.json", 1),
            ("testdata/birthmarks/udl_op-3gram-freq.json", 3),
        ] {
            let b = Birthmark::try_from(Path::new(fixture))
                .unwrap_or_else(|e| panic!("{fixture}: {e}"));
            assert_eq!(b.len(), functions, "{fixture}");
            assert_eq!(b.functions().len(), functions, "{fixture}");
            assert!(b.functions().iter().any(|f| !f.is_empty()), "{fixture}");

            let json = serde_json::to_value(&b).unwrap();
            assert!(
                json.get("elements").is_some(),
                "{fixture}: the key must stay `elements`"
            );
            assert!(
                json.get("functions").is_none(),
                "{fixture}: the field name leaked into the file"
            );
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

        // the json path is unknown until it is set
        assert_eq!(b.json_path(), None);
        b.set_json_path(PathBuf::from("/tmp/sample.json"));
        assert_eq!(b.json_path(), Some(Path::new("/tmp/sample.json")));
    }

    #[test]
    fn test_birthmark_comparable_with() {
        let b1 = sample_birthmark();
        let mut b2 = sample_birthmark();
        assert!(b1.check_comparable_with(&b2).is_ok());
        b2.metadata.birthmark_type = BirthmarkType::OpSet;
        assert!(!b1.check_comparable_with(&b2).is_ok());
    }

    /// A representation mismatch is reported as such, not as a type mismatch:
    /// the two are different reasons, and the message should say which.
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
        assert!(!b1.check_comparable_with(&b2).is_ok());
    }

    /// And a type mismatch between birthmarks of one representation is still
    /// reported as a type mismatch.
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
        Kgram::new(ops.iter().map(ToString::to_string).collect())
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
            functions: vec![Function {
                name: "main".to_string(),
                data: Data::KgramFreq(freq),
            }],
            json_path: None,
        }
    }

    /// A k-gram is a list of operations and a JSON object's keys are strings,
    /// so an `op-*gram-freq` birthmark has to be written in some other form
    /// to be written at all.
    ///
    /// A non-empty map is what makes this a test: an empty one has no key to
    /// write, and the fixture yields nothing for k >= 5.
    #[test]
    fn test_a_kgram_frequency_birthmark_survives_a_round_trip() {
        let b = kgram_freq_birthmark();
        let json = serde_json::to_string(&b).expect("a k-gram frequency must be writable");
        let back: Birthmark = serde_json::from_str(&json).expect("and readable again");

        let (Data::KgramFreq(before), Data::KgramFreq(after)) =
            (&b.functions[0].data, &back.functions[0].data)
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
        let once = serde_json::to_string(&kgram_freq_birthmark().functions[0].data).unwrap();
        for _ in 0..8 {
            let again = serde_json::to_string(&kgram_freq_birthmark().functions[0].data).unwrap();
            assert_eq!(again, once);
        }
        let data = serde_json::to_value(&kgram_freq_birthmark().functions[0].data).unwrap();
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
    /// hasher, which would let two equal birthmarks be written differently.
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

    /// The two frequency shapes refuse a repeated key and the other four
    /// correctly do not care.
    ///
    /// Both axes are pinned here, because they are easy to run together.
    /// *Which shapes check*: only the frequencies, because only a repeated
    /// count can mean two things — a repeated set member denotes the same
    /// set, a repeated sequence element is a sequence doing its job.
    /// *Which repeats are refused*: all of them, so a repeat whose counts
    /// agree is refused too. A repeated key is malformed however the values
    /// fall.
    ///
    /// serde's derived map deserializer inserts in a loop, so without the
    /// check `{"COPY": 3, "COPY": 5}` would load as 5 with nothing said.
    ///
    /// The line is ambiguity, not repetition. A `Set` repeating an element
    /// denotes the same set, so collapsing loses nothing; a `Seq` repeating
    /// one is a sequence doing its job.
    #[test]
    fn test_only_a_repeated_frequency_is_refused() {
        let refused = [
            (r#"{"Freq":{"COPY":3,"COPY":5}}"#, "COPY"),
            (
                r#"{"KgramFreq":[[["CALL","COPY"],3],[["CALL","COPY"],5]]}"#,
                "CALL",
            ),
            // and the same repeats with counts that agree, which are refused
            // for being repeats rather than for disagreeing
            (r#"{"Freq":{"COPY":3,"COPY":3}}"#, "COPY"),
            (
                r#"{"KgramFreq":[[["CALL","COPY"],3],[["CALL","COPY"],3]]}"#,
                "CALL",
            ),
        ];
        for (json, named) in refused {
            let msg = serde_json::from_str::<Data>(json)
                .expect_err("a repeated key must not be resolved by picking one")
                .to_string();
            assert!(msg.contains("listed more than once"), "{json}: {msg}");
            assert!(msg.contains(named), "does not say which one: {msg}");
        }

        // Nothing here can mean two things, so nothing is refused.
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

        let msg = serde_json::from_str::<Data>(
            r#"{"KgramFreq":[[["CALL","COPY"],3],[["CALL","COPY"],5]]}"#,
        )
        .expect_err("a repeated k-gram must not be resolved by picking one")
        .to_string();
        assert!(msg.contains("listed more than once"), "{msg}");
        assert!(msg.contains("CALL"), "does not say which one: {msg}");
    }

    /// `KgramSeq` and `KgramSet` write a k-gram as a list, the same list
    /// `KgramFreq`'s pairs hold, so the three families agree on what a k-gram
    /// looks like on disk.
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
        // the read failing, so they come back as a JSON error rather than an
        // I/O one.
        let not_utf8 = dir.path().join("not-utf8.json");
        std::fs::write(&not_utf8, b"{\"name\": \"\xff\xfe\"}").unwrap();
        assert!(matches!(
            Birthmark::try_from(not_utf8).unwrap_err(),
            Error::Json(..)
        ));
    }
}
