use anyhow::{Context, Result, bail};
use std::{
    io::Write,
    process::{Command, Stdio},
};

pub fn copy(text: &str) -> Result<()> {
    let candidates: &[(&str, &[&str])] = if cfg!(target_os = "macos") {
        &[("pbcopy", &[])]
    } else if cfg!(target_os = "windows") {
        &[("clip", &[])]
    } else {
        &[
            ("wl-copy", &[]),
            ("xclip", &["-selection", "clipboard"]),
            ("xsel", &["--clipboard", "--input"]),
        ]
    };
    let Some((program, args)) = candidates
        .iter()
        .find(|(program, _)| which::which(program).is_ok())
    else {
        bail!("no clipboard command found (install wl-copy, xclip, or xsel)");
    };
    let mut child = Command::new(program)
        .args(*args)
        .stdin(Stdio::piped())
        .spawn()
        .with_context(|| format!("failed to start {program}"))?;
    child
        .stdin
        .take()
        .context("clipboard stdin unavailable")?
        .write_all(text.as_bytes())?;
    let status = child.wait()?;
    if !status.success() {
        bail!("{program} failed with {status}");
    }
    Ok(())
}
