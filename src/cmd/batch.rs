//! Parallel batch operations (fetch, update, status) with tree-structured progress display.
//!
//! Uses [`rayon`] for parallelism and [`indicatif`] for progress spinners.

use std::path::{Path, PathBuf};
use std::process;
use std::sync::Mutex;

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use rayon::prelude::*;

use crate::tree::{self, TreeLine};

/// The outcome of a batch operation on a single repository.
#[derive(Debug, Clone)]
pub enum OpResult {
    /// The operation completed and something changed (e.g. new commits fetched).
    Success(String),
    /// The operation completed but nothing changed (e.g. already up to date).
    Skipped(String),
    /// The operation completed but the repo needs manual attention.
    Warning(String),
    /// The operation failed.
    Failed(String),
}

/// A batch operation to perform across repositories.
#[derive(Debug, Clone, Copy)]
pub enum BatchOp {
    /// Run `git fetch` on each repository.
    Fetch,
    /// Run `git fetch` followed by `git merge --ff-only`.
    Update,
    /// Report branch, dirty file count, and ahead/behind status.
    Status,
}

/// Run a batch operation across repos with a tree-structured progress display.
pub fn run(
    op: BatchOp,
    repos: &[String],
    root: &Path,
    jobs: usize,
) -> crate::error::Result<Vec<(String, OpResult)>> {
    if repos.is_empty() {
        eprintln!("{} no repositories found", "note:".dimmed());
        return Ok(Vec::new());
    }

    let (tree_lines, repo_nodes) = tree::build_tree(repos);

    let mp = MultiProgress::new();

    // Build the display: static lines for hosts/owners, spinners for repos
    let spinner_style = ProgressStyle::with_template("{prefix}{spinner:.cyan} {msg}")
        .unwrap()
        .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ ");

    let done_style = ProgressStyle::with_template("{prefix}{msg}").unwrap();

    // Map from rel_path -> ProgressBar
    let mut bars: Vec<(String, ProgressBar)> = Vec::new();

    for line in &tree_lines {
        match line {
            TreeLine::Host(host) => {
                let pb = mp.add(ProgressBar::new(0));
                pb.set_style(done_style.clone());
                pb.finish_with_message(format!("{}", host.magenta().bold()));
            }
            TreeLine::Owner { name, prefix } => {
                let pb = mp.add(ProgressBar::new(0));
                pb.set_style(done_style.clone());
                pb.finish_with_message(format!("{}{}", prefix.dimmed(), name.cyan()));
            }
            TreeLine::Repo(node) => {
                let pb = mp.add(ProgressBar::new(0));
                pb.set_style(spinner_style.clone());
                pb.set_prefix(format!("{}", node.prefix.dimmed()));
                pb.set_message(format!("{} {}", node.name, "waiting...".dimmed()));
                pb.enable_steady_tick(std::time::Duration::from_millis(80));
                bars.push((node.rel_path.clone(), pb));
            }
            TreeLine::Blank => {
                let pb = mp.add(ProgressBar::new(0));
                pb.set_style(done_style.clone());
                pb.finish_with_message("");
            }
        }
    }

    // Build work items: (index, rel_path, abs_path, prefix for done state)
    let work: Vec<(usize, String, PathBuf, String)> = repo_nodes
        .iter()
        .enumerate()
        .map(|(i, node)| {
            (
                i,
                node.rel_path.clone(),
                root.join(&node.rel_path),
                node.prefix.clone(),
            )
        })
        .collect();

    let results: Mutex<Vec<(usize, String, OpResult)>> = Mutex::new(Vec::new());

    // Configure thread pool
    let pool = rayon::ThreadPoolBuilder::new().num_threads(jobs).build()?;

    pool.install(|| {
        work.par_iter()
            .for_each(|(idx, rel_path, abs_path, _prefix)| {
                // Update spinner to show we're working
                if let Some((_, pb)) = bars.get(*idx) {
                    let name = &repo_nodes[*idx].name;
                    let label = match op {
                        BatchOp::Fetch => "fetching",
                        BatchOp::Update => "updating",
                        BatchOp::Status => "checking",
                    };
                    pb.set_message(format!("{} {}", name, label.dimmed()));
                }

                let result = execute_op(op, abs_path);

                // Update the progress bar with the result
                if let Some((_, pb)) = bars.get(*idx) {
                    let name = &repo_nodes[*idx].name;
                    pb.set_style(done_style.clone());
                    let msg = match &result {
                        OpResult::Success(detail) => {
                            format!("{} {}", name.bold(), detail.green())
                        }
                        OpResult::Skipped(detail) => {
                            format!("{} {}", name, detail.dimmed())
                        }
                        OpResult::Warning(detail) => {
                            format!("{} {}", name, detail.yellow())
                        }
                        OpResult::Failed(detail) => {
                            format!("{} {}", name, detail.red())
                        }
                    };
                    let prefix = &repo_nodes[*idx].prefix;
                    let icon = match &result {
                        OpResult::Success(_) => format!("{}", "✓ ".green()),
                        OpResult::Skipped(_) => format!("{}", "· ".dimmed()),
                        OpResult::Warning(_) => format!("{}", "! ".yellow()),
                        OpResult::Failed(_) => format!("{}", "✗ ".red()),
                    };
                    pb.set_prefix(format!("{}{}", prefix.dimmed(), icon));
                    pb.finish_with_message(msg);
                }

                results
                    .lock()
                    .expect("result collector lock poisoned")
                    .push((*idx, rel_path.clone(), result));
            });
    });

    let mut final_results: Vec<(String, OpResult)> = results
        .into_inner()
        .expect("result collector lock poisoned")
        .into_iter()
        .map(|(_, path, result)| (path, result))
        .collect();

    final_results.sort_by(|a, b| a.0.cmp(&b.0));

    // Print summary
    let mut success = 0;
    let mut skipped = 0;
    let mut warnings = 0;
    let mut failed = 0;
    for (_, result) in &final_results {
        match result {
            OpResult::Success(_) => success += 1,
            OpResult::Skipped(_) => skipped += 1,
            OpResult::Warning(_) => warnings += 1,
            OpResult::Failed(_) => failed += 1,
        }
    }

    eprintln!();
    let total = final_results.len();
    let mut parts = Vec::new();

    // Each OpResult category means something different depending on the
    // operation, so we pick labels per-op rather than reusing generic names.
    match op {
        BatchOp::Status => {
            if success > 0 {
                parts.push(format!("{}", format!("{success} behind upstream").green()));
            }
            if skipped > 0 {
                parts.push(format!("{}", format!("{skipped} clean").dimmed()));
            }
            if warnings > 0 {
                parts.push(format!("{}", format!("{warnings} need attention").yellow()));
            }
        }
        BatchOp::Fetch => {
            if success > 0 {
                parts.push(format!("{}", format!("{success} fetched").green()));
            }
            if skipped > 0 {
                parts.push(format!("{}", format!("{skipped} up to date").dimmed()));
            }
            if warnings > 0 {
                parts.push(format!("{}", format!("{warnings} need attention").yellow()));
            }
        }
        BatchOp::Update => {
            if success > 0 {
                parts.push(format!("{}", format!("{success} updated").green()));
            }
            if skipped > 0 {
                parts.push(format!("{}", format!("{skipped} up to date").dimmed()));
            }
            if warnings > 0 {
                parts.push(format!("{}", format!("{warnings} need attention").yellow()));
            }
        }
    }
    if failed > 0 {
        parts.push(format!("{}", format!("{failed} failed").red()));
    }

    eprintln!(
        "  {} {} repos: {}",
        "done:".green().bold(),
        total,
        parts.join(", ")
    );

    Ok(final_results)
}

