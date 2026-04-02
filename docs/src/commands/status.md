# status

Show the status of all repositories — branch, uncommitted changes, and ahead/behind counts.

**Alias:** `st`

## Usage

```sh
jetti status [OPTIONS]
```

## Options

```
-p, --prefix <PREFIX>  Filter repositories by prefix
-j, --jobs <N>         Number of parallel jobs (default: 8)
```

## What it shows

For each repo, status reports:

- **Branch name** — the current checked-out branch
- **Modified file count** — number of uncommitted changes
- **↑N** — commits ahead of upstream (unpushed)
- **↓N** — commits behind upstream (can pull)

## Display

```
github.com
└── rust-lang
    ├── · log main
    └── ! my-fork main · 3 modified · ↑2

codeberg.org
└── forgejo
    └── · forgejo main

  done: 3 repos: 2 clean, 1 need attention
```

Repos are categorized as:
- **clean** — on a branch, nothing modified, not ahead/behind
- **need attention** — has dirty files or unpushed commits
- **behind upstream** — can be fast-forwarded with `jetti update`

## Examples

```sh
# Check everything
jetti status

# Check only your own repos
jetti status -p github.com/your-username
```
