//! Repository specifier parsing and URL construction.
//!
//! A [`Repo`] represents a parsed repository reference with its host, owner, and
//! name extracted from various input formats (short form, full URL, SSH URL, etc.).

use std::path::PathBuf;

use crate::config::{Config, Protocol};
use crate::error::JettiError;

/// A parsed repository specifier resolved to its components.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Repo {
    /// The git host (e.g. `github.com`).
    pub host: String,
    /// The repository owner or organization (e.g. `rust-lang`).
    pub owner: String,
    /// The repository name (e.g. `log`).
    pub name: String,
    /// Protocol inferred from the input URL, if any.
    ///
    /// When present, this takes priority over the CLI flag and config default.
    pub inferred_protocol: Option<Protocol>,
}

impl Repo {
    /// Parse a repo specifier string into a Repo.
    ///
    /// Supported formats:
    /// - `owner/repo` — uses default host, no inferred protocol
    /// - `github.com/owner/repo` — explicit host, no inferred protocol
    /// - `https://github.com/owner/repo` — infers HTTPS
    /// - `https://github.com/owner/repo.git` — infers HTTPS
    /// - `git@github.com:owner/repo.git` — infers SSH
    ///
    /// If the input has two parts and the first contains a `.` (e.g. `gitlab.com/owner`),
    /// it is rejected as a likely host missing the repo name.
    pub fn parse(input: &str, config: &Config) -> crate::error::Result<Self> {
        // Try SSH URL: git@host:owner/repo.git
        if let Some(rest) = input.strip_prefix("git@") {
            let (host, path) = rest
                .split_once(':')
                .ok_or_else(|| JettiError::InvalidRepo(format!("invalid SSH URL: {input}")))?;
            return Self::from_host_and_path(host, path, Some(Protocol::Ssh));
        }

        // Try HTTPS URL: https://host/owner/repo[.git]
        if input.starts_with("https://") || input.starts_with("http://") {
            let without_scheme = input
                .strip_prefix("https://")
                .or_else(|| input.strip_prefix("http://"))
                .unwrap();
            let parts: Vec<&str> = without_scheme.splitn(3, '/').collect();
            if parts.len() < 3 {
                return Err(JettiError::InvalidRepo(format!("invalid URL: {input}")));
            }
            return Self::from_host_and_path(
                parts[0],
                &format!("{}/{}", parts[1], parts[2]),
                Some(Protocol::Https),
            );
        }

        let parts: Vec<&str> = input.splitn(3, '/').collect();

        match parts.len() {
            // host/owner/repo
            3 => Self::from_host_and_path(parts[0], &format!("{}/{}", parts[1], parts[2]), None),
            // owner/repo -> use default host, but reject if first part looks like a hostname
            2 => {
                if parts[0].contains('.') {
                    return Err(JettiError::InvalidRepo(format!(
                        "missing repo name: {input}\n\
                         Did you mean: {}/{}/REPO?",
                        parts[0], parts[1]
                    )));
                }
                Ok(Self {
                    host: config.default_host.clone(),
                    owner: parts[0].to_string(),
                    name: strip_git_suffix(parts[1]).to_string(),
                    inferred_protocol: None,
                })
            }
            _ => Err(JettiError::InvalidRepo(format!(
                "cannot parse repo specifier: {input}\n\
                 Expected: owner/repo, host/owner/repo, or a full URL"
            ))),
        }
    }

    fn from_host_and_path(
        host: &str,
        path: &str,
        inferred_protocol: Option<Protocol>,
    ) -> crate::error::Result<Self> {
        let path = path.trim_matches('/');
        let parts: Vec<&str> = path.split('/').collect();

        if parts.len() != 2 || parts.iter().any(|part| part.is_empty()) {
            return Err(JettiError::InvalidRepo(format!(
                "expected exactly host/owner/repo or owner/repo: {host}/{path}"
            )));
        }

        let owner = parts[0];
        let name = parts[1];

        Ok(Self {
            host: host.to_string(),
            owner: owner.to_string(),
            name: strip_git_suffix(name).to_string(),
            inferred_protocol,
        })
    }

