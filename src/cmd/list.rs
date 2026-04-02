//! The `list` command — discover and display cloned repositories.

use std::io::{self, IsTerminal};
use std::path::Path;

use owo_colors::OwoColorize;

use crate::config::Config;
use crate::error::JettiError;
use crate::tree::{self, TreeLine};

/// Discover repositories under root, sorted and optionally filtered by prefix.
pub fn discover_repos(prefix: Option<&str>, config: &Config) -> crate::error::Result<Vec<String>> {
    let root = &config.root;

    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut repos = Vec::new();
    find_repos(root, root, &mut repos).map_err(|e| JettiError::Io {
        action: "walk",
        path: root.to_path_buf(),
        source: e,
    })?;

    repos.sort();

    if let Some(prefix) = prefix {
        repos.retain(|r| r.starts_with(prefix));
    }

    Ok(repos)
}

/// List repositories, using a tree view in terminals or flat output when piped.
pub fn run(full_path: bool, prefix: Option<&str>, config: &Config) -> crate::error::Result<()> {
    let repos = discover_repos(prefix, config)?;
    let root = &config.root;

    // Use flat output when using --full-path or when piping (auto-detected)
    if full_path || !io::stdout().is_terminal() {
        for rel_path in &repos {
            if full_path {
                println!("{}", root.join(rel_path).display());
            } else {
                println!("{rel_path}");
            }
        }
    } else {
        print_tree(&repos);
    }

    Ok(())
}

fn print_tree(repos: &[String]) {
    let (lines, _) = tree::build_tree(repos);

    for line in &lines {
        match line {
            TreeLine::Host(host) => {
                println!("{}", host.magenta().bold());
            }
            TreeLine::Owner { name, prefix } => {
                println!("{}{}", prefix.dimmed(), name.cyan());
            }
            TreeLine::Repo(node) => {
                println!("{}{}", node.prefix.dimmed(), node.name);
            }
            TreeLine::Blank => {
                println!();
            }
        }
    }
}

/// Recursively find directories containing `.git`, returning paths relative to root.
pub fn find_repos(dir: &Path, root: &Path, out: &mut Vec<String>) -> std::io::Result<()> {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => {
            eprintln!(
                "{} cannot read {}: {e}",
                "warning:".yellow().bold(),
                dir.display()
            );
            return Ok(());
        }
    };

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        // Skip hidden directories (other than .git check below)
        if let Some(name) = path.file_name().and_then(|n| n.to_str())
            && name.starts_with('.')
        {
            continue;
        }

        if path.join(".git").exists() {
            if let Ok(rel) = path.strip_prefix(root) {
                out.push(rel.to_string_lossy().to_string());
            }
            // Don't recurse into repos (no nested repo support)
            continue;
        }

        find_repos(&path, root, out)?;
    }

    Ok(())
}
