use std::path::{Path, PathBuf};

use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use crate::Iterable;

#[derive(Debug, Serialize, Deserialize)]
pub struct Program<T> {
    #[serde(rename = "program")]
    name: String,
    path: PathBuf,
    /// Which intermediate representation the operations below are written in.
    ///
    /// Defaulted rather than required so that files lifted before the field
    /// existed still load. The default is not a guess: Ghidra is the only
    /// lifter that has ever been implemented, so every file that can predate
    /// this field was produced by it. Once a second lifter ships, its output
    /// carries the field, and the default only ever applies to files that
    /// really are Ghidra's.
    #[serde(default)]
    ir: crate::lift::Ir,
    symbols: FxHashMap<String, String>,
    functions: Vec<Function<T>>,
    #[serde(skip)]
    pub json_path: Option<PathBuf>,
}

impl<T> crate::prelude::CsvInfo for Program<T> {
    fn csv_info(&self) -> String {
        let json_path = self
            .json_path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_default();
        format!(
            "program,{},{},{},{},{}",
            crate::compare::escape_csv_string(&self.name),
            crate::compare::escape_csv_string(&self.path.display().to_string()),
            self.symbols.len(),
            self.functions.len(),
            crate::compare::escape_csv_string(&json_path)
        )
    }

    fn names(&self) -> Vec<String> {
        self.functions.iter().map(|f| f.name.clone()).collect()
    }
}

impl<T> Program<T> {
    pub fn set_json_path(&mut self, path: PathBuf) {
        self.json_path = Some(path);
    }

    pub fn new(
        name: String,
        path: PathBuf,
        ir: crate::lift::Ir,
        symbols: FxHashMap<String, String>,
        functions: Vec<Function<T>>,
    ) -> Self {
        Self {
            name,
            path,
            ir,
            symbols,
            functions,
            json_path: None,
        }
    }

    /// The intermediate representation these operations are written in.
    pub fn ir(&self) -> crate::lift::Ir {
        self.ir
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn len(&self) -> usize {
        self.functions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.functions.is_empty()
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn symbols(&self) -> impl Iterator<Item = (&str, &str)> {
        self.symbols.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }

    pub fn symbol(&self, addr: &str) -> Option<&String> {
        self.symbols.get(addr)
    }
}

impl<T> Iterable for &Program<T> {
    type Item = Function<T>;
    fn iter(&self) -> Box<dyn Iterator<Item = &Self::Item> + '_> {
        Box::new(self.functions.iter())
    }
}

impl<T> Iterable for Program<T> {
    type Item = Function<T>;
    fn iter(&self) -> Box<dyn Iterator<Item = &Self::Item> + '_> {
        Box::new(self.functions.iter())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Function<T> {
    name: String,
    ops: Vec<T>,
}

impl<T: crate::Op> Function<T> {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.ops.iter()
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.ops.get(index)
    }

    pub fn ops(&self) -> impl Iterator<Item = &str> {
        self.ops.iter().map(|op| op.mnemonic())
    }

    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }

    pub fn len(&self) -> usize {
        self.ops.len()
    }

    pub fn ops_freq(&self) -> rustc_hash::FxHashMap<String, usize> {
        crate::extractor::seq_to_freq(self.ops().map(|s| s.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lift::Ir;

    /// A file lifted before the field existed still loads, and is understood
    /// as Ghidra's — which is not a guess, since no other lifter has ever
    /// produced one.
    #[test]
    fn test_ir_defaults_to_ghidra_pcode_when_absent() {
        let json = r#"{
            "program": "sample",
            "path": "bin/sample",
            "symbols": {},
            "functions": []
        }"#;
        let program: Program<crate::ghidra::Op> =
            serde_json::from_str(json).expect("a file without the field must still load");
        assert_eq!(program.ir(), Ir::GhidraPcode);
    }

    #[test]
    fn test_ir_is_read_from_the_file() {
        let json = r#"{
            "program": "sample",
            "path": "bin/sample",
            "ir": "ghidra-pcode",
            "symbols": {},
            "functions": []
        }"#;
        let program: Program<crate::ghidra::Op> = serde_json::from_str(json).unwrap();
        assert_eq!(program.ir(), Ir::GhidraPcode);
    }

    #[test]
    fn test_ir_survives_a_round_trip() {
        let program: Program<crate::ghidra::Op> = Program::new(
            "sample".to_string(),
            PathBuf::from("bin/sample"),
            Ir::GhidraPcode,
            FxHashMap::default(),
            vec![],
        );
        let json = serde_json::to_string(&program).unwrap();
        assert!(
            json.contains("\"ir\":\"ghidra-pcode\""),
            "the field must be written, not only read: {json}"
        );
        let back: Program<crate::ghidra::Op> = serde_json::from_str(&json).unwrap();
        assert_eq!(back.ir(), Ir::GhidraPcode);
    }

    /// The fixtures are real lifter output, so they must carry what the
    /// current script writes.
    #[test]
    fn test_fixtures_record_their_ir() {
        for fixture in [
            "testdata/hello_world/pcodes/hello_clang.json",
            "testdata/hello_world/pcodes/hello_gcc.json",
        ] {
            let program: Program<crate::ghidra::Op> = Path::new(fixture)
                .try_into()
                .unwrap_or_else(|e| panic!("{fixture}: {e}"));
            assert_eq!(program.ir(), Ir::GhidraPcode, "fixture: {fixture}");
            let raw = std::fs::read_to_string(fixture).unwrap();
            assert!(
                raw.contains("\"ir\""),
                "{fixture}: the fixture predates the field and should be regenerated"
            );
        }
    }
}
