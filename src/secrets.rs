pub struct Redacted {
    pub text: String,
    pub count: usize,
}

pub fn redact(diff: &str) -> Redacted {
    let mut output = Vec::new();
    let mut count = 0;
    let mut private_key = false;
    for line in diff.lines() {
        let marker = line.chars().next().filter(|c| matches!(c, '+' | '-' | ' '));
        let content = marker.map_or(line, |_| &line[1..]);
        if content.contains("-----BEGIN") && content.contains("PRIVATE KEY-----") {
            private_key = true;
            count += 1;
            output.push(with_marker(marker, "[PRIVATE KEY REDACTED BY GITTY]"));
            continue;
        }
        if private_key {
            if content.contains("-----END") && content.contains("PRIVATE KEY-----") {
                private_key = false;
            }
            continue;
        }
        if is_diff_header(line) {
            output.push(line.to_owned());
            continue;
        }
        if let Some(redacted) = redact_assignment(content) {
            count += 1;
            output.push(with_marker(marker, &redacted));
            continue;
        }
        let (redacted, found) = redact_known_tokens(content);
        count += found;
        output.push(with_marker(marker, &redacted));
    }
    Redacted {
        text: output.join("\n") + if diff.ends_with('\n') { "\n" } else { "" },
        count,
    }
}

fn with_marker(marker: Option<char>, content: &str) -> String {
    marker.map_or_else(|| content.to_owned(), |marker| format!("{marker}{content}"))
}

fn is_diff_header(line: &str) -> bool {
    line.starts_with("+++")
        || line.starts_with("---")
        || line.starts_with("diff --git")
        || line.starts_with("@@")
}

fn redact_assignment(content: &str) -> Option<String> {
    let separator = content.find('=').or_else(|| content.find(':'))?;
    let key = content[..separator]
        .trim()
        .trim_matches(|c: char| matches!(c, '"' | '\''));
    let key = key
        .rsplit(|c: char| !c.is_ascii_alphanumeric() && c != '_')
        .next()?
        .to_ascii_uppercase();
    let sensitive = [
        "API_KEY",
        "TOKEN",
        "SECRET",
        "PASSWORD",
        "DATABASE_URL",
        "PRIVATE_KEY",
    ];
    if !sensitive
        .iter()
        .any(|name| key == *name || key.ends_with(&format!("_{name}")))
    {
        return None;
    }
    let value = content[separator + 1..].trim();
    if value.len() < 6 || value.eq_ignore_ascii_case("none") || value.contains("String") {
        return None;
    }
    Some(format!(
        "{}{} [REDACTED BY GITTY]",
        &content[..separator],
        &content[separator..=separator]
    ))
}

fn redact_known_tokens(content: &str) -> (String, usize) {
    let mut text = content.to_owned();
    let mut count = 0;
    for prefix in ["github_pat_", "ghp_", "gho_", "xoxb-", "xoxp-", "sk-"] {
        while let Some(start) = text.find(prefix) {
            let end = text[start..]
                .find(|c: char| !(c.is_ascii_alphanumeric() || matches!(c, '_' | '-')))
                .map_or(text.len(), |offset| start + offset);
            if end - start < prefix.len() + 8 {
                break;
            }
            text.replace_range(start..end, "[TOKEN REDACTED BY GITTY]");
            count += 1;
        }
    }
    (text, count)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn redacts_assignments_and_tokens() {
        let result =
            redact("+API_KEY=super-secret-value\n+token = \"ghp_1234567890abcdef\"\n context\n");
        assert_eq!(result.count, 2);
        assert!(!result.text.contains("super-secret-value"));
        assert!(!result.text.contains("ghp_1234567890abcdef"));
    }
    #[test]
    fn preserves_diff_headers() {
        let result = redact("diff --git a/x b/x\nindex abc..def 100644\n--- a/x\n+++ b/x\n");
        assert_eq!(
            result.text,
            "diff --git a/x b/x\nindex abc..def 100644\n--- a/x\n+++ b/x\n"
        );
    }
}
