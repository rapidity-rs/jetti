# fetch

Fetch updates for all repositories in parallel.

Runs `git fetch` on each repo. This updates remote refs without modifying your working tree or branches — it's always safe to run.

## Usage

```sh
jetti fetch [OPTIONS]
```

## Options

```
-p, --prefix <PREFIX>  Filter repositories by prefix
-j, --jobs <N>         Number of parallel jobs (default: 8)
```

## Examples

```sh
# Fetch all repos
jetti fetch

# Fetch only GitHub repos
jetti fetch -p github.com

# Fetch with 16 parallel jobs
jetti fetch -j 16
```

## Display

Shows a tree-structured progress view with spinners, then a summary:

```
github.com
└── rust-lang
    ├── ✓ log fetched
    └── · cfg-if up to date

  done: 2 repos: 1 fetched, 1 up to date
```
