use std::{collections::HashMap, path::Path};
use std::path::PathBuf;
use clap::ValueEnum;
use serde::{Serialize, Deserialize};

use crate::{OinkieError, Result};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Birthmark {
    pub btype: BirthmarkType,
    pub elements: Vec<Element>,
    pub path: PathBuf,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, ValueEnum)]
pub enum BirthmarkType {
    #[clap(help = "Sequence of function calls (function names)")]
    Sfc,
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

impl Birthmark {
    pub fn new(btype: BirthmarkType, path: PathBuf, elements: Vec<Element>) -> Self {
        Self {
            btype,
            elements,
            path,
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
        self.btype == other.btype
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
