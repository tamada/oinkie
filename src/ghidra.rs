use std::{path::{Path, PathBuf}, str::FromStr};

use serde::{Deserialize, Serialize, Serializer, de::DeserializeOwned};
use crate::{Result, Error};
use crate::ghidra::pcode::PcodeOp;
use crate::program::Program;

mod pcode;

impl<T> TryFrom<PathBuf> for Program<T> 
where
    T: DeserializeOwned + crate::Op
{
    type Error = Error;

    fn try_from(path: PathBuf) -> Result<Self> {
        std::fs::File::open(&path)
            .map_err(|e| Error::Io(path.clone(), e))
            .and_then(|f| serde_json::from_reader(f)
                .map_err(|e| Error::Json(path, e)))
    }
}

impl<T> TryFrom<&Path> for Program<T>
where
    T: DeserializeOwned + crate::Op
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

    fn inputs(&self) -> Vec<String> {
        self.inputs.clone()
    }

    fn ret(&self) -> Option<String> {
        self.out.clone()
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
        }.map_err(|e| Error::ParseInt(parts[1].to_string(), e))?;

        let size = parts[2].parse()
            .map_err(|e| Error::ParseInt(parts[2].to_string(), e))?;

        Ok(Value { storage, address, size })
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
            where D: serde::Deserializer<'de> 
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
    use crate::{Op, Iterable};

    use super::*;

    #[test]
    fn parse_pcode_value() {
        let s = "(ram, 0x1000497b0, 8)";
        let value: Value = s.parse()
            .expect("Failed to parse PCodeValue");
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
        let file = std::fs::File::open("testdata/bzip2/arm64-linux_gcc_O3.json")
            .expect("Failed to open JSON file");
        let r: Program<super::Op> = serde_json::from_reader(file)
            .expect("Failed to parse JSON");
        assert_eq!(r.name(), "bzip2");
        assert_eq!(r.path(), std::path::Path::new("/Users/tamada/researches/2026snpd_tamada/executables/arm64-linux/gcc_O3/bzip2"));
        assert_eq!(r.len(), 85);

        let f1 = r.iter().next().unwrap();
        assert_eq!(f1.name(), "_init");
        let op1 = f1.get(0).unwrap();
        assert_eq!(op1.mnemonic(), "SUBPIECE");
        assert_eq!(op1.inputs().len(), 2);
        assert_eq!(op1.ret(), Some("(register, 0x4000, 4)".into()));
    }


    #[test]
    fn parse_pcode_json2() {
        let file = std::fs::File::open("testdata/factorize/factorize_clang.json")
            .expect("Failed to open JSON file");
        let _r: Program<super::Op> = serde_json::from_reader(file)
            .expect("Failed to parse JSON");
    }
}