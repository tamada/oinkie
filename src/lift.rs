//! Lifting: turning a binary into a [`Program`](crate::Program), with a
//! decompiler doing the reading.
//!
//! An [`Ir`] names the intermediate representation to produce, which also
//! picks the tool -- Ghidra, IDA or Binary Ninja. A [`LifterBuilder`] makes the
//! [`Lifter`] for it, and [`Lifter::lift`] writes the lifted program as JSON,
//! which [`Program::load`](crate::Program::load) reads.
//!
//! oinkie uses these tools; it does not provide them. Installing a tool, and
//! for IDA Pro and Binary Ninja holding a licence for it, is left to the user.
//! oinkie only finds the installation -- the home given to
//! [`LifterBuilder::home`], the tool's environment variable, or the usual
//! installation directories -- and reports
//! [`Error::ToolNotFound`](crate::Error::ToolNotFound) when there is none.
//!
//! ```no_run
//! use std::path::Path;
//!
//! use oinkie::lift::{Ir, Lifter, LifterBuilder};
//!
//! # fn main() -> oinkie::Result<()> {
//! let lifter = LifterBuilder::new(Ir::GhidraPcode).build()?;
//! if let Some(notice) = lifter.notice() {
//!     eprintln!("{notice}");
//! }
//! lifter.lift(Path::new("hello"), Path::new("hello.json"))?;
//! # Ok(())
//! # }
//! ```

use crate::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub(crate) mod headless;

/// A tool that lifts binaries to one [`Ir`]. [`LifterBuilder::build`] makes
/// one.
pub trait Lifter {
    /// Lifts the binary at `input` and writes the lifted program to `output`
    /// as JSON.
    ///
    /// What was written is read back before the lift is called a success, so
    /// `Ok` means a file [`Program::load`](crate::Program::load) can read.
    /// The tools themselves report success even when their script failed;
    /// such a lift is an error here, carrying what the tool printed.
    fn lift(&self, input: &Path, output: &Path) -> Result<()>;

    /// Something the user is owed before the first lift starts, or `None`.
    ///
    /// Returned rather than printed, because whether and how to show it is the
    /// caller's to decide -- but a caller is expected to show it, and to show
    /// it where a verbosity setting cannot hide it. A notice is for a fact the
    /// user would want to know before the work begins, not a log entry about
    /// work already done.
    ///
    /// None by default: most lifters have nothing to disclose. Of oinkie's
    /// own, the Ghidra and Binary Ninja lifters return `None`, and the IDA Pro
    /// lifter returns one when the installation has a cloud decompiler, since
    /// producing microcode then sends functions to Hex-Rays' servers. With
    /// cloud decompilers for x64 and ARM64 and a local one for ARM64, it reads:
    ///
    /// ```text
    /// This IDA installation has cloud decompilers (hexcarm64.dylib,
    /// hexcx64.dylib). Microcode cannot be produced without a decompiler, and
    /// every function a cloud one handles is sent to Hex-Rays' servers.
    /// Nothing has been sent yet. It also has local decompilers
    /// (hexarm64.dylib); which of them handles this binary depends on its
    /// architecture.
    /// ```
    ///
    /// The decompilers are named by their plugin files, as installed. The last
    /// sentence is there only when local decompilers are installed too. The
    /// text is one line; it is wrapped here to be read.
    fn notice(&self) -> Option<String> {
        None
    }
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
/// post-script threw, and a script the caller supplies is code oinkie never
/// sees. Without this check, a broken lift would end as a directory of files
/// that look lifted, and a failure at `extract` naming a line and column in a
/// file the reader did not know existed.
///
/// Two things are looked at, and only one of them is a refusal. A file that
/// cannot be read is not a lift. A file that reads and holds no functions
/// might be: a binary can genuinely have none, and oinkie cannot tell that
/// apart from a tool that found none, since telling them apart would mean
/// analysing the binary itself rather than trusting the tool that was asked
/// to. So it says what it sees and goes on, which is the same call `extract`
/// makes about an empty `fc-*` family and for the same reason -- the warning
/// is what leaves the reader able to decide, and stopping would take that
/// chance away along with the rest of the run.
struct Verifying<L> {
    inner: L,
    ir: Ir,
}

impl<L: Lifter> Lifter for Verifying<L> {
    fn notice(&self) -> Option<String> {
        self.inner.notice()
    }

