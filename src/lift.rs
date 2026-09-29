use crate::Result;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub(crate) mod headless;

pub trait Lifter {
    fn lift(&self, input: &Path, output: &Path) -> Result<()>;
}

/// A lifter whose output is read back before the lift is called a success.
///
/// Wrapping here, rather than asking each [`Lifter`] to check its own output,
/// is the point. [`LifterBuilder::build`] is the one path every caller takes,
/// so a lifter added to its match gets the check without its author having to
/// remember -- the same reasoning that leaves [`crate::Op::is_call`] without a
/// default.
///
/// A writer is exactly the kind of code that looks finished while producing
/// something nothing can read. `analyzeHeadless` exits 0 whether or not its
/// post-script threw; the built-in script wrote unescaped names for as long as
/// it existed; and a replacement passed to `--script` is arbitrary Java that
/// oinkie never sees. Without this, all three end the same way: a directory of
/// files that look lifted, and a failure at `extract` naming a line and column
/// in a file the reader did not know existed.
struct Verifying<L>(L);

impl<L: Lifter> Lifter for Verifying<L> {
    fn lift(&self, input: &Path, output: &Path) -> Result<()> {
        self.0.lift(input, output)?;
        // Loaded and dropped: this is the check. Reading it is the only way to
        // know the file is one oinkie can use, and the read is what `extract`
        // would have done later anyway.
        crate::program::AnyProgram::load(output)
            .map(|_| ())
            .map_err(|e| crate::Error::UnreadableOutput(input.to_path_buf(), Box::new(e)))
    }
}

/// The intermediate representation a lifted program is written in, and the
/// only thing a caller names to ask for a lift.
///
/// Named after the representation rather than the tool that produced it,
/// because one tool can produce several and they are not interchangeable —
/// Binary Ninja lifts to LLIL, MLIL or HLIL, each with its own vocabulary.
/// What everything downstream needs to know is which vocabulary it is looking
/// at: whether two birthmarks can be compared, and which `Op` type can read
/// the file.
///
/// This used to sit beside a `LifterType` that named the tool, and `lift`
/// took both. The pair could disagree — a Ghidra lifter asked for HLIL — so
/// something had to check, report and be tested for a state that only existed
/// because there were two enums. There is one now, and that state cannot be
/// written down. It is the finer of the two: a representation implies its
/// tool, while a tool does not imply a representation.
///
/// Every variant here can be both written and read today, which has not
/// always been so: representations are declared when they are named and filled
/// in later, and while one was unreadable there was a `readable()` list and an
/// `UnsupportedIr` to report against it. Both went when IDA's maturities
/// landed and nothing was left to refuse. The guard that remains is the
/// exhaustive match in [`crate::program::AnyProgram::load`], which is what
/// stops a new variant being added without a decision about reading it.
#[derive(Debug, ValueEnum, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
#[clap(rename_all = "kebab-case")]
pub enum Ir {
    /// Ghidra's P-Code, as refined by the decompiler — what `HighFunction`
    /// yields, rather than raw lifted P-Code.
    #[default]
    GhidraPcode,
    /// The Hex-Rays microcode, the representation IDA Pro's decompiler works
    /// in, at `MMAT_LVARS` -- the last of its maturities and the one the
    /// pseudocode is rendered from.
    ///
    /// The earlier maturities are not offered. They are the decompiler's
    /// pipeline rather than representations it publishes: the same enum holds
    /// `MMAT_ZERO`, "microcode does not exist", and `MMAT_GLBOPT2` is
    /// described by Hex-Rays only as "most global optimization passes are
    /// done". Nothing says their shape is stable across releases, and a name
    /// here is permanent -- a file carrying it has to stay readable. Adding
    /// one later costs nothing; removing one breaks every file that named it.
    ///
    /// Producing microcode at all means running IDA Pro's decompiler, and on
    /// an installation whose only decompiler is the cloud one that means the
    /// function is sent to Hex-Rays' servers. `lift` says so before it starts.
    IdaMicrocode,
    /// Binary Ninja's Low Level IL: one expression per machine instruction,
    /// registers and flags still explicit.
    BinaryNinjaLlil,
    /// Binary Ninja's Medium Level IL: stack and registers resolved into
    /// variables, calls carrying their parameters.
    BinaryNinjaMlil,
    /// Binary Ninja's High Level IL: control flow recovered, the level its
    /// decompiler output is rendered from.
    BinaryNinjaHlil,
}

