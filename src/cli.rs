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
    #[arg(short, long, value_enum, env = "GITTY_PROVIDER", global = true)]
    pub provider: Option<ProviderChoice>,
    /// Override the provider's configured model.
    #[arg(short, long, env = "GITTY_MODEL", global = true)]
    pub model: Option<String>,
    /// Which changes to describe.
    #[arg(long, value_enum, default_value_t=ChangeSelection::Auto, global = true)]
    pub changes: ChangeSelection,
    /// Include staged, unstaged, and untracked changes.
    #[arg(short = 'a', long, conflicts_with = "changes", global = true)]
    pub all: bool,
    /// Commit-message style.
    #[arg(short, long, value_enum, global = true)]
    pub style: Option<MessageStyle>,
    /// Extra author intent or context.
    #[arg(long, global = true)]
    pub hint: Option<String>,
    /// Force a Conventional Commit type (for example feat or fix).
    #[arg(long = "type", value_name = "TYPE", global = true)]
    pub commit_type: Option<String>,
    /// Force a Conventional Commit scope.
    #[arg(long, value_name = "SCOPE", global = true)]
    pub scope: Option<String>,
    /// Number of alternatives.
    #[arg(short='n', long, value_parser=clap::value_parser!(u8).range(1..=5), global = true)]
    pub candidates: Option<u8>,
    /// Maximum diff bytes sent to the model.
    #[arg(long, global = true)]
    pub max_diff_bytes: Option<usize>,
    /// Repository path.
    #[arg(short = 'C', long, global = true)]
    pub repo: Option<PathBuf>,
    /// Emit a JSON array.
    #[arg(long, global = true)]
    pub json: bool,
    /// Copy the first generated candidate to the clipboard.
    #[arg(long, global = true)]
    pub copy: bool,
    /// Pick, edit, regenerate, copy, or commit candidates interactively.
    #[arg(short = 'i', long, conflicts_with_all = ["json", "dry_run", "quiet"], global = true)]
    pub interactive: bool,
    /// Send detected credentials to the provider without redaction.
    #[arg(long, global = true)]
    pub allow_secrets: bool,
    /// Print the complete model prompt without contacting a provider.
    #[arg(long, global = true)]
    pub dry_run: bool,
    /// Create a Git commit from staged changes using the generated message.
    #[arg(long, conflicts_with_all = ["json", "dry_run"], global = true)]
    pub commit: bool,
    /// Commit and push to the current branch's configured upstream.
    #[arg(long, conflicts_with_all = ["json", "dry_run"], global = true)]
    pub push: bool,
    /// Suppress progress output.
    #[arg(short, long, global = true)]
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
#[derive(Debug, Clone, Copy, ValueEnum, serde::Deserialize, serde::Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum MessageStyle {
    #[default]
    Conventional,
    Plain,
    Detailed,
}
#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Generate a commit message from repository changes.
    #[command(visible_alias = "gen")]
    Generate,
    /// Review the exact changes gitty would send to the provider.
    Diff {
        /// Show only the file and line summary.
        #[arg(long)]
        stat: bool,
    },
    /// Generate shell completions.
    Completions {
        #[arg(value_enum)]
        shell: Shell,
    },
    /// Show which supported AI CLIs are installed.
    Providers,
    /// Create or inspect gitty configuration.
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommands {
    /// Create a documented config file.
    Init {
        /// Write the user config instead of .gitty.toml in this repository.
        #[arg(long)]
        global: bool,
    },
    /// Show the merged effective configuration.
    Show,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn push_implies_commit_and_accepts_all() {
        let cli = Cli::try_parse_from(["gitty", "gen", "--all", "--push"]).unwrap();
        assert!(cli.all);
        assert!(cli.push);
    }
}