    fn lift(&self, input: &Path, output: &Path) -> Result<()> {
        self.inner.lift(input, output)?;
        // Loaded rather than merely opened: reading it is the only way to know
        // the file is one oinkie can use, and the read is what `extract` would
        // have done later anyway.
        let program = crate::program::Program::load(output)
            .map_err(|e| crate::Error::UnreadableOutput(input.to_path_buf(), Box::new(e)))?;
        if program.is_empty() {
            log::warn!("{}", no_functions_message(input, self.ir));
        }
        Ok(())
    }
}

/// What to say about a lift that came back with no functions in it.
///
/// Both readings are named, the way the `fc-*` warnings are, because oinkie
/// cannot choose between them: every birthmark from such a file is empty, two
/// empty birthmarks score as a perfect match, and whether that is the truth
/// about the binary or a broken tool is not something the file says.
///
/// For Ghidra the likely cause is named: a release that ships no decompiler
/// binary for the platform makes `DecompInterface` fail for every function,
/// and `analyzeHeadless` exits 0 regardless. The other representations get
/// the fact without the advice, since no cause is known for them, and a guess
/// would send the reader to a Ghidra directory they do not have.
fn no_functions_message(input: &Path, ir: Ir) -> String {
    let binary = input.display();
    let hint = if ir == Ir::GhidraPcode {
        ". Ghidra produces exactly this when its decompiler native binary is \
         missing: run support/gradle/gradlew buildNatives in the Ghidra installation"
    } else {
        ""
    };
    format!(
        "{binary}: the {ir} lifted from it has no functions in it, so every birthmark of it \
         will be empty -- and two empty birthmarks score as a perfect match. Either the binary \
         really has no functions, or the tool found none{hint}"
    )
}

/// The intermediate representation a lifted program is written in, and the
/// only thing a caller names to ask for a lift.
///
/// Named after the representation rather than the tool that produced it,
/// because one tool can produce several and they are not interchangeable —
/// Binary Ninja lifts to LLIL, MLIL or HLIL, each with its own vocabulary.
/// What everything downstream needs to know is which vocabulary it is looking
/// at: whether two birthmarks can be compared, and how the file is read.
///
/// The representation is all a caller names, not a tool beside it: a
/// representation implies its tool, while a tool does not imply a
/// representation, and naming both would allow a pair that disagrees -- a
/// Ghidra lift asked for HLIL.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum Ir {
    /// Ghidra's P-Code, as refined by the decompiler — what `HighFunction`
    /// yields, rather than raw lifted P-Code.
    #[default]
    GhidraPcode,
    /// The Hex-Rays microcode -- which runs IDA Pro's decompiler, so on an
    /// installation with a cloud decompiler each function is sent to Hex-Rays'
    /// servers. The lifter's [`Lifter::notice`] says so before the first lift.
    ///
    /// Read at `MMAT_LVARS`, the last of the decompiler's maturities and the
    /// one the pseudocode is rendered from. The other maturities are not
    /// offered: they are stages of the decompiler's pipeline rather than
    /// representations it publishes -- the same enum holds `MMAT_ZERO`,
    /// "microcode does not exist" -- and nothing says their shape is stable
    /// across releases, while a name here is permanent, since a file carrying
    /// it has to stay readable.
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
    /// Every representation, in the order they are declared.
    ///
    /// Written out because nothing derives it, so a test holds it to the enum
    /// with an exhaustive `match`: a variant added without being listed here
    /// stops the build instead of quietly going unlisted.
    pub const ALL: &[Ir] = &[
        Ir::GhidraPcode,
        Ir::IdaMicrocode,
        Ir::BinaryNinjaLlil,
        Ir::BinaryNinjaMlil,
        Ir::BinaryNinjaHlil,
    ];

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
    ///
    /// Every candidate is a Unix path, so on Windows the list is empty rather
    /// than wrong. `candidates` documents empty as the value for "a backend
    /// nobody has installed and checked", and nobody has checked where any of
    /// these three land on Windows -- Ghidra in particular is a zip extracted
    /// wherever the user likes, with the version in the directory name, so
    /// there is no fixed path to offer even in principle. `find_in` leaves the
    /// list out of its message when it is empty, so a Windows user is told to
    /// set `GHIDRA_HOME` and not to install it in `/opt`.
    pub(crate) fn home_spec(&self) -> HomeSpec {
        match self {
            Ir::GhidraPcode => HomeSpec {
                tool: self.tool(),
                env: "GHIDRA_HOME",
                candidates: if cfg!(windows) {
                    &[]
                } else {
                    &[
                        "/opt/homebrew/opt/ghidra/libexec",
                        "/usr/local/opt/ghidra/libexec",
                        "/opt/ghidra/libexec",
                    ]
                },
            },
            Ir::IdaMicrocode => HomeSpec {
                tool: self.tool(),
                env: "IDA_HOME",
                // The directory holding `idat`, which is the headless entry
                // point. On macOS that is inside the application bundle.
                candidates: if cfg!(windows) {
                    &[]
                } else {
                    &[
                        "/Applications/IDA Professional 9.4.app/Contents/MacOS",
                        "/Applications/IDA Classroom 9.4.app/Contents/MacOS",
                        "/opt/ida",
                    ]
                },
            },
            Ir::BinaryNinjaLlil | Ir::BinaryNinjaMlil | Ir::BinaryNinjaHlil => HomeSpec {
                tool: self.tool(),
                env: "BINARY_NINJA_HOME",
                // The directory holding `bnpython3`, which is the headless
                // entry point, rather than the bundle or the API directory
                // beside it.
                candidates: if cfg!(windows) {
                    &[]
                } else {
                    &[
                        "/Applications/Binary Ninja.app/Contents/MacOS",
                        "/opt/binaryninja",
                    ]
                },
            },
        }
    }

