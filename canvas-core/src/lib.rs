mod auth;
mod api;
mod assignments;
mod config;
mod courses;

use canvas_models::{
    AssignmentId, AssignmentName, CanvasHost, CanvasToken, CourseDates, CourseId,
    CourseVisibility, DueDate, GradingSchemeId, PointsPossible, PublishState,
    RubricAssessment, RubricSelection, Score, SubmissionId, UserId,
};
use thiserror::Error;

pub use auth::auth_check;
pub use api::{ApiError, ApiErrorCode, CanvasClient, RetryPolicy};
pub use assignments::{
    create_assignment, delete_assignment, ensure_assignment_points, get_assignment,
    grade_submission, list_assignments, list_submissions, update_assignment,
    AssignmentCreateInput, AssignmentSummary, AssignmentUpdateInput, SubmissionSummary,
};
pub use config::{config_path, load_merged_config, write_config, AuthConfig, CanvasConfig, DefaultsConfig};
pub use courses::{get_course, list_courses, persist_default_course, update_course, CourseSettingsUpdate, CourseSummary};

#[derive(Debug, Error)]
pub enum CanvasError {
    #[error("invalid course id: {0}")]
    InvalidCourseId(String),
    #[error("invalid assignment id: {0}")]
    InvalidAssignmentId(String),
    #[error("invalid assignment name: {0}")]
    InvalidAssignmentName(String),
    #[error("invalid user id: {0}")]
    InvalidUserId(String),
    #[error("invalid submission id: {0}")]
    InvalidSubmissionId(String),
    #[error("invalid score: {0}")]
    InvalidScore(String),
    #[error("invalid points possible: {0}")]
    InvalidPointsPossible(String),
    #[error("invalid due date: {0}")]
    InvalidDueDate(String),
    #[error("invalid publish state: {0}")]
    InvalidPublishState(String),
    #[error("invalid rubric selection: {0}")]
    InvalidRubricSelection(String),
    #[error("invalid rubric assessment: {0}")]
    InvalidRubricAssessment(String),
    #[error("invalid course visibility: {0}")]
    InvalidCourseVisibility(String),
    #[error("invalid grading scheme id: {0}")]
    InvalidGradingSchemeId(String),
    #[error("invalid course dates: {0}")]
    InvalidCourseDates(String),
    #[error("invalid course update: {0}")]
    InvalidCourseUpdate(String),
    #[error("invalid assignment update: {0}")]
    InvalidAssignmentUpdate(String),
    #[error("missing assignment points for assignment id {0}")]
    MissingAssignmentPoints(u64),
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

pub fn parse_assignment_name(raw: &str) -> Result<AssignmentName, CanvasError> {
    raw.parse::<AssignmentName>()
        .map_err(|_| CanvasError::InvalidAssignmentName(raw.to_string()))
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
    Score::new(value, max).map_err(|_| CanvasError::InvalidScore(raw.to_string()))
}

pub fn parse_points_possible(raw: &str) -> Result<PointsPossible, CanvasError> {
    raw.parse::<PointsPossible>()
        .map_err(|_| CanvasError::InvalidPointsPossible(raw.to_string()))
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

pub fn parse_rubric_assessment(raw: &str) -> Result<RubricAssessment, CanvasError> {
    let value: serde_json::Value =
        serde_json::from_str(raw).map_err(|_| CanvasError::InvalidRubricAssessment(raw.to_string()))?;
    match value {
        serde_json::Value::Object(entries) => RubricAssessment::new(entries)
            .map_err(|_| CanvasError::InvalidRubricAssessment(raw.to_string())),
        _ => Err(CanvasError::InvalidRubricAssessment(raw.to_string())),
    }
}

pub fn parse_course_visibility(raw: &str) -> Result<CourseVisibility, CanvasError> {
    raw.parse::<CourseVisibility>()
        .map_err(|_| CanvasError::InvalidCourseVisibility(raw.to_string()))
}

pub fn parse_grading_scheme_id(raw: &str) -> Result<GradingSchemeId, CanvasError> {
    raw.parse::<GradingSchemeId>()
        .map_err(|_| CanvasError::InvalidGradingSchemeId(raw.to_string()))
}

pub fn parse_course_dates(
    start: Option<DueDate>,
    end: Option<DueDate>,
) -> Result<CourseDates, CanvasError> {
    CourseDates::new(start, end).map_err(|_| CanvasError::InvalidCourseDates("start_after_end".to_string()))
}

pub fn parse_host(raw: &str) -> Result<CanvasHost, CanvasError> {
    raw.parse::<CanvasHost>()
        .map_err(|_| CanvasError::InvalidHost(raw.to_string()))
}

pub fn parse_token(raw: &str) -> Result<CanvasToken, CanvasError> {
    raw.parse::<CanvasToken>()
        .map_err(|_| CanvasError::InvalidToken)
}
