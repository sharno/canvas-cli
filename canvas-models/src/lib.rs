use std::str::FromStr;

use serde::Serialize;
use thiserror::Error;
use time::format_description::{self, well_known::Rfc3339};
use time::{Date, OffsetDateTime};
use url::Url;

mod assignment_advanced;
pub use assignment_advanced::{
    AssignmentOverride, AssignmentOverrideDates, AssignmentOverrideDatesError,
    AssignmentOverrideTarget, AssignmentOverrides, AssignmentOverridesError,
    GradingPostingPolicy, GradingPostingPolicyParseError, GroupAssignmentMode,
    GroupAssignmentModeParseError, GroupAssignmentSettings,
    GroupAssignmentSettingsError, GroupCategoryId, GroupCategoryIdParseError,
    MutedState, MutedStateParseError, OverrideStudentIds, OverrideStudentIdsError,
    PeerReviewMode, PeerReviewModeParseError, PeerReviewSettings,
    PeerReviewSettingsError,
};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OutcomeId(u64);

impl OutcomeId {
    pub fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum OutcomeIdParseError {
    #[error("outcome id must be a positive integer")]
    Invalid,
}

impl FromStr for OutcomeId {
    type Err = OutcomeIdParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let trimmed = raw.trim();
        let value = trimmed
            .parse::<u64>()
            .map_err(|_| OutcomeIdParseError::Invalid)?;
        if value == 0 {
            return Err(OutcomeIdParseError::Invalid);
        }
        Ok(Self(value))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OutcomeGroupId(u64);

impl OutcomeGroupId {
    pub fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum OutcomeGroupIdParseError {
    #[error("outcome group id must be a positive integer")]
    Invalid,
}

impl FromStr for OutcomeGroupId {
    type Err = OutcomeGroupIdParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let trimmed = raw.trim();
        let value = trimmed
            .parse::<u64>()
            .map_err(|_| OutcomeGroupIdParseError::Invalid)?;
        if value == 0 {
            return Err(OutcomeGroupIdParseError::Invalid);
        }
        Ok(Self(value))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RubricId(u64);

impl RubricId {
    pub fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum RubricIdParseError {
    #[error("rubric id must be a positive integer")]
    Invalid,
}

impl FromStr for RubricId {
    type Err = RubricIdParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let trimmed = raw.trim();
        let value = trimmed
            .parse::<u64>()
            .map_err(|_| RubricIdParseError::Invalid)?;
        if value == 0 {
            return Err(RubricIdParseError::Invalid);
        }
        Ok(Self(value))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RubricAssociationId(u64);

impl RubricAssociationId {
    pub fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum RubricAssociationIdParseError {
    #[error("rubric association id must be a positive integer")]
    Invalid,
}

impl FromStr for RubricAssociationId {
    type Err = RubricAssociationIdParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let trimmed = raw.trim();
        let value = trimmed
            .parse::<u64>()
            .map_err(|_| RubricAssociationIdParseError::Invalid)?;
        if value == 0 {
            return Err(RubricAssociationIdParseError::Invalid);
        }
        Ok(Self(value))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct QuizId(u64);

impl QuizId {
    pub fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum QuizIdParseError {
    #[error("quiz id must be a positive integer")]
    Invalid,
}

impl FromStr for QuizId {
    type Err = QuizIdParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let trimmed = raw.trim();
        let value = trimmed
            .parse::<u64>()
            .map_err(|_| QuizIdParseError::Invalid)?;
        if value == 0 {
            return Err(QuizIdParseError::Invalid);
        }
        Ok(Self(value))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct QuizSubmissionId(u64);

impl QuizSubmissionId {
    pub fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum QuizSubmissionIdParseError {
    #[error("quiz submission id must be a positive integer")]
    Invalid,
}

impl FromStr for QuizSubmissionId {
    type Err = QuizSubmissionIdParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let trimmed = raw.trim();
        let value = trimmed
            .parse::<u64>()
            .map_err(|_| QuizSubmissionIdParseError::Invalid)?;
        if value == 0 {
            return Err(QuizSubmissionIdParseError::Invalid);
        }
        Ok(Self(value))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct QuestionBankId(u64);

impl QuestionBankId {
    pub fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum QuestionBankIdParseError {
    #[error("question bank id must be a positive integer")]
    Invalid,
}

impl FromStr for QuestionBankId {
    type Err = QuestionBankIdParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let trimmed = raw.trim();
        let value = trimmed
            .parse::<u64>()
            .map_err(|_| QuestionBankIdParseError::Invalid)?;
        if value == 0 {
            return Err(QuestionBankIdParseError::Invalid);
        }
        Ok(Self(value))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct QuestionId(u64);

impl QuestionId {
    pub fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum QuestionIdParseError {
    #[error("question id must be a positive integer")]
    Invalid,
}

impl FromStr for QuestionId {
    type Err = QuestionIdParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let trimmed = raw.trim();
        let value = trimmed
            .parse::<u64>()
            .map_err(|_| QuestionIdParseError::Invalid)?;
        if value == 0 {
            return Err(QuestionIdParseError::Invalid);
        }
        Ok(Self(value))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CalendarEventId(u64);

impl CalendarEventId {
    pub fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum CalendarEventIdParseError {
    #[error("calendar event id must be a positive integer")]
    Invalid,
}

impl FromStr for CalendarEventId {
    type Err = CalendarEventIdParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let trimmed = raw.trim();
        let value = trimmed
            .parse::<u64>()
            .map_err(|_| CalendarEventIdParseError::Invalid)?;
        if value == 0 {
            return Err(CalendarEventIdParseError::Invalid);
        }
        Ok(Self(value))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SectionId(u64);

impl SectionId {
    pub fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum SectionIdParseError {
    #[error("section id must be a positive integer")]
    Invalid,
}

impl FromStr for SectionId {
    type Err = SectionIdParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let trimmed = raw.trim();
        let value = trimmed
            .parse::<u64>()
            .map_err(|_| SectionIdParseError::Invalid)?;
        if value == 0 {
            return Err(SectionIdParseError::Invalid);
        }
        Ok(Self(value))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExternalToolId(u64);

impl ExternalToolId {
    pub fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ExternalToolIdParseError {
    #[error("external tool id must be a positive integer")]
    Invalid,
}

impl FromStr for ExternalToolId {
    type Err = ExternalToolIdParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let trimmed = raw.trim();
        let value = trimmed
            .parse::<u64>()
            .map_err(|_| ExternalToolIdParseError::Invalid)?;
        if value == 0 {
            return Err(ExternalToolIdParseError::Invalid);
        }
        Ok(Self(value))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExternalToolName(String);

impl ExternalToolName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ExternalToolNameParseError {
    #[error("external tool name cannot be empty")]
    Empty,
}

impl FromStr for ExternalToolName {
    type Err = ExternalToolNameParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(ExternalToolNameParseError::Empty);
        }
        Ok(Self(trimmed.to_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExternalToolConfigUrl(Url);

impl ExternalToolConfigUrl {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ExternalToolConfigUrlParseError {
    #[error("external tool config url cannot be empty")]
    Empty,
    #[error("external tool config url must be a valid url")]
    InvalidUrl,
    #[error("external tool config url must use http or https")]
    InvalidScheme,
}

impl FromStr for ExternalToolConfigUrl {
    type Err = ExternalToolConfigUrlParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(ExternalToolConfigUrlParseError::Empty);
        }
        let parsed = Url::parse(trimmed).map_err(|_| ExternalToolConfigUrlParseError::InvalidUrl)?;
        match parsed.scheme() {
            "http" | "https" => Ok(Self(parsed)),
            _ => Err(ExternalToolConfigUrlParseError::InvalidScheme),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExternalToolPlacement(String);

impl ExternalToolPlacement {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ExternalToolPlacementParseError {
    #[error("external tool placement cannot be empty")]
    Empty,
}

impl FromStr for ExternalToolPlacement {
    type Err = ExternalToolPlacementParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(ExternalToolPlacementParseError::Empty);
        }
        Ok(Self(trimmed.to_string()))
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OutcomeTitle(String);

impl OutcomeTitle {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum OutcomeTitleParseError {
    #[error("outcome title must not be empty")]
    Empty,
}

impl FromStr for OutcomeTitle {
    type Err = OutcomeTitleParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(OutcomeTitleParseError::Empty);
        }
        Ok(Self(trimmed.to_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OutcomeDescription(String);

impl OutcomeDescription {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum OutcomeDescriptionParseError {
    #[error("outcome description must not be empty")]
    Empty,
}

impl FromStr for OutcomeDescription {
    type Err = OutcomeDescriptionParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(OutcomeDescriptionParseError::Empty);
        }
        Ok(Self(trimmed.to_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RubricTitle(String);

impl RubricTitle {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum RubricTitleParseError {
    #[error("rubric title must not be empty")]
    Empty,
}

impl FromStr for RubricTitle {
    type Err = RubricTitleParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(RubricTitleParseError::Empty);
        }
        Ok(Self(trimmed.to_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CalendarEventTitle(String);

impl CalendarEventTitle {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum CalendarEventTitleParseError {
    #[error("calendar event title must not be empty")]
    Empty,
}

impl FromStr for CalendarEventTitle {
    type Err = CalendarEventTitleParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(CalendarEventTitleParseError::Empty);
        }
        Ok(Self(trimmed.to_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QuizTitle(String);

impl QuizTitle {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum QuizTitleParseError {
    #[error("quiz title must not be empty")]
    Empty,
}

impl FromStr for QuizTitle {
    type Err = QuizTitleParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(QuizTitleParseError::Empty);
        }
        Ok(Self(trimmed.to_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QuestionBankTitle(String);

impl QuestionBankTitle {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum QuestionBankTitleParseError {
    #[error("question bank title must not be empty")]
    Empty,
}

impl FromStr for QuestionBankTitle {
    type Err = QuestionBankTitleParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(QuestionBankTitleParseError::Empty);
        }
        Ok(Self(trimmed.to_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QuestionName(String);

impl QuestionName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum QuestionNameParseError {
    #[error("question name must not be empty")]
    Empty,
}

impl FromStr for QuestionName {
    type Err = QuestionNameParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(QuestionNameParseError::Empty);
        }
        Ok(Self(trimmed.to_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QuestionText(String);

impl QuestionText {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum QuestionTextParseError {
    #[error("question text must not be empty")]
    Empty,
}

impl FromStr for QuestionText {
    type Err = QuestionTextParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(QuestionTextParseError::Empty);
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

impl TryFrom<u64> for UserId {
    type Error = UserIdParseError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if value == 0 {
            return Err(UserIdParseError::Invalid);
        }
        Ok(Self(value))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserRole {
    Student,
    Ta,
    Teacher,
}

impl UserRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            UserRole::Student => "student",
            UserRole::Ta => "ta",
            UserRole::Teacher => "teacher",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum UserRoleParseError {
    #[error("user role must be student, ta, or teacher")]
    Invalid,
}

impl FromStr for UserRole {
    type Err = UserRoleParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "student" | "students" => Ok(UserRole::Student),
            "ta" | "tas" | "teachingassistant" | "teaching_assistant" => {
                Ok(UserRole::Ta)
            }
            "teacher" | "teachers" => Ok(UserRole::Teacher),
            _ => Err(UserRoleParseError::Invalid),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipientIds(Vec<UserId>);

impl RecipientIds {
    pub fn new(ids: Vec<UserId>) -> Result<Self, RecipientIdsError> {
        if ids.is_empty() {
            return Err(RecipientIdsError::Empty);
        }
        Ok(Self(ids))
    }

    pub fn ids(&self) -> &[UserId] {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum RecipientIdsError {
    #[error("recipients must not be empty")]
    Empty,
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PageId(String);

impl PageId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum PageIdParseError {
    #[error("page id must not be empty")]
    Empty,
}

impl FromStr for PageId {
    type Err = PageIdParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(PageIdParseError::Empty);
        }
        Ok(Self(trimmed.to_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PageTitle(String);

impl PageTitle {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum PageTitleParseError {
    #[error("page title must not be empty")]
    Empty,
}

impl FromStr for PageTitle {
    type Err = PageTitleParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(PageTitleParseError::Empty);
        }
        Ok(Self(trimmed.to_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PageBody(String);

impl PageBody {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MessageSubject(String);

impl MessageSubject {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum MessageSubjectParseError {
    #[error("message subject must not be empty")]
    Empty,
}

impl FromStr for MessageSubject {
    type Err = MessageSubjectParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(MessageSubjectParseError::Empty);
        }
        Ok(Self(trimmed.to_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MessageBody(String);

impl MessageBody {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum MessageBodyParseError {
    #[error("message body must not be empty")]
    Empty,
}

impl FromStr for MessageBody {
    type Err = MessageBodyParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(MessageBodyParseError::Empty);
        }
        Ok(Self(trimmed.to_string()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum PageBodyParseError {
    #[error("page body must not be empty")]
    Empty,
}

impl FromStr for PageBody {
    type Err = PageBodyParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(PageBodyParseError::Empty);
        }
        Ok(Self(trimmed.to_string()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModuleId(u64);

impl ModuleId {
    pub fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ModuleIdParseError {
    #[error("module id must be a positive integer")]
    Invalid,
}

impl FromStr for ModuleId {
    type Err = ModuleIdParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let trimmed = raw.trim();
        let value = trimmed
            .parse::<u64>()
            .map_err(|_| ModuleIdParseError::Invalid)?;
        if value == 0 {
            return Err(ModuleIdParseError::Invalid);
        }
        Ok(Self(value))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModuleName(String);

impl ModuleName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ModuleNameParseError {
    #[error("module name must not be empty")]
    Empty,
}

impl FromStr for ModuleName {
    type Err = ModuleNameParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(ModuleNameParseError::Empty);
        }
        Ok(Self(trimmed.to_string()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileId(u64);

impl FileId {
    pub fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum FileIdParseError {
    #[error("file id must be a positive integer")]
    Invalid,
}

impl FromStr for FileId {
    type Err = FileIdParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let trimmed = raw.trim();
        let value = trimmed
            .parse::<u64>()
            .map_err(|_| FileIdParseError::Invalid)?;
        if value == 0 {
            return Err(FileIdParseError::Invalid);
        }
        Ok(Self(value))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FolderId(u64);

impl FolderId {
    pub fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum FolderIdParseError {
    #[error("folder id must be a positive integer")]
    Invalid,
}

impl FromStr for FolderId {
    type Err = FolderIdParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let trimmed = raw.trim();
        let value = trimmed
            .parse::<u64>()
            .map_err(|_| FolderIdParseError::Invalid)?;
        if value == 0 {
            return Err(FolderIdParseError::Invalid);
        }
        Ok(Self(value))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FolderName(String);

impl FolderName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GroupId(u64);

impl GroupId {
    pub fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum GroupIdParseError {
    #[error("group id must be a positive integer")]
    Invalid,
}

impl FromStr for GroupId {
    type Err = GroupIdParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let trimmed = raw.trim();
        let value =
            trimmed.parse::<u64>().map_err(|_| GroupIdParseError::Invalid)?;
        if value == 0 {
            return Err(GroupIdParseError::Invalid);
        }
        Ok(Self(value))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GroupName(String);

impl GroupName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum GroupNameParseError {
    #[error("group name must not be empty")]
    Empty,
}

impl FromStr for GroupName {
    type Err = GroupNameParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(GroupNameParseError::Empty);
        }
        Ok(Self(trimmed.to_string()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum FolderNameParseError {
    #[error("folder name must not be empty")]
    Empty,
}

impl FromStr for FolderName {
    type Err = FolderNameParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(FolderNameParseError::Empty);
        }
        Ok(Self(trimmed.to_string()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DiscussionId(u64);

impl DiscussionId {
    pub fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum DiscussionIdParseError {
    #[error("discussion id must be a positive integer")]
    Invalid,
}

impl FromStr for DiscussionId {
    type Err = DiscussionIdParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let trimmed = raw.trim();
        let value = trimmed
            .parse::<u64>()
            .map_err(|_| DiscussionIdParseError::Invalid)?;
        if value == 0 {
            return Err(DiscussionIdParseError::Invalid);
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
pub struct EventDateTime(OffsetDateTime);

impl EventDateTime {
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
pub enum EventDateTimeParseError {
    #[error("event timestamp must be an RFC 3339 timestamp")]
    Invalid,
}

impl FromStr for EventDateTime {
    type Err = EventDateTimeParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let trimmed = raw.trim();
        let parsed = OffsetDateTime::parse(trimmed, &Rfc3339)
            .map_err(|_| EventDateTimeParseError::Invalid)?;
        Ok(Self(parsed))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AllDayDate(Date);

impl AllDayDate {
    pub fn as_iso8601(&self) -> String {
        self.0.to_string()
    }

    pub fn inner(self) -> Date {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum AllDayDateParseError {
    #[error("all-day date must be an ISO-8601 date (YYYY-MM-DD)")]
    Invalid,
}

impl FromStr for AllDayDate {
    type Err = AllDayDateParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let trimmed = raw.trim();
        let format = format_description::parse("[year]-[month]-[day]")
            .map_err(|_| AllDayDateParseError::Invalid)?;
        let parsed = Date::parse(trimmed, &format)
            .map_err(|_| AllDayDateParseError::Invalid)?;
        Ok(Self(parsed))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalendarEventContext {
    Course(CourseId),
    Section(SectionId),
}

impl CalendarEventContext {
    pub fn context_code(self) -> String {
        match self {
            CalendarEventContext::Course(course_id) => {
                format!("course_{}", course_id.get())
            }
            CalendarEventContext::Section(section_id) => {
                format!("section_{}", section_id.get())
            }
        }
    }

    pub fn context_type(self) -> &'static str {
        match self {
            CalendarEventContext::Course(_) => "course",
            CalendarEventContext::Section(_) => "section",
        }
    }

    pub fn context_id(self) -> u64 {
        match self {
            CalendarEventContext::Course(course_id) => course_id.get(),
            CalendarEventContext::Section(section_id) => section_id.get(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuizTimeLimit(u32);

impl QuizTimeLimit {
    pub fn minutes(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum QuizTimeLimitParseError {
    #[error("quiz time limit must be a positive integer of minutes")]
    Invalid,
}

impl FromStr for QuizTimeLimit {
    type Err = QuizTimeLimitParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let trimmed = raw.trim();
        let value = trimmed
            .parse::<u32>()
            .map_err(|_| QuizTimeLimitParseError::Invalid)?;
        if value == 0 {
            return Err(QuizTimeLimitParseError::Invalid);
        }
        Ok(Self(value))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuizAccessCode(String);

impl QuizAccessCode {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum QuizAccessCodeParseError {
    #[error("quiz access code must not be empty")]
    Empty,
}

impl FromStr for QuizAccessCode {
    type Err = QuizAccessCodeParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(QuizAccessCodeParseError::Empty);
        }
        Ok(Self(trimmed.to_string()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QuestionType {
    MultipleChoice,
    MultipleAnswers,
    TrueFalse,
    Essay,
    ShortAnswer,
    Matching,
    FillInMultipleBlanks,
    MultipleDropdowns,
    Numerical,
    Calculated,
    FileUpload,
    TextOnly,
}

impl QuestionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            QuestionType::MultipleChoice => "multiple_choice_question",
            QuestionType::MultipleAnswers => "multiple_answers_question",
            QuestionType::TrueFalse => "true_false_question",
            QuestionType::Essay => "essay_question",
            QuestionType::ShortAnswer => "short_answer_question",
            QuestionType::Matching => "matching_question",
            QuestionType::FillInMultipleBlanks => "fill_in_multiple_blanks_question",
            QuestionType::MultipleDropdowns => "multiple_dropdowns_question",
            QuestionType::Numerical => "numerical_question",
            QuestionType::Calculated => "calculated_question",
            QuestionType::FileUpload => "file_upload_question",
            QuestionType::TextOnly => "text_only_question",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum QuestionTypeParseError {
    #[error(
        "question type must be multiple_choice, multiple_answers, true_false, essay, short_answer, matching, fill_in_multiple_blanks, multiple_dropdowns, numerical, calculated, file_upload, or text_only"
    )]
    Invalid,
}

impl FromStr for QuestionType {
    type Err = QuestionTypeParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "mcq"
            | "multiple_choice"
            | "multiple-choice"
            | "multiple_choice_question" => Ok(QuestionType::MultipleChoice),
            "multiple_answers" | "multiple-answers" | "multiple_answers_question" => {
                Ok(QuestionType::MultipleAnswers)
            }
            "true_false" | "true-false" | "true_false_question" => Ok(QuestionType::TrueFalse),
            "essay" | "essay_question" => Ok(QuestionType::Essay),
            "short_answer" | "short-answer" | "short_answer_question" => {
                Ok(QuestionType::ShortAnswer)
            }
            "matching" | "matching_question" => Ok(QuestionType::Matching),
            "fill_in_multiple_blanks"
            | "fill-in-multiple-blanks"
            | "fill_in_multiple_blanks_question" => Ok(QuestionType::FillInMultipleBlanks),
            "multiple_dropdowns"
            | "multiple-dropdowns"
            | "multiple_dropdowns_question" => Ok(QuestionType::MultipleDropdowns),
            "numerical" | "numerical_question" => Ok(QuestionType::Numerical),
            "calculated" | "calculated_question" => Ok(QuestionType::Calculated),
            "file_upload" | "file-upload" | "file_upload_question" => {
                Ok(QuestionType::FileUpload)
            }
            "text_only" | "text-only" | "text_only_question" => Ok(QuestionType::TextOnly),
            _ => Err(QuestionTypeParseError::Invalid),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuizAvailability {
    unlock_at: Option<DueDate>,
    due_at: Option<DueDate>,
    lock_at: Option<DueDate>,
}

impl QuizAvailability {
    pub fn new(
        unlock_at: Option<DueDate>,
        due_at: Option<DueDate>,
        lock_at: Option<DueDate>,
    ) -> Result<Self, QuizAvailabilityError> {
        if let (Some(unlock_at), Some(due_at)) = (unlock_at, due_at)
            && unlock_at.inner() > due_at.inner()
        {
            return Err(QuizAvailabilityError::UnlockAfterDue);
        }
        if let (Some(unlock_at), Some(lock_at)) = (unlock_at, lock_at)
            && unlock_at.inner() > lock_at.inner()
        {
            return Err(QuizAvailabilityError::UnlockAfterLock);
        }
        if let (Some(due_at), Some(lock_at)) = (due_at, lock_at)
            && due_at.inner() > lock_at.inner()
        {
            return Err(QuizAvailabilityError::DueAfterLock);
        }
        Ok(Self {
            unlock_at,
            due_at,
            lock_at,
        })
    }

    pub fn unlock_at(self) -> Option<DueDate> {
        self.unlock_at
    }

    pub fn due_at(self) -> Option<DueDate> {
        self.due_at
    }

    pub fn lock_at(self) -> Option<DueDate> {
        self.lock_at
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum QuizAvailabilityError {
    #[error("quiz unlock date must be on or before the due date")]
    UnlockAfterDue,
    #[error("quiz unlock date must be on or before the lock date")]
    UnlockAfterLock,
    #[error("quiz due date must be on or before the lock date")]
    DueAfterLock,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportType {
    GradebookExport,
    MissingSubmissions,
    CourseActivity,
}

impl ReportType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ReportType::GradebookExport => "grade_export",
            ReportType::MissingSubmissions => "missing_submissions",
            ReportType::CourseActivity => "course_activity",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ReportTypeParseError {
    #[error("report type must be gradebook, missing_submissions, or course_activity")]
    Invalid,
}

impl FromStr for ReportType {
    type Err = ReportTypeParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "gradebook" | "gradebook_export" | "grade_export" => {
                Ok(ReportType::GradebookExport)
            }
            "missing_submissions" | "submission_status" | "submission_status_summary" => {
                Ok(ReportType::MissingSubmissions)
            }
            "course_activity" | "activity" | "course_activity_summary" => {
                Ok(ReportType::CourseActivity)
            }
            _ => Err(ReportTypeParseError::Invalid),
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
        AllDayDate, AssignmentId, AssignmentName, CalendarEventId, CalendarEventTitle,
        CanvasHost, CanvasToken, CourseDates, CourseId, CourseVisibility, DiscussionId,
        DueDate, EventDateTime, ExternalToolConfigUrl, ExternalToolId, ExternalToolName,
        ExternalToolPlacement, FileId, FolderId, FolderName, GroupId, GroupName,
        GradingSchemeId, MessageBody, MessageSubject, ModuleId, ModuleName,
        OutcomeDescription, OutcomeGroupId, OutcomeId, OutcomeTitle, PageBody, PageId,
        PageTitle, PointsPossible, PublishState, QuestionBankId, QuestionBankTitle,
        QuestionId, QuestionName, QuestionText, QuestionType, RecipientIds, ReportType,
        RubricAssociationId, RubricAssessment, RubricId, RubricSelection, RubricTitle,
        Score, SectionId, SubmissionId, UserId, UserRole,
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
    fn outcome_id_rejects_zero() {
        let parsed: Result<OutcomeId, _> = "0".parse();
        assert!(parsed.is_err());
    }

    #[test]
    fn outcome_group_id_rejects_zero() {
        let parsed: Result<OutcomeGroupId, _> = "0".parse();
        assert!(parsed.is_err());
    }

    #[test]
    fn rubric_id_rejects_zero() {
        let parsed: Result<RubricId, _> = "0".parse();
        assert!(parsed.is_err());
    }

    #[test]
    fn rubric_association_id_rejects_zero() {
        let parsed: Result<RubricAssociationId, _> = "0".parse();
        assert!(parsed.is_err());
    }

    #[test]
    fn outcome_title_rejects_empty() {
        let parsed: Result<OutcomeTitle, _> = "   ".parse();
        assert!(parsed.is_err());
    }

    #[test]
    fn outcome_description_rejects_empty() {
        let parsed: Result<OutcomeDescription, _> = "".parse();
        assert!(parsed.is_err());
    }

    #[test]
    fn rubric_title_rejects_empty() {
        let parsed: Result<RubricTitle, _> = "  ".parse();
        assert!(parsed.is_err());
    }

    #[test]
    fn calendar_event_id_rejects_zero() {
        let parsed: Result<CalendarEventId, _> = "0".parse();
        assert!(parsed.is_err());
    }

    #[test]
    fn section_id_rejects_zero() {
        let parsed: Result<SectionId, _> = "0".parse();
        assert!(parsed.is_err());
    }

    #[test]
    fn external_tool_id_rejects_zero() {
        let parsed: Result<ExternalToolId, _> = "0".parse();
        assert!(parsed.is_err());
    }

    #[test]
    fn external_tool_name_rejects_empty() {
        let parsed: Result<ExternalToolName, _> = "  ".parse();
        assert!(parsed.is_err());
    }

    #[test]
    fn external_tool_config_url_requires_http() {
        let parsed: Result<ExternalToolConfigUrl, _> = "ftp://example.com".parse();
        assert!(parsed.is_err());
    }

    #[test]
    fn external_tool_placement_rejects_empty() {
        let parsed: Result<ExternalToolPlacement, _> = "".parse();
        assert!(parsed.is_err());
    }

    #[test]
    fn calendar_event_title_rejects_empty() {
        let parsed: Result<CalendarEventTitle, _> = "   ".parse();
        assert!(parsed.is_err());
    }

    #[test]
    fn question_bank_id_rejects_zero() {
        let parsed: Result<QuestionBankId, _> = "0".parse();
        assert!(parsed.is_err());
    }

    #[test]
    fn question_id_rejects_non_numbers() {
        let parsed: Result<QuestionId, _> = "abc".parse();
        assert!(parsed.is_err());
    }

    #[test]
    fn question_bank_title_rejects_empty() {
        let parsed: Result<QuestionBankTitle, _> = "   ".parse();
        assert!(parsed.is_err());
    }

    #[test]
    fn question_name_rejects_empty() {
        let parsed: Result<QuestionName, _> = " ".parse();
        assert!(parsed.is_err());
    }

    #[test]
    fn question_text_rejects_empty() {
        let parsed: Result<QuestionText, _> = "".parse();
        assert!(parsed.is_err());
    }

    #[test]
    fn question_type_accepts_mcq_alias() {
        let parsed: Result<QuestionType, _> = "mcq".parse();
        assert!(matches!(parsed, Ok(kind) if kind == QuestionType::MultipleChoice));
    }

    #[test]
    fn question_type_rejects_unknown() {
        let parsed: Result<QuestionType, _> = "unknown".parse();
        assert!(parsed.is_err());
    }

    #[test]
    fn event_date_time_parses_rfc3339() {
        let parsed: Result<EventDateTime, _> = "2025-01-01T12:00:00Z".parse();
        assert!(parsed.is_ok());
    }

    #[test]
    fn all_day_date_parses_iso8601() {
        let parsed: Result<AllDayDate, _> = "2025-01-01".parse();
        assert!(parsed.is_ok());
    }

    #[test]
    fn user_id_rejects_non_numbers() {
        let parsed: Result<UserId, _> = "abc".parse();
        assert!(parsed.is_err());
    }

    #[test]
    fn user_id_try_from_accepts_positive() {
        let parsed = UserId::try_from(4_u64);
        assert!(matches!(parsed, Ok(value) if value.get() == 4));
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
    fn report_type_parses_gradebook() {
        let parsed: Result<ReportType, _> = "grade_export".parse();
        assert!(matches!(parsed, Ok(ReportType::GradebookExport)));
    }

    #[test]
    fn report_type_parses_missing_submissions() {
        let parsed: Result<ReportType, _> = "missing_submissions".parse();
        assert!(matches!(parsed, Ok(ReportType::MissingSubmissions)));
    }

    #[test]
    fn report_type_rejects_unknown() {
        let parsed: Result<ReportType, _> = "unknown_report".parse();
        assert!(parsed.is_err());
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

    #[test]
    fn page_id_rejects_empty() {
        let parsed: Result<PageId, _> = " ".parse();
        assert!(parsed.is_err());
    }

    #[test]
    fn page_title_rejects_empty() {
        let parsed: Result<PageTitle, _> = "".parse();
        assert!(parsed.is_err());
    }

    #[test]
    fn page_body_rejects_empty() {
        let parsed: Result<PageBody, _> = "   ".parse();
        assert!(parsed.is_err());
    }

    #[test]
    fn module_id_rejects_zero() {
        let parsed: Result<ModuleId, _> = "0".parse();
        assert!(parsed.is_err());
    }

    #[test]
    fn module_name_rejects_empty() {
        let parsed: Result<ModuleName, _> = "  ".parse();
        assert!(parsed.is_err());
    }

    #[test]
    fn file_id_rejects_zero() {
        let parsed: Result<FileId, _> = "0".parse();
        assert!(parsed.is_err());
    }

    #[test]
    fn folder_id_rejects_zero() {
        let parsed: Result<FolderId, _> = "0".parse();
        assert!(parsed.is_err());
    }

    #[test]
    fn folder_name_rejects_empty() {
        let parsed: Result<FolderName, _> = " ".parse();
        assert!(parsed.is_err());
    }

    #[test]
    fn discussion_id_rejects_zero() {
        let parsed: Result<DiscussionId, _> = "0".parse();
        assert!(parsed.is_err());
    }

    #[test]
    fn user_role_parses_known_values() {
        let parsed: Result<UserRole, _> = "ta".parse();
        assert!(matches!(parsed, Ok(UserRole::Ta)));
    }

    #[test]
    fn message_subject_rejects_empty() {
        let parsed: Result<MessageSubject, _> = "  ".parse();
        assert!(parsed.is_err());
    }

    #[test]
    fn message_body_rejects_empty() {
        let parsed: Result<MessageBody, _> = "".parse();
        assert!(parsed.is_err());
    }

    #[test]
    fn recipient_ids_require_non_empty() {
        let result = RecipientIds::new(Vec::new());
        assert!(result.is_err());
    }

    #[test]
    fn group_id_rejects_zero() {
        let parsed: Result<GroupId, _> = "0".parse();
        assert!(parsed.is_err());
    }

    #[test]
    fn group_name_rejects_empty() {
        let parsed: Result<GroupName, _> = " ".parse();
        assert!(parsed.is_err());
    }
}
