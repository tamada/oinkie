//! Lifting with Binary Ninja: its headless Python, running `BnilLifter.py` at
//! one of LLIL, MLIL and HLIL.
//!
//! The script is embedded at build time as [`DEFAULT_BINARY_NINJA_SCRIPT`].

use crate::lift::Lifter;
use crate::lift::headless::{Headless, Invocation};
use crate::{Error, Result};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

pub const DEFAULT_BINARY_NINJA_SCRIPT: &str =
    include_str!("../../assets/lifters/binaryninja/scripts/BnilLifter.py");

/// Lifts with Binary Ninja's headless Python, at one of its three levels.
///
/// The level is not a separate option: the caller names it by naming the
/// representation -- `binary-ninja-llil`, `-mlil` or `-hlil` -- since a
/// representation implies its tool. [`LifterBuilder::build`] turns that
/// [`Ir`] into the word the script takes, and this lifter holds the word, one
/// lifter per level over the one installation the three share.
///
/// [`LifterBuilder::build`]: crate::lift::LifterBuilder::build
/// [`Ir`]: crate::lift::Ir
pub struct BinaryNinjaLifter {
    home: PathBuf,
    /// The word the lifting script takes for the level, resolved by
    /// [`crate::binaryninja::level`] before this was constructed so that
    /// nothing here has to account for a representation that is not Binary
    /// Ninja's.
    level: &'static str,
    script: Option<PathBuf>,
    intermediate_dir: Option<PathBuf>,
}

impl BinaryNinjaLifter {
    pub fn new(
        home: PathBuf,
        level: &'static str,
        script: Option<PathBuf>,
        intermediate_dir: Option<PathBuf>,
    ) -> Self {
        Self {
            home,
            level,
            script,
            intermediate_dir,
        }
    }
}

impl Lifter for BinaryNinjaLifter {
    fn lift(&self, input: &Path, output: &Path) -> Result<()> {
        // Binary Ninja ships its own interpreter with the API already
        // importable, so the script needs no PYTHONPATH and the user's python
        // is not involved.
        let bnpython = self.home.join("bnpython3");
        if !bnpython.exists() {
            return Err(Error::Parse(format!(
                "Binary Ninja's headless Python not found at {:?}",
                bnpython
            )));
        }

        Headless {
            tool: "Binary Ninja",
            program: &bnpython,
            script: self.script.as_deref(),
            default_script: ("BnilLifter.py", DEFAULT_BINARY_NINJA_SCRIPT),
            work_dir: self.intermediate_dir.as_deref(),
        }
        .lift(input, output, |i| command_line(self.level, i))
    }
}

/// What `bnpython3` is given: the script, the binary, the level, and the name
/// to write.
///
/// A named function rather than a closure so that the order can be asserted
/// without Binary Ninja installed. The script reads these positionally, so
/// swapping two of them would produce a run that fails somewhere inside
/// Binary Ninja rather than here.
fn command_line(level: &'static str, i: &Invocation) -> Vec<OsString> {
    vec![
        i.script_dir.join(&i.script_name).into_os_string(),
        i.input.clone().into_os_string(),
        level.into(),
        format!("{}.json", i.name).into(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The script is compiled into the binary, so a move of the file it comes
    /// from is a build failure rather than a lift that finds nothing. This
    /// says the text arrived, since `include_str!` of an empty file would not
    /// fail on its own.
    #[test]
    fn test_the_built_in_script_is_compiled_in() {
        assert!(
            DEFAULT_BINARY_NINJA_SCRIPT.contains("import binaryninja"),
            "the built-in script is not the lifter"
        );
    }

    /// The four arguments the script reads positionally, in order.
    ///
    /// The script's own usage line is `BnilLifter.py <binary> <level>
    /// <output-name>` after the interpreter has taken the script itself, and
    /// this is the only place that order is written on the Rust side.
    #[test]
    fn test_the_script_is_given_its_arguments_in_the_order_it_reads_them() {
        let i = Invocation {
            input: PathBuf::from("/bin/sample"),
            name: "sample".to_string(),
            work_dir: PathBuf::from("/work"),
            script_dir: PathBuf::from("/scripts"),
            script_name: OsString::from("BnilLifter.py"),
        };
        // The expected paths are built the way the code builds them rather
        // than written out as Unix strings. `Path::join` uses the platform's
        // separator, so a literal "/scripts/BnilLifter.py" asserts the
        // separator rather than the order -- and fails on Windows for saying
        // nothing about this function.
        assert_eq!(
            command_line("mlil", &i),
            vec![
                i.script_dir.join(&i.script_name).into_os_string(),
                i.input.clone().into_os_string(),
                OsString::from("mlil"),
                OsString::from("sample.json"),
            ]
        );
    }

    /// An installation with no `bnpython3` is named, rather than reported as
    /// whatever failure spawning a missing program produces.
    #[test]
    fn test_a_missing_headless_python_is_named() {
        let lifter = BinaryNinjaLifter::new(
            PathBuf::from("/oinkie-no-such-binary-ninja"),
            "llil",
            None,
            None,
        );
        let err = lifter
            .lift(Path::new("in"), Path::new("out"))
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("bnpython3"),
            "does not name what is missing: {err}"
        );
    }
}
