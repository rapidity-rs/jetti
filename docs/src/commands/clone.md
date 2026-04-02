# clone

Clone a repository into the organized directory structure.

**Aliases:** `get`, `g`

## Usage

```sh
jetti clone [OPTIONS] <REPO>
```

## Arguments

- `<REPO>` — Repository specifier in any of these formats:
  - `owner/repo` — uses the default host (github.com) and configured protocol (SSH)
  - `host/owner/repo` — explicit host
  - `https://host/owner/repo.git` — full HTTPS URL (protocol inferred)
  - `git@host:owner/repo.git` — full SSH URL (protocol inferred)

## Options

```
-d, --depth <N>   Create a shallow clone with the given depth
-s, --shallow     Shorthand for --depth 1
    --https       Clone using HTTPS instead of SSH
    --ssh         Clone using SSH instead of HTTPS
```

## Protocol resolution

The clone protocol is determined in this order:

1. **Inferred from URL** — `git@...` always uses SSH, `https://...` always uses HTTPS
2. **`--https` / `--ssh` flag** — overrides the default for this clone (mutually exclusive)
3. **Config `protocol`** — your default (SSH unless changed)

## Examples

```sh
# Clone from GitHub (SSH by default)
jetti clone rust-lang/log

# Clone from GitLab
jetti clone gitlab.com/user/project

# Shallow clone
jetti clone owner/repo --shallow

# Force HTTPS
jetti clone owner/repo --https

# Full URL — protocol inferred
jetti clone https://github.com/owner/repo.git
```

## Behavior

- Creates the directory structure `<root>/<host>/<owner>/<repo>`
- If the repo already exists, prints the path and exits
- If a directory exists but isn't a valid git repo (e.g. from a failed clone), it is removed and re-cloned
- Status messages go to stderr; only the resulting path goes to stdout

## Shell integration

Because only the path goes to stdout, you can use:

```sh
cd $(jetti clone owner/repo)
```
