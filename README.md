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
gitty --type feat --scope cli
gitty -n 3
gitty --json
gitty --copy
gitty --dry-run
gitty providers
gitty completions zsh > ~/.zfunc/_gitty
```

Add `--commit` to create a commit directly from staged changes after generation.
It refuses unstaged-only input and multiple candidates. Without that explicit
flag, gitty never modifies repository files.

Provider subprocesses run non-interactively; Codex uses a read-only sandbox,
Claude denies tool requests, and OpenCode uses its plan agent. Use `--dry-run` to
inspect the exact prompt without contacting any provider.

Use `GITTY_PROVIDER` and `GITTY_MODEL` to set defaults. Authentication and model
configuration remain owned by the selected provider CLI.
