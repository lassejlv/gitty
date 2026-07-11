# gitty

`gitty` generates sharp, repository-aware Git commit messages using the AI CLI
you already have authenticated: Codex, Claude Code, or OpenCode.

```bash
cargo install --git https://github.com/lassejlv/gitty

git add src/
gitty
```

By default, gitty uses staged changes when they exist; otherwise it includes all
working-tree changes and untracked text files. It auto-detects Codex, then Claude
Code, then OpenCode.

```bash
gitty --provider claude
gitty --provider opencode --model anthropic/claude-sonnet-4-5
gitty --all --style detailed --hint "fixes the startup race"
gitty -n 3
gitty --json
gitty completions zsh > ~/.zfunc/_gitty
```

Run `gitty --help` for every option. gitty never commits or modifies repository
files. Provider subprocesses run non-interactively; Codex uses a read-only
sandbox, Claude denies tool requests, and OpenCode uses its plan agent.

Use `GITTY_PROVIDER` and `GITTY_MODEL` to set defaults. Authentication and model
configuration remain owned by the selected provider CLI.