impl Ir {
    /// The tool that produces this representation, as it should appear in
    /// messages.
    ///
    /// Several representations share one tool, which is the asymmetry that
    /// makes the representation the thing to name and the tool the thing to
    /// derive.
    pub fn tool(&self) -> &'static str {
        match self {
            Ir::GhidraPcode => "Ghidra",
            Ir::IdaMicrocode => "IDA Pro",
            Ir::BinaryNinjaLlil | Ir::BinaryNinjaMlil | Ir::BinaryNinjaHlil => "Binary Ninja",
        }
    }

    /// Where the tool behind this representation is looked for.
    ///
    /// Keyed on the representation because that is what the caller names, and
    /// answered per tool, so the three Binary Ninja levels share one
    /// installation rather than each describing it again.
    pub fn home_spec(&self) -> HomeSpec {
        match self {
            Ir::GhidraPcode => HomeSpec {
                tool: self.tool(),
                env: "GHIDRA_HOME",
                candidates: &[
                    "/opt/homebrew/opt/ghidra/libexec",
                    "/usr/local/opt/ghidra/libexec",
                    "/opt/ghidra/libexec",
                ],
            },
            Ir::IdaMicrocode => HomeSpec {
                tool: self.tool(),
                env: "IDA_HOME",
                // The directory holding `idat`, which is the headless entry
                // point. On macOS that is inside the application bundle.
                candidates: &[
                    "/Applications/IDA Professional 9.4.app/Contents/MacOS",
                    "/Applications/IDA Classroom 9.4.app/Contents/MacOS",
                    "/opt/ida",
                ],
            },
            Ir::BinaryNinjaLlil | Ir::BinaryNinjaMlil | Ir::BinaryNinjaHlil => HomeSpec {
                tool: self.tool(),
                env: "BINARY_NINJA_HOME",
                // The directory holding `bnpython3`, which is the headless
                // entry point, rather than the bundle or the API directory
                // beside it.
                candidates: &[
                    "/Applications/Binary Ninja.app/Contents/MacOS",
                    "/opt/binaryninja",
                ],
            },
        }
    }

    /// Finds the tool's installation: what the user passed, then the
    /// environment variable, then the usual locations.
    pub fn find_home(&self, home_opt: Option<&Path>) -> Result<PathBuf> {
        if let Some(h) = home_opt {
            return Ok(h.to_path_buf());
        }
        self.home_spec()
            .find_in(|k| std::env::var(k).ok(), |p| p.exists())
    }
}

impl std::fmt::Display for Ir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Ir {
    /// This representation's name, which is also what is written into the
    /// `ir` field of a lifted file.
    ///
    /// Spelled out rather than derived, and matching what `serde`'s
    /// kebab-case rename produces. The two have to agree: a file's `ir` field
    /// is written by `serde` and a birthmark's is parsed by [`std::str::FromStr`],
    /// so a name that differed between them would make a birthmark
    /// unreadable against the file it came from.
    fn as_str(&self) -> &'static str {
        match self {
            Ir::GhidraPcode => "ghidra-pcode",
            Ir::IdaMicrocode => "ida-microcode",
            Ir::BinaryNinjaLlil => "binary-ninja-llil",
            Ir::BinaryNinjaMlil => "binary-ninja-mlil",
            Ir::BinaryNinjaHlil => "binary-ninja-hlil",
        }
    }
}

