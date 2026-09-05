//! The `oinkie mcp` subcommand: oinkie's birthmark pipeline over the Model
//! Context Protocol, on stdio.
//!
//! Behind the `mcp` cargo feature, the way `gencomp` is, so that a build that
//! does not want a server does not carry tokio and rmcp.

mod error;
mod info;
mod server;

use std::time::{Duration, Instant};

use oinkie::prelude::{Error, Result};
use rmcp::ServiceExt;

pub(crate) fn perform(_opts: &crate::cli::McpOpts) -> Result<Vec<Duration>> {
    let start = Instant::now();
    // A runtime built here rather than a `#[tokio::main]` on `main`, so that
    // every other subcommand stays synchronous and pays nothing for this one.
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| Error::Parse(format!("could not start the async runtime: {e}")))?;
    runtime.block_on(serve())?;
    Ok(vec![start.elapsed()])
}

async fn serve() -> Result<()> {
    let service = server::Oinkie::new()
        .serve(rmcp::transport::stdio())
        .await
        .map_err(|e| Error::Parse(format!("could not start the MCP server: {e}")))?;
    service
        .waiting()
        .await
        .map_err(|e| Error::Parse(format!("the MCP server stopped unexpectedly: {e}")))?;
    Ok(())
}
