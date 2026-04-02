# root

Print the root directory path where repositories are stored.

## Usage

```sh
jetti root
```

## Examples

```sh
# Print the root
jetti root
# /Users/taylor/dev

# Use in scripts
cd $(jetti root)

# Override for a single command
jetti root --root /tmp/repos
# /tmp/repos
```

## Default

The default root is `~/dev`. This can be changed in the [configuration](../configuration.md).
