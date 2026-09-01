use std::path::{Path, PathBuf};

use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::lift::Ir;
use crate::{Error, Iterable, Result};

#[derive(Debug, Serialize, Deserialize)]
pub struct Program<T> {
    #[serde(rename = "program")]
    name: String,
    path: PathBuf,
    /// Which intermediate representation the operations below are written in.
    ///
    /// Defaulted rather than required, so that files lifted before the field
    /// existed still load. Every such file was produced by Ghidra, since no
    /// other lifter has been implemented, so the default is right for all of
    /// them.
    ///
    /// It is a fallback, not a deduction: nothing here can tell a historical
    /// Ghidra file from a hand-written or third-party one that simply omits
    /// the field, and both will read as Ghidra's. Anything written by this
    /// crate carries the field, so the fallback only has to cover files it
    /// did not write.
    #[serde(default)]
    ir: crate::lift::Ir,
    symbols: FxHashMap<String, String>,
    functions: Vec<Function<T>>,
    #[serde(skip)]
    pub json_path: Option<PathBuf>,
}

impl<T> TryFrom<PathBuf> for Program<T>
where
    T: DeserializeOwned + crate::Op,
{
    type Error = Error;

    /// Read whole, then parsed from the bytes, for the reason
    /// [`AnyProgram::load`] already did it that way: serde_json's `IoRead`
    /// takes one byte at a time, so `from_reader` on an unbuffered `File` is
    /// one `read` syscall per byte — 3.3 s against 25 ms on a 10 MB lifted
    /// file (#51).
    fn try_from(path: PathBuf) -> Result<Self> {
        let bytes = std::fs::read(&path).map_err(|e| Error::Io(path.clone(), e))?;
        serde_json::from_slice(&bytes).map_err(|e| Error::Json(path, e))
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

/// A lifted program whose operation type was chosen by the file rather than by
/// the caller.
///
/// [`Program`] is generic over its operation type, but nothing was deciding
/// that type: every caller wrote `Program<ghidra::Op>` and a file lifted by
/// anything else would have been read against P-Code's vocabulary. The
/// decision belongs to the `ir` field the file carries, and this is where it
/// is made.
///
/// One variant is not an oversight. Ghidra is the only lifter implemented, so
/// it is the only representation with an operation type to read into; naming
/// the others here would mean inventing types for opcodes nobody has seen yet.
/// Adding a lifter adds a variant, and the compiler then points at every place
/// that has to account for it — which is the property the enum exists for.
pub enum AnyProgram {
    GhidraPcode(Program<crate::ghidra::Op>),
}

/// Just enough of a lifted file to learn which representation it is in.
///
/// Deserializing this skips the operations rather than building them, so
/// reading the representation costs a scan of the text and no allocation.
#[derive(Deserialize)]
struct IrProbe {
    #[serde(default)]
    ir: Ir,
}

impl AnyProgram {
    /// Reads a lifted program, choosing the operation type from the file's own
    /// `ir` field.
    ///
    /// A representation with no reader is refused by name. Read as P-Code it
    /// would fail anyway, since the opcode enum is closed — but on the first
    /// foreign opcode it happened to meet, reported as an unknown variant
    /// among seventy-five alternatives rather than as the one fact that
    /// matters.
    pub fn load(path: &Path) -> Result<Self> {
        // Read once and deserialize twice from the same bytes: the probe has
        // to see the file before the reader can be chosen, and reading it
        // again would cost more than the scan does.
        let bytes = std::fs::read(path).map_err(|e| Error::Io(path.to_path_buf(), e))?;
        let json_err = |e| Error::Json(path.to_path_buf(), e);
        let probe: IrProbe = serde_json::from_slice(&bytes).map_err(json_err)?;
        match probe.ir {
            Ir::GhidraPcode => serde_json::from_slice(&bytes)
                .map(Self::GhidraPcode)
                .map_err(json_err),
            ir => Err(Error::UnsupportedIr(path.to_path_buf(), ir)),
        }
    }

    /// The representation this program's operations are written in.
    pub fn ir(&self) -> Ir {
        match self {
            Self::GhidraPcode(p) => p.ir(),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::GhidraPcode(p) => p.name(),
        }
    }

    pub fn path(&self) -> &Path {
        match self {
            Self::GhidraPcode(p) => p.path(),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::GhidraPcode(p) => p.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn set_json_path(&mut self, path: PathBuf) {
        match self {
            Self::GhidraPcode(p) => p.set_json_path(path),
        }
    }
}

impl crate::prelude::CsvInfo for AnyProgram {
    fn csv_info(&self) -> String {
        match self {
            Self::GhidraPcode(p) => p.csv_info(),
        }
    }

    fn names(&self) -> Vec<String> {
        match self {
            Self::GhidraPcode(p) => p.names(),
        }
    }
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

    /// The point of the dispatch: the file decides, not the caller.
    #[test]
    fn test_any_program_reads_ghidra_pcode() {
        let program = AnyProgram::load(Path::new("testdata/hello_world/pcodes/hello_clang.json"))
            .expect("the fixture must load");
        assert_eq!(program.ir(), Ir::GhidraPcode);
        assert_eq!(program.name(), "hello_clang");
        assert_eq!(program.len(), 1);
    }

    /// A file written before the `ir` field existed carries no representation
    /// to dispatch on, and must still reach the reader it was written for.
    #[test]
    fn test_any_program_reads_a_file_without_the_field() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("legacy.json");
        std::fs::write(
            &path,
            r#"{"program":"legacy","path":"bin/legacy","symbols":{},"functions":[]}"#,
        )
        .unwrap();
        let program = AnyProgram::load(&path).expect("a file without the field must still load");
        assert_eq!(program.ir(), Ir::GhidraPcode);
    }

    /// Read as P-Code, a foreign file fails on whichever of its opcodes came
    /// first, reported as an unknown variant among seventy-five alternatives.
    /// Naming the representation instead says the one thing the user can act
    /// on.
    #[test]
    fn test_any_program_refuses_a_representation_it_cannot_read() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("foreign.json");
        std::fs::write(
            &path,
            r#"{"program":"foreign","path":"bin/foreign","ir":"ida-microcode",
                "symbols":{},"functions":[{"name":"main","ops":[
                  {"op":"m_call","inputs":["r0"]}]}]}"#,
        )
        .unwrap();
        match AnyProgram::load(&path) {
            Err(Error::UnsupportedIr(_, ir)) => assert_eq!(ir, Ir::IdaMicrocode),
            other => panic!("expected UnsupportedIr, got {other:?}", other = other.err()),
        }
    }

    /// The closed opcode enum is the one Ghidra assumption that fails loudly,
    /// and dispatching on the representation must not have loosened it.
    #[test]
    fn test_an_unknown_opcode_is_still_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bogus.json");
        std::fs::write(
            &path,
            r#"{"program":"bogus","path":"bin/bogus","ir":"ghidra-pcode",
                "symbols":{},"functions":[{"name":"main","ops":[
                  {"op":"LLIL_SET_REG","inputs":["(register, 0x0, 8)"]}]}]}"#,
        )
        .unwrap();
        assert!(matches!(AnyProgram::load(&path), Err(Error::Json(_, _))));
    }

    /// The loader reads the file whole and parses the bytes (#51). Reading it
    /// whole is where a change of error semantics could hide, so the three
    /// ways it fails are pinned: the file not being there is an IO error, and
    /// both ways its *content* can be wrong are JSON errors.
    ///
    /// The last of the three is the one worth having. `read_to_string` +
    /// `from_str` would look like the same thing and report `Error::Io` for
    /// bytes that are not UTF-8, moving a fact about the file's content into
    /// the category for the read failing.
    #[test]
    fn test_loading_a_program_reports_the_right_kind_of_failure() {
        let dir = tempfile::tempdir().unwrap();

        let missing = dir.path().join("missing.json");
        assert!(matches!(
            Program::<crate::ghidra::Op>::try_from(missing),
            Err(Error::Io(..))
        ));

        let broken = dir.path().join("broken.json");
        std::fs::write(&broken, b"{ not json").unwrap();
        assert!(matches!(
            Program::<crate::ghidra::Op>::try_from(broken),
            Err(Error::Json(..))
        ));

        let not_utf8 = dir.path().join("not-utf8.json");
        std::fs::write(&not_utf8, b"{\"program\": \"\xff\xfe\"}").unwrap();
        assert!(matches!(
            Program::<crate::ghidra::Op>::try_from(not_utf8),
            Err(Error::Json(..))
        ));
    }

    /// Reading the file whole must not change what is parsed out of it. The
    /// fixture is real lifter output, so this compares the loader against
    /// parsing the same text directly.
    #[test]
    fn test_reading_the_file_whole_parses_what_the_text_says() {
        let fixture = Path::new("testdata/hello_world/pcodes/hello_clang.json");
        let loaded: Program<crate::ghidra::Op> = fixture.try_into().unwrap();
        let parsed: Program<crate::ghidra::Op> =
            serde_json::from_str(&std::fs::read_to_string(fixture).unwrap()).unwrap();
        assert_eq!(loaded.name(), parsed.name());
        assert_eq!(loaded.ir(), parsed.ir());
        assert_eq!(loaded.len(), parsed.len());
        assert_eq!(loaded.path(), parsed.path());
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
