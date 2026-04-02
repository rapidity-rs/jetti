#![doc = include_str!("../README.md")]

mod cli;
mod cmd;
mod config;
mod error;
mod repo;
mod tree;

use clap::{CommandFactory, Parser};
use clap_complete::generate;
use owo_colors::OwoColorize;

use crate::cli::{Cli, Command};
use crate::config::Config;

fn main() {
    let cli = Cli::parse();
    let mut config = Config::load();

    if let Some(root) = cli.root {
        config.root = root;
    }

    let result = match cli.command {
        Command::Clone {
            repo,
            depth,
            shallow,
            https,
            ssh,
        } => {
            let depth = if shallow { Some(1) } else { depth };
            // Protocol precedence: inferred from URL > --https/--ssh CLI flag > config default.
            // The inferred-from-URL override happens in cmd::get::run via repo.inferred_protocol.
            let protocol = if https {
                config::Protocol::Https
            } else if ssh {
                config::Protocol::Ssh
            } else {
                config.protocol
            };
            cmd::get::run(&repo, depth, protocol, &config)
        }
        Command::List { full_path, prefix } => {
            cmd::list::run(full_path, prefix.as_deref(), &config)
        }
        Command::Root => {
            println!("{}", config.root.display());
            Ok(())
        }
        Command::Rm { repo, force } => cmd::rm::run(&repo, force, &config),
        Command::Completions { shell } => {
            generate(shell, &mut Cli::command(), "jetti", &mut std::io::stdout());
            Ok(())
        }
        Command::Fetch { prefix, jobs } => {
            cmd::sync::run(cmd::batch::BatchOp::Fetch, prefix.as_deref(), jobs, &config)
        }
        Command::Update { prefix, jobs } => cmd::sync::run(
            cmd::batch::BatchOp::Update,
            prefix.as_deref(),
            jobs,
            &config,
        ),
        Command::Status { prefix, jobs } => cmd::sync::run(
            cmd::batch::BatchOp::Status,
            prefix.as_deref(),
            jobs,
            &config,
        ),
        Command::Config { action } => cmd::config::run(action, &config),
    };

    if let Err(e) = result {
        eprintln!("{} {e}", "error:".red().bold());
        std::process::exit(1);
    }
}
