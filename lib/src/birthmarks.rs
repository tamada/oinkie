use std::fmt::Display;
use std::{collections::HashMap, path::Path};
use std::path::PathBuf;
use clap::ValueEnum;
use serde::{Serialize, Deserialize};

use crate::extractors::Mode;
use crate::{OinkieError, Result};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Birthmark {
    pub info: Info,
    pub elements: Vec<Element>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Info {
    pub name: String,
    pub path: PathBuf,
    pub btype: BirthmarkType,
    pub mode: Mode,
}

impl Info {
    pub fn new(name: String, path: PathBuf, btype: BirthmarkType, mode: Mode) -> Self {
        Self { name, path, btype, mode }
    }

    pub fn new_from(&self, name: String) -> Self {
        Self { name: name.clone(), path: self.path.clone(), btype: self.btype.clone(), mode: self.mode.clone() }
    }

    pub fn is_same_type(&self, other: &Info) -> bool {
        self.btype == other.btype && self.mode == other.mode
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, ValueEnum)]
pub enum BirthmarkType {
    #[clap(help = "sequence of function calls (function names)")]
    Sfc,
    #[clap(help = "frequencies of function calls (function names)")]
    Ffc,
    #[clap(help = "frequencies of opcodes")]
    OpFreq,
    #[clap(help = "set of opcodes")]
    OpSet,
    #[clap(help = "sequence of opcodes")]
    OpSeq,
    #[clap(help = "uni-grams of opcodes (1-gram)")]
    UniGram,
    #[clap(help = "bi-grams of opcodes (2-gram)")]
    BiGram,
    #[clap(help = "tri-grams of opcodes (3-gram)")]
    TriGram,
    #[clap(help = "tetra-grams of opcodes (4-gram)")]
    TetraGram,
    #[clap(help = "penta-grams of opcodes (5-gram)")]
    PentaGram,
    #[clap(help = "hexa-grams of opcodes (6-gram)")]
    HexaGram,
    #[clap(help = "hepta-grams of opcodes (7-gram)")]
    HeptaGram,
    #[clap(help = "octa-grams of opcodes (8-gram)")]
    OctaGram,
}

impl Display for BirthmarkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Birthmark {
    pub fn new(info: Info, elements: Vec<Element>) -> Self {
        Self {
            info,
            elements,
        }
    }

    pub fn from_path(path: &Path) -> Result<Self> {
        match std::fs::File::open(path) {
            Ok(file) => match serde_json::from_reader(file) {
                Ok(birthmark) => Ok(birthmark),
                Err(e) => Err(OinkieError::Json(e)),
            },
            Err(e) => Err(OinkieError::Io(e)),
        }
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Element> {
        self.elements.iter()
    }

    pub fn freq(&self) -> HashMap<&Element, usize> {
        to_freq_elements(&self.elements)
    }

    pub fn is_same_type(&self, other: &Birthmark) -> bool {
        self.info.is_same_type(&other.info)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(untagged)]
pub enum Element {
    Str(String),
    Int(i64),
    Kgram(Vec<String>),
    Freq(usize, String),
}

impl Element {
    pub fn is_same(&self, other: &Element) -> bool {
        match (self, other) {
            (Element::Str(a), Element::Str(b)) => a == b,
            (Element::Int(a), Element::Int(b)) => a == b,
            _ => false,
        }
    }
}

fn to_freq_elements(elements: &Vec<Element>) -> HashMap<&Element, usize> {
        let mut map = std::collections::HashMap::new();
        for e in elements {
            *map.entry(e).or_insert(0) += 1;
        }
        map
}
