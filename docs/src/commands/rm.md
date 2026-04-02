# rm

Remove a cloned repository.

**Alias:** `remove`

## Usage

```sh
jetti rm [OPTIONS] <REPO>
```

## Arguments

- `<REPO>` — Repository specifier (same formats as `clone`)

## Options

```
--force   Skip the confirmation prompt
```

## Behavior

- Prompts for confirmation before deleting (unless `--force`)
- Removes the repository directory
- Cleans up empty parent directories (owner, host) back to the root
- Accepts the same specifier formats as `clone`

## Examples

```sh
# Remove with confirmation
jetti rm owner/repo

# Remove without confirmation
jetti rm owner/repo --force

# Remove a GitLab repo
jetti rm gitlab.com/owner/repo --force
```
