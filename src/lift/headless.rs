//! The invocation shape every headless lifter has in common.
//!
//! Ghidra, Binary Ninja and IDA Pro are all driven the same way: run the
//! tool's headless entry point with a script that writes JSON, then move the
//! result where the caller asked for it. The command lines differ; the path
//! handling around them does not, and it is the path handling that has been
//! wrong twice already.
//!
//! Every bug fixed in this file was a real one:
//!
//! - a relative `--intermediate` was resolved twice, once by the caller and
//!   again by Ghidra against its own working directory, which is that
//!   directory nested inside itself
//! - the script writes into the process's working directory, so two lifts of
//!   same-named binaries running in parallel raced on one path in the user's
//!   current directory
//! - `rename` fails across file systems, which a temporary directory on
//!   another volume reaches every time
//!
//! A second and third lifter writing this out again would be two more chances
//! to reintroduce them, so a lifter supplies its command line and its script
//! and nothing else.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use crate::{Error, Result};

/// An external tool driven headless with a script that writes JSON.
pub(crate) struct Headless<'a> {
    /// The tool's name, for messages.
    pub tool: &'static str,
    /// The executable to run. The caller checks it exists, since only the
    /// caller knows what to say when it does not.
    pub program: &'a Path,
    /// A script supplied by the user, in place of the built-in one.
    pub script: Option<&'a Path>,
    /// The built-in script: the file name to write it under, and its text.
    pub default_script: (&'static str, &'static str),
    /// Where the tool should work. A temporary directory is used and removed
    /// when this is `None`.
    ///
    /// Every headless run needs a working directory, because that is where
    /// the script writes; keeping it is a debugging convenience rather than a
    /// Ghidra-specific need, even though Ghidra also uses it as its project
    /// directory.
    pub work_dir: Option<&'a Path>,
}

/// The paths a command line is built from, all resolved before the working
/// directory changes.
pub(crate) struct Invocation {
    /// The binary to lift, absolute.
    pub input: PathBuf,
    /// The input's file name. The script writes `{name}.json`, and Ghidra
    /// also takes it as the project name.
    pub name: String,
    /// The directory the tool runs in and writes into, absolute.
    pub work_dir: PathBuf,
    /// The directory holding the script, absolute.
    pub script_dir: PathBuf,
    /// The script's file name on its own, which is how Ghidra wants it named
    /// once `-scriptPath` has been given.
    ///
    /// Split from the directory rather than kept whole because that is the
    /// form Ghidra needs; a tool wanting the full path can rejoin them.
    pub script_name: OsString,
}

