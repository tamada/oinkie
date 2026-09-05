//! Which paths the tools are allowed to touch.
//!
//! The CLI can trust the hand that typed its arguments. A server driven by a
//! language model cannot: every path here was written by the model, and
//! nothing else in oinkie was built on that assumption.
//!
//! So the server takes `--root`, and every path -- input and output alike --
//! has to resolve inside one of them. A refusal names the roots, because the
//! useful half of "no" is what the caller may use instead.

use std::path::{Component, Path, PathBuf};

use rmcp::ErrorData;

/// The directories the tools may read and write under, canonicalized once at
/// startup so that the comparison below is between two real paths.
#[derive(Debug, Clone)]
pub struct Roots(Vec<PathBuf>);

impl Roots {
    /// Resolves the requested roots, defaulting to the working directory.
    ///
    /// A root that does not exist is refused here rather than at the first
    /// tool call: it is a mistake in how the server was started, and the
    /// person who can fix it is watching now.
    pub fn new(requested: &[PathBuf]) -> oinkie::Result<Self> {
        let requested: Vec<PathBuf> = if requested.is_empty() {
            vec![std::env::current_dir().map_err(|e| {
                oinkie::Error::Parse(format!("no working directory to serve from: {e}"))
            })?]
        } else {
            requested.to_vec()
        };
        let mut roots = Vec::with_capacity(requested.len());
        for r in requested {
            let canonical = r
                .canonicalize()
                .map_err(|e| oinkie::Error::Io(r.clone(), e))?;
            if !canonical.is_dir() {
                return Err(oinkie::Error::Parse(format!(
                    "{}: not a directory, so it cannot be a root",
                    r.display()
                )));
            }
            roots.push(canonical);
        }
        Ok(Self(roots))
    }

    /// The path the caller may use, or a refusal that says what it may use
    /// instead.
    ///
    /// The output need not exist yet -- `dest` usually does not -- so this
    /// cannot simply canonicalize and compare. It canonicalizes the deepest
    /// part that does exist, which is what resolves symlinks, and rebuilds the
    /// rest on top.
    ///
    /// The answer is true when it is given, not for ever: a symlink appearing
    /// between this and the write that follows would be followed. Guarding
    /// against that would mean writing through a root's descriptor rather than
    /// by path, and this is a local server started by the person whose files
    /// it is reading -- what it is keeping out is a mistaken path, not someone
    /// racing it.
    ///
    /// That rebuilding is why `..` is refused outright rather than resolved.
    /// A path like `<root>/nowhere/../../etc/passwd` has a canonical ancestor
    /// of `<root>`, and appending the remainder to it produces something that
    /// still *begins* with the root while pointing outside it. Refusing the
    /// component is a rule that can be stated in one line; getting the
    /// arithmetic right is not.
    pub fn resolve(&self, path: &str) -> Result<PathBuf, ErrorData> {
        let given = Path::new(path);
        if given.components().any(|c| c == Component::ParentDir) {
            return Err(ErrorData::invalid_params(
                format!(
                    "{path}: a path may not contain '..'. Write the location directly. Allowed: {}",
                    self.render()
                ),
                None,
            ));
        }

        let absolute = if given.is_absolute() {
            given.to_path_buf()
        } else {
            std::env::current_dir()
                .map_err(|e| ErrorData::internal_error(format!("no working directory: {e}"), None))?
                .join(given)
        };

        let mut walked = absolute.as_path();
        let mut tail: Vec<std::ffi::OsString> = Vec::new();
        let canonical = loop {
            match walked.canonicalize() {
                Ok(c) => break c,
                // Only "it is not there" means keep looking further up. Every
                // other failure is about a component that *is* there and could
                // not be resolved -- no permission to traverse it, a symlink
                // loop, a file where a directory was expected -- and walking
                // past one of those builds the answer out of a prefix nobody
                // established, then checks *that* against the roots. The
                // verdict would mean nothing whichever way it came out.
                Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
                    return Err(ErrorData::invalid_params(
                        format!("{path}: cannot resolve {}: {e}", walked.display()),
                        None,
                    ));
                }
                Err(_) => match (walked.file_name(), walked.parent()) {
                    (Some(name), Some(parent)) => {
                        tail.push(name.to_owned());
                        walked = parent;
                    }
                    // Walked off the top without finding anything real. On a
                    // machine where the root directory itself cannot be
                    // canonicalized there is nothing sensible left to say.
                    _ => {
                        return Err(ErrorData::invalid_params(
                            format!(
                                "{path}: no part of this path exists. Allowed: {}",
                                self.render()
                            ),
                            None,
                        ));
                    }
                },
            }
        };

