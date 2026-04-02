# list

List cloned repositories.

**Alias:** `ls`

## Usage

```sh
jetti list [OPTIONS]
```

## Options

```
-f, --full-path        Show absolute paths instead of relative
-p, --prefix <PREFIX>  Filter repositories by prefix
```

## Output modes

**Tree view** — shown by default in a terminal:

```
github.com
├── rust-lang
│   ├── cfg-if
│   └── log
└── rapidity-rs
    └── jetti

codeberg.org
└── forgejo
    └── forgejo
```

**Flat view** — used automatically when piping, or with `--full-path`:

```
github.com/rapidity-rs/jetti
github.com/rust-lang/cfg-if
github.com/rust-lang/log
codeberg.org/forgejo/forgejo
```

The flat output is one path per line, sorted, designed for piping to `fzf`, `grep`, `wc`, etc.

## Examples

```sh
# Tree view
jetti list

# Filter to a specific host or owner
jetti list -p github.com/rust-lang

# Absolute paths
jetti list --full-path

# Fuzzy find
jetti list | fzf

# Count repos
jetti list | wc -l
```
