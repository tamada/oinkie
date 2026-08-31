use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::ghidra::pcode::PcodeOp;
use crate::program::Program;
use crate::{Error, Result};
use serde::{Deserialize, Serialize, Serializer, de::DeserializeOwned};

pub(crate) mod lifter;
pub mod pcode;

impl<T> TryFrom<PathBuf> for Program<T>
where
    T: DeserializeOwned + crate::Op,
{
    type Error = Error;

    fn try_from(path: PathBuf) -> Result<Self> {
        std::fs::File::open(&path)
            .map_err(|e| Error::Io(path.clone(), e))
            .and_then(|f| serde_json::from_reader(f).map_err(|e| Error::Json(path, e)))
    }
}

impl<T> TryFrom<&Path> for Program<T>
where
    T: DeserializeOwned + crate::Op,
{
    type Error = Error;

    fn try_from(path: &Path) -> Result<Self> {
        Self::try_from(path.to_path_buf())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Op {
    op: PcodeOp,
    out: Option<String>,
    inputs: Vec<String>,
}

impl crate::Op for Op {
    fn mnemonic(&self) -> &str {
        self.op.mnemonic()
    }

    fn code(&self) -> u32 {
        self.op as u32
    }

    fn inputs(&self) -> &[String] {
        &self.inputs
    }

    fn ret(&self) -> Option<&str> {
        self.out.as_deref()
    }

    fn is_call(&self) -> bool {
        // CALLOTHER is not among these: it is Ghidra's escape hatch for
        // instruction semantics its model does not cover, and its first
        // operand indexes a table of user-defined operations rather than
        // naming a function.
        //
        // CALLIND is, because it is a call. Its target lives in a register or
        // a temporary, so `symbol_key` yields nothing for it and it
        // contributes to no birthmark today; including it keeps the predicate
        // an answer about P-Code rather than about what happens to resolve.
        matches!(self.op, PcodeOp::Call | PcodeOp::Callind)
    }

    fn symbol_key(&self) -> Option<String> {
        // Only a call into ram names an address the symbol table could carry;
        // a target held in a register or a temporary is resolved at run time
        // and has no name to find here.
        let target: Value = self.inputs.first()?.parse().ok()?;
        match target.storage {
            StorageType::Ram => Some(format!("0x{:x}", target.address)),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Value {
    pub storage: StorageType,
    pub address: u64,
    pub size: u32,
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum StorageType {
    Ram,
    Register,
    Const,
    Unique,
    Stack,
    Variable,
}

impl std::fmt::Display for StorageType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match self {
            StorageType::Ram => "ram",
            StorageType::Register => "register",
            StorageType::Const => "const",
            StorageType::Unique => "unique",
            StorageType::Stack => "stack",
            StorageType::Variable => "variable",
        };
        write!(f, "{}", s)
    }
}

/// Parsing like "(ram, 0x1000497b0, 8)".
impl FromStr for Value {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let s = s.trim_matches(|c| c == '(' || c == ')'); // strip parentheses
        let parts: Vec<&str> = s.split(',').map(|p| p.trim()).collect();

        if parts.len() != 3 {
            return Err(Error::Parse("Invalid format".to_string()));
        }

        let storage = match parts[0] {
            "ram" => StorageType::Ram,
            "register" => StorageType::Register,
            "const" => StorageType::Const,
            "unique" => StorageType::Unique,
            "stack" => StorageType::Stack,
            "VARIABLE" => StorageType::Variable,
            _ => return Err(Error::Parse(format!("Unknown storage: {}", parts[0]))),
        };

        // If prefix is 0x, parse as hexadecimal; otherwise, parse as decimal
        let address = if parts[1].starts_with("0x") {
            u64::from_str_radix(&parts[1][2..], 16)
        } else {
            parts[1].parse()
        }
        .map_err(|e| Error::ParseInt(parts[1].to_string(), e))?;

        let size = parts[2]
            .parse()
            .map_err(|e| Error::ParseInt(parts[2].to_string(), e))?;

        Ok(Value {
            storage,
            address,
            size,
        })
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("({}, 0x{:x}, {})", self.storage, self.address, self.size);
        serializer.serialize_str(&s)
    }
}

#[cfg(test)]
mod tests {
    use crate::program::Program;
    use crate::{Iterable, Op};

    use super::*;

    #[test]
    fn parse_pcode_value() {
        let s = "(ram, 0x1000497b0, 8)";
        let value: Value = s.parse().expect("Failed to parse PCodeValue");
        assert_eq!(value.storage, StorageType::Ram);
        assert_eq!(value.address, 0x1000497b0);
        assert_eq!(value.size, 8);
    }

    #[test]
    fn serialize_pcode_value() {
        let value = super::Value {
            storage: super::StorageType::Register,
            address: 0x2000,
            size: 4,
        };
        let s = serde_json::to_string(&value).unwrap();
        assert_eq!(s, "\"(register, 0x2000, 4)\"");
    }

    #[test]
    fn parse_pcode_json() {
        let file = std::fs::File::open("testdata/hello_world/pcodes/hello_clang.json")
            .expect("Failed to open JSON file");
        let r: Program<super::Op> = serde_json::from_reader(file).expect("Failed to parse JSON");
        assert_eq!(r.name(), "hello_clang");
        assert_eq!(
            r.path(),
            std::path::Path::new("testdata/hello_world/bin/hello_clang")
        );
        assert_eq!(r.len(), 1);

        let f1 = r.iter().next().unwrap();
        assert_eq!(f1.name(), "entry");
        let op1 = f1.get(0).unwrap();
        assert_eq!(op1.mnemonic(), "CALL");
        assert_eq!(op1.inputs().len(), 2);
        assert_eq!(op1.ret(), None);

        let op2 = f1.get(1).unwrap();
        assert_eq!(op2.mnemonic(), "COPY");
        assert_eq!(op2.inputs().len(), 1);
        assert_eq!(op2.ret(), Some("(unique, 0x10000009, 8)"));
    }

    fn op(kind: PcodeOp) -> super::Op {
        super::Op {
            op: kind,
            out: None,
            inputs: vec!["(ram, 0x100000480, 8)".to_string()],
        }
    }

    /// Which opcode is a call is P-Code's own business, so the answer lives
    /// with the P-Code operation rather than with the extractor that consumes
    /// it. CALLOTHER is Ghidra's escape hatch for instruction semantics its
    /// model does not cover — its first operand indexes a table of
    /// user-defined operations, not a function — so it is not a call.
    #[test]
    fn test_is_call_recognises_the_call_opcodes() {
        assert!(op(PcodeOp::Call).is_call(), "CALL is a call");
        assert!(op(PcodeOp::Callind).is_call(), "CALLIND is a call");
        for other in [
            PcodeOp::Callother,
            PcodeOp::Copy,
            PcodeOp::Branch,
            PcodeOp::Cbranch,
            PcodeOp::Branchind,
            PcodeOp::Return,
        ] {
            assert!(
                !op(other).is_call(),
                "{} is not a call",
                op(other).mnemonic()
            );
        }
    }

    #[test]
    fn parse_pcode_json2() {
        let file = std::fs::File::open("testdata/hello_world/pcodes/hello_gcc.json")
            .expect("Failed to open JSON file");
        let _r: Program<super::Op> = serde_json::from_reader(file).expect("Failed to parse JSON");
    }
}
