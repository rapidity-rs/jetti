# update

Fetch and fast-forward all repositories in parallel.

Runs `git fetch` followed by `git merge --ff-only` on each repo. This is safe:

- **Dirty repos are skipped** — repos with uncommitted changes are not touched
- **Only fast-forwards** — if your branch has diverged from upstream, the update is skipped with a warning
- **No merge commits** — only clean fast-forwards are applied

## Usage

```sh
jetti update [OPTIONS]
```

## Options

```
-p, --prefix <PREFIX>  Filter repositories by prefix
-j, --jobs <N>         Number of parallel jobs (default: 8)
```

## Examples

```sh
# Update all repos
jetti update

# Update only GitLab repos
jetti update -p gitlab.com

# Single-threaded (sequential)
jetti update -j 1
```

## Display

```
github.com
└── rust-lang
    ├── ✓ log fast-forwarded
    ├── · cfg-if up to date
    └── ! my-fork diverged, merge manually

  done: 3 repos: 1 updated, 1 up to date, 1 need attention
```
