//! Lifted programs: what a lifter writes, read back.
//!
//! [`Program`] is the public face, and is opaque. Inside, a
//! `TypedProgram<T>` holds the functions as operations of the type its
//! representation uses -- Ghidra's P-Code, Binary Ninja's three ILs, IDA's
//! microcode -- and the file's `ir` field decides which one is read.

use std::path::{Path, PathBuf};

use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::lift::Ir;
use crate::{Error, Iterable, Result};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct TypedProgram<T> {
    #[serde(rename = "program")]
    name: String,
    path: PathBuf,
    /// Which intermediate representation the operations below are written in.
    ///
    /// Defaulted to Ghidra's P-Code rather than required, so that a file
    /// without the field still loads: Ghidra's lifting script is the one that
    /// wrote such files. It is a fallback, not a deduction -- a hand-written
    /// file that omits the field reads as Ghidra's too -- and this crate always
    /// writes the field.
    #[serde(default)]
    ir: crate::lift::Ir,
    symbols: FxHashMap<String, String>,
    functions: Vec<TypedFunction<T>>,
    #[serde(skip)]
    pub(crate) json_path: Option<PathBuf>,
}

impl<T> TryFrom<PathBuf> for TypedProgram<T>
where
    T: DeserializeOwned + crate::Op,
{
    type Error = Error;

    /// Reads the file whole and parses the bytes, as [`Program::load`] does:
    /// serde_json reads a reader one byte at a time, so `from_reader` on an
    /// unbuffered `File` costs a system call per byte -- seconds for a large
    /// lifted file.
    fn try_from(path: PathBuf) -> Result<Self> {
        let bytes = std::fs::read(&path).map_err(|e| Error::Io(path.clone(), e))?;
        serde_json::from_slice(&bytes).map_err(|e| Error::Json(path, e))
    }
}

impl<T> TryFrom<&Path> for TypedProgram<T>
where
    T: DeserializeOwned + crate::Op,
{
    type Error = Error;

    fn try_from(path: &Path) -> Result<Self> {
        Self::try_from(path.to_path_buf())
    }
}

/// A lifted program, read from the file an intermediate representation was
/// written to.
///
/// Which representation it is -- and so which operations it holds -- is
/// decided by the `ir` field the file carries, not by the caller. The
/// operations themselves are not exposed: what a caller does with a program
/// is extract a birthmark from it ([`crate::extract::Extractor`]) and compare
/// the birthmark ([`crate::compare::Comparator`]), and neither needs to see
/// them.
///
/// Opaque on purpose. Adding a lifter adds a case inside, which is not a
/// change for anyone outside the crate.
#[derive(Debug)]
pub struct Program(pub(crate) Lifted);

