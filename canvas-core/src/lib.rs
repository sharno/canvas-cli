mod auth;
mod api;
mod config;

use canvas_models::{
    AssignmentId, CanvasHost, CanvasToken, CourseId, DueDate, PublishState, RubricSelection,
    Score, SubmissionId, UserId,
};
use thiserror::Error;

pub use auth::auth_check;
pub use api::{ApiError, ApiErrorCode, CanvasClient, RetryPolicy};
pub use config::{config_path, load_merged_config, write_config, AuthConfig, CanvasConfig, DefaultsConfig};

#[derive(Debug, Error)]
pub enum CanvasError {
    #[error("invalid course id: {0}")]
    InvalidCourseId(String),
    #[error("invalid assignment id: {0}")]
    InvalidAssignmentId(String),
    #[error("invalid user id: {0}")]
    InvalidUserId(String),
    #[error("invalid submission id: {0}")]
    InvalidSubmissionId(String),
    #[error("invalid score: {0}")]
    InvalidScore(String),
    #[error("invalid due date: {0}")]
    InvalidDueDate(String),
    #[error("invalid publish state: {0}")]
    InvalidPublishState(String),
    #[error("invalid rubric selection: {0}")]
    InvalidRubricSelection(String),
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
    #[error(transparent)]
    Api(#[from] ApiError),
}

pub fn parse_course_id(raw: &str) -> Result<CourseId, CanvasError> {
    raw.parse::<CourseId>()
        .map_err(|_| CanvasError::InvalidCourseId(raw.to_string()))
}

pub fn parse_assignment_id(raw: &str) -> Result<AssignmentId, CanvasError> {
    raw.parse::<AssignmentId>()
        .map_err(|_| CanvasError::InvalidAssignmentId(raw.to_string()))
}

pub fn parse_user_id(raw: &str) -> Result<UserId, CanvasError> {
    raw.parse::<UserId>()
        .map_err(|_| CanvasError::InvalidUserId(raw.to_string()))
}

pub fn parse_submission_id(raw: &str) -> Result<SubmissionId, CanvasError> {
    raw.parse::<SubmissionId>()
        .map_err(|_| CanvasError::InvalidSubmissionId(raw.to_string()))
}

pub fn parse_score(raw: &str, max: f64) -> Result<Score, CanvasError> {
    let value = raw
        .trim()
        .parse::<f64>()
        .map_err(|_| CanvasError::InvalidScore(raw.to_string()))?;
    Score::try_from((value, max)).map_err(|_| CanvasError::InvalidScore(raw.to_string()))
}

pub fn parse_due_date(raw: &str) -> Result<DueDate, CanvasError> {
    raw.parse::<DueDate>()
        .map_err(|_| CanvasError::InvalidDueDate(raw.to_string()))
}

pub fn parse_publish_state(raw: &str) -> Result<PublishState, CanvasError> {
    raw.parse::<PublishState>()
        .map_err(|_| CanvasError::InvalidPublishState(raw.to_string()))
}

pub fn parse_rubric_selection(raw: &str) -> Result<RubricSelection, CanvasError> {
    raw.parse::<RubricSelection>()
        .map_err(|_| CanvasError::InvalidRubricSelection(raw.to_string()))
}

pub fn parse_host(raw: &str) -> Result<CanvasHost, CanvasError> {
    raw.parse::<CanvasHost>()
        .map_err(|_| CanvasError::InvalidHost(raw.to_string()))
}

pub fn parse_token(raw: &str) -> Result<CanvasToken, CanvasError> {
    raw.parse::<CanvasToken>()
        .map_err(|_| CanvasError::InvalidToken)
}
