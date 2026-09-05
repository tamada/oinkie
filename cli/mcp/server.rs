//! The MCP server itself.
//!
//! Nothing here goes through the `perform_*` drivers in `cli/main.rs`. Over
//! stdio, **stdout is the JSON-RPC channel**, and those drivers `println!`
//! their progress; one such line corrupts the session. The tools call the
//! library directly for the same reason no progress bar is constructed.

use rmcp::handler::server::wrapper::Json;
use rmcp::model::{Implementation, InitializeResult, ServerCapabilities};
use rmcp::{ServerHandler, tool, tool_handler, tool_router};

use super::info::{self, Vocabulary};

#[derive(Clone)]
pub struct Oinkie {
    tool_router: rmcp::handler::server::tool::ToolRouter<Self>,
}

impl Oinkie {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }
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
