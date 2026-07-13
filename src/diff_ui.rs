use crate::git::Snapshot;
use console::style;

pub fn render(snapshot: &Snapshot, stat_only: bool) {
    let stats = Stats::from_diff(&snapshot.diff);
    println!(
        "{} {}  {}  {}",
        style("Changes").bold(),
        style(format!("({})", snapshot.label)).dim(),
        style(format!("+{}", stats.additions)).green(),
        style(format!("-{}", stats.deletions)).red()
    );
    println!();
    for line in snapshot.status.lines() {
        let (code, path) = split_status(line);
        println!("  {}  {}", status_style(code), path);
    }
    if snapshot.truncated {
        println!("\n  {} Diff limited by max_diff_bytes", style("!").yellow());
    }
    if snapshot.redactions > 0 {
        println!(
            "  {} {} potential secret(s) redacted from this view",
            style("●").yellow(),
            snapshot.redactions
        );
    }
    if stat_only {
        return;
    }
    println!("\n{}\n", style("─".repeat(72)).dim());
    for line in snapshot.diff.lines() {
        print_diff_line(line);
    }
}

fn print_diff_line(line: &str) {
    if line.starts_with("diff --git ") {
        println!("{}", style(line).cyan().bold());
    } else if line.starts_with("@@") {
        println!("{}", style(line).yellow());
    } else if line.starts_with("+++") || line.starts_with("---") {
        println!("{}", style(line).dim());
    } else if line.starts_with('+') {
        println!("{}", style(line).green());
    } else if line.starts_with('-') {
        println!("{}", style(line).red());
    } else if line.starts_with('[') {
        println!("{}", style(line).yellow());
    } else {
        println!("{line}");
    }
}

fn split_status(line: &str) -> (&str, &str) {
    if line.len() >= 3 {
        (&line[..2], line[3..].trim())
    } else {
        (line, "")
    }
}

fn status_style(code: &str) -> console::StyledObject<String> {
    let value = code.to_owned();
    if code.contains('A') || code == "??" {
        style(value).green().bold()
    } else if code.contains('D') {
        style(value).red().bold()
    } else if code.contains('M') {
        style(value).yellow().bold()
    } else {
        style(value).cyan().bold()
    }
}

#[derive(Default)]
struct Stats {
    additions: usize,
    deletions: usize,
}
impl Stats {
    fn from_diff(diff: &str) -> Self {
        let mut stats = Self::default();
        for line in diff.lines() {
            if line.starts_with('+') && !line.starts_with("+++") {
                stats.additions += 1;
            }
            if line.starts_with('-') && !line.starts_with("---") {
                stats.deletions += 1;
            }
        }
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn counts_content_lines_without_headers() {
        let stats = Stats::from_diff("--- a/x\n+++ b/x\n-old\n+new\n+more");
        assert_eq!((stats.additions, stats.deletions), (2, 1));
    }
    #[test]
    fn splits_porcelain_status() {
        assert_eq!(split_status(" M src/main.rs"), (" M", "src/main.rs"));
    }
}
