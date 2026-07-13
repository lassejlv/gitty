use anyhow::{Context, Result, bail};
use console::style;
use std::io::{self, IsTerminal, Write};

#[derive(Debug, Clone, Copy)]
pub enum Action {
    Print,
    Copy,
    Commit,
    CommitAndPush,
}
pub enum Decision {
    Accept { message: String, action: Action },
    Regenerate,
    Cancel,
}

pub fn ensure_terminal() -> Result<()> {
    if !io::stdin().is_terminal() || !io::stderr().is_terminal() {
        bail!("--interactive requires a terminal");
    }
    Ok(())
}

pub fn choose(messages: Vec<String>) -> Result<Decision> {
    ensure_terminal()?;
    eprintln!();
    for (index, message) in messages.iter().enumerate() {
        eprintln!(
            "{}\n{}\n",
            style(format!("  {}", index + 1)).cyan().bold(),
            indent(message)
        );
    }
    let selected = loop {
        let input = ask(&format!(
            "Choose 1-{}, [r]egenerate, or [q]uit: ",
            messages.len()
        ))?;
        if input.eq_ignore_ascii_case("r") {
            return Ok(Decision::Regenerate);
        }
        if input.eq_ignore_ascii_case("q") {
            return Ok(Decision::Cancel);
        }
        if let Ok(index) = input.parse::<usize>()
            && let Some(message) = messages.get(index.saturating_sub(1))
        {
            break message.clone();
        }
        eprintln!("{} Pick a candidate number, r, or q.", style("!").yellow());
    };
    action_menu(selected)
}

fn action_menu(mut message: String) -> Result<Decision> {
    loop {
        eprintln!(
            "{}",
            style("[p]rint  [c]opy  co[m]mit  commit+p[u]sh  [e]dit  [r]egenerate  [q]uit").dim()
        );
        match ask("Action [p]: ")?.to_ascii_lowercase().as_str() {
            "" | "p" => return accept(message, Action::Print),
            "c" => return accept(message, Action::Copy),
            "m" => return accept(message, Action::Commit),
            "u" => return accept(message, Action::CommitAndPush),
            "e" => {
                message = edit(&message)?;
                eprintln!("\n{}\n", indent(&message));
            }
            "r" => return Ok(Decision::Regenerate),
            "q" => return Ok(Decision::Cancel),
            _ => eprintln!("{} Unknown action.", style("!").yellow()),
        }
    }
}

fn accept(message: String, action: Action) -> Result<Decision> {
    Ok(Decision::Accept { message, action })
}

fn edit(current: &str) -> Result<String> {
    eprintln!("Enter the replacement message. Finish with a line containing only a dot.");
    eprintln!("{}", style(format!("Current: {current}")).dim());
    let mut lines = Vec::new();
    loop {
        let line = ask("│ ")?;
        if line == "." {
            break;
        }
        lines.push(line);
    }
    let edited = lines.join("\n").trim().to_owned();
    if edited.is_empty() {
        bail!("edited message cannot be empty");
    }
    Ok(edited)
}

fn ask(prompt: &str) -> Result<String> {
    eprint!("{prompt}");
    io::stderr().flush()?;
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .context("failed to read terminal input")?;
    if input.is_empty() {
        bail!("interactive input closed");
    }
    Ok(input.trim_end().to_owned())
}

fn indent(message: &str) -> String {
    message
        .lines()
        .map(|line| format!("    {line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn indents_multiline_messages() {
        assert_eq!(
            indent("feat: add picker\n\nExplain why"),
            "    feat: add picker\n    \n    Explain why"
        );
    }
}
