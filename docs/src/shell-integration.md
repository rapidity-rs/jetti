# Shell Integration

## Jump to a repo with fzf

The most common integration is a function that lets you fuzzy-find and `cd` into a repo:

### Bash / Zsh

Add to your `~/.bashrc` or `~/.zshrc`:

```sh
j() {
  local dir
  dir=$(jetti list --full-path | fzf) && cd "$dir"
}
```

### Fish

Add to your `~/.config/fish/functions/j.fish`:

```fish
function j
    set dir (jetti list --full-path | fzf)
    and cd $dir
end
```

## Clone and cd in one step

```sh
jc() {
  cd "$(jetti clone "$@")"
}
```

This works because `jetti clone` prints only the path to stdout. Usage:

```sh
jc rust-lang/log
# You're now in ~/dev/github.com/rust-lang/log
```

## Shell completions

Generate completions so your shell can tab-complete jetti commands and options:

```sh
# Bash
jetti completions bash > ~/.local/share/bash-completion/completions/jetti

# Zsh
mkdir -p ~/.zfunc
jetti completions zsh > ~/.zfunc/_jetti

# Fish
jetti completions fish > ~/.config/fish/completions/jetti.fish
```

For Zsh, make sure `~/.zfunc` is in your `fpath`:

```sh
# In .zshrc, before compinit:
fpath=(~/.zfunc $fpath)
autoload -Uz compinit && compinit
```

## Using with tmux or scripts

Since jetti separates status output (stderr) from data (stdout), it works cleanly in scripts:

```sh
# Get the path to a repo without any status noise
path=$(jetti clone owner/repo 2>/dev/null)

# List all repos as an array
repos=($(jetti list))

# Count repos per host
jetti list | cut -d/ -f1 | sort | uniq -c
```