impl Headless<'_> {
    /// Runs the tool and moves the JSON it wrote to `output`.
    ///
    /// `args` receives paths that are already absolute, because the process
    /// runs with the working directory set to `Invocation::work_dir` and a
    /// relative path would be resolved against that instead of against the
    /// caller's.
    pub(crate) fn lift(
        &self,
        input: &Path,
        output: &Path,
        args: impl FnOnce(&Invocation) -> Vec<OsString>,
    ) -> Result<()> {
        // Both temporary directories are removed when dropped, so they are
        // held until the command has finished with them.
        let (script, _script_tmp) = self.resolve_script()?;
        let (work_dir, _work_tmp) = self.resolve_work_dir()?;
        let invocation = Self::resolve(input, work_dir, script)?;

        let mut command = std::process::Command::new(self.program);
        command
            .args(args(&invocation))
            .current_dir(&invocation.work_dir);

        log::info!("Executing {}: {:?}", self.tool, command);
        let result = command
            .output()
            .map_err(|e| Error::Io(self.program.to_path_buf(), e))?;
        if !result.status.success() {
            return Err(Error::Parse(format!(
                "{} failed with status {}.\nSTDOUT: {}\nSTDERR: {}",
                self.tool,
                result.status,
                String::from_utf8_lossy(&result.stdout),
                String::from_utf8_lossy(&result.stderr),
            )));
        }

        self.take_output(&invocation, output)
    }

    /// Writes the built-in script to a temporary directory unless the user
    /// supplied one.
    fn resolve_script(&self) -> Result<(PathBuf, Option<tempfile::TempDir>)> {
        match self.script {
            Some(s) => {
                let s = std::fs::canonicalize(s).map_err(|e| Error::Io(s.to_path_buf(), e))?;
                Ok((s, None))
            }
            None => {
                let dir = Self::temp_dir("oinkie_script")?;
                let (name, text) = self.default_script;
                let path = dir.path().join(name);
                std::fs::write(&path, text).map_err(|e| Error::Io(path.clone(), e))?;
                Ok((path, Some(dir)))
            }
        }
    }

    /// The directory the tool runs in, created if the user named one.
    fn resolve_work_dir(&self) -> Result<(PathBuf, Option<tempfile::TempDir>)> {
        match self.work_dir {
            Some(d) => {
                // Created because every other destination directory in the CLI
                // is, and canonicalized because it is passed to the tool as an
                // argument while also being its working directory -- left
                // relative, the tool resolves it a second time against itself.
                std::fs::create_dir_all(d).map_err(|e| Error::Io(d.to_path_buf(), e))?;
                let d = std::fs::canonicalize(d).map_err(|e| Error::Io(d.to_path_buf(), e))?;
                Ok((d, None))
            }
            None => {
                let dir = Self::temp_dir("oinkie_work")?;
                Ok((dir.path().to_path_buf(), Some(dir)))
            }
        }
    }

    /// Ghidra rejects any path element beginning with a dot, which is what
    /// `tempfile`'s default prefix produces, so every temporary directory here
    /// is named explicitly.
    fn temp_dir(prefix: &str) -> Result<tempfile::TempDir> {
        tempfile::Builder::new()
            .prefix(prefix)
            .tempdir()
            .map_err(|e| Error::Io(PathBuf::from(prefix), e))
    }

    fn resolve(input: &Path, work_dir: PathBuf, script: PathBuf) -> Result<Invocation> {
        let invalid = |what: &str, p: &Path| Error::Parse(format!("Invalid {what} path: {p:?}"));
        let name = input
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| invalid("input", input))?
            .to_string();
        let script_dir = script
            .parent()
            .ok_or_else(|| invalid("script", &script))?
            .to_path_buf();
        let script_name = script
            .file_name()
            .ok_or_else(|| invalid("script", &script))?
            .to_os_string();
        // The tool's working directory is not the caller's, so a relative
        // input would be resolved against the wrong one.
        let input = std::fs::canonicalize(input).map_err(|e| Error::Io(input.to_path_buf(), e))?;
        Ok(Invocation {
            input,
            name,
            work_dir,
            script_dir,
            script_name,
        })
    }

    /// Moves the JSON the script wrote to where the caller asked for it.
    fn take_output(&self, invocation: &Invocation, output: &Path) -> Result<()> {
        let generated = invocation
            .work_dir
            .join(format!("{}.json", invocation.name));
        if !generated.exists() {
            return Err(Error::Parse(format!(
                "Expected {} to generate {:?}, but it was not found.",
                self.tool, generated
            )));
        }
        // rename fails across file systems, which a temporary directory on
        // another volume reaches every time; fall back to copy + remove.
        if std::fs::rename(&generated, output).is_err() {
            std::fs::copy(&generated, output).map_err(|e| Error::Io(output.to_path_buf(), e))?;
            let _ = std::fs::remove_file(&generated);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `true` ignores its arguments and writes nothing, which is the whole
    /// point: it exercises the path handling without needing a decompiler.
    fn headless<'a>(work_dir: Option<&'a Path>, script: Option<&'a Path>) -> Headless<'a> {
        Headless {
            tool: "test",
            program: Path::new("/usr/bin/true"),
            script,
            default_script: ("script.txt", "built-in"),
            work_dir,
        }
    }

    /// The tool runs with the working directory set to its own scratch
    /// directory, so a relative input would be resolved against that rather
    /// than the caller's -- the shape of the bug that made `lift -i irs` look
    /// for `irs/irs`.
    #[test]
    fn test_paths_reach_the_command_absolute() {
        let dir = tempfile::Builder::new()
            .prefix("oinkie_t")
            .tempdir()
            .unwrap();
        let input = dir.path().join("sample.bin");
        std::fs::write(&input, b"binary").unwrap();
        let relative = pathdiff(&input);

        let seen = std::cell::RefCell::new(None);
        let _ = headless(None, None).lift(&relative, &dir.path().join("out.json"), |i| {
            *seen.borrow_mut() = Some((i.input.clone(), i.work_dir.clone(), i.name.clone()));
            vec![]
        });
        let (resolved, work_dir, name) = seen.into_inner().expect("args were never built");
        assert!(
            resolved.is_absolute(),
            "input stayed relative: {resolved:?}"
        );
        assert!(
            work_dir.is_absolute(),
            "work dir stayed relative: {work_dir:?}"
        );
        assert_eq!(name, "sample.bin");
    }

    /// A working directory the user named is created rather than required to
    /// exist, and is handed over resolved so the tool cannot resolve it again.
    #[test]
    fn test_named_work_dir_is_created_and_absolute() {
        let dir = tempfile::Builder::new()
            .prefix("oinkie_t")
            .tempdir()
            .unwrap();
        let input = dir.path().join("sample.bin");
        std::fs::write(&input, b"binary").unwrap();
        let work = dir.path().join("nested/work");
        assert!(!work.exists());

        let seen = std::cell::RefCell::new(None);
        let _ = headless(Some(&work), None).lift(&input, &dir.path().join("out.json"), |i| {
            *seen.borrow_mut() = Some(i.work_dir.clone());
            vec![]
        });
        assert!(work.is_dir(), "the working directory was not created");
        let used = seen.into_inner().expect("args were never built");
        assert!(used.is_absolute());
        assert_eq!(used, std::fs::canonicalize(&work).unwrap());
    }

    /// Without a script of its own a lifter gets the built-in one written out,
    /// under the name it asked for and in a directory of its own.
    #[test]
    fn test_the_built_in_script_is_written_out() {
        let dir = tempfile::Builder::new()
            .prefix("oinkie_t")
            .tempdir()
            .unwrap();
        let input = dir.path().join("sample.bin");
        std::fs::write(&input, b"binary").unwrap();

        let seen = std::cell::RefCell::new(None);
        let _ = headless(None, None).lift(&input, &dir.path().join("out.json"), |i| {
            let script = i.script_dir.join(&i.script_name);
            *seen.borrow_mut() = Some((
                std::fs::read_to_string(script).unwrap(),
                i.script_name.clone(),
            ));
            vec![]
        });
        let (text, name) = seen.into_inner().expect("args were never built");
        assert_eq!(text, "built-in");
        assert_eq!(name, "script.txt");
    }

    /// A tool that produced nothing must say so rather than leave the caller
    /// with a missing file and a zero exit status.
    #[test]
    fn test_a_missing_result_is_reported() {
        let dir = tempfile::Builder::new()
            .prefix("oinkie_t")
            .tempdir()
            .unwrap();
        let input = dir.path().join("sample.bin");
        std::fs::write(&input, b"binary").unwrap();

        match headless(None, None).lift(&input, &dir.path().join("out.json"), |_| vec![]) {
            Err(Error::Parse(msg)) => assert!(
                msg.contains("sample.bin.json") && msg.contains("test"),
                "unhelpful message: {msg}"
            ),
            other => panic!("expected a missing-output error, got {:?}", other.err()),
        }
    }

    /// The output is moved even when the working directory is on another file
    /// system, where `rename` fails and only copy + remove works. The move
    /// itself is what is checked here; the cross-volume case cannot be staged
    /// portably.
    #[test]
    fn test_the_result_is_moved_to_the_destination() {
        let dir = tempfile::Builder::new()
            .prefix("oinkie_t")
            .tempdir()
            .unwrap();
        let input = dir.path().join("sample.bin");
        std::fs::write(&input, b"binary").unwrap();
        let work = dir.path().join("work");
        let output = dir.path().join("out.json");

        // `true` writes nothing, so stage what the script would have written.
        std::fs::create_dir_all(&work).unwrap();
        std::fs::write(work.join("sample.bin.json"), r#"{"program":"sample"}"#).unwrap();

        headless(Some(&work), None)
            .lift(&input, &output, |_| vec![])
            .expect("the staged output should have been moved");
        assert_eq!(
            std::fs::read_to_string(&output).unwrap(),
            r#"{"program":"sample"}"#
        );
        assert!(
            !work.join("sample.bin.json").exists(),
            "the result was copied rather than moved"
        );
    }

    /// A path relative to the current directory, so that the caller's working
    /// directory is what a relative input would be resolved against.
    fn pathdiff(absolute: &Path) -> PathBuf {
        let cwd = std::env::current_dir().unwrap();
        match absolute.strip_prefix(&cwd) {
            Ok(rest) => rest.to_path_buf(),
            // Not under the current directory, so leave it absolute: the test
            // still checks that what reaches the command is absolute.
            Err(_) => absolute.to_path_buf(),
        }
    }
}
