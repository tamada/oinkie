use crate::Result;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub trait Lifter {
    fn lift(&self, input: &Path, output: &Path) -> Result<()>;
}

#[derive(Debug, ValueEnum, Clone, Copy, Serialize, Deserialize)]
#[clap(rename_all = "kebab-case")]
pub enum LifterType {
    Ghidra,
    Angr,
    IDAPro,
    BinaryNinja,
}

/// The intermediate representation a lifted program is written in.
///
/// Named after the representation rather than the tool that produced it,
/// because one tool can produce several and they are not interchangeable —
/// Binary Ninja lifts to LLIL, MLIL or HLIL, each with its own vocabulary.
/// What everything downstream needs to know is which vocabulary it is looking
/// at: whether two birthmarks can be compared, and which `Op` type can read
/// the file.
///
/// Variants are declared ahead of the lifters that produce them, the way
/// [`LifterType`] already declares backends that are not implemented yet.
/// Binary Ninja is absent on purpose: it lifts to LLIL, MLIL or HLIL, and
/// which of those is worth using has to be settled by measurement rather than
/// named in advance.
#[derive(Debug, ValueEnum, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
#[clap(rename_all = "kebab-case")]
pub enum Ir {
    /// Ghidra's P-Code, as refined by the decompiler — what `HighFunction`
    /// yields, rather than raw lifted P-Code.
    #[default]
    GhidraPcode,
    /// The Hex-Rays microcode, the representation IDA Pro's decompiler works
    /// in. Unlike Binary Ninja's, there is only one of it, so it can be named
    /// before the lifter that reads it exists.
    IdaMicrocode,
}

impl Ir {
    /// The representations this build can actually read a lifted file in.
    ///
    /// [`Ir`] names representations ahead of the code that reads them, so the
    /// two lists are not the same and the difference is what
    /// [`crate::Error::UnsupportedIr`] reports.
    pub fn readable() -> &'static [Ir] {
        &[Ir::GhidraPcode]
    }
}

impl std::fmt::Display for Ir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ir::GhidraPcode => write!(f, "ghidra-pcode"),
            Ir::IdaMicrocode => write!(f, "ida-microcode"),
        }
    }
}

impl std::str::FromStr for Ir {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "ghidra-pcode" => Ok(Ir::GhidraPcode),
            "ida-microcode" => Ok(Ir::IdaMicrocode),
            _ => Err(crate::Error::Parse(format!(
                "{s}: unknown intermediate representation"
            ))),
        }
    }
}

pub struct LifterBuilder {
    lifter_type: LifterType,
    home: Option<PathBuf>,
    script: Option<PathBuf>,
    intermediate_dir: Option<PathBuf>,
}

impl LifterBuilder {
    pub fn new(lifter_type: LifterType) -> Self {
        Self {
            lifter_type,
            home: None,
            script: None,
            intermediate_dir: None,
        }
    }

    pub fn home(mut self, home: Option<PathBuf>) -> Self {
        self.home = home;
        self
    }

    pub fn script(mut self, script: Option<PathBuf>) -> Self {
        self.script = script;
        self
    }

    pub fn intermediate_dir(mut self, dir: Option<PathBuf>) -> Self {
        self.intermediate_dir = dir;
        self
    }

    pub fn build(self) -> Result<Box<dyn Lifter + Sync>> {
        match self.lifter_type {
            LifterType::Ghidra => {
                let home = find_ghidra_home(self.home.as_deref())?;
                Ok(Box::new(crate::ghidra::lifter::GhidraLifter::new(
                    home,
                    self.script,
                    self.intermediate_dir,
                )))
            }
            LifterType::Angr => Err(crate::Error::Parse(
                "angr lifter is not yet implemented.".to_string(),
            )),
            LifterType::IDAPro => Err(crate::Error::Parse(
                "IDA Pro lifter is not yet implemented.".to_string(),
            )),
            LifterType::BinaryNinja => Err(crate::Error::Parse(
                "Binary Ninja lifter is not yet implemented.".to_string(),
            )),
        }
    }
}

pub fn find_ghidra_home(home_opt: Option<&Path>) -> Result<PathBuf> {
    crate::ghidra::lifter::find_ghidra_home(home_opt)
}
