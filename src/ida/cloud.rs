//! Whether an IDA installation can only decompile in the cloud.
//!
//! Producing microcode means running the decompiler, and a cloud decompiler
//! sends the function to Hex-Rays' servers. For a tool whose whole purpose is
//! deciding whether a binary was stolen, the binaries it is pointed at are
//! exactly the ones whose owner may not want them leaving the machine. So
//! `lift` says so before it starts -- before, because a warning that arrives
//! once the function has left is a log entry.
//!
//! # How this is known
//!
//! Not from the API. `ida_hexrays` offers `get_hexrays_version`, which reports
//! a version and nothing about where the work happens, and `MERR_CLOUD`, which
//! is an error code -- it can say a decompilation already failed for a cloud
//! reason, which is after the fact.
//!
//! From the installation instead. A decompiler plugin is named `hex<arch>`
//! when it decompiles locally and `hexc<arch>` when it calls out, so an
//! installation carrying only `hexc*` has no local decompiler to use. That
//! naming convention is not documented anywhere citable, which is why
//! [`Verdict`] carries what was seen rather than only a yes or no: if the
//! convention is wrong, the mistake should be legible in the message.
//!
//! Erring towards silence is deliberate. A false warning tells someone with a
//! local licence that their functions are being uploaded, which is worse than
//! a missing one -- it is a claim about their data that is not true.

use std::path::Path;

/// What an installation's plugin directory says about its decompiler.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Verdict {
    /// Only cloud decompiler plugins are installed, so the only decompiler
    /// that can run sends the function away. Carries their names.
    CloudOnly(Vec<String>),
    /// At least one local decompiler plugin is installed. Carries its name.
    LocalAvailable(String),
    /// No decompiler plugin was found, or the directory could not be read.
    ///
    /// Not a warning. Without a decompiler there is no microcode, so the lift
    /// will fail on its own and say so in IDA's words, which are better than a
    /// guess made here.
    Unknown,
}

/// Reads `<home>/plugins` and decides.
pub(crate) fn inspect(home: &Path) -> Verdict {
    let Ok(entries) = std::fs::read_dir(home.join("plugins")) else {
        return Verdict::Unknown;
    };
    let mut cloud = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let Some(stem) = name.split('.').next() else {
            continue;
        };
        if !stem.starts_with("hex") {
            continue;
        }
        if stem.starts_with("hexc") {
            cloud.push(name);
        } else {
            // One local decompiler is enough: it is the one that will run.
            return Verdict::LocalAvailable(name);
        }
    }
    if cloud.is_empty() {
        Verdict::Unknown
    } else {
        cloud.sort();
        Verdict::CloudOnly(cloud)
    }
}

/// What to tell the user before the first function is sent, or `None` when
/// nothing is owed.
pub(crate) fn warning(home: &Path) -> Option<String> {
    match inspect(home) {
        Verdict::CloudOnly(plugins) => Some(format!(
            "This IDA installation has only cloud decompilers ({}), and \
             microcode cannot be produced without a decompiler. Each function \
             lifted is sent to Hex-Rays' servers. Nothing has been sent yet.",
            plugins.join(", ")
        )),
        Verdict::LocalAvailable(_) | Verdict::Unknown => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn install(plugins: &[&str]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join("plugins")).unwrap();
        for p in plugins {
            fs::write(dir.path().join("plugins").join(p), b"").unwrap();
        }
        dir
    }

    /// The installation this was written against: two cloud decompilers and no
    /// local one.
    #[test]
    fn test_only_cloud_plugins_is_cloud_only() {
        let dir = install(&["hexcx64.dylib", "hexcarm.dylib", "dbg.dylib"]);
        assert_eq!(
            inspect(dir.path()),
            Verdict::CloudOnly(vec!["hexcarm.dylib".into(), "hexcx64.dylib".into()])
        );
        let warning = warning(dir.path()).expect("no warning for a cloud-only install");
        assert!(warning.contains("hexcx64.dylib"), "{warning}");
        assert!(warning.contains("Hex-Rays"), "{warning}");
    }

    /// A local licence must not be told its functions are being uploaded. One
    /// local plugin is enough, even beside cloud ones.
    #[test]
    fn test_one_local_plugin_silences_the_warning() {
        let dir = install(&["hexcx64.dylib", "hexarm64.dylib"]);
        assert_eq!(
            inspect(dir.path()),
            Verdict::LocalAvailable("hexarm64.dylib".into())
        );
        assert_eq!(warning(dir.path()), None);
    }

    /// `hexc` is a prefix of `hexcarm` but `hex` is a prefix of both, so the
    /// order of the two checks is the whole test: reversed, every cloud plugin
    /// reads as local and the warning never fires.
    #[test]
    fn test_a_cloud_plugin_is_not_mistaken_for_a_local_one() {
        let dir = install(&["hexcarm.dylib"]);
        assert!(matches!(inspect(dir.path()), Verdict::CloudOnly(_)));
    }

    /// No decompiler at all is not a warning. The lift fails on its own, in
    /// IDA's words rather than in a guess made here.
    #[test]
    fn test_no_decompiler_is_not_a_warning() {
        let dir = install(&["dbg.dylib", "pdb.dylib"]);
        assert_eq!(inspect(dir.path()), Verdict::Unknown);
        assert_eq!(warning(dir.path()), None);
    }

    /// An installation with no plugins directory says nothing rather than
    /// failing. `--home` takes what the user passed without checking it, so
    /// this is reached by a typo as much as by anything.
    #[test]
    fn test_an_unreadable_installation_says_nothing() {
        assert_eq!(inspect(Path::new("/oinkie-no-such-ida")), Verdict::Unknown);
        assert_eq!(warning(Path::new("/oinkie-no-such-ida")), None);
    }
}