/// The program inside a [`Program`], typed by its representation's operations.
///
/// [`TypedProgram`] is generic over its operation type, and the type is not
/// the caller's to choose: it belongs to the `ir` field the file carries, and
/// this is where that decision is made. Adding a lifter adds a variant, and
/// the compiler then points at every place that has to account for it.
#[derive(Debug)]
pub(crate) enum Lifted {
    GhidraPcode(TypedProgram<crate::ghidra::Op>),
    BinaryNinjaLlil(TypedProgram<crate::binaryninja::Op<crate::binaryninja::Llil>>),
    BinaryNinjaMlil(TypedProgram<crate::binaryninja::Op<crate::binaryninja::Mlil>>),
    BinaryNinjaHlil(TypedProgram<crate::binaryninja::Op<crate::binaryninja::Hlil>>),
    IdaMicrocode(TypedProgram<crate::ida::Op>),
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

/// Applies one expression to whichever [`TypedProgram`] a [`Program`] holds.
///
/// Every method below forwards identically, and writing the four arms out
/// each time is four chances to forward one of them to the wrong thing.
/// Adding a variant still stops the build, because the match here becomes
/// non-exhaustive -- the compiler asks once instead of a dozen times.
macro_rules! dispatch {
    ($self:expr, $p:ident => $body:expr) => {
        match $self {
            Lifted::GhidraPcode($p) => $body,
            Lifted::BinaryNinjaLlil($p) => $body,
            Lifted::BinaryNinjaMlil($p) => $body,
            Lifted::BinaryNinjaHlil($p) => $body,
            Lifted::IdaMicrocode($p) => $body,
        }
    };
}

impl Program {
    /// Reads a lifted program, choosing the operation type from the file's own
    /// `ir` field.
    ///
    /// The representation is read first so that each file is read by its own
    /// representation's reader. Read as P-Code, a file of another
    /// representation would fail on the first foreign opcode, reported as an
    /// unknown variant among seventy-five alternatives rather than as the one
    /// fact that matters.
    pub fn load(path: &Path) -> Result<Self> {
        // Read once and deserialize twice from the same bytes: the probe has
        // to see the file before the reader can be chosen, and reading it
        // again would cost more than the scan does.
        let bytes = std::fs::read(path).map_err(|e| Error::Io(path.to_path_buf(), e))?;
        let json_err = |e| Error::Json(path.to_path_buf(), e);
        let probe: IrProbe = serde_json::from_slice(&bytes).map_err(json_err)?;
        match probe.ir {
            Ir::GhidraPcode => serde_json::from_slice(&bytes)
                .map(|p| Self(Lifted::GhidraPcode(p)))
                .map_err(json_err),
            Ir::BinaryNinjaLlil => serde_json::from_slice(&bytes)
                .map(|p| Self(Lifted::BinaryNinjaLlil(p)))
                .map_err(json_err),
            Ir::BinaryNinjaMlil => serde_json::from_slice(&bytes)
                .map(|p| Self(Lifted::BinaryNinjaMlil(p)))
                .map_err(json_err),
            Ir::BinaryNinjaHlil => serde_json::from_slice(&bytes)
                .map(|p| Self(Lifted::BinaryNinjaHlil(p)))
                .map_err(json_err),
            Ir::IdaMicrocode => serde_json::from_slice(&bytes)
                .map(|p| Self(Lifted::IdaMicrocode(p)))
                .map_err(json_err),
        }
    }

    /// The representation this program's operations are written in.
    pub fn ir(&self) -> Ir {
        dispatch!(&self.0, p => p.ir())
    }

    /// The program's name, as the lifter recorded it: the binary's file name.
    pub fn name(&self) -> &str {
        dispatch!(&self.0, p => p.name())
    }

    /// The path of the binary it was lifted from, as the lifter recorded it.
    pub fn path(&self) -> &Path {
        dispatch!(&self.0, p => p.path())
    }

    /// How many functions it holds.
    pub fn len(&self) -> usize {
        dispatch!(&self.0, p => p.len())
    }

    /// Whether it holds no functions.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Records the file this program was read from, which
    /// [`Program::json_path`] then returns. [`Program::load`] does not record
    /// it by itself.
    pub fn set_json_path(&mut self, path: PathBuf) {
        dispatch!(&mut self.0, p => p.set_json_path(path))
    }

    /// Where this program was read from, once [`Program::set_json_path`] has
    /// said so.
    pub fn json_path(&self) -> Option<&Path> {
        dispatch!(&self.0, p => p.json_path.as_deref())
    }

    /// How many symbols the lifted file names.
    pub fn symbol_count(&self) -> usize {
        dispatch!(&self.0, p => p.symbols.len())
    }

    /// The name of each function, in the order the file lists them.
    pub fn function_names(&self) -> Vec<&str> {
        dispatch!(&self.0, p => p.functions.iter().map(|f| f.name.as_str()).collect())
    }
}

impl<T> TypedProgram<T> {
    pub fn set_json_path(&mut self, path: PathBuf) {
        self.json_path = Some(path);
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

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn symbol(&self, addr: &str) -> Option<&String> {
        self.symbols.get(addr)
    }
}

impl<T> Iterable for TypedProgram<T> {
    type Item = TypedFunction<T>;
    fn iter(&self) -> Box<dyn Iterator<Item = &Self::Item> + '_> {
        Box::new(self.functions.iter())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct TypedFunction<T> {
    name: String,
    ops: Vec<T>,
}

impl<T: crate::Op> TypedFunction<T> {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.ops.iter()
    }

    pub fn ops(&self) -> impl Iterator<Item = &str> {
        self.ops.iter().map(crate::Op::mnemonic)
    }

    pub fn ops_freq(&self) -> rustc_hash::FxHashMap<String, usize> {
        crate::extract::seq_to_freq(self.ops().map(ToString::to_string))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lift::Ir;

    /// A file without the `ir` field loads, and is understood as Ghidra's.
    #[test]
    fn test_ir_defaults_to_ghidra_pcode_when_absent() {
        let json = r#"{
            "program": "sample",
            "path": "bin/sample",
            "symbols": {},
            "functions": []
        }"#;
        let program: TypedProgram<crate::ghidra::Op> =
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
        let program: TypedProgram<crate::ghidra::Op> = serde_json::from_str(json).unwrap();
        assert_eq!(program.ir(), Ir::GhidraPcode);
    }

