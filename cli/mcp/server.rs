//! The MCP server itself.
//!
//! Nothing here goes through the `perform_*` drivers in `cli/main.rs`. Over
//! stdio, **stdout is the JSON-RPC channel**, and those drivers `println!`
//! their progress; one such line corrupts the session. The tools call the
//! library directly for the same reason no progress bar is constructed.

use std::str::FromStr;

use oinkie::prelude::Aggregator;
use rmcp::handler::server::wrapper::{Json, Parameters};
use rmcp::model::{Implementation, InitializeResult, ServerCapabilities};
use rmcp::{ErrorData, ServerHandler, schemars, tool, tool_handler, tool_router};
use serde::{Deserialize, Serialize};

use super::info::{self, Vocabulary};
use super::paths::Roots;

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
pub struct Score {
    /// The pair's number, as it appears in the score directory's file names.
    pub index: usize,
    pub left: String,
    pub right: String,
    /// Between 0 and 1. 1.0 means the two birthmarks matched exactly, which
    /// for unrelated programs is a reason to look at the inputs rather than a
    /// conclusion about them.
    pub similarity: f64,
    pub duration_ms: u64,
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
                 whole decompiler process per binary, for minutes at a time, and a \
                 replacement lifting script is arbitrary code. Run `oinkie lift` yourself \
                 first.\n\n\
                 Call oinkie_info before naming a birthmark type, an algorithm or an \
                 analysis. The names are precise and the lists are generated from the parser."
                .to_string(),
        );
        info
    }
}
