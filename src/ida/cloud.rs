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

/// What an installation's plugin directory says about its decompilers.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Verdict {
    /// At least one cloud decompiler is installed, so a function may be sent.
    /// Carries the cloud plugins, and any local ones beside them.
    SomeCloud {
        cloud: Vec<String>,
        local: Vec<String>,
    },
    /// Every decompiler installed decompiles locally, or none is installed at
    /// all.
    ///
    /// No decompiler is not a warning: without one there is no microcode, so
    /// the lift fails on its own and says so in IDA's words, which are better
    /// than a guess made here.
    NoCloud,
}

/// Reads `<home>/plugins` and sorts the decompiler plugins it finds.
pub(crate) fn inspect(home: &Path) -> Verdict {
    let Ok(entries) = std::fs::read_dir(home.join("plugins")) else {
        return Verdict::NoCloud;
    };
    let (mut cloud, mut local) = (Vec::new(), Vec::new());
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let Some(stem) = name.split('.').next() else {
            continue;
        };
        if !stem.starts_with("hex") {
            continue;
        }
        // `hexc` before `hex`: every cloud name begins with both, so testing
        // the shorter one first reads every cloud plugin as local.
        if stem.starts_with("hexc") {
            cloud.push(name);
        } else {
            local.push(name);
        }
    }
    if cloud.is_empty() {
        return Verdict::NoCloud;
    }
    cloud.sort();
    local.sort();
    Verdict::SomeCloud { cloud, local }
}

/// What to tell the user before the first function is sent, or `None` when
/// nothing is owed.
///
/// A cloud plugin anywhere in the installation earns a warning, even beside
/// local ones, because each plugin covers its own architectures: `hexcx64`
/// next to `hexarm64` decompiles x64 in the cloud whatever the ARM64 plugin
/// does. Deciding otherwise would need the binary's architecture, which is not
/// known until IDA has read it -- after the point where a warning is still
/// worth giving.
///
/// So the message says what is installed rather than what will happen, and
/// lets the reader see which half their binary falls in. That is weaker than a
/// verdict and stronger than a guess: an over-warning costs attention, while a
/// missed one costs a function that has already left the machine.
pub(crate) fn warning(home: &Path) -> Option<String> {
    let Verdict::SomeCloud { cloud, local } = inspect(home) else {
        return None;
    };
    let mut message = format!(
        "This IDA installation has cloud decompilers ({}). Microcode cannot be \
         produced without a decompiler, and every function a cloud one handles \
         is sent to Hex-Rays' servers. Nothing has been sent yet.",
        cloud.join(", ")
    );
    if !local.is_empty() {
        message.push_str(&format!(
            " It also has local decompilers ({}); which of them handles this \
             binary depends on its architecture.",
            local.join(", ")
        ));
    }
    Some(message)
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
    fn test_only_cloud_plugins_warns() {
        let dir = install(&["hexcx64.dylib", "hexcarm.dylib", "dbg.dylib"]);
        assert_eq!(
            inspect(dir.path()),
            Verdict::SomeCloud {
                cloud: vec!["hexcarm.dylib".into(), "hexcx64.dylib".into()],
                local: vec![],
            }
        );
        let warning = warning(dir.path()).expect("no warning for a cloud-only install");
        assert!(warning.contains("hexcx64.dylib"), "{warning}");
        assert!(warning.contains("Hex-Rays"), "{warning}");
        assert!(
            !warning.contains("local decompilers"),
            "claims a local decompiler that is not there: {warning}"
        );
    }

    /// A local plugin beside a cloud one does not silence the warning, because
    /// each covers its own architectures: `hexcx64` next to `hexarm64`
    /// decompiles x64 in the cloud whatever the ARM64 plugin does.
    ///
    /// This is the case the first version got wrong. It returned on the first
    /// local plugin it saw and reported that a local decompiler would run,
    /// which for an x64 binary on this installation is false -- and the cost of
    /// being wrong that way is a function that has already left the machine.
    #[test]
    fn test_a_local_plugin_for_another_architecture_does_not_silence_the_warning() {
        let dir = install(&["hexcx64.dylib", "hexarm64.dylib"]);
        assert_eq!(
            inspect(dir.path()),
            Verdict::SomeCloud {
                cloud: vec!["hexcx64.dylib".into()],
                local: vec!["hexarm64.dylib".into()],
            }
        );
        let warning = warning(dir.path()).expect("no warning where a cloud plugin exists");
        assert!(warning.contains("hexcx64.dylib"), "{warning}");
        assert!(
            warning.contains("hexarm64.dylib") && warning.contains("architecture"),
            "does not say the answer depends on the architecture: {warning}"
        );
    }

    /// Only local decompilers: nothing is owed, and saying otherwise would be a
    /// claim about someone's data that is not true.
    #[test]
    fn test_only_local_plugins_says_nothing() {
        let dir = install(&["hexx64.dylib", "hexarm64.dylib"]);
        assert_eq!(inspect(dir.path()), Verdict::NoCloud);
        assert_eq!(warning(dir.path()), None);
    }

    /// `hexc` is a prefix of `hexcarm` but `hex` is a prefix of both, so the
    /// order of the two checks is the whole test: reversed, every cloud plugin
    /// reads as local and the warning never fires.
    #[test]
    fn test_a_cloud_plugin_is_not_mistaken_for_a_local_one() {
        let dir = install(&["hexcarm.dylib"]);
        assert!(matches!(inspect(dir.path()), Verdict::SomeCloud { .. }));
    }

    /// No decompiler at all is not a warning. The lift fails on its own, in
    /// IDA's words rather than in a guess made here.
    #[test]
    fn test_no_decompiler_is_not_a_warning() {
        let dir = install(&["dbg.dylib", "pdb.dylib"]);
        assert_eq!(inspect(dir.path()), Verdict::NoCloud);
        assert_eq!(warning(dir.path()), None);
    }

    /// An installation with no plugins directory says nothing rather than
    /// failing. A home is taken as the caller gave it, without checking it,
    /// so this is reached by a typo as much as by anything.
    #[test]
    fn test_an_unreadable_installation_says_nothing() {
        assert_eq!(inspect(Path::new("/oinkie-no-such-ida")), Verdict::NoCloud);
        assert_eq!(warning(Path::new("/oinkie-no-such-ida")), None);
    }
}