    /// Finds the tool's installation: what the user passed, then the
    /// environment variable, then the usual locations.
    pub(crate) fn find_home(&self, home_opt: Option<&Path>) -> Result<PathBuf> {
        if let Some(h) = home_opt {
            return Ok(h.to_path_buf());
        }
        self.home_spec()
            .find_in(|k| std::env::var(k).ok(), Path::exists)
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
pub(crate) struct HomeSpec {
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
    /// describe a machine instead of having to become one. The environment is
    /// process-global: tests that set it would see each other's writes, and
    /// `set_var` is `unsafe` because a concurrent read of it is undefined
    /// behaviour. A described machine also lets the "not found" case be
    /// asserted on one that has the tool installed.
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
        Err(crate::Error::ToolNotFound {
            tool: self.tool,
            env: self.env,
            candidates: self.candidates,
        })
    }
}

/// How to make a [`Lifter`]: the representation to produce, and optionally
/// where the tool is, which script it runs and where it works.
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

    /// The tool's installation directory.
    ///
    /// Without one -- the default -- the tool's environment variable is read
    /// (`GHIDRA_HOME` for Ghidra), and then the usual installation directories
    /// are looked in. [`Error::ToolNotFound`](crate::Error::ToolNotFound)
    /// names both when neither finds it.
    pub fn home(mut self, home: Option<PathBuf>) -> Self {
        self.home = home;
        self
    }

    /// A lifting script to run instead of the built-in one, in the tool's own
    /// language: Java for Ghidra, Python for IDA and Binary Ninja. It must
    /// write `{input file name}.json` into its working directory.
    pub fn script(mut self, script: Option<PathBuf>) -> Self {
        self.script = script;
        self
    }

    /// The directory the tool works in, kept afterwards. Without one -- the
    /// default -- a temporary directory is used and deleted.
    pub fn intermediate_dir(mut self, dir: Option<PathBuf>) -> Self {
        self.intermediate_dir = dir;
        self
    }