fn execute_op(op: BatchOp, path: &Path) -> OpResult {
    match op {
        BatchOp::Status => execute_status(path),
        BatchOp::Fetch => execute_fetch(path),
        BatchOp::Update => execute_update(path),
    }
}

fn execute_status(path: &Path) -> OpResult {
    let branch = current_branch(path).unwrap_or_else(|| "HEAD".into());
    let dirty_count = dirty_file_count(path);
    let (ahead, behind) = ahead_behind(path);

    let mut parts = Vec::new();
    parts.push(branch);

    if dirty_count > 0 {
        parts.push(format!("{dirty_count} modified"));
    }
    if ahead > 0 {
        parts.push(format!("↑{ahead}"));
    }
    if behind > 0 {
        parts.push(format!("↓{behind}"));
    }

    let summary = parts.join(" · ");

    if dirty_count > 0 || ahead > 0 {
        OpResult::Warning(summary)
    } else if behind > 0 {
        OpResult::Success(summary)
    } else {
        OpResult::Skipped(summary)
    }
}

fn execute_fetch(path: &Path) -> OpResult {
    // Compare the upstream tracking ref before/after fetch, since git fetch
    // updates remote-tracking refs, not HEAD.
    let before = upstream_rev(path).or_else(|| fetch_head_rev(path));

    let fetch = process::Command::new("git")
        .arg("fetch")
        .current_dir(path)
        .stdout(process::Stdio::null())
        .stderr(process::Stdio::null())
        .status();

    match fetch {
        Ok(s) if !s.success() => return OpResult::Failed("fetch failed".into()),
        Err(e) => return OpResult::Failed(format!("git error: {e}")),
        _ => {}
    }

    let after = upstream_rev(path).or_else(|| fetch_head_rev(path));
    if before == after {
        OpResult::Skipped("up to date".into())
    } else {
        OpResult::Success("fetched".into())
    }
}

