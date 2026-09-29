use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::Once;

use crate::lift::Lifter;
use crate::lift::headless::{Headless, Invocation};
use crate::{Error, Result};

pub const DEFAULT_IDA_SCRIPT: &str =
    include_str!("../../assets/lifters/ida/scripts/MicrocodeLifter.py");

/// Said at most once per process, however many files `-j` lifts at a time.
static WARNED: Once = Once::new();

/// Lifts with IDA Pro's headless `idat`, at one maturity of the microcode.
pub struct IdaLifter {
    home: PathBuf,
    script: Option<PathBuf>,
    intermediate_dir: Option<PathBuf>,
}

impl IdaLifter {
    pub fn new(home: PathBuf, script: Option<PathBuf>, intermediate_dir: Option<PathBuf>) -> Self {
        Self {
            home,
            script,
            intermediate_dir,
        }
    }
}

impl Lifter for IdaLifter {
    fn lift(&self, input: &Path, output: &Path) -> Result<()> {
        let idat = self.home.join("idat");
        if !idat.exists() {
            return Err(Error::Parse(format!(
                "IDA Pro's headless analyser not found at {:?}",
                idat
            )));
        }

        // Straight to stderr, not through `log`. `--level error` and
        // `--level off` are ordinary things to pass, and either would silence
        // a disclosure about where someone's code is going. A privacy warning
        // that a verbosity flag can turn off is one the user never agreed to
        // turn off.
        //
        // Before the process starts, because a warning that arrives once the
        // function has left the machine is a log entry rather than a warning.
        if let Some(message) = crate::ida::cloud::warning(&self.home) {
            WARNED.call_once(|| eprintln!("oinkie: {message}"));
        }

        Headless {
            tool: "IDA Pro",
            program: &idat,
            script: self.script.as_deref(),
            default_script: ("MicrocodeLifter.py", DEFAULT_IDA_SCRIPT),
            work_dir: self.intermediate_dir.as_deref(),
        }
        .lift(input, output, command_line)
    }
}

/// What `idat` is given.
///
/// A named function rather than a closure so that it can be asserted without
/// IDA installed. Two of these are load-bearing in ways that are invisible
/// from the shape:
///
/// - `-o` puts the database in the working directory. Without it IDA writes a
///   `.i64` beside the binary, so lifting `testdata/bin/hello_clang` would
///   leave a file in the repository.
/// - `-S` takes the script *and its arguments* as one argument, split by IDA
///   rather than by a shell.
fn command_line(i: &Invocation) -> Vec<OsString> {
    let script = i.script_dir.join(&i.script_name);
    vec![
        "-A".into(),
        "-c".into(),
        {
            let mut o = OsString::from("-o");
            o.push(i.work_dir.join(&i.name));
            o
        },
        {
            let mut s = OsString::from("-S");
            s.push(&script);
            s.push(" ");
            s.push(format!("{}.json", i.name));
            s
        },
        i.input.clone().into_os_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_built_in_script_is_compiled_in() {
        assert!(
            DEFAULT_IDA_SCRIPT.contains("import ida_hexrays"),
            "the built-in script is not the lifter"
        );
    }

    /// The five arguments, in the order `idat` reads them.
    ///
    /// `-o` and `-S` are checked for their content rather than their presence:
    /// a `-o` pointing anywhere but the working directory leaves a database in
    /// the user's repository, and `-S` carries the script's own arguments
    /// inside it, where a missing separator would reach the script as one
    /// unparsable word.
    #[test]
    fn test_idat_is_given_its_arguments_in_the_order_it_reads_them() {
        let i = Invocation {
            input: PathBuf::from("/bin/sample"),
            name: "sample".to_string(),
            work_dir: PathBuf::from("/work"),
            script_dir: PathBuf::from("/scripts"),
            script_name: OsString::from("MicrocodeLifter.py"),
        };
        let args = command_line(&i);
        assert_eq!(
            args,
            vec![
                OsString::from("-A"),
                OsString::from("-c"),
                OsString::from("-o/work/sample"),
                OsString::from("-S/scripts/MicrocodeLifter.py sample.json"),
                OsString::from("/bin/sample"),
            ]
        );
    }

    /// An installation with no `idat` is named, rather than reported as
    /// whatever failure spawning a missing program produces.
    #[test]
    fn test_a_missing_idat_is_named() {
        let lifter = IdaLifter::new(PathBuf::from("/oinkie-no-such-ida"), None, None);
        let err = match lifter.lift(Path::new("in"), Path::new("out")) {
            Err(e) => e.to_string(),
            Ok(()) => panic!("a lift succeeded without IDA"),
        };
        assert!(err.contains("idat"), "does not name what is missing: {err}");
    }
}