    /// The lifter, or [`Error::ToolNotFound`](crate::Error::ToolNotFound)
    /// when its tool cannot be found.
    pub fn build(self) -> Result<Box<dyn Lifter + Sync>> {
        // Every representation is spelled out rather than falling through to
        // one "not implemented" arm, so that adding a variant to `Ir` stops
        // the build here and asks whether this one can be written.
        match self.ir {
            Ir::GhidraPcode => {
                let home = self.ir.find_home(self.home.as_deref())?;
                Ok(Box::new(Verifying {
                    inner: crate::ghidra::lifter::GhidraLifter::new(
                        home,
                        self.script,
                        self.intermediate_dir,
                    ),
                    ir: self.ir,
                }))
            }
            Ir::IdaMicrocode => {
                let home = self.ir.find_home(self.home.as_deref())?;
                Ok(Box::new(Verifying {
                    inner: crate::ida::lifter::IdaLifter::new(
                        home,
                        self.script,
                        self.intermediate_dir,
                    ),
                    ir: self.ir,
                }))
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
                Ok(Box::new(Verifying {
                    inner: crate::binaryninja::lifter::BinaryNinjaLifter::new(
                        home,
                        level,
                        self.script,
                        self.intermediate_dir,
                    ),
                    ir: self.ir,
                }))
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
    /// `serde` derives it from `rename_all`, [`Ir::as_str`] is a hand-written
    /// match, and [`std::str::FromStr`] is another. All three are load-bearing
    /// and none of them consults the others: a lifted program's `ir` field is
    /// written and read by `serde`, a birthmark's metadata is parsed by
    /// `FromStr`, and the name a person types is whatever `Display` prints.
    ///
    /// A variant they disagreed about would not fail to compile. It would
    /// accept a name and write something else into the file, or extract a
    /// birthmark that no longer matches the program it came from.
    #[test]
    fn test_every_representation_spells_itself_the_same_way_three_times() {
        for ir in Ir::ALL {
            let json = serde_json::to_string(ir).unwrap();
            let via_serde = json.trim_matches('"');
            assert_eq!(via_serde, ir.to_string(), "serde and Display disagree");

            assert_eq!(
                &<Ir as FromStr>::from_str(via_serde).unwrap(),
                ir,
                "FromStr does not read what serde writes: {via_serde}"
            );
        }
    }

    /// A name that is no representation is refused, and the refusal names
    /// what was given.
    #[test]
    fn test_an_unknown_representation_is_refused_by_name() {
        let err = "ghidra-pcodes".parse::<Ir>().unwrap_err();
        assert!(matches!(err, crate::Error::Parse(_)), "{err:?}");
        assert!(err.to_string().contains("ghidra-pcodes"), "{err}");
    }

    /// `Ir::ALL` is written by hand, so it is held to the enum: the `match`
    /// below stops compiling when a variant is added, and the assertion fails
    /// when it is added there but not to the list.
    #[test]
    fn test_all_lists_every_representation_exactly_once() {
        fn index(ir: Ir) -> usize {
            match ir {
                Ir::GhidraPcode => 0,
                Ir::IdaMicrocode => 1,
                Ir::BinaryNinjaLlil => 2,
                Ir::BinaryNinjaMlil => 3,
                Ir::BinaryNinjaHlil => 4,
            }
        }
        let mut seen = Ir::ALL.iter().map(|ir| index(*ir)).collect::<Vec<_>>();
        seen.sort_unstable();
        assert_eq!(seen, vec![0, 1, 2, 3, 4]);
    }

    /// Every backend has to say what to set for itself, since a message naming GHIDRA_HOME for Binary Ninja is worse than no
    /// message at all.
    #[test]
    fn test_each_backend_names_its_own_environment_variable() {
        for (ir, env) in [
            (Ir::GhidraPcode, "GHIDRA_HOME"),
            (Ir::IdaMicrocode, "IDA_HOME"),
            (Ir::BinaryNinjaLlil, "BINARY_NINJA_HOME"),
            (Ir::BinaryNinjaMlil, "BINARY_NINJA_HOME"),
            (Ir::BinaryNinjaHlil, "BINARY_NINJA_HOME"),
        ] {
            assert_eq!(ir.home_spec().env, env, "{ir}");
        }
    }

    /// Every representation resolves to a tool, which is what makes naming the
    /// representation enough.
    #[test]
    fn test_every_representation_names_a_tool() {
        for ir in Ir::ALL {
            assert!(!ir.tool().is_empty(), "{ir} names no tool");
            assert!(!ir.home_spec().env.is_empty(), "{ir} names no variable");
        }
    }

    /// The search never runs: a home the caller gives is taken as given,
    /// without checking that it exists, so that the error names the path the user actually
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

    /// A spec with somewhere to look, built here rather than taken from an
    /// [`Ir`].
    ///
    /// `Ir::GhidraPcode`'s own list is empty on Windows, where oinkie knows of
    /// no install location. What the two tests below are about is the search,
    /// not which paths a platform happens to offer, and a constructed spec
    /// exercises it on all of them.
    fn a_spec_with_candidates() -> HomeSpec {
        HomeSpec {
            tool: "Ghidra",
            env: "GHIDRA_HOME",
            candidates: &["/first/ghidra", "/second/ghidra", "/third/ghidra"],
        }
    }

    #[test]
    fn test_the_usual_locations_are_searched_when_the_variable_is_unset() {
        let spec = a_spec_with_candidates();
        let last = Path::new(spec.candidates.last().unwrap());
        let home = spec.find_in(|_| None, |p| p == last).unwrap();
        assert_eq!(home, last);
    }

    /// Two installations are a real situation on a Mac with both Homebrew
    /// prefixes populated, and the order the candidates are listed in is the
    /// answer to it.
    #[test]
    fn test_the_first_of_several_installations_wins() {
        let spec = a_spec_with_candidates();
        let home = spec.find_in(|_| None, |_| true).unwrap();
        assert_eq!(home, PathBuf::from(spec.candidates[0]));
    }

    /// What a machine with no Ghidra sees -- asserted on a described machine,
    /// so that it holds on one that has Ghidra installed.
    #[test]
    fn test_a_ghidra_that_is_nowhere_says_what_to_set_and_where_it_looked() {
        let spec = Ir::GhidraPcode.home_spec();
        let err = spec.find_in(|_| None, |_| false).unwrap_err().to_string();
        // The library says what to set and where it looked, and nothing about
        // how a particular caller passes a home: that is the caller's to add.
        assert!(
            !err.contains("--"),
            "names a command-line option the library has no business knowing: {err}"
        );
        assert!(
            err.contains("GHIDRA_HOME"),
            "does not name the variable: {err}"
        );
        // Empty on Windows, where there is nowhere usual to look, so this
        // half of the message -- and of this test's name -- is the platform's
        // answer rather than an assertion that always has something to make.
        for c in spec.candidates {
            assert!(err.contains(c), "does not say it looked in {c}: {err}");
        }
    }

    /// A backend with no candidates must stop after the variable rather than
    /// invite the user to install it in one of nowhere.
    ///
    /// The spec is built here rather than taken from an [`Ir`], because on the
    /// platforms where a representation names somewhere to look it names
    /// several, and on Windows -- where none of them does -- this would be
    /// asserting the same thing as every other test in this module.
    /// An empty list is what a backend looks like before anyone has checked
    /// where it installs.
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
    ///
    /// Not on Windows, where the honest answer is that oinkie knows of no
    /// install location for any of the three; the test below is that side of
    /// it, so the `cfg` is a different assertion rather than an absent one.
    #[cfg(not(windows))]
    #[test]
    fn test_every_representation_names_somewhere_to_look() {
        for ir in Ir::ALL {
            assert!(
                !ir.home_spec().candidates.is_empty(),
                "{ir} offers no install locations"
            );
        }
    }

    /// And on Windows every list is empty, deliberately. A candidate that
    /// happens to exist is found silently and fails somewhere less obvious,
    /// which is worse than being asked for the home; a candidate spelled for
    /// another operating system cannot even be found, and only makes the
    /// message advise something impossible.
    #[cfg(windows)]
    #[test]
    fn test_no_representation_guesses_where_windows_keeps_things() {
        for ir in Ir::ALL {
            assert!(
                ir.home_spec().candidates.is_empty(),
                "{ir} offers an install location this platform does not have"
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

    /// It has a function, and that is not incidental: a file that reads and
    /// holds nothing is the case the verifier warns about, not the example of
    /// everything having worked.
    const A_READABLE_PROGRAM: &str = r#"{
        "program": "sample",
        "path": "bin/sample",
        "ir": "ghidra-pcode",
        "symbols": {},
        "functions": [
            {"name": "main", "ops": [{"op": "COPY", "inputs": ["r0"]}]}
        ]
    }"#;

    /// The same file with the functions taken out: readable, well-formed, and
    /// empty.
    const A_PROGRAM_WITH_NO_FUNCTIONS: &str = r#"{
        "program": "sample",
        "path": "bin/sample",
        "ir": "ghidra-pcode",
        "symbols": {},
        "functions": []
    }"#;

    fn lift_with<L: Lifter>(lifter: L, input: &str) -> (Result<()>, tempfile::TempDir) {
        lift_as(lifter, input, Ir::GhidraPcode)
    }

    /// A file already on disk, so a test can describe output a lifter would
    /// not write -- a representation with no reader, or one this build spells
    /// differently.
    fn lift_written(json: &str, ir: Ir) -> (Result<()>, tempfile::TempDir) {
        struct Nothing;
        impl Lifter for Nothing {
            fn lift(&self, _i: &Path, _o: &Path) -> Result<()> {
                Ok(())
            }
        }
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("sample.json");
        std::fs::write(&output, json).unwrap();
        let r = Verifying { inner: Nothing, ir }.lift(Path::new("bin/sample"), &output);
        (r, dir)
    }

    fn lift_as<L: Lifter>(lifter: L, input: &str, ir: Ir) -> (Result<()>, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("sample.json");
        let r = Verifying { inner: lifter, ir }.lift(Path::new(input), &output);
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

    /// An output whose representation this build does not know -- what a
    /// lifter written against a newer oinkie would write -- is caught here
    /// rather than at `extract`.
    #[test]
    fn test_an_output_in_an_unknown_representation_fails_the_lift() {
        let json = A_READABLE_PROGRAM.replace("ghidra-pcode", "llvm-ir");
        let (r, _dir) = lift_written(&json, Ir::GhidraPcode);
        let e = r.expect_err("a representation with no reader is not a successful lift");
        assert!(e.to_string().contains("llvm-ir"), "{e}");
    }

    /// A file with no functions is a successful lift, and says so out loud.
    ///
    /// A binary can genuinely have no functions, and nothing in the file
    /// distinguishes that from a tool that found none. Stopping would decide
    /// the question on the reader's behalf, and take the rest of a batch with
    /// it.
    #[test]
    fn test_an_output_with_no_functions_is_still_a_successful_lift() {
        let (r, _dir) = lift_with(Writes(A_PROGRAM_WITH_NO_FUNCTIONS), "bin/sample");
        assert!(
            r.is_ok(),
            "a program with no functions is not a failure: {:?}",
            r.map_err(|e| e.to_string())
        );
    }

    /// What the warning says, asserted on the message rather than through the
    /// log, since capturing a global logger from one test would make every
    /// other test in this binary depend on the order it ran in.
    ///
    /// Both readings are named, because oinkie cannot choose between them.
    #[test]
    fn test_the_no_functions_warning_names_both_readings() {
        let m = no_functions_message(Path::new("bin/sample"), Ir::GhidraPcode);
        assert!(m.contains("bin/sample:"), "{m}");
        assert!(m.contains("no functions"), "{m}");
        // the reading oinkie cannot rule out
        assert!(m.contains("really has no functions"), "{m}");
        // the reading that makes it worth saying at all
        assert!(m.contains("the tool found none"), "{m}");
        // and why an empty one matters downstream
        assert!(m.contains("perfect match"), "{m}");
    }

    /// Ghidra gets its likely cause named: a release that ships no decompiler
    /// binary for the platform, and a `DecompInterface` that then fails for
    /// every function while analyzeHeadless still exits 0.
    ///
    /// The others do not, because no cause is known for them, and advice
    /// invented for them would send the reader to a Ghidra directory they do
    /// not have.
    #[test]
    fn test_only_ghidra_is_told_about_the_decompiler() {
        let ghidra = no_functions_message(Path::new("bin/sample"), Ir::GhidraPcode);
        assert!(ghidra.contains("buildNatives"), "{ghidra}");

        for ir in [Ir::BinaryNinjaHlil, Ir::IdaMicrocode] {
            let other = no_functions_message(Path::new("bin/sample"), ir);
            assert!(other.contains("no functions"), "{other}");
            assert!(other.contains(&ir.to_string()), "{other}");
            assert!(!other.contains("buildNatives"), "{other}");
        }
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