    /// Build the clone URL for this repo using the given protocol.
    ///
    /// Uses the host's configured prefix if available, otherwise constructs a
    /// conventional URL from the hostname.
    pub fn clone_url(&self, config: &Config, protocol: Protocol) -> String {
        match protocol {
            Protocol::Ssh => {
                if let Some(host_config) = config.host(&self.host) {
                    format!("{}{}/{}.git", host_config.ssh_prefix, self.owner, self.name)
                } else {
                    format!("git@{}:{}/{}.git", self.host, self.owner, self.name)
                }
            }
            Protocol::Https => {
                if let Some(host_config) = config.host(&self.host) {
                    format!(
                        "{}{}/{}.git",
                        host_config.https_prefix, self.owner, self.name
                    )
                } else {
                    format!("https://{}/{}/{}.git", self.host, self.owner, self.name)
                }
            }
        }
    }

    /// The local directory path under the root (`<root>/<host>/<owner>/<name>`).
    pub fn local_path(&self, config: &Config) -> PathBuf {
        config
            .root
            .join(&self.host)
            .join(&self.owner)
            .join(&self.name)
    }
}

fn strip_git_suffix(s: &str) -> &str {
    s.strip_suffix(".git").unwrap_or(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> Config {
        Config::default()
    }

    #[test]
    fn parse_owner_repo() {
        let repo = Repo::parse("rapidity-rs/jetti", &test_config()).unwrap();
        assert_eq!(repo.host, "github.com");
        assert_eq!(repo.owner, "rapidity-rs");
        assert_eq!(repo.name, "jetti");
    }

    #[test]
    fn parse_host_owner_repo() {
        let repo = Repo::parse("gitlab.com/user/project", &test_config()).unwrap();
        assert_eq!(repo.host, "gitlab.com");
        assert_eq!(repo.owner, "user");
        assert_eq!(repo.name, "project");
    }

    #[test]
    fn parse_https_url() {
        let repo = Repo::parse("https://github.com/rapidity-rs/jetti.git", &test_config()).unwrap();
        assert_eq!(repo.host, "github.com");
        assert_eq!(repo.owner, "rapidity-rs");
        assert_eq!(repo.name, "jetti");
    }

    #[test]
    fn parse_ssh_url() {
        let repo = Repo::parse("git@github.com:rapidity-rs/jetti.git", &test_config()).unwrap();
        assert_eq!(repo.host, "github.com");
        assert_eq!(repo.owner, "rapidity-rs");
        assert_eq!(repo.name, "jetti");
    }

    #[test]
    fn parse_codeberg() {
        let repo = Repo::parse("codeberg.org/user/repo", &test_config()).unwrap();
        assert_eq!(repo.host, "codeberg.org");
        assert_eq!(repo.owner, "user");
        assert_eq!(repo.name, "repo");
    }

    #[test]
    fn clone_url_ssh_known_host() {
        let repo = Repo::parse("rapidity-rs/jetti", &test_config()).unwrap();
        assert_eq!(
            repo.clone_url(&test_config(), Protocol::Ssh),
            "git@github.com:rapidity-rs/jetti.git"
        );
    }

    #[test]
    fn clone_url_https_known_host() {
        let repo = Repo::parse("rapidity-rs/jetti", &test_config()).unwrap();
        assert_eq!(
            repo.clone_url(&test_config(), Protocol::Https),
            "https://github.com/rapidity-rs/jetti.git"
        );
    }

    #[test]
    fn clone_url_ssh_unknown_host() {
        let repo = Repo::parse("sr.ht/user/repo", &test_config()).unwrap();
        assert_eq!(
            repo.clone_url(&test_config(), Protocol::Ssh),
            "git@sr.ht:user/repo.git"
        );
    }

    #[test]
    fn clone_url_https_unknown_host() {
        let repo = Repo::parse("sr.ht/user/repo", &test_config()).unwrap();
        assert_eq!(
            repo.clone_url(&test_config(), Protocol::Https),
            "https://sr.ht/user/repo.git"
        );
    }

    #[test]
    fn local_path() {
        let config = test_config();
        let repo = Repo::parse("rapidity-rs/jetti", &config).unwrap();
        let expected = config.root.join("github.com/rapidity-rs/jetti");
        assert_eq!(repo.local_path(&config), expected);
    }

    #[test]
    fn rejects_bare_name() {
        assert!(Repo::parse("jetti", &test_config()).is_err());
    }

    #[test]
    fn parse_http_url() {
        let repo = Repo::parse("http://github.com/owner/repo", &test_config()).unwrap();
        assert_eq!(repo.host, "github.com");
        assert_eq!(repo.owner, "owner");
        assert_eq!(repo.name, "repo");
    }

    #[test]
    fn parse_https_url_without_git_suffix() {
        let repo = Repo::parse("https://github.com/owner/repo", &test_config()).unwrap();
        assert_eq!(repo.name, "repo");
    }

    #[test]
    fn parse_ssh_infers_protocol() {
        let repo = Repo::parse("git@github.com:owner/repo.git", &test_config()).unwrap();
        assert_eq!(repo.inferred_protocol, Some(Protocol::Ssh));
    }

    #[test]
    fn parse_https_infers_protocol() {
        let repo = Repo::parse("https://github.com/owner/repo", &test_config()).unwrap();
        assert_eq!(repo.inferred_protocol, Some(Protocol::Https));
    }

    #[test]
    fn parse_owner_repo_no_inferred_protocol() {
        let repo = Repo::parse("owner/repo", &test_config()).unwrap();
        assert_eq!(repo.inferred_protocol, None);
    }

    #[test]
    fn parse_host_owner_repo_no_inferred_protocol() {
        let repo = Repo::parse("gitlab.com/owner/repo", &test_config()).unwrap();
        assert_eq!(repo.inferred_protocol, None);
    }

    #[test]
    fn parse_strips_trailing_slashes() {
        let repo = Repo::parse("https://github.com/owner/repo/", &test_config()).unwrap();
        assert_eq!(repo.name, "repo");
    }

    #[test]
    fn parse_invalid_ssh_url() {
        assert!(Repo::parse("git@github.com", &test_config()).is_err());
    }

    #[test]
    fn parse_invalid_https_url() {
        assert!(Repo::parse("https://github.com", &test_config()).is_err());
    }

    #[test]
    fn rejects_https_url_with_extra_segments() {
        assert!(Repo::parse("https://github.com/owner/repo/tree/main", &test_config()).is_err());
    }

    #[test]
    fn rejects_host_path_with_extra_segments() {
        assert!(Repo::parse("github.com/owner/repo/extra", &test_config()).is_err());
    }

    #[test]
    fn local_path_different_host() {
        let config = test_config();
        let repo = Repo::parse("gitlab.com/user/project", &config).unwrap();
        let expected = config.root.join("gitlab.com/user/project");
        assert_eq!(repo.local_path(&config), expected);
    }

    #[test]
    fn rejects_host_without_repo_name() {
        let err = Repo::parse("gitlab.com/inkscape", &test_config()).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("missing repo name"), "got: {msg}");
        assert!(msg.contains("gitlab.com/inkscape/REPO"), "got: {msg}");
    }

    #[test]
    fn owner_without_dot_still_uses_default_host() {
        let repo = Repo::parse("owner/repo", &test_config()).unwrap();
        assert_eq!(repo.host, "github.com");
        assert_eq!(repo.owner, "owner");
        assert_eq!(repo.name, "repo");
    }

    #[test]
    fn clone_url_uses_host_config() {
        let config = test_config();
        let repo = Repo::parse("gitlab.com/user/project", &config).unwrap();
        assert_eq!(
            repo.clone_url(&config, Protocol::Ssh),
            "git@gitlab.com:user/project.git"
        );
        assert_eq!(
            repo.clone_url(&config, Protocol::Https),
            "https://gitlab.com/user/project.git"
        );
    }
}
