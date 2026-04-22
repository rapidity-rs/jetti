//! Configuration loading and management.
//!
//! Jetti reads its configuration from `~/.config/jetti/config.toml` (or
//! `$XDG_CONFIG_HOME/jetti/config.toml`). If the file is missing or
//! unparseable, sensible defaults are used.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// The protocol used to clone repositories.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    #[default]
    Ssh,
    Https,
}

/// Top-level configuration for jetti.
///
/// All fields have defaults so an empty (or missing) config file is valid.
#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Root directory where repositories are cloned
    pub root: PathBuf,
    /// Default host when only owner/repo is given
    pub default_host: String,
    /// Default clone protocol (ssh or https)
    pub protocol: Protocol,
    /// Known hosts and their clone URL templates
    pub hosts: Vec<HostConfig>,
}

/// A known git host with its clone URL prefixes.
///
/// Jetti uses these to construct clone URLs for known hosts. Unknown hosts
/// fall back to conventional `git@host:` / `https://host/` patterns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostConfig {
    /// The hostname (e.g. `github.com`).
    pub name: String,
    /// SSH clone URL prefix (e.g. `git@github.com:`).
    pub ssh_prefix: String,
    /// HTTPS clone URL prefix (e.g. `https://github.com/`).
    pub https_prefix: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            root: dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("~"))
                .join("dev"),
            default_host: String::from("github.com"),
            protocol: Protocol::default(),
            hosts: vec![
                HostConfig {
                    name: String::from("github.com"),
                    ssh_prefix: String::from("git@github.com:"),
                    https_prefix: String::from("https://github.com/"),
                },
                HostConfig {
                    name: String::from("gitlab.com"),
                    ssh_prefix: String::from("git@gitlab.com:"),
                    https_prefix: String::from("https://gitlab.com/"),
                },
                HostConfig {
                    name: String::from("codeberg.org"),
                    ssh_prefix: String::from("git@codeberg.org:"),
                    https_prefix: String::from("https://codeberg.org/"),
                },
            ],
        }
    }
}

impl Config {
    /// Load configuration from disk, falling back to defaults.
    ///
    /// Prints a warning to stderr if the file exists but cannot be read or parsed.
    pub fn load() -> Self {
        Self::load_from_path(&Self::path())
    }

    pub fn load_from_path(config_path: &Path) -> Self {
        if config_path.exists() {
            match std::fs::read_to_string(config_path) {
                Ok(contents) => match toml::from_str(&contents) {
                    Ok(config) => return config,
                    Err(e) => {
                        eprintln!("Warning: failed to parse config: {e}");
                    }
                },
                Err(e) => {
                    eprintln!("Warning: failed to read config: {e}");
                }
            }
        }
        Self::default()
    }

    /// Return the path to the config file.
    ///
    /// Uses `$XDG_CONFIG_HOME/jetti/config.toml`, falling back to
    /// `~/.config/jetti/config.toml`.
    pub fn path() -> PathBuf {
        // Prefer $XDG_CONFIG_HOME, fall back to ~/.config (not macOS Library)
        let base = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("~"))
                    .join(".config")
            });
        base.join("jetti").join("config.toml")
    }

    /// Look up a host by name, returning its URL prefixes if found.
    pub fn host(&self, name: &str) -> Option<&HostConfig> {
        self.hosts.iter().find(|h| h.name == name)
    }

    /// Load config from a TOML string.
    #[cfg(test)]
    pub fn from_toml(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let config = Config::default();
        assert!(config.root.ends_with("dev"));
        assert_eq!(config.default_host, "github.com");
        assert_eq!(config.protocol, Protocol::Ssh);
        assert_eq!(config.hosts.len(), 3);
    }

    #[test]
    fn default_protocol_is_ssh() {
        assert_eq!(Protocol::default(), Protocol::Ssh);
    }

    #[test]
    fn host_lookup_found() {
        let config = Config::default();
        let host = config.host("github.com").unwrap();
        assert_eq!(host.name, "github.com");
        assert_eq!(host.ssh_prefix, "git@github.com:");
        assert_eq!(host.https_prefix, "https://github.com/");
    }

    #[test]
    fn host_lookup_not_found() {
        let config = Config::default();
        assert!(config.host("sr.ht").is_none());
    }

    #[test]
    fn host_lookup_all_defaults() {
        let config = Config::default();
        assert!(config.host("github.com").is_some());
        assert!(config.host("gitlab.com").is_some());
        assert!(config.host("codeberg.org").is_some());
    }

    #[test]
    fn from_toml_minimal() {
        let config = Config::from_toml("").unwrap();
        assert!(config.root.ends_with("dev"));
        assert_eq!(config.default_host, "github.com");
    }

    #[test]
    fn from_toml_custom_root() {
        let config = Config::from_toml("root = \"/tmp/repos\"").unwrap();
        assert_eq!(config.root, PathBuf::from("/tmp/repos"));
    }

    #[test]
    fn from_toml_protocol_https() {
        let config = Config::from_toml("protocol = \"https\"").unwrap();
        assert_eq!(config.protocol, Protocol::Https);
    }

    #[test]
    fn from_toml_protocol_ssh() {
        let config = Config::from_toml("protocol = \"ssh\"").unwrap();
        assert_eq!(config.protocol, Protocol::Ssh);
    }

    #[test]
    fn from_toml_custom_host() {
        let toml = r#"
            default_host = "gitlab.com"
            [[hosts]]
            name = "git.example.com"
            ssh_prefix = "git@git.example.com:"
            https_prefix = "https://git.example.com/"
        "#;
        let config = Config::from_toml(toml).unwrap();
        assert_eq!(config.default_host, "gitlab.com");
        assert_eq!(config.hosts.len(), 1);
        assert_eq!(config.hosts[0].name, "git.example.com");
    }

    #[test]
    fn from_toml_invalid() {
        assert!(Config::from_toml("not valid toml {{{{").is_err());
    }

    #[test]
    fn config_path_uses_xdg() {
        // This test verifies the path ends with jetti/config.toml
        let path = Config::path();
        assert!(path.ends_with("jetti/config.toml"));
    }

    #[test]
    fn load_returns_default_when_no_file() {
        let path =
            std::env::temp_dir().join(format!("jetti-config-missing-{}", std::process::id()));
        let config = Config::load_from_path(&path);
        assert_eq!(config.default_host, "github.com");
    }

    #[test]
    fn protocol_serde_roundtrip() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).unwrap();
        let reloaded: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(reloaded.protocol, config.protocol);
        assert_eq!(reloaded.default_host, config.default_host);
        assert_eq!(reloaded.hosts.len(), config.hosts.len());
    }
}
