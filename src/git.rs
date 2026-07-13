use crate::cli::ChangeSelection;
use anyhow::{Context, Result, bail};
use std::{
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

pub struct Repository {
    pub root: PathBuf,
}
pub struct Snapshot {
    pub status: String,
    pub diff: String,
    pub label: &'static str,
    pub truncated: bool,
    pub includes_all_changes: bool,
}

impl Repository {
    pub fn has_staged_changes(&self) -> Result<bool> {
        Ok(!self
            .git(&["diff", "--cached", "--name-only"])?
            .trim()
            .is_empty())
    }

    pub fn commit(&self, message: &str) -> Result<()> {
        let mut child = Command::new("git")
            .arg("-C")
            .arg(&self.root)
            .args(["commit", "--file", "-"])
            .stdin(Stdio::piped())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .context("failed to start git commit")?;
        child
            .stdin
            .take()
            .context("git commit stdin unavailable")?
            .write_all(message.as_bytes())?;
        let status = child.wait()?;
        if !status.success() {
            bail!("git commit failed with {status}");
        }
        Ok(())
    }

    pub fn stage_all(&self) -> Result<()> {
        let status = Command::new("git")
            .arg("-C")
            .arg(&self.root)
            .args(["add", "--all"])
            .status()
            .context("failed to stage all changes")?;
        if !status.success() {
            bail!("git add --all failed with {status}");
        }
        Ok(())
    }

    pub fn ensure_push_target(&self) -> Result<()> {
        let branch = self.git(&["branch", "--show-current"])?;
        if branch.trim().is_empty() {
            bail!("--push is unavailable in detached HEAD state");
        }
        let upstream = Command::new("git")
            .arg("-C")
            .arg(&self.root)
            .args([
                "rev-parse",
                "--abbrev-ref",
                "--symbolic-full-name",
                "@{upstream}",
            ])
            .output()
            .context("failed to inspect Git upstream")?;
        if !upstream.status.success() {
            bail!(
                "current branch has no upstream; run `git push --set-upstream origin {}` once",
                branch.trim()
            );
        }
        Ok(())
    }

    pub fn push(&self) -> Result<()> {
        let status = Command::new("git")
            .arg("-C")
            .arg(&self.root)
            .arg("push")
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .context("failed to start git push")?;
        if !status.success() {
            bail!("git push failed with {status}");
        }
        Ok(())
    }

    pub fn discover(path: Option<&Path>) -> Result<Self> {
        let cwd = path.unwrap_or_else(|| Path::new("."));
        let out = Command::new("git")
            .arg("-C")
            .arg(cwd)
            .args(["rev-parse", "--show-toplevel"])
            .output()
            .context("could not run git; is it installed?")?;
        if !out.status.success() {
            bail!("not inside a Git repository");
        }
        Ok(Self {
            root: PathBuf::from(String::from_utf8_lossy(&out.stdout).trim()),
        })
    }

    pub fn snapshot(&self, requested: ChangeSelection, limit: usize) -> Result<Snapshot> {
        let full_status = self.git(&["status", "--short"])?;
        let staged = !self
            .git(&["diff", "--cached", "--name-only"])?
            .trim()
            .is_empty();
        let selection = match requested {
            ChangeSelection::Auto if staged => ChangeSelection::Staged,
            ChangeSelection::Auto => ChangeSelection::All,
            other => other,
        };
        let includes_all_changes = matches!(selection, ChangeSelection::All);
        let (mut diff, status, label) = match selection {
            ChangeSelection::Staged => (
                self.git(&["diff", "--cached", "--no-ext-diff", "--no-color"])?,
                staged_status(&full_status),
                "staged changes",
            ),
            _ => {
                let mut all = self.git(&["diff", "--cached", "--no-ext-diff", "--no-color"])?;
                all.push_str(&self.git(&["diff", "--no-ext-diff", "--no-color"])?);
                all.push_str(&self.untracked()?);
                (all, full_status, "all changes")
            }
        };
        let truncated = diff.len() > limit;
        if truncated {
            let mut boundary = limit;
            while !diff.is_char_boundary(boundary) {
                boundary -= 1;
            }
            diff.truncate(boundary);
            diff.push_str("\n\n[diff truncated by gitty]\n");
        }
        Ok(Snapshot {
            status,
            diff,
            label,
            truncated,
            includes_all_changes,
        })
    }

    fn untracked(&self) -> Result<String> {
        let files = self.git(&["ls-files", "--others", "--exclude-standard"])?;
        let mut out = String::new();
        for file in files.lines() {
            out.push_str(&format!("\n--- /dev/null\n+++ b/{file}\n"));
            let path = self.root.join(file);
            if std::fs::symlink_metadata(&path)
                .is_ok_and(|metadata| metadata.file_type().is_symlink())
            {
                out.push_str("+[symbolic link content omitted]\n");
                continue;
            }
            match std::fs::read(path) {
                Ok(bytes) if bytes.iter().take(8_192).all(|b| *b != 0) => {
                    let text = String::from_utf8_lossy(&bytes[..bytes.len().min(32_000)]);
                    for line in text.lines() {
                        out.push_str(&format!("+{line}\n"));
                    }
                    if bytes.len() > 32_000 {
                        out.push_str("+[file truncated]\n");
                    }
                }
                Ok(_) => out.push_str("+[binary file]\n"),
                Err(_) => out.push_str("+[unreadable file]\n"),
            }
        }
        Ok(out)
    }

    fn git(&self, args: &[&str]) -> Result<String> {
        let out = Command::new("git")
            .arg("-C")
            .arg(&self.root)
            .args(args)
            .output()
            .with_context(|| format!("failed to run git {}", args.join(" ")))?;
        if !out.status.success() {
            bail!(
                "git {} failed: {}",
                args.join(" "),
                String::from_utf8_lossy(&out.stderr).trim()
            );
        }
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    }
}

fn staged_status(status: &str) -> String {
    status
        .lines()
        .filter(|line| {
            line.as_bytes()
                .first()
                .is_some_and(|code| *code != b' ' && *code != b'?')
        })
        .map(|line| format!("{line}\n"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn staged_status_excludes_unstaged_and_untracked_files() {
        let status = "M  staged.rs\n M unstaged.rs\nMM both.rs\n?? new.rs\n";
        assert_eq!(staged_status(status), "M  staged.rs\nMM both.rs\n");
    }
}
