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
gitty --commit --push
gitty providers
gitty completions zsh > ~/.zfunc/_gitty
```

Add `--commit` to create a commit directly from staged changes after generation.
Add `--push` to push that commit to the current branch's configured upstream.
The command refuses unstaged-only input, multiple candidates, detached HEAD, and
branches without an upstream. Without those explicit flags, gitty never modifies
the repository or remote.

Provider subprocesses run non-interactively; Codex uses a read-only sandbox,
Claude denies tool requests, and OpenCode uses its plan agent. Use `--dry-run` to
inspect the exact prompt without contacting any provider.

Use `GITTY_PROVIDER` and `GITTY_MODEL` to set defaults. Authentication and model
configuration remain owned by the selected provider CLI.

## Configuration

Create a repository config with `gitty config init`, or a user-wide config with
`gitty config init --global`. Settings are layered in this order:

1. User config at `~/.config/gitty/config.toml`
2. Repository config at `.gitty.toml`
3. Environment variables and CLI flags

```toml
provider = "codex"
style = "conventional"
language = "English"
candidates = 1
max_diff_bytes = 120000
allowed_types = ["feat", "fix", "docs", "refactor", "test", "chore"]
allowed_scopes = ["cli", "config", "providers"]
```

Run `gitty config show` to print the merged configuration and its source files.
Unknown keys are rejected, so a typo can't silently change generation behavior.
