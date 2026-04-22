# jetti

A fast, cross-platform tool for organizing git repositories into a consistent directory structure by host, owner, and repo name.

## Install

```sh
cargo install jetti
```

Requires `git` in your PATH.

## Usage

### Clone a repository

```sh
# GitHub (default host, SSH by default)
jetti clone owner/repo

# Explicit host
jetti clone gitlab.com/owner/repo
jetti clone codeberg.org/owner/repo

# Full URLs — protocol is inferred
jetti clone https://github.com/owner/repo.git
jetti clone git@github.com:owner/repo.git

# Shallow clone
jetti clone owner/repo --shallow

# Force HTTPS
jetti clone owner/repo --https

# Force SSH (useful when config defaults to HTTPS)
jetti clone owner/repo --ssh
```

Repositories are cloned to `<root>/<host>/<owner>/<repo>`:

```text
~/dev/
├── github.com/
│   └── rust-lang/
│       └── log/
├── gitlab.com/
│   └── user/
│       └── project/
└── codeberg.org/
    └── user/
        └── repo/
```

If the destination already exists and is not a git repository, `jetti clone`
stops with an error instead of deleting the directory.

### List repositories

```sh
jetti list                              # tree view in terminal
jetti list --full-path                  # absolute paths (flat)
jetti list -p github.com/rust-lang      # filter by prefix
jetti list | fzf                        # fuzzy find (auto-flat)
```

### Fetch and update

```sh
jetti fetch                 # git fetch all repos in parallel
jetti update                # fetch + fast-forward (safe, skips dirty repos)
jetti update -p github.com  # filter by prefix
jetti fetch -j 16           # 16 parallel jobs
```

### Status dashboard

```sh
jetti status                # branch, dirty files, ahead/behind for all repos
jetti status -p github.com  # filter by prefix
```

### Remove a repository

```sh
jetti rm owner/repo           # prompts for confirmation
jetti rm owner/repo --force   # skip confirmation
```

`jetti rm` refuses to remove directories that are not git repositories.

### Configuration

```sh
jetti config          # show current settings
jetti config init     # create default config file
jetti config edit     # open in $EDITOR
```

Config file at `~/.config/jetti/config.toml`:

```toml
root = "/home/user/dev"
default_host = "github.com"
protocol = "ssh"

[[hosts]]
name = "github.com"
ssh_prefix = "git@github.com:"
https_prefix = "https://github.com/"
```

### Shell completions

Add one of the following to your shell configuration:

```sh
# Bash (add to ~/.bashrc)
eval "$(jetti completions bash)"

# Zsh (add to ~/.zshrc)
eval "$(jetti completions zsh)"

# Fish (add to ~/.config/fish/config.fish)
jetti completions fish | source
```

## Shell integration

Jump to a repo with fzf:

```sh
# Add to .bashrc / .zshrc
j() {
  local dir
  dir=$(jetti list --full-path | fzf) && cd "$dir"
}
```

Clone and cd in one step:

```sh
jc() {
  cd "$(jetti clone "$@")"
}
```

## Release

This repository uses `cocogitto` for conventional commits, version bumps, tags,
and changelog generation.

Install the release tooling:

```sh
cargo install cocogitto cargo-edit
```

Install the git hooks locally:

```sh
cog install-hook --all
```

Cut a release:

```sh
# Preview the generated changelog
cog changelog

# Create the bump commit and tag from conventional commits
cog bump --auto
```

`cog bump` uses `cog.toml` to:

- update `Cargo.toml`
- update `CHANGELOG.md`
- create the release commit
- create a `v<version>` git tag

After the tag is pushed, GitHub Actions builds release artifacts and publishes to
crates.io.

All new commits should follow the Conventional Commits format. CI verifies commit
history with `cocogitto`.

## License

MIT
