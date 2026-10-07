//! The MCP server itself.
//!
//! Nothing here goes through the `perform_*` drivers in `cli/main.rs`. Over
//! stdio, **stdout is the JSON-RPC channel**, and those drivers `println!`
//! their progress; one such line corrupts the session. The tools call the
//! library directly for the same reason no progress bar is constructed.

use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::vocabulary::{ALGORITHMS, STRATEGIES, by_name};
use oinkie::birthmarks::{AnalysisType, BirthmarkType};
use oinkie::compare::{Aggregator, PairingStrategy};
use rmcp::handler::server::wrapper::{Json, Parameters};
use rmcp::model::{Implementation, InitializeResult, ServerCapabilities};
use rmcp::{ErrorData, ServerHandler, schemars, tool, tool_handler, tool_router};
use serde::{Deserialize, Serialize};

use super::analysis::{self, Extracted, Score};
use super::info::{self, Vocabulary};
use super::paths::Roots;

/// How many pairs a single call will compare before refusing.
///
/// `all-and-self` over N files is N(N+1)/2 pairs, and a model handed a
/// directory will pass all of it. This is a guard against that arriving by
/// accident, not a limit on what may be asked: `max_pairs` raises it, and
/// raising it is then a deliberate act rather than a surprise.
const DEFAULT_MAX_PAIRS: usize = 500;

#[derive(Clone)]
pub struct Oinkie {
    roots: Roots,
    /// The threads every tool computes on, shared by all of them so that
    /// `--threads` bounds the server as a whole.
    engine: oinkie::Oinkie,
    tool_router: rmcp::handler::server::tool::ToolRouter<Self>,
}

impl Oinkie {
    pub fn new(roots: Roots, engine: oinkie::Oinkie) -> Self {
        Self {
            roots,
            engine,
            tool_router: Self::tool_router(),
        }
    }

    /// Runs `f` off the async runtime, on the server's threads: the tools'
    /// work would otherwise stall everything else the server has to answer,
    /// cancellation included.
    async fn compute<T: Send + 'static>(
        &self,
        f: impl FnOnce() -> T + Send + 'static,
    ) -> Result<T, ErrorData> {
        let engine = self.engine.clone();
        tokio::task::spawn_blocking(move || engine.install(f))
            .await
            .map_err(joined)
    }
}

/// A blocking task that did not come back -- panicked, or was cancelled.
///
/// Not the caller's doing, whatever they asked for, so it is reported as an
/// internal failure rather than as a bad argument.
fn joined(e: tokio::task::JoinError) -> ErrorData {
    ErrorData::internal_error(format!("the work did not finish: {e}"), None)
}

/// A refusal that points at where the accepted names are.
///
/// Every one of these is a value the caller chose, so it is reported as their
/// mistake -- which the shared conversion cannot do, since the library refuses
/// several of them through `Error::Parse` and that variant spans both sides.
fn refuse(e: impl std::fmt::Display) -> ErrorData {
    ErrorData::invalid_params(
        format!("{e}. Call oinkie_info for the names this accepts."),
        None,
    )
}

impl Oinkie {
    fn resolve_all(&self, files: &[String]) -> Result<Vec<PathBuf>, ErrorData> {
        if files.is_empty() {
            return Err(ErrorData::invalid_params(
                "no files given; there is nothing to do".to_string(),
                None,
            ));
        }
        files.iter().map(|f| self.roots.resolve(f)).collect()
    }

    fn dest(&self, dest: Option<&str>) -> Result<Option<PathBuf>, ErrorData> {
        dest.map(|d| self.roots.resolve(d)).transpose()
    }

    /// Resolves each of the files a tool writes below a destination it was
    /// given. `dest` was confined, but the files in it are named by the tool,
    /// not the caller, and one already there can be a symlink out of every
    /// root, which resolving `dest` does not look through. So they are
    /// resolved too, before anything is written.
    fn confine(&self, written: impl IntoIterator<Item = PathBuf>) -> Result<(), ErrorData> {
        written.into_iter().try_for_each(|p| {
            let text = p.to_str().ok_or_else(|| {
                ErrorData::invalid_params(
                    format!("{}: not a path the server can confine", p.display()),
                    None,
                )
            })?;
            self.roots.resolve(text).map(|_| ())
        })
    }

