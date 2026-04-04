//! The `config` command — view, create, and edit configuration.

use std::process;

use owo_colors::OwoColorize;

use crate::cli::ConfigAction;
use crate::config::Config;
use crate::error::JettiError;

/// Run a config subcommand, or show the current config if no subcommand is given.
pub fn run(action: Option<ConfigAction>, config: &Config) -> crate::error::Result<()> {
    match action {
        // No subcommand: show current config
        None => show(config),
        Some(ConfigAction::Edit) => edit(),
        Some(ConfigAction::Path) => {
            println!("{}", Config::path().display());
            Ok(())
        }
        Some(ConfigAction::Init) => init(config),
    }
}

fn show(config: &Config) -> crate::error::Result<()> {
    let path = Config::path();

    println!(
        "{} {}",
        "config:".cyan().bold(),
        path.display().to_string().dimmed()
    );
    println!();
    println!("  {} {}", "root:".bold(), config.root.display());
    println!("  {} {}", "default_host:".bold(), config.default_host);
    println!(
        "  {} {}",
        "protocol:".bold(),
        match config.protocol {
            crate::config::Protocol::Ssh => "ssh",
            crate::config::Protocol::Https => "https",
        }
    );
    println!("  {} {}", "use_jj:".bold(), config.use_jj);
    println!();
    println!("  {}", "hosts:".bold());
    for host in &config.hosts {
        println!("    {} {}", "·".dimmed(), host.name.cyan());
        println!("      {} {}", "ssh:".dimmed(), host.ssh_prefix.dimmed());
        println!("      {} {}", "https:".dimmed(), host.https_prefix.dimmed());
    }

    if !path.exists() {
        println!();
        println!(
            "  {} no config file found, using defaults. Run {} to create one.",
            "note:".yellow().bold(),
            "jetti config init".cyan().bold()
        );
    }

    Ok(())
}

fn edit() -> crate::error::Result<()> {
    let path = Config::path();

    // Create default config if it doesn't exist
    if !path.exists() {
        write_default_config(&path)?;
        println!(
            "{} created default config at {}",
            "init:".green().bold(),
            path.display()
        );
    }

    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .map_err(|_| {
            JettiError::Precondition(String::from(
                "$EDITOR is not set. Set it to your preferred editor (e.g. export EDITOR=vim)",
            ))
        })?;

    let status = process::Command::new(&editor)
        .arg(&path)
        .status()
        .map_err(|e| JettiError::Subprocess(format!("failed to launch {editor}: {e}")))?;

    if !status.success() {
        return Err(JettiError::Subprocess(format!(
            "{editor} exited with {status}"
        )));
    }

    Ok(())
}

fn init(_config: &Config) -> crate::error::Result<()> {
    let path = Config::path();

    if path.exists() {
        println!(
            "{} config already exists at {}",
            "exists:".cyan().bold(),
            path.display()
        );
        return Ok(());
    }

    write_default_config(&path)?;

    println!(
        "{} created config at {}",
        "init:".green().bold(),
        path.display().to_string().bold()
    );
    println!();
    println!(
        "  Run {} to customize it.",
        "jetti config edit".cyan().bold()
    );

    Ok(())
}

fn write_default_config(path: &std::path::Path) -> crate::error::Result<()> {
    let config = Config::default();
    let root = config.root.display();

    // Write as a static template so we can include comments. TOML comments are
    // lost through serde roundtrip, so we build the string manually.
    let toml = format!(
        r#"# jetti configuration
# See: https://github.com/rapidity-rs/jetti

# Root directory where repositories are cloned.
root = "{root}"

# Default host when only owner/repo is given (e.g. `jetti clone owner/repo`).
default_host = "{default_host}"

# Default clone protocol: "ssh" or "https".
protocol = "{protocol}"

# Use Jujutsu (jj git clone) instead of git clone by default.
use_jj = false

# Known hosts and their clone URL prefixes.
# NOTE: This list *replaces* the built-in defaults — it does not extend them.
# Remove or add entries as needed for your setup.
[[hosts]]
name = "github.com"
ssh_prefix = "git@github.com:"
https_prefix = "https://github.com/"

[[hosts]]
name = "gitlab.com"
ssh_prefix = "git@gitlab.com:"
https_prefix = "https://gitlab.com/"

[[hosts]]
name = "codeberg.org"
ssh_prefix = "git@codeberg.org:"
https_prefix = "https://codeberg.org/"
"#,
        default_host = config.default_host,
        protocol = match config.protocol {
            crate::config::Protocol::Ssh => "ssh",
            crate::config::Protocol::Https => "https",
        },
    );

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| JettiError::Io {
            action: "create directory",
            path: parent.to_path_buf(),
            source: e,
        })?;
    }

    std::fs::write(path, toml).map_err(|e| JettiError::Io {
        action: "write",
        path: path.to_path_buf(),
        source: e,
    })?;

    Ok(())
}
