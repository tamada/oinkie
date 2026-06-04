use std::path::{Path, PathBuf};
use crate::{Result, Error, Lifter};

pub const DEFAULT_GHIDRA_SCRIPT: &str = include_str!("../../lifter/scripts/HighPCodeLifter.java");

pub struct GhidraLifter {
    home: PathBuf,
    script: Option<PathBuf>,
    intermediate_dir: Option<PathBuf>,
}

impl GhidraLifter {
    pub fn new(home: PathBuf, script: Option<PathBuf>, intermediate_dir: Option<PathBuf>) -> Self {
        Self { home, script, intermediate_dir }
    }
}

impl Lifter for GhidraLifter {
    fn lift(&self, input: &Path, output: &Path) -> Result<()> {
        let analyze_headless = self.home.join("support/analyzeHeadless");
        if !analyze_headless.exists() {
            return Err(Error::Parse(format!("Ghidra headless analyzer not found at {:?}", analyze_headless)));
        }

        let (script_path, _temp_dir) = if let Some(s) = &self.script {
            (s.to_path_buf(), None)
        } else {
            let temp_dir = tempfile::Builder::new().prefix("oinkie_script").tempdir().map_err(|e| Error::Io(PathBuf::from("temp"), e))?;
            let script_file = temp_dir.path().join("HighPCodeLifter.java");
            std::fs::write(&script_file, DEFAULT_GHIDRA_SCRIPT)
                .map_err(|e| Error::Io(script_file.clone(), e))?;
            (script_file, Some(temp_dir))
        };

        let (proj_dir, _temp_proj_dir) = if let Some(i) = &self.intermediate_dir {
            (i.to_path_buf(), None)
        } else {
            let temp_proj = tempfile::Builder::new().prefix("oinkie_proj").tempdir().map_err(|e| Error::Io(PathBuf::from("temp"), e))?;
            (temp_proj.path().to_path_buf(), Some(temp_proj))
        };

        let proj_name = input.file_name().ok_or_else(|| Error::Parse(format!("Invalid input path: {:?}", input)))?.to_str().unwrap();
        
        let mut command = std::process::Command::new(&analyze_headless);
        command.arg(&proj_dir)
               .arg(proj_name)
               .arg("-import").arg(input)
               .arg("-scriptPath").arg(script_path.parent().unwrap())
               .arg("-postScript").arg(script_path.file_name().unwrap());

        log::info!("Executing Ghidra: {:?}", command);
        let output_res = command.output().map_err(|e| Error::Io(analyze_headless, e))?;
        if !output_res.status.success() {
            let stderr = String::from_utf8_lossy(&output_res.stderr);
            let stdout = String::from_utf8_lossy(&output_res.stdout);
            return Err(Error::Parse(format!("Ghidra failed with status {}.\nSTDOUT: {}\nSTDERR: {}", output_res.status, stdout, stderr)));
        }

        // Move the generated JSON to the destination
        let generated_json = std::env::current_dir().unwrap().join(format!("{}.json", proj_name));
        if generated_json.exists() {
            std::fs::rename(&generated_json, output)
                .map_err(|e| Error::Io(output.to_path_buf(), e))?;
        } else {
            return Err(Error::Parse(format!("Expected Ghidra to generate {:?}, but it was not found.", generated_json)));
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
