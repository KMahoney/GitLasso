# GitLasso

This is my tool for managing multiple Git repositories. I made it, all on my own. It's little, and broken, but still good. Yeah, still good.

- Quickly see the status of multiple Git repositories in a pretty table.
- A sensible command line interface.
- Work on a subset of repositories with `context`.
- Run commands against them serially or in parallel.

![Example](doc/images/example.gif)

## Installation

```bash
$ cargo install gitlasso
```

Make sure you have your `cargo` binary path in your `$PATH`.

## Usage

In short:

- Use the `register` command to add repositories.
- Run `gitlasso` on its own to see a summary of those repositories.
- Use `context` to select which repositories you want to operate on.
- Use `fetch` to update all repositories in parallel.
- Use `git` to run git commands.
- Run `exec` to run arbitrary commands, and `exec -p` to run them in parallel.

Tip: alias `gitlasso` to something short, like `gl`.

## Shell Completion

You can use the `completions` command to print shell completions. Either evaluate the output directly, or pipe the output to a file and include it in your shell configuration.

### Fish

```
$ gitlasso completions fish | source
```

### Zsh

```
$ eval "$(gitlasso completions zsh)"
```

### Bash

```
$ eval "$(gitlasso completions bash)"
```

Bash does not complete aliases, but you can do this:

```
$ alias gl=gitlasso
$ eval "$(gitlasso completions bash --binary gl)"
```
