use anyhow::{Context, Result, bail};
use clap::ValueEnum;
use std::{
    fmt,
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

#[derive(Debug, Clone, Copy, ValueEnum, serde::Deserialize, serde::Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ProviderChoice {
    #[default]
    Auto,
    Codex,
    Claude,
    Opencode,
}
#[derive(Debug, Clone, Copy)]
pub enum Provider {
    Codex,
    Claude,
    Opencode,
}

impl Provider {
    pub fn resolve_all(choice: ProviderChoice) -> Result<Vec<Self>> {
        let choices: &[Provider] = match choice {
            ProviderChoice::Auto => &[Self::Codex, Self::Claude, Self::Opencode],
            ProviderChoice::Codex => &[Self::Codex],
            ProviderChoice::Claude => &[Self::Claude],
            ProviderChoice::Opencode => &[Self::Opencode],
        };
        let providers: Vec<Self> = choices
            .iter()
            .copied()
            .filter(|p| which::which(p.binary()).is_ok())
            .collect();
        if providers.is_empty() {
            bail!("no supported AI CLI found; install codex, claude, or opencode");
        }
        Ok(providers)
    }

    pub fn generate(self, repo: &Path, prompt: &str, model: Option<&str>) -> Result<String> {
        let mut cmd = Command::new(self.binary());
        cmd.current_dir(repo)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        match self {
            Self::Codex => {
                cmd.args([
                    "exec",
                    "--ephemeral",
                    "--sandbox",
                    "read-only",
                    "--color",
                    "never",
                ]);
                if let Some(m) = model {
                    cmd.args(["--model", m]);
                }
                cmd.arg("-");
            }
            Self::Claude => {
                cmd.args([
                    "--print",
                    "--no-session-persistence",
                    "--permission-mode",
                    "dontAsk",
                ]);
                if let Some(m) = model {
                    cmd.args(["--model", m]);
                }
            }
            Self::Opencode => {
                cmd.args(["run", "--format", "json", "--agent", "plan"]);
                if let Some(m) = model {
                    cmd.args(["--model", m]);
                }
                cmd.arg(prompt);
            }
        }
        let mut child = cmd
            .spawn()
            .with_context(|| format!("failed to start {self}"))?;
        if !matches!(self, Self::Opencode) {
            child
                .stdin
                .take()
                .context("provider stdin unavailable")?
                .write_all(prompt.as_bytes())?;
        }
        let output = child.wait_with_output()?;
        if !output.status.success() {
            bail!(
                "{self} exited with {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr).trim()
            );
        }
        let stdout =
            String::from_utf8(output.stdout).context("provider returned non-UTF-8 output")?;
        if matches!(self, Self::Opencode) {
            parse_opencode(&stdout)
        } else {
            Ok(stdout)
        }
    }
    pub fn binary(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::Claude => "claude",
            Self::Opencode => "opencode",
        }
    }
}

pub fn print_provider_status() {
    for provider in [Provider::Codex, Provider::Claude, Provider::Opencode] {
        let status = which::which(provider.binary())
            .map(|path| path.display().to_string())
            .unwrap_or_else(|_| "not installed".to_owned());
        println!("{:<12} {status}", provider.to_string());
    }
}
impl fmt::Display for Provider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Codex => "Codex",
            Self::Claude => "Claude Code",
            Self::Opencode => "OpenCode",
        })
    }
}

fn parse_opencode(output: &str) -> Result<String> {
    let mut text = String::new();
    for line in output.lines() {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(line)
            && v.get("type").and_then(|v| v.as_str()) == Some("text")
            && let Some(s) = v
                .get("part")
                .and_then(|p| p.get("text"))
                .and_then(|v| v.as_str())
        {
            text.push_str(s);
        }
    }
    if text.trim().is_empty() {
        bail!("OpenCode returned no text");
    }
    Ok(text)
}
