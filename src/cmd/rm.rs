//! The `rm` command — remove a cloned repository.

use std::io::{self, Write};

use owo_colors::OwoColorize;

use crate::config::Config;
use crate::error::JettiError;
use crate::repo::Repo;

/// Remove a repository directory and clean up empty parent directories.
///
/// Prompts for confirmation unless `force` is true.
pub fn run(repo_spec: &str, force: bool, config: &Config) -> crate::error::Result<()> {
    let repo = Repo::parse(repo_spec, config)?;
    let dest = repo.local_path(config);

    if !dest.exists() {
        return Err(JettiError::Precondition(format!(
            "repository {} not found at {}",
            format!("{}/{}", repo.owner, repo.name).bold(),
            dest.display()
        )));
    }

    if !dest.join(".git").exists() {
        return Err(JettiError::Precondition(format!(
            "refusing to remove non-git directory at {}",
            dest.display()
        )));
    }

    if !force {
        eprint!(
            "{} remove {}? [y/N] ",
            "confirm:".yellow().bold(),
            dest.display().to_string().bold()
        );
        io::stderr().flush().unwrap();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|e| JettiError::Io {
                action: "read input from",
                path: "stdin".into(),
                source: e,
            })?;

        if !matches!(input.trim(), "y" | "Y" | "yes" | "Yes") {
            eprintln!("{} aborted", "rm:".cyan().bold());
            return Ok(());
        }
    }

    std::fs::remove_dir_all(&dest).map_err(|e| JettiError::Io {
        action: "remove",
        path: dest.clone(),
        source: e,
    })?;

    eprintln!(
        "{} removed {}",
        "rm:".green().bold(),
        dest.display().to_string().bold()
    );

    // Clean up empty parent directories up to the root
    let mut parent = dest.parent();
    while let Some(dir) = parent {
        if dir == config.root {
            break;
        }
        // Try to remove — will fail if not empty, which is fine
        if std::fs::remove_dir(dir).is_err() {
            break;
        }
        parent = dir.parent();
    }

    Ok(())
}
