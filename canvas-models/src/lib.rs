use std::str::FromStr;

use serde::Serialize;
use thiserror::Error;
use time::format_description::well_known::Rfc3339;  
use time::OffsetDateTime;
use url::Url;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CourseId(u64);

impl CourseId {
    pub fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum CourseIdParseError {
    #[error("course id must be a positive integer")]
    Invalid,
}

impl FromStr for CourseId {
    type Err = CourseIdParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let trimmed = raw.trim();
        let value = trimmed.parse::<u64>().map_err(|_| CourseIdParseError::Invalid)?;
        if value == 0 {
            return Err(CourseIdParseError::Invalid);
        }
        Ok(Self(value))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AssignmentId(u64);

impl AssignmentId {
    pub fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum AssignmentIdParseError {
    #[error("assignment id must be a positive integer")]
    Invalid,
}

impl FromStr for AssignmentId {
    type Err = AssignmentIdParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let trimmed = raw.trim();
        let value = trimmed
            .parse::<u64>()
            .map_err(|_| AssignmentIdParseError::Invalid)?;
        if value == 0 {
            return Err(AssignmentIdParseError::Invalid);
        }
        Ok(Self(value))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AssignmentName(String);

impl AssignmentName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum AssignmentNameParseError {
    #[error("assignment name must not be empty")]
    Empty,
}

impl FromStr for AssignmentName {
    type Err = AssignmentNameParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(AssignmentNameParseError::Empty);
        }
        Ok(Self(trimmed.to_string()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UserId(u64);

impl UserId {
    pub fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum UserIdParseError {
    #[error("user id must be a positive integer")]
    Invalid,
}

impl FromStr for UserId {
    type Err = UserIdParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let trimmed = raw.trim();
        let value = trimmed.parse::<u64>().map_err(|_| UserIdParseError::Invalid)?;
        if value == 0 {
            return Err(UserIdParseError::Invalid);
        }
        Ok(Self(value))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubmissionId(u64);

impl SubmissionId {
    pub fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum SubmissionIdParseError {
    #[error("submission id must be a positive integer")]
    Invalid,
}

impl FromStr for SubmissionId {
    type Err = SubmissionIdParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let trimmed = raw.trim();
        let value = trimmed
            .parse::<u64>()
            .map_err(|_| SubmissionIdParseError::Invalid)?;
        if value == 0 {
            return Err(SubmissionIdParseError::Invalid);
        }
        Ok(Self(value))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointsPossible(f64);

impl PointsPossible {
    pub fn value(self) -> f64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum PointsPossibleParseError {
    #[error("points must be a positive finite number")]
    Invalid,
}

impl FromStr for PointsPossible {
    type Err = PointsPossibleParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let trimmed = raw.trim();
        let value = trimmed
            .parse::<f64>()
            .map_err(|_| PointsPossibleParseError::Invalid)?;
        if !value.is_finite() || value <= 0.0 {
            return Err(PointsPossibleParseError::Invalid);
        }
        Ok(Self(value))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Score {
    value: f64,
    max: f64,
}

impl Score {
    pub fn new(value: f64, max: f64) -> Result<Self, ScoreParseError> {
        if !max.is_finite() || max <= 0.0 {
            return Err(ScoreParseError::InvalidMax);
        }
        if !value.is_finite() {
            return Err(ScoreParseError::InvalidValue);
        }
        if !(0.0..=max).contains(&value) {
            return Err(ScoreParseError::OutOfRange);
        }
        Ok(Self { value, max })
    }

    pub fn value(self) -> f64 {
        self.value
    }

    pub fn max(self) -> f64 {
        self.max
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ScoreParseError {
    #[error("score must be a finite number")]
    InvalidValue,
    #[error("score max must be a positive finite number")]
    InvalidMax,
    #[error("score must be within 0..=max")]
    OutOfRange,
}

impl FromStr for Score {
    type Err = ScoreParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let trimmed = raw.trim();
        let value = trimmed.parse::<f64>().map_err(|_| ScoreParseError::InvalidValue)?;
        Score::new(value, 100.0)
    }
}

impl TryFrom<(f64, f64)> for Score {
    type Error = ScoreParseError;

    fn try_from(raw: (f64, f64)) -> Result<Self, Self::Error> {
        Score::new(raw.0, raw.1)
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(transparent)]
pub struct RubricAssessment(serde_json::Map<String, serde_json::Value>);

impl RubricAssessment {
    pub fn new(
        entries: serde_json::Map<String, serde_json::Value>,
    ) -> Result<Self, RubricAssessmentError> {
        if entries.is_empty() {
            return Err(RubricAssessmentError::Empty);
        }
        Ok(Self(entries))
    }

    pub fn entries(&self) -> &serde_json::Map<String, serde_json::Value> {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum RubricAssessmentError {
    #[error("rubric assessment must be a non-empty object")]
    Empty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DueDate(OffsetDateTime);

impl DueDate {
    pub fn as_rfc3339(&self) -> String {
        self.0
            .format(&Rfc3339)
            .unwrap_or_else(|_| self.0.to_string())
    }

    pub fn inner(self) -> OffsetDateTime {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum DueDateParseError {
    #[error("due date must be an RFC 3339 timestamp")]
    Invalid,
}

impl FromStr for DueDate {
    type Err = DueDateParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let trimmed = raw.trim();
        let parsed =
            OffsetDateTime::parse(trimmed, &Rfc3339).map_err(|_| DueDateParseError::Invalid)?;
        Ok(Self(parsed))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PublishState {
    Published,
    Unpublished,
}

impl PublishState {
    pub fn as_str(&self) -> &'static str {
        match self {
            PublishState::Published => "published",
            PublishState::Unpublished => "unpublished",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum PublishStateParseError {
    #[error("publish state must be 'published' or 'unpublished'")]
    Invalid,
}

impl FromStr for PublishState {
    type Err = PublishStateParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "published" => Ok(PublishState::Published),
            "unpublished" => Ok(PublishState::Unpublished),
            _ => Err(PublishStateParseError::Invalid),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RubricSelection {
    None,
    Rubric,
}

impl RubricSelection {
    pub fn as_str(&self) -> &'static str {
        match self {
            RubricSelection::None => "none",
            RubricSelection::Rubric => "rubric",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum RubricSelectionParseError {
    #[error("rubric selection must be 'none' or 'rubric'")]
    Invalid,
}

impl FromStr for RubricSelection {
    type Err = RubricSelectionParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "none" => Ok(RubricSelection::None),
            "rubric" => Ok(RubricSelection::Rubric),
            _ => Err(RubricSelectionParseError::Invalid),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CourseVisibility {
    Public,
    Institution,
    Course,
}

impl CourseVisibility {
    pub fn as_str(&self) -> &'static str {
        match self {
            CourseVisibility::Public => "public",
            CourseVisibility::Institution => "institution",
            CourseVisibility::Course => "course",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum CourseVisibilityParseError {
    #[error("course visibility must be 'public', 'institution', or 'course'")]
    Invalid,
}

impl FromStr for CourseVisibility {
    type Err = CourseVisibilityParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "public" => Ok(CourseVisibility::Public),
            "institution" => Ok(CourseVisibility::Institution),
            "course" => Ok(CourseVisibility::Course),
            _ => Err(CourseVisibilityParseError::Invalid),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GradingSchemeId(u64);

impl GradingSchemeId {
    pub fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum GradingSchemeIdParseError {
    #[error("grading scheme id must be a positive integer")]
    Invalid,
}

impl FromStr for GradingSchemeId {
    type Err = GradingSchemeIdParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let trimmed = raw.trim();
        let value = trimmed
            .parse::<u64>()
            .map_err(|_| GradingSchemeIdParseError::Invalid)?;
        if value == 0 {
            return Err(GradingSchemeIdParseError::Invalid);
        }
        Ok(Self(value))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CourseDates {
    start: Option<DueDate>,
    end: Option<DueDate>,
}

impl CourseDates {
    pub fn new(start: Option<DueDate>, end: Option<DueDate>) -> Result<Self, CourseDatesError> {
        if let (Some(start), Some(end)) = (start, end)
            && start.inner() > end.inner()
        {
            return Err(CourseDatesError::StartAfterEnd);
        }
        Ok(Self { start, end })
    }

    pub fn start(self) -> Option<DueDate> {
        self.start
    }

    pub fn end(self) -> Option<DueDate> {
        self.end
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum CourseDatesError {
    #[error("course start date must be on or before the end date")]
    StartAfterEnd,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CanvasHost(String);

impl CanvasHost {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum CanvasHostParseError {
    #[error("host must not be empty")]
    Empty,
    #[error("host must be a valid URL")]
    InvalidUrl,
    #[error("host must use http or https")]
    InvalidScheme,
    #[error("host must include a domain")]
    MissingHost,
}

impl FromStr for CanvasHost {
    type Err = CanvasHostParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(CanvasHostParseError::Empty);
        }

        let parsed = Url::parse(trimmed).map_err(|_| CanvasHostParseError::InvalidUrl)?;
        match parsed.scheme() {
            "http" | "https" => {}
            _ => return Err(CanvasHostParseError::InvalidScheme),
        }
        if parsed.host_str().is_none() {
            return Err(CanvasHostParseError::MissingHost);
        }

        let normalized = trimmed.trim_end_matches('/');
        Ok(Self(normalized.to_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CanvasToken(String);

impl CanvasToken {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum CanvasTokenParseError {
    #[error("token must not be empty")]
    Empty,
    #[error("token must not contain whitespace")]
    ContainsWhitespace,
}

impl FromStr for CanvasToken {
    type Err = CanvasTokenParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(CanvasTokenParseError::Empty);
        }
        if trimmed.chars().any(char::is_whitespace) {
            return Err(CanvasTokenParseError::ContainsWhitespace);
        }
        Ok(Self(trimmed.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AssignmentId, AssignmentName, CanvasHost, CanvasToken, CourseDates, CourseId,
        CourseVisibility, DueDate, GradingSchemeId, PointsPossible, PublishState,
        RubricAssessment, RubricSelection, Score, SubmissionId, UserId,
    };

    #[test]
    fn course_id_parses_positive_integers() {
        let parsed: Result<CourseId, _> = "42".parse();
        assert!(matches!(parsed, Ok(value) if value.get() == 42));
    }

    #[test]
    fn course_id_rejects_zero() {
        let parsed: Result<CourseId, _> = "0".parse();
        assert!(parsed.is_err());
    }

    #[test]
    fn course_id_rejects_non_numbers() {
        let parsed: Result<CourseId, _> = "abc".parse();
        assert!(parsed.is_err());
    }

    #[test]
    fn assignment_id_rejects_zero() {
        let parsed: Result<AssignmentId, _> = "0".parse();
        assert!(parsed.is_err());
    }

    #[test]
    fn assignment_name_rejects_empty() {
        let parsed: Result<AssignmentName, _> = "   ".parse();
        assert!(parsed.is_err());
    }

    #[test]
    fn assignment_name_accepts_trimmed() {
        let parsed: Result<AssignmentName, _> = "  Quiz 1  ".parse();
        assert!(matches!(parsed, Ok(value) if value.as_str() == "Quiz 1"));
    }

    #[test]
    fn user_id_rejects_non_numbers() {
        let parsed: Result<UserId, _> = "abc".parse();
        assert!(parsed.is_err());
    }

    #[test]
    fn submission_id_parses_positive_integers() {
        let parsed: Result<SubmissionId, _> = "11".parse();
        assert!(matches!(parsed, Ok(value) if value.get() == 11));
    }

    #[test]
    fn score_accepts_values_within_bounds() {
        let parsed = Score::new(25.5, 30.0);
        assert!(matches!(parsed, Ok(score) if score.value() == 25.5));
    }

    #[test]
    fn score_rejects_out_of_range_values() {
        let parsed = Score::new(31.0, 30.0);
        assert!(parsed.is_err());
    }

    #[test]
    fn score_from_str_defaults_to_100_max() {
        let parsed: Result<Score, _> = "90".parse();
        assert!(matches!(parsed, Ok(score) if score.max() == 100.0));
    }

    #[test]
    fn points_possible_rejects_zero() {
        let parsed: Result<PointsPossible, _> = "0".parse();
        assert!(parsed.is_err());
    }

    #[test]
    fn points_possible_accepts_positive() {
        let parsed: Result<PointsPossible, _> = "15.5".parse();
        assert!(matches!(parsed, Ok(points) if points.value() == 15.5));
    }

    #[test]
    fn rubric_assessment_requires_entries() {
        let empty = serde_json::Map::new();
        let parsed = RubricAssessment::new(empty);
        assert!(parsed.is_err());
    }

    #[test]
    fn due_date_parses_rfc3339() {
        let parsed: Result<DueDate, _> = "2025-01-01T12:00:00Z".parse();
        assert!(parsed.is_ok());
    }

    #[test]
    fn due_date_rejects_invalid_format() {
        let parsed: Result<DueDate, _> = "01/01/2025".parse();
        assert!(parsed.is_err());
    }

    #[test]
    fn publish_state_parses_known_values() {
        let parsed: Result<PublishState, _> = "published".parse();
        assert!(matches!(parsed, Ok(PublishState::Published)));
    }

    #[test]
    fn rubric_selection_parses_known_values() {
        let parsed: Result<RubricSelection, _> = "rubric".parse();
        assert!(matches!(parsed, Ok(RubricSelection::Rubric)));
    }

    #[test]
    fn course_visibility_parses_values() {
        let parsed: Result<CourseVisibility, _> = "institution".parse();
        assert!(matches!(parsed, Ok(CourseVisibility::Institution)));
    }

    #[test]
    fn grading_scheme_id_rejects_zero() {
        let parsed: Result<GradingSchemeId, _> = "0".parse();
        assert!(parsed.is_err());
    }

    #[test]
    fn course_dates_rejects_reverse_order() {
        let start: DueDate = "2025-01-02T00:00:00Z".parse().expect("start");
        let end: DueDate = "2025-01-01T00:00:00Z".parse().expect("end");
        let dates = CourseDates::new(Some(start), Some(end));
        assert!(dates.is_err());
    }

    #[test]
    fn canvas_host_accepts_http_urls() {
        let host: Result<CanvasHost, _> = "https://example.instructure.com".parse();
        assert!(matches!(host, Ok(host) if host.as_str() == "https://example.instructure.com"));
    }

    #[test]
    fn canvas_host_rejects_missing_scheme() {
        let parsed: Result<CanvasHost, _> = "example.instructure.com".parse();
        assert!(parsed.is_err());
    }

    #[test]
    fn canvas_token_rejects_whitespace() {
        let parsed: Result<CanvasToken, _> = "tok en".parse();
        assert!(parsed.is_err());
    }

    #[test]
    fn canvas_token_accepts_non_empty() {
        let token: Result<CanvasToken, _> = "token123".parse();
        assert!(matches!(token, Ok(token) if token.as_str() == "token123"));
    }
}
