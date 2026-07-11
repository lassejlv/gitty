use crate::ai::ProviderChoice;
use clap::{Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "gitty",
    version,
    about = "Generate excellent Git commit messages with Codex, Claude, or OpenCode"
)]
pub struct Cli {
    /// AI CLI to use. Auto tries Codex, Claude, then OpenCode.
    #[arg(short, long, value_enum, default_value_t=ProviderChoice::Auto, env="GITTY_PROVIDER")]
    pub provider: ProviderChoice,
    /// Override the provider's configured model.
    #[arg(short, long, env = "GITTY_MODEL")]
    pub model: Option<String>,
    /// Which changes to describe.
    #[arg(long, value_enum, default_value_t=ChangeSelection::Auto)]
    pub changes: ChangeSelection,
    /// Include staged, unstaged, and untracked changes.
    #[arg(short = 'a', long, conflicts_with = "changes")]
    pub all: bool,
    /// Commit-message style.
    #[arg(short, long, value_enum, default_value_t=MessageStyle::Conventional)]
    pub style: MessageStyle,
    /// Extra author intent or context.
    #[arg(long)]
    pub hint: Option<String>,
    /// Force a Conventional Commit type (for example feat or fix).
    #[arg(long = "type", value_name = "TYPE")]
    pub commit_type: Option<String>,
    /// Force a Conventional Commit scope.
    #[arg(long, value_name = "SCOPE")]
    pub scope: Option<String>,
    /// Number of alternatives.
    #[arg(short='n', long, default_value_t=1, value_parser=clap::value_parser!(u8).range(1..=5))]
    pub candidates: u8,
    /// Maximum diff bytes sent to the model.
    #[arg(long, default_value_t = 120_000)]
    pub max_diff_bytes: usize,
    /// Repository path.
    #[arg(short = 'C', long)]
    pub repo: Option<PathBuf>,
    /// Emit a JSON array.
    #[arg(long)]
    pub json: bool,
    /// Copy the first generated candidate to the clipboard.
    #[arg(long)]
    pub copy: bool,
    /// Print the complete model prompt without contacting a provider.
    #[arg(long)]
    pub dry_run: bool,
    /// Create a Git commit from staged changes using the generated message.
    #[arg(long, conflicts_with_all=["all", "json", "dry_run"])]
    pub commit: bool,
    /// Suppress progress output.
    #[arg(short, long)]
    pub quiet: bool,
    #[command(subcommand)]
    pub command: Option<Commands>,
}
impl Cli {
    pub fn effective_changes(&self) -> ChangeSelection {
        if self.all {
            ChangeSelection::All
        } else {
            self.changes
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ChangeSelection {
    Auto,
    Staged,
    All,
}
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum MessageStyle {
    Conventional,
    Plain,
    Detailed,
}
#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Generate shell completions.
    Completions {
        #[arg(value_enum)]
        shell: Shell,
    },
    /// Show which supported AI CLIs are installed.
    Providers,
}
