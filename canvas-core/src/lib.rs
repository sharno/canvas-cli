mod auth;
mod config;

use canvas_models::{CanvasHost, CanvasToken, CourseId};
use thiserror::Error;

pub use auth::auth_check;
pub use config::{config_path, load_merged_config, write_config, AuthConfig, CanvasConfig, DefaultsConfig};

#[derive(Debug, Error)]
pub enum CanvasError {
    #[error("invalid course id: {0}")]
    InvalidCourseId(String),
    #[error("invalid host: {0}")]
    InvalidHost(String),
    #[error("invalid token")]
    InvalidToken,
    #[error("missing configuration: {0}")]
    MissingConfig(String),
    #[error("failed to read config file at {0}: {1}")]
    ConfigRead(String, String),
    #[error("failed to parse config file at {0}: {1}")]
    ConfigParse(String, String),
    #[error("failed to write config file at {0}: {1}")]
    ConfigWrite(String, String),
    #[error("auth check failed: {0}")]
    AuthCheckFailed(String),
    #[error("http error: {0}")]
    Http(String),
}

pub fn parse_course_id(raw: &str) -> Result<CourseId, CanvasError> {
    raw.parse::<CourseId>()
        .map_err(|_| CanvasError::InvalidCourseId(raw.to_string()))
}

pub fn parse_host(raw: &str) -> Result<CanvasHost, CanvasError> {
    raw.parse::<CanvasHost>()
        .map_err(|_| CanvasError::InvalidHost(raw.to_string()))
}

pub fn parse_token(raw: &str) -> Result<CanvasToken, CanvasError> {
    raw.parse::<CanvasToken>()
        .map_err(|_| CanvasError::InvalidToken)
}
