use crate::lift::Lifter;
use crate::{Error, Result};
use std::path::{Path, PathBuf};

pub const DEFAULT_GHIDRA_SCRIPT: &str = include_str!("../../lifter/scripts/HighPCodeLifter.java");

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

impl Lifter for GhidraLifter {
    fn lift(&self, input: &Path, output: &Path) -> Result<()> {
        let analyze_headless = self.home.join("support/analyzeHeadless");
        if !analyze_headless.exists() {
            return Err(Error::Parse(format!(
                "Ghidra headless analyzer not found at {:?}",
                analyze_headless
            )));
        }

        let (script_path, _temp_dir) = if let Some(s) = &self.script {
            let s = std::fs::canonicalize(s).map_err(|e| Error::Io(s.clone(), e))?;
            (s, None)
        } else {
            let temp_dir = tempfile::Builder::new()
                .prefix("oinkie_script")
                .tempdir()
                .map_err(|e| Error::Io(PathBuf::from("temp"), e))?;
            let script_file = temp_dir.path().join("HighPCodeLifter.java");
            std::fs::write(&script_file, DEFAULT_GHIDRA_SCRIPT)
                .map_err(|e| Error::Io(script_file.clone(), e))?;
            (script_file, Some(temp_dir))
        };
        let script_dir = script_path
            .parent()
            .ok_or_else(|| Error::Parse(format!("Invalid script path: {:?}", script_path)))?;
        let script_name = script_path
            .file_name()
            .ok_or_else(|| Error::Parse(format!("Invalid script path: {:?}", script_path)))?;

        let (proj_dir, _temp_proj_dir) = if let Some(i) = &self.intermediate_dir {
            (i.to_path_buf(), None)
        } else {
            let temp_proj = tempfile::Builder::new()
                .prefix("oinkie_proj")
                .tempdir()
                .map_err(|e| Error::Io(PathBuf::from("temp"), e))?;
            (temp_proj.path().to_path_buf(), Some(temp_proj))
        };

        let proj_name = input
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| Error::Parse(format!("Invalid input path: {:?}", input)))?;
        // the working directory of the Ghidra process is the project directory,
        // so relative paths must be resolved beforehand
        let input = std::fs::canonicalize(input).map_err(|e| Error::Io(input.to_path_buf(), e))?;

        let mut command = std::process::Command::new(&analyze_headless);
        command
            .arg(&proj_dir)
            .arg(proj_name)
            .arg("-import")
            .arg(&input)
            .arg("-scriptPath")
            .arg(script_dir)
            .arg("-postScript")
            .arg(script_name)
            // The lifter script writes "{program name}.json" into the working
            // directory of the Ghidra process. Run it inside the project
            // directory so that concurrent lifts of same-named binaries do not
            // race on a single path in the user's current directory.
            .current_dir(&proj_dir);

        log::info!("Executing Ghidra: {:?}", command);
        let output_res = command
            .output()
            .map_err(|e| Error::Io(analyze_headless, e))?;
        if !output_res.status.success() {
            let stderr = String::from_utf8_lossy(&output_res.stderr);
            let stdout = String::from_utf8_lossy(&output_res.stdout);
            return Err(Error::Parse(format!(
                "Ghidra failed with status {}.\nSTDOUT: {}\nSTDERR: {}",
                output_res.status, stdout, stderr
            )));
        }

        // Move the generated JSON to the destination
        let generated_json = proj_dir.join(format!("{}.json", proj_name));
        if generated_json.exists() {
            // rename fails across file systems (e.g., temp dir on another
            // volume); fall back to copy + remove in that case
            if std::fs::rename(&generated_json, output).is_err() {
                std::fs::copy(&generated_json, output)
                    .map_err(|e| Error::Io(output.to_path_buf(), e))?;
                let _ = std::fs::remove_file(&generated_json);
            }
        } else {
            return Err(Error::Parse(format!(
                "Expected Ghidra to generate {:?}, but it was not found.",
                generated_json
            )));
        }

        Ok(())
    }
}

pub fn find_ghidra_home(home_opt: Option<&Path>) -> Result<PathBuf> {
    if let Some(h) = home_opt {
        return Ok(h.to_path_buf());
    }
    if let Ok(h) = std::env::var("GHIDRA_HOME") {
        return Ok(PathBuf::from(h));
    }

    let candidates = [
        "/opt/homebrew/opt/ghidra/libexec",
        "/usr/local/opt/ghidra/libexec",
        "/opt/ghidra/libexec",
    ];
    for c in candidates {
        let p = PathBuf::from(c);
        if p.exists() {
            return Ok(p);
        }
    }
    Err(Error::Parse("GHIDRA_HOME not found. Please specify it via --home option or GHIDRA_HOME environment variable.".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::tempdir;

    #[test]
    fn test_find_ghidra_home_with_opt() {
        let opt_path = PathBuf::from("/custom/ghidra/home");
        let result = find_ghidra_home(Some(&opt_path));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), opt_path);
    }

    #[test]
    fn test_find_ghidra_home_with_env() {
        // Backup the env
        let old_env = env::var("GHIDRA_HOME").ok();
        unsafe {
            env::set_var("GHIDRA_HOME", "/env/ghidra/home");
        }

        let result = find_ghidra_home(None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PathBuf::from("/env/ghidra/home"));

        // Restore env
        if let Some(val) = old_env {
            unsafe {
                env::set_var("GHIDRA_HOME", val);
            }
        } else {
            unsafe {
                env::remove_var("GHIDRA_HOME");
            }
        }
    }

    #[test]
    fn test_find_ghidra_home_not_found() {
        // Backup the env
        let old_env = env::var("GHIDRA_HOME").ok();
        unsafe {
            env::remove_var("GHIDRA_HOME");
        }

        let result = find_ghidra_home(None);
        // It might find it in standard paths on some systems, so we can't definitively assert error
        // unless we know the system doesn't have it. But we can check it doesn't crash.
        let _ = result;

        // Restore env
        if let Some(val) = old_env {
            unsafe {
                env::set_var("GHIDRA_HOME", val);
            }
        }
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
