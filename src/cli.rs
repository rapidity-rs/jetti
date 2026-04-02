//! Command-line argument parsing via [`clap`].

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use clap_complete::Shell;

#[derive(Debug, Parser)]
#[command(
    name = "jetti",
    version,
    about = "Organize your git repositories",
    long_about = "A fast, cross-platform tool for organizing git repositories",
    styles = styles(),
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Override the root directory
    #[arg(long, global = true)]
    pub root: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Clone a repository into the organized directory structure
    #[command(aliases = ["get", "g"])]
    Clone {
        /// Repository specifier (owner/repo, host/owner/repo, or full URL)
        repo: String,

        /// Create a shallow clone with the given depth
        #[arg(long, short = 'd')]
        depth: Option<u32>,

        /// Shorthand for --depth 1
        #[arg(long, short = 's', conflicts_with = "depth")]
        shallow: bool,

        /// Clone using HTTPS instead of SSH
        #[arg(long, conflicts_with = "ssh")]
        https: bool,

        /// Clone using SSH instead of HTTPS
        #[arg(long, conflicts_with = "https")]
        ssh: bool,
    },

    /// List cloned repositories
    #[command(visible_alias = "ls")]
    List {
        /// Show absolute paths instead of relative
        #[arg(long, short = 'f')]
        full_path: bool,

        /// Filter repositories by prefix
        #[arg(long, short = 'p')]
        prefix: Option<String>,
    },

    /// Print the root directory path
    Root,

    /// Remove a cloned repository
    #[command(visible_alias = "remove")]
    Rm {
        /// Repository specifier to remove
        repo: String,

        /// Skip confirmation prompt
        #[arg(long)]
        force: bool,
    },

    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        shell: Shell,
    },

    /// Fetch updates for all repositories
    Fetch {
        /// Filter repositories by prefix
        #[arg(long, short = 'p')]
        prefix: Option<String>,

        /// Number of parallel jobs (default: 8)
        #[arg(long, short = 'j', default_value = "8")]
        jobs: usize,
    },

    /// Fetch and fast-forward all repositories
    Update {
        /// Filter repositories by prefix
        #[arg(long, short = 'p')]
        prefix: Option<String>,

        /// Number of parallel jobs (default: 8)
        #[arg(long, short = 'j', default_value = "8")]
        jobs: usize,
    },

    /// Show status of all repositories (branch, dirty files, ahead/behind)
    #[command(visible_alias = "st")]
    Status {
        /// Filter repositories by prefix
        #[arg(long, short = 'p')]
        prefix: Option<String>,

        /// Number of parallel jobs (default: 8)
        #[arg(long, short = 'j', default_value = "8")]
        jobs: usize,
    },

    /// View or manage configuration
    #[command(visible_alias = "cfg")]
    Config {
        #[command(subcommand)]
        action: Option<ConfigAction>,
    },
}

#[derive(Debug, Subcommand)]
pub enum ConfigAction {
    /// Open the config file in $EDITOR
    Edit,

    /// Print the path to the config file
    Path,

    /// Create the default config file if it doesn't exist
    Init,
}

fn styles() -> clap::builder::Styles {
    use clap::builder::styling::*;
    Styles::styled()
        .header(
            Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Magenta))),
        )
        .usage(
            Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Magenta))),
        )
        .literal(
            Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Cyan))),
        )
        .placeholder(Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green))))
        .valid(Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green))))
        .invalid(Style::new().fg_color(Some(Color::Ansi(AnsiColor::Red))))
        .error(
            Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Red))),
        )
}
