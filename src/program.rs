use std::path::{Path, PathBuf};

use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Program<T> {
    #[serde(rename = "program")]
    name: String,
    path: PathBuf,
    symbols: FxHashMap<String, String>,
    functions: Vec<Function<T>>,
}

impl<T> Program<T> {
    pub fn new(name: String, path: PathBuf, symbols: FxHashMap<String, String>, functions: Vec<Function<T>>) -> Self {
        Self { name, path, symbols, functions }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn len(&self) -> usize {
        self.functions.len()
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn iter(&self) -> impl Iterator<Item = &Function<T>> {
        self.functions.iter()
    }

    pub fn symbol(&self, addr: &str) -> Option<&String> {
        self.symbols.get(addr)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Function<T> {
    name: String,
    ops: Vec<T>
}

impl<T: crate::Op> Function<T> {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.ops.get(index)
    }

    pub fn ops(&self) -> impl Iterator<Item = &str> {
        self.ops.iter().map(|op| op.mnemonic())
    }

    pub fn len(&self) -> usize {
        self.ops.len()
    }

    pub fn ops_freq(&self) -> rustc_hash::FxHashMap<String, usize> {
        crate::extractor::seq_to_freq(self.ops().map(|s| s.to_string()))
    }
}
