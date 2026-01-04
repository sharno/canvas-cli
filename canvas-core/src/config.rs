use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use canvas_models::{CanvasHost, CanvasToken, CourseId};
use serde::{Deserialize, Serialize};

use crate::{CanvasError, parse_course_id, parse_host, parse_token};

#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub host: CanvasHost,
    pub token: CanvasToken,
}

#[derive(Debug, Clone, Default)]
pub struct DefaultsConfig {
    pub course_id: Option<CourseId>,
}

#[derive(Debug, Clone)]
pub struct CanvasConfig {
    pub auth: AuthConfig,
    pub defaults: DefaultsConfig,
}

#[derive(Debug, Default, Deserialize)]
struct FileConfig {
    auth: Option<FileAuthConfig>,
    defaults: Option<FileDefaultsConfig>,
}

#[derive(Debug, Default, Deserialize)]
struct FileAuthConfig {
    host: Option<String>,
    token: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct FileDefaultsConfig {
    course_id: Option<u64>,
}

#[derive(Debug, Serialize)]
struct WriteConfig {
    auth: WriteAuthConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    defaults: Option<WriteDefaultsConfig>,
}

#[derive(Debug, Serialize)]
struct WriteAuthConfig {
    host: String,
    token: String,
}

#[derive(Debug, Serialize)]
struct WriteDefaultsConfig {
    course_id: u64,
}

pub fn config_path() -> PathBuf {
    let base = dirs::home_dir()
        .map(|home| home.join(".config"))
        .or_else(dirs::config_dir)
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("canvas-cli").join("config.toml")
}

pub fn load_merged_config() -> Result<CanvasConfig, CanvasError> {
    let path = config_path();
    let file_config = load_config_file(&path)?;
    let env_config = FileConfig::from_env();
    let merged = merge_file_configs(file_config, env_config);
    CanvasConfig::try_from(merged)
}

pub fn write_config(path: &Path, config: &CanvasConfig) -> Result<(), CanvasError> {
    let serialized = WriteConfig {
        auth: WriteAuthConfig {
            host: config.auth.host.as_str().to_string(),
            token: config.auth.token.as_str().to_string(),
        },
        defaults: config.defaults.course_id.map(|course_id| WriteDefaultsConfig {
            course_id: course_id.get(),
        }),
    };

    let contents =
        toml::to_string_pretty(&serialized).map_err(|err| CanvasError::ConfigWrite(
            path.display().to_string(),
            err.to_string(),
        ))?;
    fs::write(path, contents).map_err(|err| {
        CanvasError::ConfigWrite(path.display().to_string(), err.to_string())
    })?;
    Ok(())
}

fn load_config_file(path: &Path) -> Result<Option<FileConfig>, CanvasError> {
    if !path.exists() {
        return Ok(None);
    }
    let contents = fs::read_to_string(path)
        .map_err(|err| CanvasError::ConfigRead(path.display().to_string(), err.to_string()))?;
    let parsed = toml::from_str(&contents)
        .map_err(|err| CanvasError::ConfigParse(path.display().to_string(), err.to_string()))?;
    Ok(Some(parsed))
}

fn merge_file_configs(file: Option<FileConfig>, env: FileConfig) -> FileConfig {
    let file = file.unwrap_or_default();
    FileConfig {
        auth: merge_auth(file.auth, env.auth),
        defaults: merge_defaults(file.defaults, env.defaults),
    }
}

fn merge_auth(file: Option<FileAuthConfig>, env: Option<FileAuthConfig>) -> Option<FileAuthConfig> {
    match (file, env) {
        (None, None) => None,
        (Some(file), None) => Some(file),
        (None, Some(env)) => Some(env),
        (Some(file), Some(env)) => Some(FileAuthConfig {
            host: env.host.or(file.host),
            token: env.token.or(file.token),
        }),
    }
}

fn merge_defaults(
    file: Option<FileDefaultsConfig>,
    env: Option<FileDefaultsConfig>,
) -> Option<FileDefaultsConfig> {
    match (file, env) {
        (None, None) => None,
        (Some(file), None) => Some(file),
        (None, Some(env)) => Some(env),
        (Some(file), Some(env)) => Some(FileDefaultsConfig {
            course_id: env.course_id.or(file.course_id),
        }),
    }
}

impl FileConfig {
    fn from_env() -> Self {
        let host = env::var("CANVAS_HOST")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        let token = env::var("CANVAS_TOKEN")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());

        let auth = if host.is_some() || token.is_some() {
            Some(FileAuthConfig { host, token })
        } else {
            None
        };

        Self {
            auth,
            defaults: None,
        }
    }
}

impl TryFrom<FileConfig> for CanvasConfig {
    type Error = CanvasError;

    fn try_from(raw: FileConfig) -> Result<Self, Self::Error> {
        let auth = raw.auth.unwrap_or_default();
        let host_raw = auth
            .host
            .ok_or_else(|| CanvasError::MissingConfig("host (CANVAS_HOST or auth.host)".to_string()))?;
        let token_raw = auth
            .token
            .ok_or_else(|| CanvasError::MissingConfig("token (CANVAS_TOKEN or auth.token)".to_string()))?;
        let host = parse_host(&host_raw)?;
        let token = parse_token(&token_raw)?;

        let defaults = match raw.defaults {
            Some(defaults) => DefaultsConfig {
                course_id: defaults
                    .course_id
                    .map(|course_id| parse_course_id(&course_id.to_string()))
                    .transpose()?,
            },
            None => DefaultsConfig::default(),
        };

        Ok(CanvasConfig {
            auth: AuthConfig { host, token },
            defaults,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{merge_file_configs, CanvasConfig, FileAuthConfig, FileConfig, FileDefaultsConfig};
    use crate::{parse_host, parse_token};

    fn host() -> String {
        "https://example.instructure.com".to_string()
    }

    fn token() -> String {
        "token123".to_string()
    }

    #[test]
    fn env_overrides_file_values() {
        let file = FileConfig {
            auth: Some(FileAuthConfig {
                host: Some(host()),
                token: Some(token()),
            }),
            defaults: None,
        };
        let env = FileConfig {
            auth: Some(FileAuthConfig {
                host: Some("https://override.example.com".to_string()),
                token: None,
            }),
            defaults: None,
        };

        let merged = merge_file_configs(Some(file), env);
        let config = CanvasConfig::try_from(merged).expect("merged config");
        assert_eq!(config.auth.host.as_str(), "https://override.example.com");
        assert_eq!(config.auth.token.as_str(), "token123");
    }

    #[test]
    fn missing_config_is_reported() {
        let raw = FileConfig::default();
        let result = CanvasConfig::try_from(raw);
        assert!(result.is_err());
    }

    #[test]
    fn defaults_parse_course_id() {
        let raw = FileConfig {
            auth: Some(FileAuthConfig {
                host: Some(host()),
                token: Some(token()),
            }),
            defaults: Some(FileDefaultsConfig { course_id: Some(42) }),
        };

        let config = CanvasConfig::try_from(raw).expect("config");
        assert_eq!(config.defaults.course_id.expect("course id").get(), 42);
        let _ = parse_host("https://example.com").expect("host");
        let _ = parse_token("token123").expect("token");
    }
}
