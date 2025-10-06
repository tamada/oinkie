use std::path::{Path, PathBuf};

use clap::ValueEnum;

use crate::birthmarks::{Birthmark, BirthmarkType, Element, Info};
use crate::{OinkieError, Result};

mod functions;
mod opcodes;
mod operands;

pub enum Source {
    BC,
    IR,
}

/// The extraction mode of the birthmark.
#[derive(Clone, Debug, PartialEq, Eq, Hash, ValueEnum, serde::Serialize, serde::Deserialize)]
pub enum Mode {
    /// a single IR code into a birthmark object.
    File,
    /// a function in IR code into a birthmark object.
    Function,
    /// a basic block in functions into a birthmark object.
    BasicBlock,
}

pub fn from_str(string: &str, bt: &BirthmarkType, mode: &Mode) -> Result<Vec<Birthmark>> {
    match llvm_ir::Module::from_ir_str(string) {
        Ok(module) => extract(&module, PathBuf::from("<str>"), bt, mode),
        Err(e) => Err(OinkieError::Format(format!("Failed to parse IR from reader: {}", e))),
    }
}

pub fn from_path<P: AsRef<Path>>(path: P, bt: &BirthmarkType, mode: &Mode) -> Result<Vec<Birthmark>> {
    let path = path.as_ref().to_path_buf();
    match find_type(&path) {
        Ok(t) => match parse_impl(&path, t) {
            Ok(module) => extract(&module, path, bt, mode),
            Err(e) => Err(e),
        },
        Err(e) => Err(e),
    }
}

pub fn extract<P: AsRef<Path>>(module: &llvm_ir::Module, path: P, bt: &BirthmarkType, mode: &Mode) -> Result<Vec<Birthmark>> {
    let path = path.as_ref().to_path_buf();
    let mut extractor = build_extractor(bt, mode);
    extract_birthmarks_impl(&module, &mut extractor, &path)
}

