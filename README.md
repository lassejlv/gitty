# gitty

`gitty` generates sharp, repository-aware Git commit messages using the AI CLI
you already have authenticated: Codex, Claude Code, or OpenCode.

```bash
cargo install --git https://github.com/lassejlv/gitty

git add src/
gitty gen
```

By default, gitty uses staged changes when they exist; otherwise it includes all
working-tree changes and untracked text files. It auto-detects Codex, then Claude
Code, then OpenCode. In auto mode, a failed or unauthenticated provider falls
through to the next installed backend; an explicit `--provider` never falls back.

```bash
gitty gen --provider claude
gitty diff
gitty diff --all --stat
gitty generate --provider opencode --model anthropic/claude-sonnet-4-5
gitty generate --all --style detailed --hint "fixes the startup race"
gitty generate --type feat --scope cli
gitty generate -n 3
gitty gen --interactive
gitty generate --json
gitty generate --copy
gitty generate --dry-run
gitty generate --commit --push
gitty providers
gitty completions zsh > ~/.zfunc/_gitty
```

Running bare `gitty` prints the command overview. Use `gitty gen` (or the long
form `gitty generate`) for generation; global options work before or after the
subcommand.

Review exactly what will be sent to the provider before generating anything:

```bash
gitty diff          # staged changes when available, otherwise all changes
gitty diff --all    # staged, unstaged, and untracked changes
gitty diff --stat   # compact file and line summary
```

Potential credentials are redacted before the diff reaches any provider. Use
`--allow-secrets` only when sending the raw diff is genuinely intentional;
`gitty diff` shows the same redacted payload used for generation.

Interactive mode generates three candidates by default, then lets you select,
edit, regenerate, copy, commit, or commit and push without leaving the terminal:

```bash
git add src/
gitty gen -i
```

Add `--commit` to commit staged changes, or combine it with `--all` to stage and
commit every visible change after generation succeeds. `--push` implies commit,
so `gitty gen --all --push` is the shortest full workflow. Detached HEAD and
branches without an upstream still fail before generation. Without these flags,
gitty never modifies the repository or remote.

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