        let mut resolved = canonical;
        for part in tail.into_iter().rev() {
            resolved.push(part);
        }

        if self.0.iter().any(|root| resolved.starts_with(root)) {
            Ok(resolved)
        } else {
            Err(ErrorData::invalid_params(
                format!(
                    "{path} resolves to {}, which is outside every allowed directory. Allowed: {}",
                    resolved.display(),
                    self.render()
                ),
                None,
            ))
        }
    }

    fn render(&self) -> String {
        self.0
            .iter()
            .map(|r| r.display().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fixture {
        _dir: tempfile::TempDir,
        root: PathBuf,
        roots: Roots,
    }

    /// A root holding one file and one directory, plus a sibling directory
    /// outside it to aim at.
    fn fixture() -> Fixture {
        let dir = tempfile::Builder::new()
            .prefix("oinkie_roots")
            .tempdir()
            .unwrap();
        let root = dir.path().join("inside");
        std::fs::create_dir_all(root.join("pcodes")).unwrap();
        std::fs::write(root.join("pcodes/a.json"), "{}").unwrap();
        std::fs::create_dir_all(dir.path().join("outside")).unwrap();
        std::fs::write(dir.path().join("outside/secret"), "s").unwrap();
        let roots = Roots::new(std::slice::from_ref(&root)).unwrap();
        Fixture {
            root: root.canonicalize().unwrap(),
            roots,
            _dir: dir,
        }
    }

    #[test]
    fn test_a_path_inside_a_root_is_allowed() {
        let f = fixture();
        let p = f
            .roots
            .resolve(f.root.join("pcodes/a.json").to_str().unwrap())
            .unwrap();
        assert_eq!(p, f.root.join("pcodes/a.json"));
    }

    /// `dest` usually does not exist yet, so a path that is not there is not
    /// by itself a refusal -- only one that lands outside is.
    #[test]
    fn test_a_path_that_does_not_exist_yet_is_allowed_inside_a_root() {
        let f = fixture();
        let p = f
            .roots
            .resolve(
                f.root
                    .join("similarities/deep/results.csv")
                    .to_str()
                    .unwrap(),
            )
            .unwrap();
        assert_eq!(p, f.root.join("similarities/deep/results.csv"));
    }

    #[test]
    fn test_a_path_outside_every_root_is_refused_and_says_what_is_allowed() {
        let f = fixture();
        let outside = f.root.parent().unwrap().join("outside/secret");
        let e = f.roots.resolve(outside.to_str().unwrap()).unwrap_err();
        assert!(e.message.contains("outside every allowed"), "{}", e.message);
        assert!(
            e.message.contains(&f.root.display().to_string()),
            "the refusal has to name the root: {}",
            e.message
        );
    }

    /// The case a textual prefix check would wave through: the path begins
    /// with the root and does not stay there.
    #[test]
    fn test_a_parent_component_is_refused() {
        let f = fixture();
        for attempt in [
            "pcodes/../../outside/secret",
            "../outside/secret",
            "pcodes/../a.json",
        ] {
            let e = f
                .roots
                .resolve(f.root.join(attempt).to_str().unwrap())
                .unwrap_err();
            assert!(e.message.contains(".."), "{attempt}: {}", e.message);
        }
    }

    /// And the case canonicalizing exists to catch: a link that lives inside
    /// a root and points out of it.
    #[cfg(unix)]
    #[test]
    fn test_a_symlink_out_of_a_root_is_refused() {
        let f = fixture();
        let link = f.root.join("escape");
        std::os::unix::fs::symlink(f.root.parent().unwrap().join("outside"), &link).unwrap();

        // the link itself
        let e = f.roots.resolve(link.to_str().unwrap()).unwrap_err();
        assert!(e.message.contains("outside every allowed"), "{}", e.message);

        // and something reached through it, which is the shape that matters:
        // every component of the written path is inside the root
        let e = f
            .roots
            .resolve(link.join("secret").to_str().unwrap())
            .unwrap_err();
        assert!(e.message.contains("outside every allowed"), "{}", e.message);
    }

    /// A symlink that stays inside a root is allowed, and that is deliberate
    /// rather than an oversight: what is checked is where a path leads, not
    /// how it is spelled. Pinned because it is the behaviour that rules out
    /// the tempting one-line "open with O_NOFOLLOW" -- that refuses a final
    /// component that is a link whatever it points at, including this.
    #[cfg(unix)]
    #[test]
    fn test_a_symlink_that_stays_inside_a_root_is_allowed() {
        let f = fixture();
        let link = f.root.join("shortcut.json");
        std::os::unix::fs::symlink(f.root.join("pcodes/a.json"), &link).unwrap();
        let resolved = f.roots.resolve(link.to_str().unwrap()).unwrap();
        assert_eq!(resolved, f.root.join("pcodes/a.json"));
    }

    /// A relative path is taken against the working directory, not against a
    /// root -- a root is a boundary, not a base. It still has to land inside.
    #[test]
    fn test_a_relative_path_is_resolved_against_the_working_directory() {
        let cwd = std::env::current_dir().unwrap();
        let roots = Roots::new(std::slice::from_ref(&cwd)).unwrap();
        let p = roots.resolve("Cargo.toml").unwrap();
        assert_eq!(p, cwd.canonicalize().unwrap().join("Cargo.toml"));
    }

    #[test]
    fn test_more_than_one_root_is_honoured() {
        let f = fixture();
        let other = f.root.parent().unwrap().join("outside");
        let roots = Roots::new(&[f.root.clone(), other.clone()]).unwrap();
        assert!(
            roots
                .resolve(f.root.join("pcodes/a.json").to_str().unwrap())
                .is_ok()
        );
        assert!(
            roots
                .resolve(other.join("secret").to_str().unwrap())
                .is_ok()
        );
    }

    /// A component that exists but cannot be resolved is reported as itself.
    /// It used to be treated as "not there" and walked past, which built the
    /// answer out of a prefix nobody had established -- and then checked that
    /// against the roots, so the verdict meant nothing either way.
    ///
    /// `ENOTDIR` stands in for the family. A permission error is the other
    /// everyday member and cannot be arranged portably in a test, since a
    /// suite running as root would not get one.
    #[test]
    fn test_a_component_that_cannot_be_resolved_is_not_walked_past() {
        let f = fixture();
        let through_a_file = f.root.join("pcodes/a.json/deeper/out.csv");
        let e = f
            .roots
            .resolve(through_a_file.to_str().unwrap())
            .unwrap_err();
        assert!(
            e.message.contains("cannot resolve"),
            "a file used as a directory should say so: {}",
            e.message
        );
        assert!(
            e.message.contains("a.json"),
            "and name the component that failed: {}",
            e.message
        );
    }

    /// A root that is wrong is a mistake in how the server was started, and
    /// the person who can fix it is watching at that moment rather than when
    /// a tool is called.
    #[test]
    fn test_a_root_that_is_not_there_is_refused_at_startup() {
        let dir = tempfile::tempdir().unwrap();
        assert!(Roots::new(&[dir.path().join("nope")]).is_err());
        let file = dir.path().join("a-file");
        std::fs::write(&file, "x").unwrap();
        let e = Roots::new(&[file]).unwrap_err().to_string();
        assert!(e.contains("not a directory"), "{e}");
    }

    #[test]
    fn test_no_root_given_means_the_working_directory() {
        let roots = Roots::new(&[]).unwrap();
        assert!(roots.resolve("Cargo.toml").is_ok());
    }
}
