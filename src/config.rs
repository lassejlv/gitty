use crate::{ai::ProviderChoice, cli::MessageStyle};
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub provider: Option<ProviderChoice>,
    pub model: Option<String>,
    pub style: Option<MessageStyle>,
    #[serde(rename = "type")]
    pub commit_type: Option<String>,
    pub scope: Option<String>,
    pub candidates: Option<u8>,
    pub max_diff_bytes: Option<usize>,
    pub language: Option<String>,
    pub allowed_types: Option<Vec<String>>,
    pub allowed_scopes: Option<Vec<String>>,
}

pub struct LoadedConfig {
    pub value: Config,
    pub sources: Vec<PathBuf>,
}

impl Config {
    pub fn load(repo: &Path) -> Result<LoadedConfig> {
        let mut value = Config::default();
        let mut sources = Vec::new();
        if let Some(path) = global_path() {
            load_one(&path, &mut value, &mut sources)?;
        }
        load_one(&repo.join(".gitty.toml"), &mut value, &mut sources)?;
        value.apply_defaults();
        validate(&value)?;
        Ok(LoadedConfig { value, sources })
    }

    fn overlay(&mut self, newer: Config) {
        macro_rules! take {
            ($field:ident) => {
                if newer.$field.is_some() {
                    self.$field = newer.$field;
                }
            };
        }
        take!(provider);
        take!(model);
        take!(style);
        take!(commit_type);
        take!(scope);
        take!(candidates);
        take!(max_diff_bytes);
        take!(language);
        take!(allowed_types);
        take!(allowed_scopes);
    }

    fn apply_defaults(&mut self) {
        self.provider.get_or_insert_default();
        self.style.get_or_insert_default();
        self.candidates.get_or_insert(1);
        self.max_diff_bytes.get_or_insert(120_000);
        self.language.get_or_insert_with(|| "English".into());
    }
}

fn load_one(path: &Path, value: &mut Config, sources: &mut Vec<PathBuf>) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    let text =
        fs::read_to_string(path).with_context(|| format!("could not read {}", path.display()))?;
    let parsed = toml::from_str(&text)
        .with_context(|| format!("invalid configuration in {}", path.display()))?;
    value.overlay(parsed);
    sources.push(path.to_owned());
    Ok(())
}

fn validate(config: &Config) -> Result<()> {
    if config
        .candidates
        .is_some_and(|count| !(1..=5).contains(&count))
    {
        bail!("config candidates must be between 1 and 5");
    }
    if config.max_diff_bytes == Some(0) {
        bail!("config max_diff_bytes must be greater than zero");
    }
    if (config.commit_type.is_some() || config.scope.is_some())
        && !matches!(config.style, Some(MessageStyle::Conventional))
    {
        bail!("config type and scope require style = \"conventional\"");
    }
    Ok(())
}

pub fn init(repo: Option<&Path>, global: bool) -> Result<PathBuf> {
    let path = if global {
        global_path().context("could not determine user config directory")?
    } else {
        repo.context("repository required")?.join(".gitty.toml")
    };
    if path.exists() {
        bail!("{} already exists", path.display());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, TEMPLATE).with_context(|| format!("could not write {}", path.display()))?;
    Ok(path)
}

fn global_path() -> Option<PathBuf> {
    env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .map(|p| p.join("gitty/config.toml"))
        .or_else(|| {
            env::var_os("HOME")
                .map(PathBuf::from)
                .map(|p| p.join(".config/gitty/config.toml"))
        })
}

const TEMPLATE: &str = r#"# CLI flags override repository config, which overrides user config.
provider = "auto"
style = "conventional"
candidates = 1
max_diff_bytes = 120000
language = "English"
allowed_types = ["feat", "fix", "docs", "refactor", "test", "chore"]
# model = "your-provider-model"
# type = "feat"
# scope = "cli"
# allowed_scopes = ["cli", "config", "providers"]
"#;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn overlay_prefers_new_values() {
        let mut base = Config {
            model: Some("global".into()),
            language: Some("English".into()),
            ..Default::default()
        };
        base.overlay(Config {
            model: Some("repo".into()),
            ..Default::default()
        });
        assert_eq!(base.model.as_deref(), Some("repo"));
        assert_eq!(base.language.as_deref(), Some("English"));
    }
    #[test]
    fn rejects_unknown_keys() {
        assert!(toml::from_str::<Config>("mdoel = 'oops'").is_err());
    }
}
