mod ai;
mod cli;
mod clipboard;
mod config;
mod git;
mod message;
mod prompt;
mod ui;

use anyhow::{Context, Result, bail};
use clap::{CommandFactory, Parser};
use clap_complete::generate;
use cli::{Cli, Commands, ConfigCommands};
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
    if std::env::args_os().len() == 1 {
        ui::about();
        return Ok(());
    }
    let cli = Cli::parse();
    match &cli.command {
        Some(Commands::Completions { shell }) => {
            generate(*shell, &mut Cli::command(), "gitty", &mut io::stdout());
            return Ok(());
        }
        Some(Commands::Providers) => {
            ai::print_provider_status();
            return Ok(());
        }
        Some(Commands::Config {
            command: ConfigCommands::Init { global },
        }) => {
            let repo = if *global {
                None
            } else {
                Some(git::Repository::discover(cli.repo.as_deref())?)
            };
            let path = config::init(repo.as_ref().map(|repo| repo.root.as_path()), *global)?;
            println!("created {}", path.display());
            return Ok(());
        }
        Some(Commands::Config {
            command: ConfigCommands::Show,
        }) => {
            let repo = git::Repository::discover(cli.repo.as_deref())?;
            let loaded = config::Config::load(&repo.root)?;
            println!("{}", toml::to_string_pretty(&loaded.value)?);
            for path in loaded.sources {
                eprintln!("loaded {}", path.display());
            }
            return Ok(());
        }
        Some(Commands::Generate) => {}
        None => {}
    }
    let repo = git::Repository::discover(cli.repo.as_deref())?;
    let loaded = config::Config::load(&repo.root)?;
    let settings = loaded.value;
    let provider_choice = cli.provider.or(settings.provider).unwrap_or_default();
    let model = cli.model.as_deref().or(settings.model.as_deref());
    let style = cli.style.or(settings.style).unwrap_or_default();
    let commit_type = cli
        .commit_type
        .as_deref()
        .or(settings.commit_type.as_deref());
    let scope = cli.scope.as_deref().or(settings.scope.as_deref());
    let candidates = cli.candidates.or(settings.candidates).unwrap_or(1);
    let max_diff_bytes = cli
        .max_diff_bytes
        .or(settings.max_diff_bytes)
        .unwrap_or(120_000);
    if (commit_type.is_some() || scope.is_some())
        && !matches!(style, cli::MessageStyle::Conventional)
    {
        bail!("--type and --scope require --style conventional");
    }
    if cli.commit && candidates != 1 {
        bail!("--commit requires exactly one candidate");
    }
    if cli.commit && matches!(cli.effective_changes(), cli::ChangeSelection::All) {
        bail!("--commit cannot use all changes; stage what you want and use --changes staged");
    }
    if cli.commit && !repo.has_staged_changes()? {
        bail!("--commit requires staged changes");
    }
    if cli.push {
        repo.ensure_push_target()?;
    }
    let snapshot = repo.snapshot(cli.effective_changes(), max_diff_bytes)?;
    if snapshot.diff.trim().is_empty() {
        bail!("no changes found (stage files, or pass --all to include unstaged changes)");
    }
    let request = prompt::Request {
        snapshot: &snapshot,
        style,
        hint: cli.hint.as_deref(),
        candidates,
        commit_type,
        scope,
        language: settings.language.as_deref(),
        allowed_types: settings.allowed_types.as_deref(),
        allowed_scopes: settings.allowed_scopes.as_deref(),
    };
    let prompt = request.render();
    if cli.dry_run {
        print!("{prompt}");
        return Ok(());
    }
    let provider = ai::Provider::resolve(provider_choice)?;
    if !cli.quiet {
        ui::generating(provider, snapshot.label);
    }
    let raw = provider
        .generate(&repo.root, &prompt, model)
        .with_context(|| format!("{provider} could not generate a commit message"))?;
    let messages = message::parse_candidates(&raw, candidates)?;
    if cli.copy {
        clipboard::copy(&messages[0])?;
        if !cli.quiet {
            ui::success("Copied first candidate to clipboard");
        }
    }
    if cli.commit {
        repo.commit(&messages[0])?;
        if !cli.quiet {
            ui::success("Created commit");
        }
    }
    if cli.push {
        repo.push()?;
        if !cli.quiet {
            ui::success("Pushed current branch");
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
