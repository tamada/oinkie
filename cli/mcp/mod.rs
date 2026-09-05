//! The `oinkie mcp` subcommand: oinkie's birthmark pipeline over the Model
//! Context Protocol, on stdio.
//!
//! Behind the `mcp` cargo feature, the way `gencomp` is, so that a build that
//! does not want a server does not carry tokio and rmcp.

mod error;
mod info;
mod paths;
mod server;

use std::time::{Duration, Instant};

use oinkie::prelude::{Error, Result};
use rmcp::ServiceExt;

/// Starts the server, and returns only when the client goes away.
///
/// The failures here are reported as `Error::Parse`, which `error.rs` does not
/// classify as the caller's fault -- correctly, since no argument would fix a
/// runtime that will not start. They do not become MCP errors in any case:
/// this returns to `main`, which prints and exits, and by definition there is
/// no session yet to report them into.
pub(crate) fn perform(opts: &crate::cli::McpOpts) -> Result<Vec<Duration>> {
    let start = Instant::now();
    // A runtime built here rather than a `#[tokio::main]` on `main`, so that
    // every other subcommand stays synchronous and pays nothing for this one.
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| Error::Parse(format!("could not start the async runtime: {e}")))?;
    // Resolved before the server starts: a root that is wrong is a mistake in
    // how this was launched, and the person who can fix it is watching now
    // rather than when a tool is first called.
    let roots = paths::Roots::new(opts.roots())?;
    runtime.block_on(serve(roots))?;
    Ok(vec![start.elapsed()])
}

async fn serve(roots: paths::Roots) -> Result<()> {
    let service = server::Oinkie::new(roots)
        .serve(rmcp::transport::stdio())
        .await
        .map_err(|e| Error::Parse(format!("could not start the MCP server: {e}")))?;
    service
        .waiting()
        .await
        .map_err(|e| Error::Parse(format!("the MCP server stopped unexpectedly: {e}")))?;
    Ok(())
}
