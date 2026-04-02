# completions

Generate shell completions for bash, zsh, or fish.

## Usage

```sh
jetti completions <SHELL>
```

## Supported shells

- `bash`
- `zsh`
- `fish`

## Setup

### Bash

```sh
jetti completions bash > ~/.local/share/bash-completion/completions/jetti
```

### Zsh

```sh
# Ensure the directory is in your fpath
mkdir -p ~/.zfunc
jetti completions zsh > ~/.zfunc/_jetti

# Add to .zshrc if not already present:
# fpath=(~/.zfunc $fpath)
# autoload -Uz compinit && compinit
```

### Fish

```sh
jetti completions fish > ~/.config/fish/completions/jetti.fish
```

Completions take effect in new shell sessions, or after sourcing the file.
