use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use clap::ValueEnum;
use crate::Result;

pub trait Lifter {
    fn lift(&self, input: &Path, output: &Path) -> Result<()>;
}

#[derive(Debug, ValueEnum, Clone, Copy, Serialize, Deserialize)]
#[clap(rename_all = "kebab-case")]
pub enum LifterType {
    Ghidra,
    Llvm,
    BinaryNinja,
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
            LifterType::Llvm => Err(crate::Error::Parse("LLVM lifter is not yet implemented.".to_string())),
            LifterType::BinaryNinja => Err(crate::Error::Parse("Binary Ninja lifter is not yet implemented.".to_string())),
        }
    }
}

pub fn find_ghidra_home(home_opt: Option<&Path>) -> Result<PathBuf> {
    crate::ghidra::lifter::find_ghidra_home(home_opt)
}
