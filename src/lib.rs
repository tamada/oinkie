use std::path::PathBuf;

use ndarray::ShapeError;

use crate::prelude::BirthmarkType;

pub mod ghidra;
pub mod prelude;
pub mod extractor;
mod compare;
mod birthmarks;
mod program;
mod llvm;
mod ninja;

pub type Result<T> = std::result::Result<T, Error>;

impl std::error::Error for Error {}

#[derive(Debug)]
pub enum Error {
    Array(Vec<Self>),
    BirthmarkType(String),
    Clap(clap::Error),
    Csv(csv::Error),
    InvalidPcode(u32),
    Io(PathBuf, std::io::Error),
    Json(PathBuf, serde_json::Error),
    LapJV(lapjv::LapJVError),
    Mismatch(BirthmarkType, BirthmarkType),
    Parse(String),
    ParseFloat(String, std::num::ParseFloatError),
    ParseInt(String, std::num::ParseIntError),
    ShapeError(ShapeError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Array(errs) => {
                let _ = write!(f, "Multiple errors:");
                for (i, err) in errs.iter().enumerate() {
                    write!(f, "\n  {}. {}", i + 1, err)?;
                }
                Ok(())
            },
            Error::BirthmarkType(t) => write!(f, "{t}: unknown birthmark type"),
            Error::Csv(e) => write!(f, "CSV error: {}", e),
            Error::InvalidPcode(code) => write!(f, "invalid pcode: {code}"),
            Error::Io(path, e) => write!(f, "IO error for {}: {}", path.display(), e),
            Error::Json(file, e) => write!(f, "{}: JSON error: {}", file.display(), e),
            Error::LapJV(e) => write!(f, "LapJV error: {}", e),
            Error::Mismatch(t1, t2) => write!(f, "Mismatched birthmark types: {} and {}", t1, t2),
            Error::ParseFloat(s, e) => write!(f, "{s}: Parse float error {e}"),
            Error::Parse(s) => write!(f, "Parse error: {}", s),
            Error::ParseInt(s, e) => write!(f, "{s}: Parse int error {e}"),
            Error::Clap(e) => write!(f, "Clap error: {}", e),
            Error::ShapeError(e) => write!(f, "Shape error: {}", e),
        }
    }
}

impl Error {
    pub fn vec_result_to_result_vec<T>(vec: Vec<Result<T>>) -> Result<Vec<T>> {
        let mut results = Vec::new();
        let mut errs = Vec::new();
        for r in vec {
            match r {
                Ok(v) => results.push(v),
                Err(e) => errs.push(e),
            }
        }
        Self::error_or(results, errs)
    }

    pub fn error_or<T>(result: T, errs: Vec<Self>) -> Result<T> {
        if errs.is_empty() {
            Ok(result)
        } else if errs.len() == 1 {
            Err(errs.into_iter().next().unwrap())
        } else {
            Err(Self::Array(errs))
        }
    }
}

pub trait Op {
    /// returns the mnemonic of the operation, e.g., "ADD", "SUB", etc.
    fn mnemonic(&self) -> &str;

    /// returns the unique code of the operation, e.g., the pcode opcode.
    fn code(&self) -> u32;

    /// returns the inputs of the operation, e.g., the source registers or memory locations.
    fn inputs(&self) -> Vec<String>;

    /// returns the output of the operation, e.g., the destination register or memory location.
    fn ret(&self) -> Option<String>;
}

pub trait Iterable {
    type Item;
    fn iter(&self) -> Box<dyn Iterator<Item = &Self::Item> + '_>;
}

pub trait Lifter {
    fn lift(&self, input: &std::path::Path, output: &std::path::Path) -> Result<()>;
}
