use std::ffi::OsString;
use std::path::{Path, PathBuf};

use crate::lift::Lifter;
use crate::lift::headless::{Headless, Invocation};
use crate::{Error, Result};

pub const DEFAULT_IDA_SCRIPT: &str =
    include_str!("../../assets/lifters/ida/scripts/MicrocodeLifter.py");

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
    /// The cloud-upload disclosure, when this installation has a cloud
    /// decompiler.
    ///
    /// Returned instead of printed, and meant to be shown *before* the first
    /// lift: a warning that arrives once a function has left the machine is a
    /// log entry rather than a warning. It is also meant to be shown where a
    /// verbosity setting cannot hide it, since a privacy disclosure that can be
    /// turned off is one the user never agreed to turn off.
    fn notice(&self) -> Option<String> {
        crate::ida::cloud::warning(&self.home)
    }

    fn lift(&self, input: &Path, output: &Path) -> Result<()> {
        let idat = self.home.join("idat");
        if !idat.exists() {
            return Err(Error::Parse(format!(
                "IDA Pro's headless analyser not found at {:?}",
                idat
            )));
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
        // The expected paths are built the way the code builds them rather
        // than written out as Unix strings: `Path::join` uses the platform's
        // separator, so a literal "-o/work/sample" asserts the separator
        // rather than the flag.
        let mut database = OsString::from("-o");
        database.push(i.work_dir.join(&i.name));
        let mut script = OsString::from("-S");
        script.push(i.script_dir.join(&i.script_name));
        script.push(" sample.json");
        assert_eq!(
            command_line(&i),
            vec![
                OsString::from("-A"),
                OsString::from("-c"),
                database,
                script,
                i.input.clone().into_os_string(),
            ]
        );
    }

    /// The notice is the cloud warning, and only when there is something to
    /// warn about. An installation with only local decompilers owes nothing.
    #[test]
    fn test_the_notice_is_the_cloud_warning_and_only_then() {
        let install = |plugins: &[&str]| {
            let dir = tempfile::tempdir().unwrap();
            std::fs::create_dir(dir.path().join("plugins")).unwrap();
            for p in plugins {
                std::fs::write(dir.path().join("plugins").join(p), b"").unwrap();
            }
            dir
        };

        let cloud = install(&["hexcx64.so"]);
        let lifter = IdaLifter::new(cloud.path().to_path_buf(), None, None);
        let notice = lifter
            .notice()
            .expect("a cloud decompiler is owed a notice");
        assert!(notice.contains("hexcx64.so"), "{notice}");

        let local = install(&["hex64.so"]);
        let lifter = IdaLifter::new(local.path().to_path_buf(), None, None);
        assert_eq!(lifter.notice(), None);
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
