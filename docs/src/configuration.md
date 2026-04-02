# Configuration

Jetti looks for a config file at `~/.config/jetti/config.toml`. If the file doesn't exist, sensible defaults are used.

## Managing the config

```sh
# View current settings (shows defaults if no file exists)
jetti config

# Create the default config file
jetti config init

# Open in your editor
jetti config edit

# Print the config file path
jetti config path
```

## Config file reference

```toml
# Root directory for repositories (default: ~/dev)
root = "/home/user/dev"

# Default host when only owner/repo is given (default: github.com)
default_host = "github.com"

# Clone protocol: "ssh" (default) or "https"
protocol = "ssh"

# Known hosts — each entry defines SSH and HTTPS URL prefixes
[[hosts]]
name = "github.com"
ssh_prefix = "git@github.com:"
https_prefix = "https://github.com/"

[[hosts]]
name = "gitlab.com"
ssh_prefix = "git@gitlab.com:"
https_prefix = "https://gitlab.com/"

[[hosts]]
name = "codeberg.org"
ssh_prefix = "git@codeberg.org:"
https_prefix = "https://codeberg.org/"
```

All fields are optional. Missing fields use the defaults shown above.

## Settings

### `root`

The base directory where all repositories are cloned. Repos are stored at `<root>/<host>/<owner>/<repo>`.

**Default:** `~/dev`

Can also be overridden per-command with the `--root` flag.

### `default_host`

The host assumed when you provide only `owner/repo` without a host prefix.

**Default:** `github.com`

### `protocol`

The default protocol for cloning. When a full URL is provided (e.g. `https://...` or `git@...`), the protocol is inferred from the URL instead.

**Default:** `ssh`

**Values:** `ssh`, `https`

Can also be overridden per-clone with `jetti clone --https` or `jetti clone --ssh`.

### `hosts`

A list of known git hosts. Each host has:

- `name` — the hostname (e.g. `github.com`)
- `ssh_prefix` — the prefix for SSH clone URLs (e.g. `git@github.com:`)
- `https_prefix` — the prefix for HTTPS clone URLs (e.g. `https://github.com/`)

**Defaults:** github.com, gitlab.com, codeberg.org

> **Note:** If you define a `[[hosts]]` section in your config file, it *replaces* the built-in defaults entirely — it does not extend them. Include all hosts you need.

Unknown hosts work too — jetti constructs reasonable SSH and HTTPS URLs from the hostname automatically. You only need to add a host entry if its URL format is non-standard.

## Adding a custom host

To add a self-hosted GitLab, Gitea, or other forge:

```toml
[[hosts]]
name = "git.example.com"
ssh_prefix = "git@git.example.com:"
https_prefix = "https://git.example.com/"
```

Then:

```sh
jetti clone git.example.com/team/project
```

## Environment variables

- `XDG_CONFIG_HOME` — if set, the config file is at `$XDG_CONFIG_HOME/jetti/config.toml` instead of `~/.config/jetti/config.toml`