    /// What `run` writes below `dest`: its birthmarks, the pairs' CSVs and
    /// their index.
    fn confine_run(
        &self,
        files: &[PathBuf],
        dest: &Path,
        strategy: &PairingStrategy,
    ) -> Result<(), ErrorData> {
        let birthmarks = analysis::birthmark_files(files, dest).map_err(super::error::to_mcp)?;
        self.confine(
            std::iter::once(analysis::birthmarks_dir(dest))
                .chain(birthmarks)
                .chain(analysis::score_files(dest, strategy.compare_count(files))),
        )
    }

    fn strategy(name: Option<&str>) -> Result<PairingStrategy, ErrorData> {
        by_name(STRATEGIES, name.unwrap_or("all-and-self")).map_err(refuse)
    }

    fn aggregator(name: Option<&str>) -> Result<Aggregator, ErrorData> {
        Aggregator::from_str(name.unwrap_or("hungarian")).map_err(refuse)
    }

    /// Refuses before reading anything, since the count is known from the
    /// strategy and the number of files alone.
    fn bound(
        strategy: &PairingStrategy,
        files: &[PathBuf],
        max: Option<usize>,
    ) -> Result<(), ErrorData> {
        let max = max.unwrap_or(DEFAULT_MAX_PAIRS);
        let count = strategy.compare_count(files);
        if count > max {
            return Err(ErrorData::invalid_params(
                format!(
                    "{count} pairs from {} files, which is more than the {max} this will do at \
                     once. Pass fewer files, choose a strategy that pairs them differently, or \
                     raise max_pairs deliberately.",
                    files.len()
                ),
                None,
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ExtractParams {
    /// Paths to lifted programs -- the JSON that `oinkie lift` writes.
    pub files: Vec<String>,
    /// Directory to write the birthmarks into. Created if it is not there.
    ///
    /// Required, and deliberately without a default. The CLI defaults this to
    /// "birthmarks" because the person running it chose the working directory
    /// and can see it; a server's working directory is wherever the client
    /// happened to start it. A default would resolve against that, land
    /// outside the roots, and be refused -- naming a value the caller never
    /// supplied.
    pub dest: String,
    /// Which birthmark to extract, for example "op-seq" or "op-3gram-set".
    /// Defaults to "op-seq". Call oinkie_info for the full list.
    #[serde(default)]
    pub birthmark_type: Option<String>,
    /// Leave alone any birthmark file that already exists. Default false.
    #[serde(default)]
    pub skip: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RunParams {
    /// Paths to lifted programs -- the JSON that `oinkie lift` writes.
    pub files: Vec<String>,
    /// "{birthmark}-{algorithm}", for example "op-set-jaccard". Defaults to
    /// "op-set-jaccard". Call oinkie_info for the names that pair.
    #[serde(default)]
    pub analysis: Option<String>,
    /// Which pairs to compare. Defaults to "all-and-self".
    #[serde(default)]
    pub strategy: Option<String>,
    /// Defaults to "hungarian".
    #[serde(default)]
    pub aggregator: Option<String>,
    /// Optional. Write the per-pair matrices here, as `oinkie run -d` does.
    /// The scores come back either way; this is for the detail behind them,
    /// and for handing the directory to oinkie_review afterwards.
    #[serde(default)]
    pub dest: Option<String>,
    /// Refuse rather than compare more than this many pairs. Defaults to 500.
    #[serde(default)]
    pub max_pairs: Option<usize>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CompareParams {
    /// Paths to birthmark files -- what oinkie_extract wrote.
    pub files: Vec<String>,
    /// Defaults to "jaccard". It has to operate on the birthmarks' shape;
    /// call oinkie_info for which does what.
    #[serde(default)]
    pub algorithm: Option<String>,
    /// Which pairs to compare. Defaults to "all-and-self".
    #[serde(default)]
    pub strategy: Option<String>,
    /// Defaults to "hungarian".
    #[serde(default)]
    pub aggregator: Option<String>,
    /// Optional. Write the per-pair matrices here, as `oinkie compare -d` does.
    #[serde(default)]
    pub dest: Option<String>,
    /// Refuse rather than compare more than this many pairs. Defaults to 500.
    #[serde(default)]
    pub max_pairs: Option<usize>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Compared {
    pub scores: Vec<Score>,
    pub dest: Option<String>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ExtractedAll {
    pub birthmark_type: String,
    /// One per file written. Inputs that name the same file -- one path given
    /// twice, or two copies of a program under one stem -- are extracted once
    /// and listed under the first of them.
    pub birthmarks: Vec<Extracted>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct StatsParams {
    /// Birthmark files, or directories holding them -- what oinkie_extract
    /// writes. A directory contributes the *.json directly inside it. A file
    /// that does not read as a birthmark is skipped, and listed as skipped.
    pub paths: Vec<String>,
    /// Descend into the subdirectories of the given directories. Default false.
    #[serde(default)]
    pub recursive: Option<bool>,
    /// Report each birthmark file as well as each group. Default false.
    #[serde(default)]
    pub per_file: Option<bool>,
    /// Report this many of the most frequent elements of each group.
    #[serde(default)]
    pub top: Option<usize>,
}

/// What `oinkie stats -f json` writes, with the per-file rows only when they
/// were asked for: a model pays for every row it is handed.
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub(crate) struct Statistics {
    /// One per intermediate representation and birthmark type: the
    /// birthmarks that can be compared with each other.
    pub groups: Vec<crate::stats::GroupStats>,
    /// One per birthmark file, when per_file was given.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<crate::stats::FileStats>>,
    /// The files that were given or found and are not birthmarks.
    pub skipped: Vec<crate::stats::Skipped>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ReviewParams {
    /// Directory holding the pair CSVs of function similarities that `compare` or
    /// `run` wrote -- the one they were given as their destination.
    pub score_directory: String,
    /// How to combine a pair's function similarities into its similarity:
    /// "hungarian" (the default), "topn:all", "topn:" and a count,
    /// "containment", "matched:" and a threshold, or "weighted".
    /// Call oinkie_info for what these mean.
    #[serde(default)]
    pub aggregator: Option<String>,
    /// Optional. Also write the recomputed scores to this CSV, as
    /// `oinkie review` does. The scores come back either way.
    #[serde(default)]
    pub dest_file: Option<String>,
    /// Optional. Drop each pair's functions with fewer elements than this
    /// before aggregating: a count such as "5", or a multiple of the mean
    /// count over every function of every birthmark in the directory, such
    /// as "0.3x". A bare "0.3" is refused. Short functions agree with each
    /// other by chance, and this removes that agreement; scores under
    /// different thresholds are not comparable with each other. Needs the
    /// birthmark files the score CSVs name.
    #[serde(default)]
    pub min_elements: Option<String>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Reviewed {
    pub aggregator: String,
    pub scores: Vec<Score>,
    /// Where the CSV was written, when one was asked for.
    pub dest_file: Option<String>,
    /// The threshold the scores were recomputed under, when one was given.
    pub min_elements: Option<MinElementsApplied>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct MinElementsApplied {
    /// As it was given: "5" or "0.3x".
    pub given: String,
    /// The element count it came to. A function with fewer was dropped.
    pub threshold: f64,
}

#[tool_router]
impl Oinkie {
    #[tool(
        name = "oinkie_info",
        description = "The vocabulary oinkie accepts: the birthmark types, the similarity \
                       algorithms and the shapes they operate on, every canonical \
                       {birthmark}-{algorithm} analysis name, the pairing strategies and the \
                       aggregators. Ask this before naming any of them -- the lists are \
                       generated from the same code that parses the names, and a name not \
                       derived from them may be refused."
    )]
    fn oinkie_info(&self) -> Json<Vocabulary> {
        Json(info::vocabulary())
    }

    #[tool(
        name = "oinkie_extract",
        description = "Extract a birthmark from each lifted program and write it to a \
                       directory. The birthmarks are what oinkie_compare takes. If you only \
                       want similarity scores, oinkie_run does both steps and keeps nothing."
    )]
    async fn oinkie_extract(
        &self,
        Parameters(params): Parameters<ExtractParams>,
    ) -> Result<Json<ExtractedAll>, ErrorData> {
        let files = self.resolve_all(&params.files)?;
        let dest = self.roots.resolve(&params.dest)?;
        let name = params
            .birthmark_type
            .unwrap_or_else(|| "op-seq".to_string());
        let birthmark_type = BirthmarkType::try_from(name.as_str()).map_err(refuse)?;
        let skip = params.skip.unwrap_or(false);
        self.confine(analysis::extracted_files(&files, &dest).map_err(super::error::to_mcp)?)?;

        let birthmarks = self
            .compute(move || analysis::extract(&files, &birthmark_type, &dest, skip))
            .await?
            .map_err(super::error::to_mcp)?;

        Ok(Json(ExtractedAll {
            birthmark_type: name,
            birthmarks,
        }))
    }

    #[tool(
        name = "oinkie_run",
        description = "Compare lifted programs and report how similar each pair is, in one \
                       step. Given a dest, the scores and the birthmarks they came from are \
                       written there. This is the usual way to ask whether one program is a \
                       copy of another: a high score suggests it is."
    )]
    async fn oinkie_run(
        &self,
        Parameters(params): Parameters<RunParams>,
    ) -> Result<Json<Compared>, ErrorData> {
        let files = self.resolve_all(&params.files)?;
        let dest = self.dest(params.dest.as_deref())?;
        let analysis_type =
            AnalysisType::try_from(params.analysis.as_deref().unwrap_or("op-set-jaccard"))
                .map_err(refuse)?;
        let strategy = Self::strategy(params.strategy.as_deref())?;
        let aggregator = Self::aggregator(params.aggregator.as_deref())?;
        Self::bound(&strategy, &files, params.max_pairs)?;
        if let Some(d) = &dest {
            self.confine_run(&files, d, &strategy)?;
        }

        let written = dest.clone();
        let comparator = self.engine.comparator(analysis_type.algorithm());
        let scores = self
            .compute(move || {
                analysis::run(
                    &files,
                    &analysis_type,
                    &comparator,
                    &strategy,
                    &aggregator,
                    written.as_deref(),
                )
            })
            .await?
            .map_err(super::error::to_mcp)?;

        Ok(Json(Compared {
            scores,
            dest: dest.map(|d| d.display().to_string()),
        }))
    }

    #[tool(
        name = "oinkie_compare",
        description = "Compare birthmarks that oinkie_extract already wrote, and report how \
                       similar each pair is. Use this to try a different algorithm without \
                       re-reading the programs; oinkie_run is the shorter path from programs \
                       to scores."
    )]
    async fn oinkie_compare(
        &self,
        Parameters(params): Parameters<CompareParams>,
    ) -> Result<Json<Compared>, ErrorData> {
        let files = self.resolve_all(&params.files)?;
        let dest = self.dest(params.dest.as_deref())?;
        let algorithm = by_name(ALGORITHMS, params.algorithm.as_deref().unwrap_or("jaccard"))
            .map_err(refuse)?;
        let strategy = Self::strategy(params.strategy.as_deref())?;
        let aggregator = Self::aggregator(params.aggregator.as_deref())?;
        Self::bound(&strategy, &files, params.max_pairs)?;
        if let Some(d) = &dest {
            self.confine(analysis::score_files(d, strategy.compare_count(&files)))?;
        }

        let written = dest.clone();
        let comparator = self.engine.comparator(&algorithm);
        let scores = self
            .compute(move || {
                analysis::compare(
                    &files,
                    &comparator,
                    &strategy,
                    &aggregator,
                    written.as_deref(),
                )
            })
            .await?
            .map_err(super::error::to_mcp)?;

        Ok(Json(Compared {
            scores,
            dest: dest.map(|d| d.display().to_string()),
        }))
    }

    #[tool(
        name = "oinkie_stats",
        description = "Summarise a set of birthmarks before comparing them: per group of \
                       comparable birthmarks, how many files, how many functions each holds, \
                       how many elements each function's birthmark has, how many are empty, \
                       and optionally the most frequent elements. Use it to describe a \
                       dataset, or to choose min_elements for oinkie_review."
    )]
    async fn oinkie_stats(
        &self,
        Parameters(params): Parameters<StatsParams>,
    ) -> Result<Json<Statistics>, ErrorData> {
        let inputs = self.resolve_all(&params.paths)?;
        let recursive = params.recursive.unwrap_or(false);
        let found =
            tokio::task::spawn_blocking(move || crate::stats::collect_paths(&inputs, recursive))
                .await
                .map_err(joined)?
                .map_err(super::error::to_mcp)?;
        // The directories were confined, but not what is in them: a file
        // found there can be a symlink out of every root.
        self.confine(found.iter().cloned())?;

        let top = params.top;
        let report = self
            .compute(move || crate::stats::compute(&found, top))
            .await?;
        Ok(Json(Statistics {
            groups: report.groups,
            files: params.per_file.unwrap_or(false).then_some(report.files),
            skipped: report.skipped,
        }))
    }

    #[tool(
        name = "oinkie_review",
        description = "Recompute the similarity of every pair in a directory of pair CSVs \
                       similarity CSVs, using a different aggregator, without comparing \
                       anything again. Use this to ask what the same comparison would have \
                       scored under 'topn' or 'containment' rather than 'hungarian', or without the functions \
                       too short to be evidence (min_elements)."
    )]
    async fn oinkie_review(
        &self,
        Parameters(params): Parameters<ReviewParams>,
    ) -> Result<Json<Reviewed>, ErrorData> {
        let score_dir = self.roots.resolve(&params.score_directory)?;
        let dest = params
            .dest_file
            .as_deref()
            .map(|d| self.roots.resolve(d))
            .transpose()?;

        // Parsed here, and refused here, rather than through the shared
        // conversion. `Aggregator::from_str` fails with `Error::Parse`, which
        // is a catch-all that spans both sides and so is not reported as the
        // caller's fault -- but this one is theirs, and they can fix it.
        let name = params.aggregator.unwrap_or_else(|| "hungarian".to_string());
        let aggregator = Aggregator::from_str(&name).map_err(|e| {
            ErrorData::invalid_params(
                format!("{e}. Call oinkie_info for the aggregators this accepts."),
                None,
            )
        })?;

        let min_elements = params
            .min_elements
            .as_deref()
            .map(crate::cli::MinElements::from_str)
            .transpose()
            .map_err(|e| ErrorData::invalid_params(format!("min_elements: {e}"), None))?;

        let written = dest.clone();
        let roots = self.roots.clone();
        let scores = self
            .compute(move || {
                let start = std::time::Instant::now();
                // The birthmarks are named by the score CSVs, not by the caller,
                // and are read only if they are under a root like everything else.
                let confine = |p: &Path| -> oinkie::Result<()> {
                    let refused = |message: String| {
                        oinkie::Error::Io(
                            p.to_path_buf(),
                            std::io::Error::new(std::io::ErrorKind::PermissionDenied, message),
                        )
                    };
                    let text = p
                        .to_str()
                        .ok_or_else(|| refused("not a path the server can confine".to_string()))?;
                    roots
                        .resolve(text)
                        .map(|_| ())
                        .map_err(|e| refused(e.message.to_string()))
                };
                let review = crate::review::review_all(
                    &score_dir,
                    &aggregator,
                    min_elements.as_ref(),
                    &confine,
                    // Its stderr is not the user's to draw on.
                    &crate::progress::Bars::new(false),
                )?;
                let scores = review
                    .results
                    .iter()
                    .map(|r| Score {
                        index: r.index,
                        left: r.path1.display().to_string(),
                        right: r.path2.display().to_string(),
                        similarity: r.similarity,
                        duration_ms: r.duration.as_millis() as u64,
                    })
                    .collect::<Vec<_>>();
                let applied =
                    review
                        .min_elements
                        .as_ref()
                        .map(|(given, threshold)| MinElementsApplied {
                            given: given.to_string(),
                            threshold: *threshold,
                        });
                if let Some(d) = written {
                    crate::review::store(review, &d, start)?;
                }
                Ok::<_, oinkie::Error>((scores, applied))
            })
            .await?
            .map_err(super::error::to_mcp)?;

        let (scores, min_elements) = scores;
        Ok(Json(Reviewed {
            aggregator: name,
            scores,
            dest_file: dest.map(|d| d.display().to_string()),
            min_elements,
        }))
    }
}

// `router = self.tool_router` rather than the default. The default expands to
// `Self::tool_router()`, which builds the router afresh on every request and
// leaves the stored one unread.
#[tool_handler(router = self.tool_router)]
impl ServerHandler for Oinkie {
    fn get_info(&self) -> InitializeResult {
        // Built by mutating a default rather than with a struct expression:
        // `InitializeResult` is `#[non_exhaustive]`, so a literal will not
        // compile outside rmcp -- which is the point of the attribute, since a
        // field added upstream would otherwise break this build.
        let mut info = InitializeResult::default();
        info.capabilities = ServerCapabilities::builder().enable_tools().build();
        info.server_info = Implementation::from_build_env();
        info.instructions = Some(
            "oinkie detects software theft by comparing software birthmarks -- \
                 characteristics extracted from a program's lifted intermediate \
                 representation. A high similarity between two birthmarks suggests one \
                 program is a copy of the other.\n\n\
                 Inputs are files already lifted to the Oinkie IR, the JSON that \
                 `oinkie lift` writes. Lifting is deliberately not exposed here: it runs a \
                 whole decompiler process per binary, for as long as that binary takes, and \
                 a replacement lifting script is arbitrary code. Run `oinkie lift` yourself \
                 first -- on a host with Ghidra, or in the `ghidra` image.\n\n\
                 Call oinkie_info before naming a birthmark type, an algorithm or an \
                 analysis. The names are precise and the lists are generated from the parser."
                .to_string(),
        );
        info
    }
}