impl std::str::FromStr for Ir {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "ghidra-pcode" => Ok(Ir::GhidraPcode),
            "ida-microcode" => Ok(Ir::IdaMicrocode),
            "binary-ninja-llil" => Ok(Ir::BinaryNinjaLlil),
            "binary-ninja-mlil" => Ok(Ir::BinaryNinjaMlil),
            "binary-ninja-hlil" => Ok(Ir::BinaryNinjaHlil),
            _ => Err(crate::Error::Parse(format!(
                "{s}: unknown intermediate representation"
            ))),
        }
    }
}

/// How a backend's installation is found.
///
/// The three steps are the same for every tool -- what the user passed, then
/// an environment variable, then the places it is usually installed -- and
/// only the names differ, so the names are the data and the search is written
/// once.
pub struct HomeSpec {
    /// The tool's name, for messages.
    pub tool: &'static str,
    /// The environment variable oinkie reads.
    ///
    /// This is oinkie's own convention rather than something the tools
    /// define: Ghidra's own installer sets `GHIDRA_INSTALL_DIR`, not
    /// `GHIDRA_HOME`. Naming the others the same way keeps the convention
    /// guessable.
    pub env: &'static str,
    /// Where to look when the variable is unset.
    ///
    /// Empty for a backend nobody has installed and checked. A guessed path
    /// that happens to exist is worse than asking, because it is found
    /// silently and only fails later, somewhere less obvious.
    pub candidates: &'static [&'static str],
}

impl HomeSpec {
    /// Runs the search -- the environment variable, then the usual install
    /// locations -- against a given environment and a given test for whether a
    /// path exists.
    ///
    /// Both are handed in rather than read directly so that a test can
    /// describe a machine instead of having to become one. The tests used to
    /// set `GHIDRA_HOME` and put it back afterwards; the environment is
    /// process-global, so two of them running at once in the same test binary
    /// saw each other's writes, and in edition 2024 `set_var` is `unsafe`
    /// precisely because a concurrent read of it is undefined behaviour rather
    /// than merely a wrong answer (#24).
    ///
    /// It also makes the "not found" case testable at all. It could assert
    /// nothing before, because the machine running the test might genuinely
    /// have Ghidra at one of the candidates.
    pub(crate) fn find_in(
        &self,
        env: impl Fn(&str) -> Option<String>,
        exists: impl Fn(&Path) -> bool,
    ) -> Result<PathBuf> {
        if let Some(h) = env(self.env) {
            return Ok(PathBuf::from(h));
        }
        for c in self.candidates {
            let p = PathBuf::from(c);
            if exists(&p) {
                return Ok(p);
            }
        }
        let looked_in = if self.candidates.is_empty() {
            String::new()
        } else {
            format!(", or install it in one of: {}", self.candidates.join(", "))
        };
        Err(crate::Error::Parse(format!(
            "{} not found. Specify it with --home, set {}{looked_in}",
            self.tool, self.env
        )))
    }
}

pub struct LifterBuilder {
    ir: Ir,
    home: Option<PathBuf>,
    script: Option<PathBuf>,
    intermediate_dir: Option<PathBuf>,
}

