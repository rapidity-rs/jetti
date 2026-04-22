//! The `clone` (aka `get`) command — clone a repository into the organized directory structure.

use std::process;

use owo_colors::OwoColorize;

use crate::config::{Config, Protocol};
use crate::error::JettiError;
use crate::repo::Repo;

/// Clone a repository into `<root>/<host>/<owner>/<repo>`.
///
/// If the destination already exists and contains a valid git repo, prints the
/// path and returns. If it exists but is not a git repo (e.g. a failed partial
/// clone), the directory is removed and the clone is retried.
pub fn run(
    repo_spec: &str,
    depth: Option<u32>,
    protocol: Protocol,
    config: &Config,
) -> crate::error::Result<()> {
    require_git()?;
    let repo = Repo::parse(repo_spec, config)?;
    let dest = repo.local_path(config);

    if dest.exists() {
        if dest.join(".git").exists() {
            eprintln!(
                "{} {} already exists at {}",
                "exists:".cyan().bold(),
                format!("{}/{}", repo.owner, repo.name).bold(),
                dest.display()
            );
            println!("{}", dest.display());
            return Ok(());
        }

        return Err(JettiError::Precondition(format!(
            "destination already exists and is not a git repository: {}\nremove it manually before cloning",
            dest.display()
        )));
    }

    // Inferred protocol from URL takes priority, then CLI flag, then config
    let effective_protocol = repo.inferred_protocol.unwrap_or(protocol);
    let url = repo.clone_url(config, effective_protocol);

    eprintln!(
        "{} {} into {}",
        "cloning:".green().bold(),
        url.bold(),
        dest.display()
    );

    // Ensure parent directories exist
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|e| JettiError::Io {
            action: "create directory",
            path: parent.to_path_buf(),
            source: e,
        })?;
    }

    let mut cmd = process::Command::new("git");
    cmd.arg("clone");

    if let Some(d) = depth {
        cmd.arg("--depth").arg(d.to_string());
    }

    cmd.arg(&url).arg(&dest);

    let status = cmd
        .status()
        .map_err(|e| JettiError::Subprocess(format!("failed to run git: {e}")))?;

    if !status.success() {
        return Err(JettiError::Subprocess(format!(
            "git clone exited with {status}"
        )));
    }

    eprintln!(
        "{} cloned to {}",
        "done:".green().bold(),
        dest.display().to_string().bold()
    );

    // stdout gets only the path — clean for shell capture: cd $(jetti clone ...)
    println!("{}", dest.display());

    Ok(())
}

fn require_git() -> crate::error::Result<()> {
    let status = process::Command::new("git")
        .arg("--version")
        .stdout(process::Stdio::null())
        .stderr(process::Stdio::null())
        .status()
        .map_err(|_| {
            JettiError::Precondition(String::from(
                "git is not installed or not in PATH — jetti requires git to clone repositories",
            ))
        })?;

    if !status.success() {
        return Err(JettiError::Precondition(String::from(
            "git is not installed or not in PATH — jetti requires git to clone repositories",
        )));
    }

    Ok(())
}
