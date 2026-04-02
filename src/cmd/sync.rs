//! Orchestrator for batch operations (fetch, update, status) across all repositories.

use crate::cmd::batch::{self, BatchOp};
use crate::cmd::list::discover_repos;
use crate::config::Config;

/// Discover repositories and run a batch operation across them.
pub fn run(
    op: BatchOp,
    prefix: Option<&str>,
    jobs: usize,
    config: &Config,
) -> crate::error::Result<()> {
    let repos = discover_repos(prefix, config)?;

    batch::run(op, &repos, &config.root, jobs)?;

    Ok(())
}