impl LifterBuilder {
    /// Takes the representation to produce, which is also what picks the tool.
    pub fn new(ir: Ir) -> Self {
        Self {
            ir,
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
        // Every representation is spelled out rather than falling through to
        // one "not implemented" arm, so that adding a variant to `Ir` stops
        // the build here and asks whether this one can be written.
        match self.ir {
            Ir::GhidraPcode => {
                let home = self.ir.find_home(self.home.as_deref())?;
                Ok(Box::new(Verifying(
                    crate::ghidra::lifter::GhidraLifter::new(
                        home,
                        self.script,
                        self.intermediate_dir,
                    ),
                )))
            }
            Ir::IdaMicrocode => {
                let home = self.ir.find_home(self.home.as_deref())?;
                Ok(Box::new(Verifying(crate::ida::lifter::IdaLifter::new(
                    home,
                    self.script,
                    self.intermediate_dir,
                ))))
            }
            Ir::BinaryNinjaLlil | Ir::BinaryNinjaMlil | Ir::BinaryNinjaHlil => {
                // `level` is exhaustive over `Ir`, so a `None` here would mean
                // a representation reached this arm without being given a
                // level -- an inconsistency between two matches rather than
                // anything a caller did. Refusing says so; panicking would
                // not, and would be a branch nothing can exercise.
                let Some(level) = crate::binaryninja::level(self.ir) else {
                    return Err(not_implemented(self.ir));
                };
                let home = self.ir.find_home(self.home.as_deref())?;
                Ok(Box::new(Verifying(
                    crate::binaryninja::lifter::BinaryNinjaLifter::new(
                        home,
                        level,
                        self.script,
                        self.intermediate_dir,
                    ),
                )))
            }
        }
    }
}

/// What a representation nobody has written a lifter for reports.
///
/// Names the representation as well as the tool, because the caller asked for
/// the representation and "Binary Ninja is not implemented" would not say
/// which of its three they were refused.
fn not_implemented(ir: Ir) -> crate::Error {
    crate::Error::Parse(format!(
        "no lifter writes {ir} yet: the {} backend is not implemented",
        ir.tool()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    /// Every representation has one name, spelled three independent times.
    ///
    /// `serde` derives it from `rename_all`, `clap` derives its own from
    /// another `rename_all`, and [`Ir::as_str`] is a hand-written match. All
    /// three are load-bearing and none of them consults the others: a lifted
    /// program's `ir` field is written and read by `serde`, a birthmark's
    /// metadata is parsed by `FromStr`, and `--ir` is parsed by `clap`.
    ///
    /// A variant they disagreed about would not fail to compile. It would
    /// accept `--ir binary-ninja-llil` and write something else into the file,
    /// or extract a birthmark that no longer matches the program it came from.
    #[test]
    fn test_every_representation_spells_itself_the_same_way_three_times() {
        for ir in Ir::value_variants() {
            let json = serde_json::to_string(ir).unwrap();
            let via_serde = json.trim_matches('"');
            assert_eq!(via_serde, ir.to_string(), "serde and Display disagree");

            let via_clap = ir.to_possible_value().unwrap();
            assert_eq!(
                via_clap.get_name(),
                ir.to_string(),
                "clap and Display disagree"
            );

            assert_eq!(
                &<Ir as FromStr>::from_str(via_serde).unwrap(),
                ir,
                "FromStr does not read what serde writes: {via_serde}"
            );
        }
    }

    /// Every representation resolves to a tool, which is the property that let
    /// the tool stop being a separate argument.
    #[test]
    fn test_every_representation_names_a_tool() {
        for ir in Ir::value_variants() {
            assert!(!ir.tool().is_empty(), "{ir} names no tool");
            assert!(!ir.home_spec().env.is_empty(), "{ir} names no variable");
        }
    }

    /// The search never runs: `--home` is taken as given, without checking
    /// that it exists, so that the error names the path the user actually
    /// passed rather than a guess.
    #[test]
    fn test_the_home_the_user_passed_wins() {
        let opt = PathBuf::from("/custom/ghidra/home");
        assert_eq!(Ir::GhidraPcode.find_home(Some(&opt)).unwrap(), opt);
    }

    #[test]
    fn test_the_environment_variable_is_read_when_no_home_was_passed() {
        let spec = Ir::GhidraPcode.home_spec();
        let home = spec
            .find_in(
                |k| (k == "GHIDRA_HOME").then(|| "/env/ghidra/home".to_string()),
                |_| panic!("the usual locations were searched despite GHIDRA_HOME being set"),
            )
            .unwrap();
        assert_eq!(home, PathBuf::from("/env/ghidra/home"));
    }

    /// An installed Ghidra does not override the one the user pointed at. The
    /// `exists` above never runs, so this says the same thing from the other
    /// side: with both available, the variable is what comes back.
    #[test]
    fn test_the_environment_variable_beats_an_installed_ghidra() {
        let spec = Ir::GhidraPcode.home_spec();
        let home = spec
            .find_in(|_| Some("/env/ghidra/home".to_string()), |_| true)
            .unwrap();
        assert_eq!(home, PathBuf::from("/env/ghidra/home"));
    }

    #[test]
    fn test_the_usual_locations_are_searched_when_the_variable_is_unset() {
        let spec = Ir::GhidraPcode.home_spec();
        let last = Path::new(spec.candidates.last().unwrap());
        let home = spec.find_in(|_| None, |p| p == last).unwrap();
        assert_eq!(home, last);
    }

    /// Two installations are a real situation on a Mac with both Homebrew
    /// prefixes populated, and the order the candidates are listed in is the
    /// answer to it.
    #[test]
    fn test_the_first_of_several_installations_wins() {
        let spec = Ir::GhidraPcode.home_spec();
        let home = spec.find_in(|_| None, |_| true).unwrap();
        assert_eq!(home, PathBuf::from(spec.candidates[0]));
    }

    /// This is what a machine with no Ghidra sees, and until the search took
    /// its environment as a parameter it could not be asserted: the test ran
    /// on a machine that might have Ghidra at one of the candidates, so it
    /// checked only that nothing panicked.
    #[test]
    fn test_a_ghidra_that_is_nowhere_says_what_to_set_and_where_it_looked() {
        let spec = Ir::GhidraPcode.home_spec();
        let err = spec.find_in(|_| None, |_| false).unwrap_err().to_string();
        assert!(err.contains("--home"), "does not offer --home: {err}");
        assert!(
            err.contains("GHIDRA_HOME"),
            "does not name the variable: {err}"
        );
        for c in spec.candidates {
            assert!(err.contains(c), "does not say it looked in {c}: {err}");
        }
    }

    /// A backend with no candidates must stop after the variable rather than
    /// invite the user to install it in one of nowhere.
    ///
    /// The spec is built here rather than taken from an [`Ir`], because every
    /// representation now names somewhere to look. [`HomeSpec`] is public with
    /// public fields, so the empty case is still reachable -- and it is what a
    /// backend added before anyone has checked where it installs looks like,
    /// which is how both IDA Pro and Binary Ninja began.
    #[test]
    fn test_a_backend_with_no_usual_locations_does_not_offer_an_empty_list() {
        let spec = HomeSpec {
            tool: "Nowhere",
            env: "NOWHERE_HOME",
            candidates: &[],
        };
        let err = spec
            .find_in(|_| None, |_| panic!("there is nothing to look at"))
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("NOWHERE_HOME"),
            "does not name the variable: {err}"
        );
        assert!(
            !err.contains("install it in one of"),
            "offers an empty list: {err}"
        );
    }

    /// Every representation names somewhere to look, which is the other half
    /// of the test above: it is what makes that spec a constructed one.
    #[test]
    fn test_every_representation_names_somewhere_to_look() {
        for ir in Ir::value_variants() {
            assert!(
                !ir.home_spec().candidates.is_empty(),
                "{ir} offers no install locations"
            );
        }
    }
}

#[cfg(test)]
mod verifying_tests {
    use super::*;
    use crate::Error;

    /// A lifter that writes exactly what it is given, so that what the
    /// verifier does with each kind of output can be described rather than
    /// arranged through a real decompiler.
    struct Writes(&'static str);

    impl Lifter for Writes {
        fn lift(&self, _input: &Path, output: &Path) -> Result<()> {
            std::fs::write(output, self.0).map_err(|e| Error::Io(output.to_path_buf(), e))
        }
    }

    /// A lifter that fails before writing anything.
    struct Fails;

    impl Lifter for Fails {
        fn lift(&self, _input: &Path, _output: &Path) -> Result<()> {
            Err(Error::Parse("the decompiler said no".to_string()))
        }
    }

    const A_READABLE_PROGRAM: &str = r#"{
        "program": "sample",
        "path": "bin/sample",
        "ir": "ghidra-pcode",
        "symbols": {},
        "functions": []
    }"#;

    fn lift_with<L: Lifter>(lifter: L, input: &str) -> (Result<()>, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("sample.json");
        let r = Verifying(lifter).lift(Path::new(input), &output);
        (r, dir)
    }

    #[test]
    fn test_a_readable_output_is_a_successful_lift() {
        let (r, _dir) = lift_with(Writes(A_READABLE_PROGRAM), "bin/sample");
        assert!(r.is_ok(), "{:?}", r.unwrap_err().to_string());
    }

    /// The case this exists for. The lifter reported success -- `Writes`
    /// returns `Ok` -- and the file it left behind is not one oinkie can read.
    #[test]
    fn test_an_unparseable_output_fails_the_lift() {
        let (r, _dir) = lift_with(Writes("{\"functions\": ["), "bin/sample");
        let e = r.expect_err("a file that does not parse is not a successful lift");
        let rendered = e.to_string();
        // The binary, with the colon that follows it, because that is what the
        // caller asked about -- and because a bare "sample" would also match
        // the output's own name and so assert nothing.
        assert!(rendered.contains("bin/sample:"), "{rendered}");
        // the file, because that is the one that is wrong
        assert!(rendered.contains("sample.json"), "{rendered}");
        // and that the lifter claimed to have succeeded, which is what points
        // at the script rather than at the binary
        assert!(rendered.contains("reported success"), "{rendered}");
    }

    /// The cause has to survive, or the message says a file is unreadable
    /// without saying what is wrong with it.
    #[test]
    fn test_the_underlying_error_is_kept_as_the_cause() {
        use std::error::Error as _;
        let (r, _dir) = lift_with(Writes("{\"functions\": ["), "bin/sample");
        let e = r.unwrap_err();
        let source = e.source().expect("the parse failure is the cause");
        assert!(source.to_string().contains("JSON error"), "{source}");
        assert!(e.to_string().contains("JSON error"), "{e}");
    }

    /// An output whose representation this build does not know is caught here
    /// rather than at `extract`.
    ///
    /// It used to name a declared-but-unreadable representation. Every
    /// declared one has a reader now, so what is left is a name from a future
    /// version -- which is what a lifter written against a newer oinkie would
    /// write.
    #[test]
    fn test_an_output_in_an_unknown_representation_fails_the_lift() {
        let json = A_READABLE_PROGRAM.replace("ghidra-pcode", "llvm-ir");
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("sample.json");
        std::fs::write(&output, &json).unwrap();
        struct Nothing;
        impl Lifter for Nothing {
            fn lift(&self, _i: &Path, _o: &Path) -> Result<()> {
                Ok(())
            }
        }
        let e = Verifying(Nothing)
            .lift(Path::new("bin/sample"), &output)
            .expect_err("a representation with no reader is not a successful lift");
        assert!(e.to_string().contains("llvm-ir"), "{e}");
    }

    /// A lifter that fails is reported as itself. Wrapping its error in
    /// "cannot be read back" would blame the output for a file that was never
    /// written.
    #[test]
    fn test_a_failing_lifter_is_not_reported_as_an_unreadable_output() {
        let (r, _dir) = lift_with(Fails, "bin/sample");
        let e = r.unwrap_err();
        assert_eq!(e.to_string(), "Parse error: the decompiler said no");
        assert!(!e.to_string().contains("cannot be read back"), "{e}");
    }
}