fn build_extractor(bt: &BirthmarkType, mode: &Mode) -> Box<dyn Extractor> {
    use BirthmarkType::*;
    let extractor: Box<dyn Extractor> = match bt {
        OpSeq => Box::new(opcodes::SeqExtractor::new()),
        OpSet => Box::new(opcodes::SetExtractor::new()),
        OpFreq => Box::new(opcodes::FreqExtractor::new()),
        UniGram => Box::new(opcodes::KGramExtractor::new(1)),
        BiGram => Box::new(opcodes::KGramExtractor::new(2)),
        TriGram => Box::new(opcodes::KGramExtractor::new(3)),
        TetraGram => Box::new(opcodes::KGramExtractor::new(4)),
        PentaGram => Box::new(opcodes::KGramExtractor::new(5)),
        HexaGram => Box::new(opcodes::KGramExtractor::new(6)),
        HeptaGram => Box::new(opcodes::KGramExtractor::new(7)),
        OctaGram => Box::new(opcodes::KGramExtractor::new(8)),
        Sfc => Box::new(functions::SeqNames::new()),
        Ffc => Box::new(functions::FreqNames::new()),
    };
    match mode {
        Mode::Function => Box::new(FunctionModeExtractor::new(extractor)),
        Mode::BasicBlock => Box::new(BBModeExtractor::new(extractor)),
        Mode::File => Box::new(FileModeExtractor::new(extractor)),
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

pub trait Extractor {
    fn btype(&self) -> BirthmarkType;
    fn visit(&mut self, module: &llvm_ir::Module, path: &PathBuf);
    fn visit_func(&mut self, func: &llvm_ir::Function);
    fn visit_bb(&mut self, bb: &llvm_ir::basicblock::BasicBlock);
    fn visit_inst(&mut self, instr: &llvm_ir::Instruction) -> Result<Option<Element>>;
    fn visit_bb_end(&mut self, term: &llvm_ir::terminator::Terminator) -> Result<Vec<Element>>;
    fn visit_func_end(&mut self, func: &llvm_ir::Function) -> Result<Vec<Element>>;
    fn visit_end(&mut self, module: &llvm_ir::Module) -> Result<Vec<Element>>;
    fn finish(&self) -> Result<Vec<Birthmark>>;
    fn clear(&mut self);
}

struct FunctionModeExtractor {
    info: Option<Info>,
    delegates: Box<dyn Extractor>,
    birthmarks: Vec<Birthmark>,
}

impl FunctionModeExtractor {
    pub fn new(delegate: Box<dyn Extractor>) -> Self {
        Self {
            info: None,
            delegates: delegate,
            birthmarks: vec![],
        }
    }
}
impl Extractor for FunctionModeExtractor {
    fn btype(&self) -> BirthmarkType {
        self.delegates.btype()
    }
    fn visit_func(&mut self, func: &llvm_ir::Function) {
        self.delegates.visit_func(func);
        self.info = self.info.as_mut().map(|info| Info::new_from(&info, func.name.clone()));
    }
    fn visit_func_end(&mut self, func: &llvm_ir::Function) -> Result<Vec<Element>> {
        match self.delegates.visit_func_end(func) {
            Ok(elements) => {
                let birthmark = Birthmark::new(self.info.clone().unwrap(), elements.clone());
                self.birthmarks.push(birthmark);
                self.clear();
                Ok(elements)
            },
            Err(e) => Err(e),
        }
    }
    fn clear(&mut self) {
        self.delegates.clear();
    }
    
    fn visit(&mut self, module: &llvm_ir::Module, path: &PathBuf) {
        self.info = Some(Info::new(path.to_string_lossy().into(), path.clone(), self.btype(), Mode::Function));
        self.delegates.visit(module, path);
    }
    
    fn visit_bb(&mut self, bb: &llvm_ir::basicblock::BasicBlock) {
        self.delegates.visit_bb(bb);
    }
    
    fn visit_inst(&mut self, instr: &llvm_ir::Instruction) -> Result<Option<Element>> {
        self.delegates.visit_inst(instr)
    }
    
    fn visit_bb_end(&mut self, term: &llvm_ir::terminator::Terminator) -> Result<Vec<Element>> {
        self.delegates.visit_bb_end(term)
    }
    
    fn visit_end(&mut self, module: &llvm_ir::Module) -> Result<Vec<Element>> {
        self.delegates.visit_end(module)
    }
    fn finish(&self) -> Result<Vec<Birthmark>> {
        Ok(self.birthmarks.clone())
    }
}

struct BBModeExtractor {
    info: Option<Info>,
    delegates: Box<dyn Extractor>,
    birthmarks: Vec<Birthmark>,
}

impl BBModeExtractor {
    pub fn new(delegate: Box<dyn Extractor>) -> Self {
        Self {
            info: None,
            delegates: delegate,
            birthmarks: vec![],
        }
    }
}
impl Extractor for BBModeExtractor {
    fn visit(&mut self, module: &llvm_ir::Module, path: &PathBuf) {
        self.delegates.visit(module, path);
        self.info = Some(Info::new(path.to_string_lossy().into(), path.clone(), self.btype(), Mode::Function));
    }

    fn visit_func(&mut self, func: &llvm_ir::Function) {
        self.delegates.visit_func(func);
    }

    fn visit_bb(&mut self, bb: &llvm_ir::basicblock::BasicBlock) {
        self.delegates.visit_bb(bb);
        self.info = self.info.as_mut().map(|info| Info::new_from(&info, bb.name.clone().to_string()));
    }

    fn visit_inst(&mut self, instr: &llvm_ir::Instruction) -> Result<Option<Element>> {
        self.delegates.visit_inst(instr)
    }

    fn visit_bb_end(&mut self, term: &llvm_ir::terminator::Terminator) -> Result<Vec<Element>> {
        match self.delegates.visit_bb_end(term) {
            Err(e) => Err(e),
            Ok(elements) => {
                let birthmark = Birthmark::new(self.info.clone().unwrap(), elements.clone());
                self.birthmarks.push(birthmark);
                self.clear();
                Ok(elements)
            }
        }
    }

    fn visit_func_end(&mut self, func: &llvm_ir::Function) -> Result<Vec<Element>> {
        self.delegates.visit_func_end(func)
    }

    fn visit_end(&mut self, module: &llvm_ir::Module) -> Result<Vec<Element>> {
        self.delegates.visit_end(module)
    }

    fn clear(&mut self) {
        self.delegates.clear();
    }
    
    fn btype(&self) -> BirthmarkType {
        self.delegates.btype()
    }

    fn finish(&self) -> Result<Vec<Birthmark>> {
        Ok(self.birthmarks.clone())
    }
}

struct FileModeExtractor {
    info: Option<Info>,
    delegates: Box<dyn Extractor>,
    birthmarks: Vec<Birthmark>,
}
impl FileModeExtractor {
    pub fn new(delegate: Box<dyn Extractor>) -> Self {
        Self {
            info: None,
            delegates: delegate,
            birthmarks: vec![],
        }
    }
}
impl Extractor for FileModeExtractor {
    fn visit(&mut self, module: &llvm_ir::Module, path: &PathBuf) {
        self.delegates.visit(module, path);
        self.info = Some(Info::new(path.to_string_lossy().into(), path.clone(), self.btype(), Mode::Function));
    }

    fn visit_func(&mut self, func: &llvm_ir::Function) {
        self.delegates.visit_func(func);
    }

    fn visit_bb(&mut self, bb: &llvm_ir::basicblock::BasicBlock) {
        self.delegates.visit_bb(bb);
    }

    fn visit_inst(&mut self, instr: &llvm_ir::Instruction) -> Result<Option<Element>> {
        self.delegates.visit_inst(instr)
    }

    fn visit_bb_end(&mut self, term: &llvm_ir::terminator::Terminator) -> Result<Vec<Element>> {
        self.delegates.visit_bb_end(term)
    }

    fn visit_func_end(&mut self, func: &llvm_ir::Function) -> Result<Vec<Element>> {
        self.delegates.visit_func_end(func)
    }

    fn visit_end(&mut self, module: &llvm_ir::Module) -> Result<Vec<Element>> {
        match self.delegates.visit_end(module) {
            Err(e) => Err(e),
            Ok(elements) => {
                let birthmark = Birthmark::new(self.info.clone().unwrap(), elements.clone());
                self.birthmarks.push(birthmark);
                self.clear();
                Ok(elements)
            }
        }
    }

    fn clear(&mut self) {
        self.delegates.clear();
    }
    
    fn btype(&self) -> BirthmarkType {
        self.delegates.btype()
    }
    
    fn finish(&self) -> Result<Vec<Birthmark>> {
        Ok(self.birthmarks.clone())
    }
}

fn extract_birthmarks_impl(module: &llvm_ir::Module, extractor: &mut Box<dyn Extractor>, path: &PathBuf) -> Result<Vec<Birthmark>> {
    extractor.visit(module, &path);
    for func in &module.functions {
        extractor.visit_func(func);
        for bb in &func.basic_blocks {
            extractor.visit_bb(bb);
            for instr in &bb.instrs {
                let _ = extractor.visit_inst(instr);
            }
            let _ = extractor.visit_bb_end(&bb.term);
        }
        let _ = extractor.visit_func_end(func);
    }
    let _ = extractor.visit_end(module);
    extractor.finish()
}