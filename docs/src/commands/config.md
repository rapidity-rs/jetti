# config

View or manage configuration.

**Alias:** `cfg`

## Usage

```sh
jetti config [SUBCOMMAND]
```

## Subcommands

### `jetti config` (no subcommand)

Print the current configuration, including all settings and known hosts.

```
config: ~/.config/jetti/config.toml

  root: /Users/taylor/dev
  default_host: github.com
  protocol: ssh

  hosts:
    · github.com
    · gitlab.com
    · codeberg.org
```

If no config file exists, shows defaults and a hint to create one.

### `jetti config init`

Create the default config file at `~/.config/jetti/config.toml`. Does nothing if it already exists.

### `jetti config edit`

Open the config file in `$EDITOR` (or `$VISUAL`). Creates the default config first if it doesn't exist.

### `jetti config path`

Print the path to the config file.

## Examples

```sh
# See current settings
jetti config

# Create a config file
jetti config init

# Edit in your preferred editor
jetti config edit

# Find the config file
cat $(jetti config path)
```
