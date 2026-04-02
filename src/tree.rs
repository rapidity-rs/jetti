//! Tree layout builder for displaying repositories in a hierarchical format.
//!
//! Converts a flat list of `host/owner/repo` paths into a tree structure with
//! proper ASCII art connectors (├──, └──, │) for terminal display.

use std::collections::BTreeMap;

/// A node in the repo tree, representing a single repo with its display position.
#[derive(Debug, Clone)]
pub struct RepoNode {
    /// Full relative path (e.g. "github.com/rust-lang/log")
    pub rel_path: String,
    /// Just the repo name (e.g. "log")
    pub name: String,
    /// The tree prefix for this node (e.g. "│   ├── ")
    pub prefix: String,
    /// The continuation prefix for content after the name (e.g. "│   │   ")
    #[allow(dead_code)]
    pub continuation_prefix: String,
}

/// All the lines needed to render the tree, including host and owner headers.
#[derive(Debug)]
pub enum TreeLine {
    /// A host header (e.g. "github.com")
    Host(String),
    /// An owner line with its tree prefix (e.g. "├── rust-lang")
    Owner { name: String, prefix: String },
    /// A repo leaf
    Repo(RepoNode),
    /// Blank line between host groups
    Blank,
}

/// Build a tree layout from sorted `host/owner/repo` relative paths.
/// Returns a list of TreeLines for rendering and a list of RepoNodes
/// (in the same order they appear in the tree) for batch operations.
pub fn build_tree(paths: &[String]) -> (Vec<TreeLine>, Vec<RepoNode>) {
    // Group: host -> owner -> Vec<repo_name>
    let mut groups: BTreeMap<&str, BTreeMap<&str, Vec<&str>>> = BTreeMap::new();
    // Keep the full paths mapped for lookup
    let mut path_map: BTreeMap<(&str, &str, &str), &str> = BTreeMap::new();

    for path in paths {
        let parts: Vec<&str> = path.splitn(3, '/').collect();
        if parts.len() == 3 {
            groups
                .entry(parts[0])
                .or_default()
                .entry(parts[1])
                .or_default()
                .push(parts[2]);
            path_map.insert((parts[0], parts[1], parts[2]), path.as_str());
        }
    }

    let mut lines = Vec::new();
    let mut repo_nodes = Vec::new();
    let host_count = groups.len();

    for (host_idx, (host, owners)) in groups.iter().enumerate() {
        let is_last_host = host_idx == host_count - 1;

        lines.push(TreeLine::Host(host.to_string()));

        let owner_count = owners.len();
        for (owner_idx, (owner, repo_names)) in owners.iter().enumerate() {
            let is_last_owner = owner_idx == owner_count - 1;
            let owner_branch = if is_last_owner {
                "└── "
            } else {
                "├── "
            };
            let owner_cont = if is_last_owner { "    " } else { "│   " };

            lines.push(TreeLine::Owner {
                name: owner.to_string(),
                prefix: owner_branch.to_string(),
            });

            for (repo_idx, repo_name) in repo_names.iter().enumerate() {
                let is_last_repo = repo_idx == repo_names.len() - 1;
                let repo_branch = if is_last_repo {
                    "└── "
                } else {
                    "├── "
                };
                let repo_cont = if is_last_repo { "    " } else { "│   " };

                let prefix = format!("{owner_cont}{repo_branch}");
                let continuation_prefix = format!("{owner_cont}{repo_cont}");
                let rel_path = path_map
                    .get(&(*host, *owner, *repo_name))
                    .unwrap_or(repo_name)
                    .to_string();

                let node = RepoNode {
                    rel_path,
                    name: repo_name.to_string(),
                    prefix,
                    continuation_prefix,
                };

                lines.push(TreeLine::Repo(node.clone()));
                repo_nodes.push(node);
            }
        }

        if !is_last_host {
            lines.push(TreeLine::Blank);
        }
    }

    (lines, repo_nodes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input() {
        let (lines, nodes) = build_tree(&[]);
        assert!(lines.is_empty());
        assert!(nodes.is_empty());
    }

    #[test]
    fn single_repo() {
        let paths = vec!["github.com/owner/repo".to_string()];
        let (lines, nodes) = build_tree(&paths);

        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].rel_path, "github.com/owner/repo");
        assert_eq!(nodes[0].name, "repo");

        // Should have: Host, Owner, Repo (no trailing Blank)
        assert_eq!(lines.len(), 3);
        assert!(matches!(&lines[0], TreeLine::Host(h) if h == "github.com"));
        assert!(matches!(&lines[1], TreeLine::Owner { name, .. } if name == "owner"));
        assert!(matches!(&lines[2], TreeLine::Repo(n) if n.name == "repo"));
    }

    #[test]
    fn single_host_multiple_repos() {
        let paths = vec![
            "github.com/rust-lang/cfg-if".to_string(),
            "github.com/rust-lang/log".to_string(),
        ];
        let (lines, nodes) = build_tree(&paths);

        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].name, "cfg-if");
        assert_eq!(nodes[1].name, "log");

        // Last owner should use └──
        assert!(matches!(&lines[1], TreeLine::Owner { prefix, .. } if prefix == "└── "));
        // First repo should use ├──, last should use └──
        assert!(nodes[0].prefix.contains("├── "));
        assert!(nodes[1].prefix.contains("└── "));
    }

    #[test]
    fn multiple_owners() {
        let paths = vec![
            "github.com/alice/project".to_string(),
            "github.com/bob/tool".to_string(),
        ];
        let (lines, nodes) = build_tree(&paths);

        assert_eq!(nodes.len(), 2);
        // First owner gets ├──, last gets └──
        assert!(
            matches!(&lines[1], TreeLine::Owner { name, prefix } if name == "alice" && prefix == "├── ")
        );
        assert!(
            matches!(&lines[3], TreeLine::Owner { name, prefix } if name == "bob" && prefix == "└── ")
        );
    }

    #[test]
    fn multiple_hosts_have_blank_separator() {
        let paths = vec![
            "codeberg.org/user/repo".to_string(),
            "github.com/user/repo".to_string(),
        ];
        let (lines, _) = build_tree(&paths);

        // Should have blank between hosts
        let blank_count = lines
            .iter()
            .filter(|l| matches!(l, TreeLine::Blank))
            .count();
        assert_eq!(blank_count, 1);
    }

    #[test]
    fn no_trailing_blank() {
        let paths = vec![
            "github.com/user/repo".to_string(),
            "gitlab.com/user/repo".to_string(),
        ];
        let (lines, _) = build_tree(&paths);

        // Last line should not be Blank
        assert!(!matches!(lines.last().unwrap(), TreeLine::Blank));
    }

    #[test]
    fn continuation_prefixes() {
        let paths = vec![
            "github.com/owner/first".to_string(),
            "github.com/owner/second".to_string(),
        ];
        let (_, nodes) = build_tree(&paths);

        // Non-last repo should have │ in continuation
        assert!(nodes[0].continuation_prefix.contains("│"));
        // Last repo should have spaces in continuation
        assert!(!nodes[1].continuation_prefix.contains("│"));
    }

    #[test]
    fn paths_with_fewer_than_three_parts_are_skipped() {
        let paths = vec![
            "github.com/owner/repo".to_string(),
            "malformed".to_string(),
            "also/malformed".to_string(),
        ];
        let (_, nodes) = build_tree(&paths);
        assert_eq!(nodes.len(), 1);
    }

    #[test]
    fn repo_nodes_preserve_full_path() {
        let paths = vec!["github.com/rust-lang/log".to_string()];
        let (_, nodes) = build_tree(&paths);
        assert_eq!(nodes[0].rel_path, "github.com/rust-lang/log");
    }

    #[test]
    fn hosts_are_sorted() {
        let paths = vec![
            "gitlab.com/user/repo".to_string(),
            "codeberg.org/user/repo".to_string(),
            "github.com/user/repo".to_string(),
        ];
        let (lines, _) = build_tree(&paths);

        let hosts: Vec<&str> = lines
            .iter()
            .filter_map(|l| match l {
                TreeLine::Host(h) => Some(h.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(hosts, vec!["codeberg.org", "github.com", "gitlab.com"]);
    }
}
