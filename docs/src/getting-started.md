# Getting Started

## Install

```sh
cargo install jetti
```

Jetti requires `git` to be installed and available in your `PATH`.

## Clone your first repo

```sh
jetti clone owner/repo
```

This clones `git@github.com:owner/repo.git` into `~/dev/github.com/owner/repo/`.

Jetti supports several specifier formats:

```sh
# Short form — uses default host (github.com) and SSH
jetti clone owner/repo

# Explicit host
jetti clone gitlab.com/owner/repo
jetti clone codeberg.org/owner/repo

# Full URLs — protocol is inferred from the URL
jetti clone https://github.com/owner/repo.git
jetti clone git@github.com:owner/repo.git
```

## Browse your repos

```sh
# Tree view (in a terminal)
jetti list

# Flat output (for piping)
jetti list | fzf

# Absolute paths
jetti list --full-path
```

## Keep repos up to date

```sh
# Fetch all repos in parallel
jetti fetch

# Fetch and fast-forward all repos
jetti update

# Check for uncommitted work or unpushed commits
jetti status
```

## Next steps

- See [Commands](./commands.md) for the full reference
- Set up [Shell Integration](./shell-integration.md) for the `j` shortcut
- Customize your setup in [Configuration](./configuration.md)
