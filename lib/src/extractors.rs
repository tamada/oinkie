use std::path::{Path, PathBuf};

use crate::birthmarks::{Birthmark, BirthmarkType, Element};
use crate::{OinkieError, Result};

mod functions;
mod opcodes;
mod operands;

pub enum Source {
    BC,
    IR,
}

pub fn from_str(string: &str, bt: BirthmarkType) -> Result<Birthmark> {
    match llvm_ir::Module::from_ir_str(string) {
        Ok(module) => extract_impl(module, PathBuf::from("<str>"), bt),
        Err(e) => Err(OinkieError::Format(format!("Failed to parse IR from reader: {}", e))),
    }
}

pub fn from_path<P: AsRef<Path>>(path: P, bt: BirthmarkType) -> Result<Birthmark> {
    let path = path.as_ref().to_path_buf();
    match find_type(&path) {
        Ok(t) => match parse_impl(&path, t) {
            Ok(module) => extract_impl(module, path, bt),
            Err(e) => Err(e),
        },
        Err(e) => Err(e),
    }
}

pub fn extract_elements(module: llvm_ir::Module, bt: &BirthmarkType) -> Result<Vec<Element>> {
    use BirthmarkType::*;
    match bt {
        UniGram | OpSeq => opcodes::extract_seq(&module),
        OpSet => opcodes::extract_set(&module),
        OpFreq => opcodes::extract_freq(&module),
        BiGram => opcodes::extract_kgram(&module, 2),
        TriGram => opcodes::extract_kgram(&module, 3),
        TetraGram => opcodes::extract_kgram(&module, 4),
        PentaGram => opcodes::extract_kgram(&module, 5),
        HexaGram => opcodes::extract_kgram(&module, 6),
        HeptaGram => opcodes::extract_kgram(&module, 7),
        OctaGram => opcodes::extract_kgram(&module, 8),
        Sfc => functions::extract_names(&module),
    }
}

fn extract_impl<P: AsRef<Path>>(module: llvm_ir::Module, path: P, bt: BirthmarkType) -> Result<Birthmark> {
    let path = path.as_ref().to_path_buf();
    match extract_elements(module, &bt) {
        Ok(elements) => Ok(Birthmark::new(bt, path, elements)),
        Err(e) => Err(e),
    }
}

fn parse_impl(path: &PathBuf, source: Source) -> Result<llvm_ir::Module> {
    match source {
        Source::BC => parse_bc(path),
        Source::IR => parse_ir(path),
    }
}

fn parse_bc(path: &PathBuf) -> Result<llvm_ir::Module> {
    llvm_ir::Module::from_bc_path(path)
        .map_err(|e| OinkieError::Format(format!("Failed to parse BC file: {}", e)))
}

fn parse_ir(path: &PathBuf) -> Result<llvm_ir::Module> {
    llvm_ir::Module::from_ir_path(path)
        .map_err(|e| OinkieError::Format(format!("Failed to parse IR file: {}", e)))
}

fn find_type(path: &PathBuf) -> Result<Source> {
    if !path.exists() {
        Err(OinkieError::NotFound(path.clone()))
    } else if path.is_dir() {
        Err(OinkieError::NotFile(path.clone()))
    } else {
        match path.extension().and_then(|s| s.to_str()) {
            Some("bc") => Ok(Source::BC),
            Some("ll") => Ok(Source::IR),
            Some(ext) => Err(OinkieError::UnsupportedFormat(ext.to_string())),
            None => Err(OinkieError::NoExtension(path.to_string_lossy().to_string())),
        }
    }
}
