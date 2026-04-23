use std::env;
use std::fmt;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

const APP_DIR_NAME: &str = ".config/localmind";
const CONFIG_FILE_NAME: &str = "config.toml";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ModelProvider {
    #[default]
    OpenAi,
    Anthropic,
    Gemini,
    Local,
}

impl ModelProvider {
    pub fn variants() -> &'static [ModelProvider] {
        &[
            ModelProvider::OpenAi,
            ModelProvider::Anthropic,
            ModelProvider::Gemini,
            ModelProvider::Local,
        ]
    }
}

impl fmt::Display for ModelProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            ModelProvider::OpenAi => "openai",
            ModelProvider::Anthropic => "anthropic",
            ModelProvider::Gemini => "gemini",
            ModelProvider::Local => "local",
        })
    }
}

impl FromStr for ModelProvider {
    type Err = io::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "openai" | "open_ai" => Ok(ModelProvider::OpenAi),
            "anthropic" => Ok(ModelProvider::Anthropic),
            "gemini" => Ok(ModelProvider::Gemini),
            "local" => Ok(ModelProvider::Local),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid model provider",
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    model_provider: ModelProvider,
    model_name: String,
    api_key: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            model_provider: ModelProvider::OpenAi,
            model_name: "gpt-3.5-turbo".to_string(),
            api_key: String::new(),
        }
    }
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load() -> io::Result<Self> {
        let config_path = Self::config_path()?;
        let contents = fs::read_to_string(&config_path)?;
        toml::from_str(&contents).map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("failed to parse {}: {error}", config_path.display()),
            )
        })
    }

    pub fn load_or_setup_interactive() -> io::Result<Self> {
        match Self::load() {
            Ok(config) => Ok(config),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                let config = Self::prompt_new(None)?;
                config.save()?;
                Ok(config)
            }
            Err(error) => Err(error),
        }
    }

    pub fn prompt_new(existing: Option<&Self>) -> io::Result<Self> {
        let default_provider = existing.map(|config| config.model_provider).unwrap_or_default();
        let default_model_name = existing
            .map(|config| config.model_name.as_str())
            .filter(|value| !value.trim().is_empty())
            .unwrap_or("gpt-3.5-turbo");
        let existing_api_key = existing
            .map(|config| config.api_key.as_str())
            .filter(|value| !value.trim().is_empty());

        let model_provider = Self::prompt_model_provider(default_provider)?;
        let model_name = Self::prompt_line(
            "Model name",
            Some(default_model_name),
            true,
        )?;
        let api_key = Self::prompt_password(
            "API key",
            existing_api_key,
            existing.is_none(),
        )?;

        Ok(Self {
            model_provider,
            model_name,
            api_key,
        })
    }

    pub fn edit_interactive() -> io::Result<Self> {
        let existing = Self::load().ok();
        let config = Self::prompt_new(existing.as_ref())?;
        config.save()?;
        Ok(config)
    }

    pub fn reset() -> io::Result<()> {
        let config_path = Self::config_path()?;
        if config_path.exists() {
            fs::remove_file(config_path)?;
        }
        Ok(())
    }

    pub fn save(&self) -> io::Result<()> {
        let config_path = Self::config_path()?;
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let serialized = toml::to_string_pretty(self).map_err(|error| {
            io::Error::new(io::ErrorKind::InvalidData, error.to_string())
        })?;

        let temp_path = config_path.with_extension("toml.tmp");
        if temp_path.exists() {
            let _ = fs::remove_file(&temp_path);
        }

        fs::write(&temp_path, serialized)?;
        fs::rename(&temp_path, &config_path)?;
        Ok(())
    }

    pub fn config_path() -> io::Result<PathBuf> {
        let base_dir = match env::var_os("HOME") {
            Some(home) => PathBuf::from(home).join(APP_DIR_NAME),
            None => env::current_dir()?.join(".localmind"),
        };

        Ok(base_dir.join(CONFIG_FILE_NAME))
    }

    pub fn view_string(&self) -> io::Result<String> {
        Ok(format!(
            "Config file: {}\nModel provider: {}\nModel name: {}\nAPI key: {}\n",
            Self::config_path()?.display(),
            self.model_provider,
            self.model_name,
            self.masked_api_key()
        ))
    }

    pub fn summary(&self) -> String {
        format!("{} / {}", self.model_provider, self.model_name)
    }

    pub fn model_provider(&self) -> ModelProvider {
        self.model_provider
    }

    pub fn model_name(&self) -> &str {
        &self.model_name
    }

    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    fn masked_api_key(&self) -> String {
        if self.api_key.is_empty() {
            return "<empty>".to_string();
        }

        let visible_count = self.api_key.chars().count().min(4);
        let suffix: String = self
            .api_key
            .chars()
            .rev()
            .take(visible_count)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();

        format!("{}{}", "*".repeat(self.api_key.chars().count().saturating_sub(visible_count)), suffix)
    }

    fn prompt_model_provider(default: ModelProvider) -> io::Result<ModelProvider> {
        loop {
            println!("Select model provider:");
            for (index, provider) in ModelProvider::variants().iter().enumerate() {
                let default_marker = if *provider == default { " [default]" } else { "" };
                println!("  {}. {}{}", index + 1, provider, default_marker);
            }

            let selection = Self::prompt_line("Provider", Some(&default.to_string()), true)?;
            if selection.trim().is_empty() {
                return Ok(default);
            }

            if let Ok(provider) = ModelProvider::from_str(&selection) {
                return Ok(provider);
            }

            if let Ok(index) = selection.trim().parse::<usize>() {
                if let Some(provider) = ModelProvider::variants().get(index.saturating_sub(1)) {
                    return Ok(*provider);
                }
            }

            println!("Invalid provider. Try again.");
        }
    }

    fn prompt_line(label: &str, default: Option<&str>, allow_empty: bool) -> io::Result<String> {
        loop {
            if let Some(default) = default {
                print!("{} [{}]: ", label, default);
            } else {
                print!("{}: ", label);
            }
            io::stdout().flush()?;

            let mut value = String::new();
            io::stdin().read_line(&mut value)?;
            let trimmed = value.trim().to_string();

            if trimmed.is_empty() {
                if let Some(default) = default {
                    return Ok(default.to_string());
                }

                if allow_empty {
                    return Ok(String::new());
                }

                println!("{} cannot be empty.", label);
                continue;
            }

            return Ok(trimmed);
        }
    }

    fn prompt_password(
        label: &str,
        existing: Option<&str>,
        require_value: bool,
    ) -> io::Result<String> {
        loop {
            match existing {
                Some(default) => {
                    let value = rpassword::prompt_password(format!("{} [press Enter to keep current]: ", label))?;
                    if value.trim().is_empty() {
                        return Ok(default.to_string());
                    }
                    return Ok(value.trim().to_string());
                }
                None => {
                    let value = rpassword::prompt_password(format!("{}: ", label))?;
                    let trimmed = value.trim().to_string();
                    if trimmed.is_empty() && require_value {
                        println!("{} cannot be empty.", label);
                        continue;
                    }
                    return Ok(trimmed);
                }
            }
        }
    }
}