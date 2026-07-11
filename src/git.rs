use crate::cli::ChangeSelection;
use anyhow::{Context, Result, bail};
use std::{
    path::{Path, PathBuf},
    process::Command,
};

pub struct Repository {
    pub root: PathBuf,
}
pub struct Snapshot {
    pub status: String,
    pub diff: String,
    pub label: &'static str,
    pub truncated: bool,
}

impl Repository {
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
        let status = self.git(&["status", "--short"])?;
        let staged = !self
            .git(&["diff", "--cached", "--name-only"])?
            .trim()
            .is_empty();
        let selection = match requested {
            ChangeSelection::Auto if staged => ChangeSelection::Staged,
            ChangeSelection::Auto => ChangeSelection::All,
            other => other,
        };
        let (mut diff, label) = match selection {
            ChangeSelection::Staged => (
                self.git(&["diff", "--cached", "--no-ext-diff", "--no-color"])?,
                "staged changes",
            ),
            _ => {
                let mut all = self.git(&["diff", "--cached", "--no-ext-diff", "--no-color"])?;
                all.push_str(&self.git(&["diff", "--no-ext-diff", "--no-color"])?);
                all.push_str(&self.untracked()?);
                (all, "all changes")
            }
        };
        let truncated = diff.len() > limit;
        if truncated {
            diff.truncate(diff.floor_char_boundary(limit));
            diff.push_str("\n\n[diff truncated by gitty]\n");
        }
        Ok(Snapshot {
            status,
            diff,
            label,
            truncated,
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
