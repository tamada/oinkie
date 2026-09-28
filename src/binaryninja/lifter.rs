use crate::lift::headless::Headless;
use crate::lift::{Ir, Lifter};
use crate::{Error, Result};
use std::path::{Path, PathBuf};

pub const DEFAULT_BINARY_NINJA_SCRIPT: &str =
    include_str!("../../assets/lifters/binaryninja/scripts/BnilLifter.py");

/// Lifts with Binary Ninja's headless Python, at one of its three levels.
///
/// The level is carried as an [`Ir`] rather than as a separate argument for
/// the reason `lift` takes only one: a representation implies its tool, and
/// the three levels share this installation.
pub struct BinaryNinjaLifter {
    home: PathBuf,
    ir: Ir,
    script: Option<PathBuf>,
    intermediate_dir: Option<PathBuf>,
}

impl BinaryNinjaLifter {
    pub fn new(
        home: PathBuf,
        ir: Ir,
        script: Option<PathBuf>,
        intermediate_dir: Option<PathBuf>,
    ) -> Self {
        Self {
            home,
            ir,
            script,
            intermediate_dir,
        }
    }

    /// The word the lifting script takes for this representation.
    ///
    /// An `Ir` that is not one of Binary Ninja's cannot reach here, because
    /// `LifterBuilder::build` is the only construction site and it matches on
    /// the representation first.
    fn level(&self) -> &'static str {
        match self.ir {
            Ir::BinaryNinjaLlil => "llil",
            Ir::BinaryNinjaMlil => "mlil",
            Ir::BinaryNinjaHlil => "hlil",
            other => unreachable!("{other} is not a Binary Ninja representation"),
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

        let level = self.level();
        Headless {
            tool: "Binary Ninja",
            program: &bnpython,
            script: self.script.as_deref(),
            default_script: ("BnilLifter.py", DEFAULT_BINARY_NINJA_SCRIPT),
            work_dir: self.intermediate_dir.as_deref(),
        }
        .lift(input, output, |i| {
            vec![
                i.script_dir.join(&i.script_name).into_os_string(),
                i.input.clone().into_os_string(),
                level.into(),
                format!("{}.json", i.name).into(),
            ]
        })
    }
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

    /// Each representation names its own level, and nothing else does.
    #[test]
    fn test_each_representation_names_its_level() {
        for (ir, level) in [
            (Ir::BinaryNinjaLlil, "llil"),
            (Ir::BinaryNinjaMlil, "mlil"),
            (Ir::BinaryNinjaHlil, "hlil"),
        ] {
            let lifter = BinaryNinjaLifter::new(PathBuf::from("/nowhere"), ir, None, None);
            assert_eq!(lifter.level(), level, "{ir}");
        }
    }

    /// An installation with no `bnpython3` is named, rather than reported as
    /// whatever failure spawning a missing program produces.
    #[test]
    fn test_a_missing_headless_python_is_named() {
        let lifter = BinaryNinjaLifter::new(
            PathBuf::from("/oinkie-no-such-binary-ninja"),
            Ir::BinaryNinjaLlil,
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