    #[test]
    fn test_ir_survives_a_round_trip() {
        let program: TypedProgram<crate::ghidra::Op> = TypedProgram {
            name: "sample".to_string(),
            path: PathBuf::from("bin/sample"),
            ir: Ir::GhidraPcode,
            symbols: FxHashMap::default(),
            functions: vec![],
            json_path: None,
        };
        let json = serde_json::to_string(&program).unwrap();
        assert!(
            json.contains("\"ir\":\"ghidra-pcode\""),
            "the field must be written, not only read: {json}"
        );
        let back: TypedProgram<crate::ghidra::Op> = serde_json::from_str(&json).unwrap();
        assert_eq!(back.ir(), Ir::GhidraPcode);
    }

    /// The point of the dispatch: the file decides, not the caller.
    #[test]
    fn test_any_program_reads_ghidra_pcode() {
        let program = Program::load(Path::new("testdata/lifted/pcodes/hello_clang.json"))
            .expect("the fixture must load");
        assert_eq!(program.ir(), Ir::GhidraPcode);
        assert_eq!(program.name(), "hello_clang");
        assert_eq!(program.len(), 1);
    }

    /// A file without the `ir` field carries no representation to dispatch on,
    /// and still reaches Ghidra's reader.
    #[test]
    fn test_any_program_reads_a_file_without_the_field() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("legacy.json");
        std::fs::write(
            &path,
            r#"{"program":"legacy","path":"bin/legacy","symbols":{},"functions":[]}"#,
        )
        .unwrap();
        let program = Program::load(&path).expect("a file without the field must still load");
        assert_eq!(program.ir(), Ir::GhidraPcode);
    }

    /// An `ir` this build does not know -- what a file from a future version
    /// looks like -- is refused by the deserializer rather than read as
    /// something else.
    #[test]
    fn test_any_program_refuses_a_representation_it_does_not_know() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("foreign.json");
        std::fs::write(
            &path,
            r#"{"program":"foreign","path":"bin/foreign","ir":"llvm-ir",
                "symbols":{},"functions":[{"name":"main","ops":[
                  {"op":"call","inputs":["r0"]}]}]}"#,
        )
        .unwrap();
        let err = match Program::load(&path) {
            Err(e) => e.to_string(),
            Ok(_) => panic!("a representation with no reader was loaded"),
        };
        assert!(
            err.contains("llvm-ir"),
            "does not name what it could not read: {err}"
        );
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
        assert!(matches!(Program::load(&path), Err(Error::Json(_, _))));
    }

    /// The loader reads the file whole and parses the bytes. The three ways it
    /// fails are pinned: the file not being there is an IO error, and both ways
    /// its *content* can be wrong are JSON errors.
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
            TypedProgram::<crate::ghidra::Op>::try_from(missing),
            Err(Error::Io(..))
        ));

        let broken = dir.path().join("broken.json");
        std::fs::write(&broken, b"{ not json").unwrap();
        assert!(matches!(
            TypedProgram::<crate::ghidra::Op>::try_from(broken),
            Err(Error::Json(..))
        ));

        let not_utf8 = dir.path().join("not-utf8.json");
        std::fs::write(&not_utf8, b"{\"program\": \"\xff\xfe\"}").unwrap();
        assert!(matches!(
            TypedProgram::<crate::ghidra::Op>::try_from(not_utf8),
            Err(Error::Json(..))
        ));
    }

    /// Reading the file whole must not change what is parsed out of it. The
    /// fixture is real lifter output, so this compares the loader against
    /// parsing the same text directly.
    #[test]
    fn test_reading_the_file_whole_parses_what_the_text_says() {
        let fixture = Path::new("testdata/lifted/pcodes/hello_clang.json");
        let loaded: TypedProgram<crate::ghidra::Op> = fixture.try_into().unwrap();
        let parsed: TypedProgram<crate::ghidra::Op> =
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
            "testdata/lifted/pcodes/hello_clang.json",
            "testdata/lifted/pcodes/hello_gcc.json",
        ] {
            let program: TypedProgram<crate::ghidra::Op> = Path::new(fixture)
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
