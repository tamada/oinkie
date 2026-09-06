//! The MCP server itself.
//!
//! Nothing here goes through the `perform_*` drivers in `cli/main.rs`. Over
//! stdio, **stdout is the JSON-RPC channel**, and those drivers `println!`
//! their progress; one such line corrupts the session. The tools call the
//! library directly for the same reason no progress bar is constructed.

use std::path::PathBuf;
use std::str::FromStr;

use clap::ValueEnum;
use oinkie::prelude::{Aggregator, Algorithm, AnalysisType, BirthmarkType, PairingStrategy};
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
    tool_router: rmcp::handler::server::tool::ToolRouter<Self>,
}

impl Oinkie {
    pub fn new(roots: Roots) -> Self {
        Self {
            roots,
            tool_router: Self::tool_router(),
        }
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

    fn strategy(name: Option<&str>) -> Result<PairingStrategy, ErrorData> {
        PairingStrategy::from_str(name.unwrap_or("all-and-self"), true).map_err(refuse)
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
    /// and for handing the directory to oinkie_reaggregate afterwards.
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
    pub birthmarks: Vec<Extracted>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ReaggregateParams {
    /// Directory holding the element-wise similarity CSVs that `compare` or
    /// `run` wrote -- the one they were given as their destination.
    pub score_directory: String,
    /// How to combine element-wise scores into one score per pair:
    /// "hungarian" (the default), "topn:all", or "topn:" and a count.
    /// Call oinkie_info for what these mean.
    #[serde(default)]
    pub aggregator: Option<String>,
    /// Optional. Also write the recomputed scores to this CSV, as
    /// `oinkie reaggregate` does. The scores come back either way.
    #[serde(default)]
    pub dest_file: Option<String>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Reaggregated {
    pub aggregator: String,
    pub scores: Vec<Score>,
    /// Where the CSV was written, when one was asked for.
    pub dest_file: Option<String>,
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

        let birthmarks = tokio::task::spawn_blocking(move || {
            analysis::extract(&files, &birthmark_type, &dest, skip)
        })
        .await
        .map_err(joined)?
        .map_err(super::error::to_mcp)?;

        Ok(Json(ExtractedAll {
            birthmark_type: name,
            birthmarks,
        }))
    }

    #[tool(
        name = "oinkie_run",
        description = "Compare lifted programs and report how similar each pair is, in one \
                       step and without writing birthmarks. This is the usual way to ask \
                       whether one program is a copy of another: a high score suggests it is."
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

        let written = dest.clone();
        let scores = tokio::task::spawn_blocking(move || {
            analysis::run(
                &files,
                &analysis_type,
                &strategy,
                &aggregator,
                written.as_deref(),
            )
        })
        .await
        .map_err(joined)?
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
        let algorithm = Algorithm::from_str(params.algorithm.as_deref().unwrap_or("jaccard"), true)
            .map_err(refuse)?;
        let strategy = Self::strategy(params.strategy.as_deref())?;
        let aggregator = Self::aggregator(params.aggregator.as_deref())?;
        Self::bound(&strategy, &files, params.max_pairs)?;

        let written = dest.clone();
        let scores = tokio::task::spawn_blocking(move || {
            analysis::compare(
                &files,
                &algorithm.comparator(),
                &strategy,
                &aggregator,
                written.as_deref(),
            )
        })
        .await
        .map_err(joined)?
        .map_err(super::error::to_mcp)?;

        Ok(Json(Compared {
            scores,
            dest: dest.map(|d| d.display().to_string()),
        }))
    }

    #[tool(
        name = "oinkie_reaggregate",
        description = "Recompute the score for every pair in a directory of element-wise \
                       similarity CSVs, using a different aggregator, without comparing \
                       anything again. Use this to ask what the same comparison would have \
                       scored under 'topn' rather than 'hungarian'."
    )]
    async fn oinkie_reaggregate(
        &self,
        Parameters(params): Parameters<ReaggregateParams>,
    ) -> Result<Json<Reaggregated>, ErrorData> {
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

        // Off the async runtime: this reads every CSV in the directory and
        // runs an assignment problem per pair, so it is exactly the kind of
        // work that would otherwise stall everything else the server has to
        // answer, cancellation included.
        let written = dest.clone();
        let scores = tokio::task::spawn_blocking(move || {
            let start = std::time::Instant::now();
            let results = crate::reaggregator::reaggregate_all(&score_dir, &aggregator)?;
            let scores = results
                .iter()
                .map(|r| Score {
                    index: r.index,
                    left: r.path1.display().to_string(),
                    right: r.path2.display().to_string(),
                    similarity: r.similarity,
                    duration_ms: r.duration.as_millis() as u64,
                })
                .collect::<Vec<_>>();
            if let Some(d) = written {
                crate::store_and_get_durations(results, &d, start)?;
            }
            Ok::<_, oinkie::Error>(scores)
        })
        .await
        .map_err(|e| {
            ErrorData::internal_error(format!("the reaggregation did not finish: {e}"), None)
        })?
        .map_err(super::error::to_mcp)?;

        Ok(Json(Reaggregated {
            aggregator: name,
            scores,
            dest_file: dest.map(|d| d.display().to_string()),
        }))
    }
}

// `router = self.tool_router` rather than the default. The default expands to
// `Self::tool_router()`, which builds the router afresh on every request and
// leaves the stored one unread -- the dead-code warning was telling the truth.
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
                 first -- on a host with Ghidra, or in the `full` image.\n\n\
                 Call oinkie_info before naming a birthmark type, an algorithm or an \
                 analysis. The names are precise and the lists are generated from the parser."
                .to_string(),
        );
        info
    }
}
