use anyhow::{Result, bail};

pub fn parse_candidates(raw: &str, expected: u8) -> Result<Vec<String>> {
    let cleaned = raw
        .trim()
        .trim_start_matches("```text")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();
    let mut messages: Vec<String> = cleaned
        .split("\n---\n")
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned)
        .collect();
    if messages.is_empty() {
        bail!("provider returned an empty response");
    }
    messages.truncate(expected as usize);
    for message in &messages {
        let subject = message.lines().next().unwrap_or_default();
        if subject.is_empty() {
            bail!("provider returned a message without a subject");
        }
        if subject.chars().count() > 100 {
            bail!("provider returned an implausibly long subject: {subject}");
        }
    }
    Ok(messages)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn strips_fences_and_splits_candidates() {
        assert_eq!(
            parse_candidates("```text\nfeat: add parser\n---\nfix: handle EOF\n```", 2).unwrap(),
            ["feat: add parser", "fix: handle EOF"]
        );
    }
}
