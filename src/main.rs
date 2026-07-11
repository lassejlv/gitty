mod ai;
mod cli;
mod clipboard;
mod git;
mod message;
mod prompt;

use anyhow::{Context, Result, bail};
use clap::{CommandFactory, Parser};
use clap_complete::generate;
use cli::{Cli, Commands};
use std::{io, process::ExitCode};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Some(Commands::Completions { shell }) => {
            generate(shell, &mut Cli::command(), "gitty", &mut io::stdout());
            return Ok(());
        }
        Some(Commands::Providers) => {
            ai::print_provider_status();
            return Ok(());
        }
        None => {}
    }
    let repo = git::Repository::discover(cli.repo.as_deref())?;
    if (cli.commit_type.is_some() || cli.scope.is_some())
        && !matches!(cli.style, cli::MessageStyle::Conventional)
    {
        bail!("--type and --scope require --style conventional");
    }
    if cli.commit && cli.candidates != 1 {
        bail!("--commit requires exactly one candidate");
    }
    if cli.commit && matches!(cli.effective_changes(), cli::ChangeSelection::All) {
        bail!("--commit cannot use all changes; stage what you want and use --changes staged");
    }
    if cli.commit && !repo.has_staged_changes()? {
        bail!("--commit requires staged changes");
    }
    let snapshot = repo.snapshot(cli.effective_changes(), cli.max_diff_bytes)?;
    if snapshot.diff.trim().is_empty() {
        bail!("no changes found (stage files, or pass --all to include unstaged changes)");
    }
    let request = prompt::Request {
        snapshot: &snapshot,
        style: cli.style,
        hint: cli.hint.as_deref(),
        candidates: cli.candidates,
        commit_type: cli.commit_type.as_deref(),
        scope: cli.scope.as_deref(),
    };
    let prompt = request.render();
    if cli.dry_run {
        print!("{prompt}");
        return Ok(());
    }
    let provider = ai::Provider::resolve(cli.provider)?;
    if !cli.quiet {
        eprintln!(
            "{} asking {provider} to describe {}…",
            console::style("gitty").bold(),
            snapshot.label
        );
    }
    let raw = provider
        .generate(&repo.root, &prompt, cli.model.as_deref())
        .with_context(|| format!("{provider} could not generate a commit message"))?;
    let messages = message::parse_candidates(&raw, cli.candidates)?;
    if cli.copy {
        clipboard::copy(&messages[0])?;
        if !cli.quiet {
            eprintln!(
                "{} copied first candidate to clipboard",
                console::style("✓").green()
            );
        }
    }
    if cli.commit {
        repo.commit(&messages[0])?;
        if !cli.quiet {
            eprintln!("{} created commit", console::style("✓").green());
        }
    }
    if cli.json {
        println!("{}", serde_json::to_string_pretty(&messages)?);
    } else {
        for (i, msg) in messages.iter().enumerate() {
            if i > 0 {
                println!("\n---\n");
            }
            println!("{msg}");
        }
    }
    Ok(())
}
