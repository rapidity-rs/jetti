# Jetti

A fast, cross-platform tool for organizing git repositories into a consistent directory structure by host, owner, and repo name.

Jetti is a Rust alternative to [ghq](https://github.com/x-motemen/ghq), installable via Cargo with no dependencies beyond `git`.

## Why Jetti?

- **Cross-platform** — installs anywhere Cargo runs, no Go toolchain needed
- **Multi-host** — GitHub, GitLab, Codeberg, and any git host out of the box
- **SSH by default** — uses SSH clone URLs, matching GitHub's recommendation
- **Fast batch operations** — parallel `fetch`, `update`, and `status` across all repos
- **Scriptable** — clean stdout/stderr separation for shell integration
- **Pretty** — colored tree view for humans, flat output for pipes

## Directory structure

Jetti organizes repositories into `~/dev/<host>/<owner>/<repo>`:

```
~/dev/
├── github.com/
│   ├── rust-lang/
│   │   ├── cargo/
│   │   └── log/
│   └── rapidity-rs/
│       └── jetti/
├── gitlab.com/
│   └── user/
│       └── project/
└── codeberg.org/
    └── forgejo/
        └── forgejo/
```

## Quick example

```sh
# Clone a repo
jetti clone rust-lang/log

# See all your repos
jetti list

# Check status across everything
jetti status

# Fetch all repos in parallel
jetti fetch

# Jump to a repo with fzf
cd $(jetti list --full-path | fzf)
```
