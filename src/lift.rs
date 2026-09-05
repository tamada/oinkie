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

impl LifterType {
    /// The tool's name as it should appear in messages.
    pub fn name(&self) -> &'static str {
        match self {
            LifterType::Ghidra => "Ghidra",
            LifterType::Angr => "angr",
            LifterType::IDAPro => "IDA Pro",
            LifterType::BinaryNinja => "Binary Ninja",
        }
    }

    /// Where this backend's installation is looked for, or `None` for one that
    /// has no installation directory to find.
    ///
    /// angr is the `None`: it is a Python library, imported rather than
    /// installed somewhere oinkie could point at. It is here as the reminder
    /// that the shape does not fit every backend.
    pub fn home_spec(&self) -> Option<HomeSpec> {
        match self {
            LifterType::Ghidra => Some(HomeSpec {
                tool: self.name(),
                env: "GHIDRA_HOME",
                candidates: &[
                    "/opt/homebrew/opt/ghidra/libexec",
                    "/usr/local/opt/ghidra/libexec",
                    "/opt/ghidra/libexec",
                ],
            }),
            LifterType::Angr => None,
            LifterType::IDAPro => Some(HomeSpec {
                tool: self.name(),
                env: "IDA_HOME",
                candidates: &[],
            }),
            LifterType::BinaryNinja => Some(HomeSpec {
                tool: self.name(),
                env: "BINARY_NINJA_HOME",
                candidates: &[],
            }),
        }
    }

    /// Finds this backend's installation: what the user passed, then the
    /// environment variable, then the usual locations.
    pub fn find_home(&self, home_opt: Option<&Path>) -> Result<PathBuf> {
        if let Some(h) = home_opt {
            return Ok(h.to_path_buf());
        }
        let Some(spec) = self.home_spec() else {
            return Err(crate::Error::Parse(format!(
                "{} has no installation directory to find, so --home means nothing for it",
                self.name()
            )));
        };
        spec.find_in(|k| std::env::var(k).ok(), |p| p.exists())
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
                let home = self.lifter_type.find_home(self.home.as_deref())?;
                Ok(Box::new(Verifying(
                    crate::ghidra::lifter::GhidraLifter::new(
                        home,
                        self.script,
                        self.intermediate_dir,
                    ),
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The search never runs: `--home` is taken as given, without checking
    /// that it exists, so that the error names the path the user actually
    /// passed rather than a guess.
    #[test]
    fn test_the_home_the_user_passed_wins() {
        let opt = PathBuf::from("/custom/ghidra/home");
        assert_eq!(LifterType::Ghidra.find_home(Some(&opt)).unwrap(), opt);
    }

    #[test]
    fn test_the_environment_variable_is_read_when_no_home_was_passed() {
        let spec = LifterType::Ghidra.home_spec().unwrap();
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
        let spec = LifterType::Ghidra.home_spec().unwrap();
        let home = spec
            .find_in(|_| Some("/env/ghidra/home".to_string()), |_| true)
            .unwrap();
        assert_eq!(home, PathBuf::from("/env/ghidra/home"));
    }

    #[test]
    fn test_the_usual_locations_are_searched_when_the_variable_is_unset() {
        let spec = LifterType::Ghidra.home_spec().unwrap();
        let last = Path::new(spec.candidates.last().unwrap());
        let home = spec.find_in(|_| None, |p| p == last).unwrap();
        assert_eq!(home, last);
    }

    /// Two installations are a real situation on a Mac with both Homebrew
    /// prefixes populated, and the order the candidates are listed in is the
    /// answer to it.
    #[test]
    fn test_the_first_of_several_installations_wins() {
        let spec = LifterType::Ghidra.home_spec().unwrap();
        let home = spec.find_in(|_| None, |_| true).unwrap();
        assert_eq!(home, PathBuf::from(spec.candidates[0]));
    }

    /// This is what a machine with no Ghidra sees, and until the search took
    /// its environment as a parameter it could not be asserted: the test ran
    /// on a machine that might have Ghidra at one of the candidates, so it
    /// checked only that nothing panicked.
    #[test]
    fn test_a_ghidra_that_is_nowhere_says_what_to_set_and_where_it_looked() {
        let spec = LifterType::Ghidra.home_spec().unwrap();
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

    /// A backend nobody has installed and checked has no candidates, and the
    /// message has to stop after the variable rather than invite the user to
    /// install it in one of nowhere.
    #[test]
    fn test_a_backend_with_no_usual_locations_does_not_offer_an_empty_list() {
        let spec = LifterType::IDAPro.home_spec().unwrap();
        let err = spec
            .find_in(|_| None, |_| panic!("there is nothing to look at"))
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("IDA_HOME"),
            "does not name the variable: {err}"
        );
        assert!(
            !err.contains("install it in one of"),
            "offers an empty list: {err}"
        );
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

    /// A representation this build cannot read is caught here too, rather
    /// than at `extract`.
    #[test]
    fn test_an_output_in_an_unreadable_representation_fails_the_lift() {
        let json = A_READABLE_PROGRAM.replace("ghidra-pcode", "ida-microcode");
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
        assert!(e.to_string().contains("no reader for ida-microcode"), "{e}");
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