fn execute_update(path: &Path) -> OpResult {
    if is_dirty(path) {
        return OpResult::Warning("dirty working tree, skipped".into());
    }

    let before = head_rev(path);

    let fetch = process::Command::new("git")
        .arg("fetch")
        .current_dir(path)
        .stdout(process::Stdio::null())
        .stderr(process::Stdio::null())
        .status();

    match fetch {
        Ok(s) if !s.success() => return OpResult::Failed("fetch failed".into()),
        Err(e) => return OpResult::Failed(format!("git error: {e}")),
        _ => {}
    }

    // Check upstream exists
    let upstream = process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"])
        .current_dir(path)
        .stdout(process::Stdio::piped())
        .stderr(process::Stdio::null())
        .output();

    match upstream {
        Ok(out) if !out.status.success() => {
            return OpResult::Skipped("no upstream".into());
        }
        Err(_) => return OpResult::Failed("failed to check upstream".into()),
        _ => {}
    }

    let merge = process::Command::new("git")
        .args(["merge", "--ff-only"])
        .current_dir(path)
        .stdout(process::Stdio::null())
        .stderr(process::Stdio::null())
        .status();

    match merge {
        Ok(s) if s.success() => {
            let after = head_rev(path);
            if before == after {
                OpResult::Skipped("up to date".into())
            } else {
                OpResult::Success("fast-forwarded".into())
            }
        }
        Ok(_) => OpResult::Warning("diverged, merge manually".into()),
        Err(e) => OpResult::Failed(format!("merge failed: {e}")),
    }
}

fn is_dirty(path: &Path) -> bool {
    process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(path)
        .stdout(process::Stdio::piped())
        .stderr(process::Stdio::null())
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false)
}

fn head_rev(path: &Path) -> Option<String> {
    process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(path)
        .stdout(process::Stdio::piped())
        .stderr(process::Stdio::null())
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}

fn upstream_rev(path: &Path) -> Option<String> {
    process::Command::new("git")
        .args(["rev-parse", "@{u}"])
        .current_dir(path)
        .stdout(process::Stdio::piped())
        .stderr(process::Stdio::null())
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}

fn fetch_head_rev(path: &Path) -> Option<String> {
    process::Command::new("git")
        .args(["rev-parse", "FETCH_HEAD"])
        .current_dir(path)
        .stdout(process::Stdio::piped())
        .stderr(process::Stdio::null())
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}

fn current_branch(path: &Path) -> Option<String> {
    process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(path)
        .stdout(process::Stdio::piped())
        .stderr(process::Stdio::null())
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}

fn dirty_file_count(path: &Path) -> usize {
    process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(path)
        .stdout(process::Stdio::piped())
        .stderr(process::Stdio::null())
        .output()
        .ok()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .filter(|l| !l.is_empty())
                .count()
        })
        .unwrap_or(0)
}

fn ahead_behind(path: &Path) -> (usize, usize) {
    let output = process::Command::new("git")
        .args(["rev-list", "--left-right", "--count", "HEAD...@{u}"])
        .current_dir(path)
        .stdout(process::Stdio::piped())
        .stderr(process::Stdio::null())
        .output();

    match output {
        Ok(o) if o.status.success() => {
            let text = String::from_utf8_lossy(&o.stdout);
            let parts: Vec<&str> = text.trim().split('\t').collect();
            if parts.len() == 2 {
                let ahead = parts[0].parse().unwrap_or(0);
                let behind = parts[1].parse().unwrap_or(0);
                (ahead, behind)
            } else {
                (0, 0)
            }
        }
        // No upstream configured
        _ => (0, 0),
    }
}
