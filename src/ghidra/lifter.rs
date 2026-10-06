//! Lifting with Ghidra: `analyzeHeadless`, running `HighPCodeLifter.java` as
//! its post-script.
//!
//! The script is embedded at build time as [`DEFAULT_GHIDRA_SCRIPT`], and is
//! what runs unless the caller gave one of their own. Running the process and
//! collecting its output is [`crate::lift::headless`]'s.

use crate::lift::Lifter;
use crate::lift::headless::Headless;
use crate::{Error, Result};
use std::path::{Path, PathBuf};

pub const DEFAULT_GHIDRA_SCRIPT: &str =
    include_str!("../../assets/lifters/ghidra/scripts/HighPCodeLifter.java");

pub struct GhidraLifter {
    home: PathBuf,
    script: Option<PathBuf>,
    intermediate_dir: Option<PathBuf>,
}

impl GhidraLifter {
    pub fn new(home: PathBuf, script: Option<PathBuf>, intermediate_dir: Option<PathBuf>) -> Self {
        Self {
            home,
            script,
            intermediate_dir,
        }
    }
}

/// The headless entry point inside a Ghidra installation.
///
/// `support/analyzeHeadless` is a bash script and `support/analyzeHeadless.bat`
/// is the Windows one. A public release carries both -- they are
/// `Ghidra/RuntimeScripts/Linux/support/analyzeHeadless` and
/// `.../Windows/support/analyzeHeadless.bat` in the source tree -- and macOS
/// uses the Linux one, so there is no third spelling.
///
/// Nothing else is needed to run the `.bat`: `std::process::Command` on Windows
/// recognises a `.bat` or `.cmd` program, substitutes `cmd.exe`, and builds a
/// batch-specific command line with its own escaping.
///
/// `windows` is a parameter rather than a `cfg!` read in here, so both
/// spellings can be asserted from either platform. A `cfg!` would compile one
/// branch away, and leave it unexercised everywhere else.
fn headless_entry_point(home: &Path, windows: bool) -> PathBuf {
    home.join(if windows {
        "support/analyzeHeadless.bat"
    } else {
        "support/analyzeHeadless"
    })
}

impl Lifter for GhidraLifter {
    fn lift(&self, input: &Path, output: &Path) -> Result<()> {
        let analyze_headless = headless_entry_point(&self.home, cfg!(windows));
        if !analyze_headless.exists() {
            return Err(Error::Parse(format!(
                "Ghidra headless analyzer not found at {:?}",
                analyze_headless
            )));
        }

        Headless {
            tool: "Ghidra",
            program: &analyze_headless,
            script: self.script.as_deref(),
            default_script: ("HighPCodeLifter.java", DEFAULT_GHIDRA_SCRIPT),
            // Ghidra keeps its project here as well as working in it, which is
            // why the option exists; the working directory itself is what
            // every headless lifter needs.
            work_dir: self.intermediate_dir.as_deref(),
        }
        .lift(input, output, |i| {
            vec![
                i.work_dir.clone().into_os_string(),
                i.name.clone().into(),
                "-import".into(),
                i.input.clone().into_os_string(),
                "-scriptPath".into(),
                i.script_dir.clone().into_os_string(),
                "-postScript".into(),
                i.script_name.clone(),
            ]
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    // Where Ghidra's installation is looked for is tested in `crate::lift`,
    // alongside the search itself.

    /// Both spellings, from whichever platform is running this. The point of
    /// the `windows` parameter is that neither branch waits for a runner.
    #[test]
    fn test_the_headless_entry_point_is_spelled_for_the_platform() {
        let home = Path::new("ghidra_home");
        assert_eq!(
            headless_entry_point(home, false),
            home.join("support/analyzeHeadless"),
        );
        assert_eq!(
            headless_entry_point(home, true),
            home.join("support/analyzeHeadless.bat"),
        );
    }

    /// The two differ: a lift on Windows that looked for the bash script would
    /// find nothing, and report Ghidra as missing from an installation that has
    /// it.
    #[test]
    fn test_the_two_spellings_are_not_the_same_file() {
        let home = Path::new("ghidra_home");
        assert_ne!(
            headless_entry_point(home, true),
            headless_entry_point(home, false),
        );
    }

    #[test]
    fn test_ghidra_lifter_new() {
        let home = PathBuf::from("/dummy/home");
        let script = Some(PathBuf::from("/dummy/script.java"));
        let intermediate_dir = Some(PathBuf::from("/dummy/intermediate"));

        let lifter = GhidraLifter::new(home.clone(), script.clone(), intermediate_dir.clone());
        assert_eq!(lifter.home, home);
        assert_eq!(lifter.script, script);
        assert_eq!(lifter.intermediate_dir, intermediate_dir);
    }

    #[test]
    fn test_ghidra_lifter_lift_headless_not_found() {
        // Use a dummy home where analyzeHeadless definitely doesn't exist
        let temp_dir = tempdir().unwrap();
        let lifter = GhidraLifter::new(temp_dir.path().to_path_buf(), None, None);

        let input = PathBuf::from("dummy_input");
        let output = PathBuf::from("dummy_output");

        let result = lifter.lift(&input, &output);
        assert!(result.is_err());
        match result {
            Err(Error::Parse(msg)) => {
                assert!(msg.contains("Ghidra headless analyzer not found at"));
            }
            _ => panic!("Expected Error::Parse"),
        }
    }
}
