use crate::ai::Provider;
use console::style;

pub fn about() {
    println!(
        "{} {}\n\n{}\n\n{}\n  gitty <command> [options]\n\n{}\n  {}  Generate a commit message\n  {}  Review the changes sent to AI\n  {}  Show installed AI backends\n  {}  Create or inspect configuration\n  {}  Generate shell completions\n\n{}\n  git add src/\n  gitty diff\n  gitty gen\n  gitty gen --interactive\n  gitty gen --commit --push\n\n{}",
        style("gitty").bold().cyan(),
        style(env!("CARGO_PKG_VERSION")).dim(),
        style("AI commit messages that don't suck.").bold(),
        style("Usage:").bold(),
        style("Commands:").bold(),
        style("generate, gen").cyan(),
        style("diff").cyan(),
        style("providers").cyan(),
        style("config").cyan(),
        style("completions").cyan(),
        style("Quick start:").bold(),
        style("Codex · Claude Code · OpenCode").dim()
    );
}

pub fn generating(provider: Provider, changes: &str) {
    eprintln!(
        "{} {} {}",
        style("◆").cyan(),
        style(provider).bold(),
        style(format!("is writing from {changes}…")).dim()
    );
}

pub fn success(message: &str) {
    eprintln!("{} {message}", style("✓").green().bold());
}

pub fn nothing_to_do() {
    eprintln!(
        "{} Working tree is clean — nothing to generate.",
        style("✓").green().bold()
    );
}
