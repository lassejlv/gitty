mod ai;
mod cli;
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
    if let Some(Commands::Completions { shell }) = cli.command {
        generate(shell, &mut Cli::command(), "gitty", &mut io::stdout());
        return Ok(());
    }
    let repo = git::Repository::discover(cli.repo.as_deref())?;
    let snapshot = repo.snapshot(cli.effective_changes(), cli.max_diff_bytes)?;
    if snapshot.diff.trim().is_empty() {
        bail!("no changes found (stage files, or pass --all to include unstaged changes)");
    }
    let provider = ai::Provider::resolve(cli.provider)?;
    if !cli.quiet {
        eprintln!(
            "{} asking {provider} to describe {}…",
            console::style("gitty").bold(),
            snapshot.label
        );
    }
    let request = prompt::Request {
        snapshot: &snapshot,
        style: cli.style,
        hint: cli.hint.as_deref(),
        candidates: cli.candidates,
    };
    let raw = provider
        .generate(&repo.root, &request.render(), cli.model.as_deref())
        .with_context(|| format!("{provider} could not generate a commit message"))?;
    let messages = message::parse_candidates(&raw, cli.candidates)?;
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
