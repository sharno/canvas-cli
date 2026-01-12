mod auth;
mod api;
mod assignments;
mod calendar;
mod conferences;
mod collaborations;
mod config;
mod courses;
mod content;
mod content_migrations;
mod external_tools;
mod gradebook;
mod people;
mod reports;
mod quizzes;
mod outcomes;
mod rubrics;
mod question_banks;

use canvas_models::{
    AllDayDate, AssignmentId, AssignmentName, AssignmentOverride, AssignmentOverrideDates,
    AssignmentOverrideTarget, AssignmentOverrides, CalendarEventContext, CalendarEventId,
    CalendarEventTitle, CanvasHost, CanvasToken, ConferenceDescription, ConferenceDuration,
    CollaborationId, CollaborationTitle, CollaborationType, ConferenceId,
    ConferenceTitle, ContentMigrationId, ContentMigrationType, CourseDates,
    CourseId, CourseVisibility, DiscussionId, DueDate, EventDateTime,
    ExternalToolConfigUrl, ExternalToolId, ExternalToolName, ExternalToolPlacement,
    FileId, FolderId, FolderName, GradingPostingPolicy, GradingSchemeId,
    GroupAssignmentMode, GroupAssignmentSettings, GroupCategoryId, GroupId,
    GroupName, MessageBody, MessageSubject, ModuleId, ModuleItemId, ModuleName,
    ModulePrerequisites, ModuleRequirement, ModuleRequirementType, ModuleRequirements,
    MutedState, OutcomeDescription,
    OutcomeGroupId, OutcomeId, OutcomeTitle, OverrideStudentIds, PageBody, PageId,
    PageTitle, PeerReviewMode, PeerReviewSettings, PointsPossible, PublishState,
    QuestionBankId, QuestionBankTitle, QuestionId, QuestionName, QuestionText,
    QuestionType, QuizAccessCode, QuizAvailability, QuizId, QuizSubmissionId,
    QuizTimeLimit, QuizTitle, RecipientIds, ReportType, RubricAssessment,
    RubricAssociationId, RubricId, RubricSelection, RubricTitle, Score, SectionId,
    SubmissionId, UserId, UserRole,
};
use thiserror::Error;

pub use auth::auth_check;
pub use api::{ApiError, ApiErrorCode, CanvasClient, RetryPolicy};
pub use assignments::{
    create_assignment, delete_assignment, ensure_assignment_points, get_assignment,
    grade_submission, list_assignments, list_submissions, update_assignment,
    AssignmentCreateInput, AssignmentGroupSummary, AssignmentPeerReviewSummary,
    AssignmentSummary, AssignmentUpdateInput, SubmissionSummary,
};
pub use calendar::{
    create_calendar_event, delete_calendar_event, list_calendar_events,
    update_calendar_event, CalendarEventCreateInput, CalendarEventSummary,
    CalendarEventTiming, CalendarEventUpdateInput,
};
pub use conferences::{
    create_conference, delete_conference, list_conferences, update_conference,
    ConferenceCreateInput, ConferenceSchedule, ConferenceSummary, ConferenceUpdateInput,
};
pub use collaborations::{
    create_collaboration, delete_collaboration, list_collaborations,
    CollaborationCreateInput, CollaborationSummary, Collaborator, Collaborators,
};
pub use config::{config_path, load_merged_config, write_config, AuthConfig, CanvasConfig, DefaultsConfig};
pub use courses::{get_course, list_courses, persist_default_course, update_course, CourseSettingsUpdate, CourseSummary};
pub use content::{
    create_announcement, create_discussion, create_folder, create_module,
    create_page, delete_file, get_module, list_announcements, list_discussions,
    list_files, list_folders, list_modules, list_pages, reorder_modules,
    update_module, update_module_requirements, update_page, upload_file,
    AnnouncementSummary, DiscussionSummary, FileSummary, FolderSummary,
    ModuleCreateInput, ModulePrerequisiteSummary, ModuleRequirementSummary,
    ModuleRequirementsUpdateInput, ModuleSummary, ModuleUpdateInput,
    PageCreateInput, PageSummary, PageUpdateInput, UploadFileInput,
};
pub use content_migrations::{
    create_content_migration, get_content_migration, get_content_migration_progress,
    list_content_migrations, wait_for_content_migration, ContentMigrationCreateInput,
    ContentMigrationProgress, ContentMigrationState, ContentMigrationSummary,
    ContentMigrationTypeValue, ContentMigrationWaitOptions,
};
pub use external_tools::{
    create_external_tool, delete_external_tool, list_external_tools,
    update_external_tool, ExternalToolConfig, ExternalToolConfigJson,
    ExternalToolConfigXml, ExternalToolCreateInput, ExternalToolPlacementList,
    ExternalToolPlacementSettings, ExternalToolSummary, ExternalToolUpdateInput,
};
pub use gradebook::{
    get_grading_period, get_posting_policy, list_grading_periods,
    update_posting_policy, GradingPeriodSummary, PostingPolicyScope,
    PostingPolicySummary,
};
pub use people::{
    create_group, list_groups, list_users, send_message, GroupSummary,
    MessageSendInput, MessageSendResult, UserSummary,
};
pub use quizzes::{
    create_quiz, delete_quiz, ensure_quiz_points, get_quiz, grade_quiz_submission,
    list_quiz_submissions, list_quizzes, update_quiz, QuizCreateInput,
    QuizSubmissionSummary, QuizSummary, QuizUpdateInput,
};
pub use outcomes::{
    create_outcome, delete_outcome, list_outcomes, update_outcome,
    OutcomeCreateInput, OutcomeRatingSummary, OutcomeSummary, OutcomeUpdateInput,
};
pub use rubrics::{
    attach_rubric, create_rubric, delete_rubric, detach_rubric, list_rubrics,
    update_rubric, RubricAssociationInput, RubricAssociationSummary,
    RubricAssociationTarget, RubricCreateInput, RubricCriteria,
    RubricCriterionSummary, RubricSummary, RubricUpdateInput,
};
pub use question_banks::{
    create_question, create_question_bank, delete_question, delete_question_bank,
    list_question_banks, list_questions, update_question, update_question_bank,
    QuestionAnswerSummary, QuestionBankCreateInput, QuestionBankSummary,
    QuestionBankUpdateInput, QuestionCreateInput, QuestionSummary, QuestionUpdateInput,
};
pub use reports::{
    download_report_to_writer, get_course_activity_summary, get_report, request_report,
    wait_for_report, CourseActivitySummary, ReportState, ReportSummary, ReportWaitOptions,
};

#[derive(Debug, Error)]
pub enum CanvasError {
    #[error("invalid course id: {0}")]
    InvalidCourseId(String),
    #[error("invalid grading period id: {0}")]
    InvalidGradingPeriodId(String),
    #[error("invalid assignment id: {0}")]
    InvalidAssignmentId(String),
    #[error("invalid assignment name: {0}")]
    InvalidAssignmentName(String),
    #[error("invalid group category id: {0}")]
    InvalidGroupCategoryId(String),
    #[error("invalid group assignment mode: {0}")]
    InvalidGroupAssignmentMode(String),
    #[error("invalid group assignment settings: {0}")]
    InvalidGroupAssignmentSettings(String),
    #[error("invalid peer review mode: {0}")]
    InvalidPeerReviewMode(String),
    #[error("invalid peer review settings: {0}")]
    InvalidPeerReviewSettings(String),
    #[error("invalid assignment overrides: {0}")]
    InvalidAssignmentOverrides(String),
    #[error("invalid assignment override target: {0}")]
    InvalidAssignmentOverrideTarget(String),
    #[error("invalid assignment override dates: {0}")]
    InvalidAssignmentOverrideDates(String),
    #[error("invalid grading posting policy: {0}")]
    InvalidGradingPostingPolicy(String),
    #[error("invalid muted state: {0}")]
    InvalidMutedState(String),
    #[error("invalid outcome id: {0}")]
    InvalidOutcomeId(String),
    #[error("invalid outcome group id: {0}")]
    InvalidOutcomeGroupId(String),
    #[error("invalid outcome title: {0}")]
    InvalidOutcomeTitle(String),
    #[error("invalid outcome description: {0}")]
    InvalidOutcomeDescription(String),
    #[error("invalid rubric id: {0}")]
    InvalidRubricId(String),
    #[error("invalid rubric association id: {0}")]
    InvalidRubricAssociationId(String),
    #[error("invalid rubric title: {0}")]
    InvalidRubricTitle(String),
    #[error("invalid quiz id: {0}")]
    InvalidQuizId(String),
    #[error("invalid quiz submission id: {0}")]
    InvalidQuizSubmissionId(String),
    #[error("invalid quiz title: {0}")]
    InvalidQuizTitle(String),
    #[error("invalid question bank id: {0}")]
    InvalidQuestionBankId(String),
    #[error("invalid question id: {0}")]
    InvalidQuestionId(String),
    #[error("invalid question bank title: {0}")]
    InvalidQuestionBankTitle(String),
    #[error("invalid question name: {0}")]
    InvalidQuestionName(String),
    #[error("invalid question text: {0}")]
    InvalidQuestionText(String),
    #[error("invalid question type: {0}")]
    InvalidQuestionType(String),
    #[error("invalid question bank update: {0}")]
    InvalidQuestionBankUpdate(String),
    #[error("invalid question update: {0}")]
    InvalidQuestionUpdate(String),
    #[error("invalid question bank json: {0}")]
    InvalidQuestionBankJson(String),
    #[error("invalid question json: {0}")]
    InvalidQuestionJson(String),
    #[error("invalid calendar event id: {0}")]
    InvalidCalendarEventId(String),
    #[error("invalid conference id: {0}")]
    InvalidConferenceId(String),
    #[error("invalid collaboration id: {0}")]
    InvalidCollaborationId(String),
    #[error("invalid section id: {0}")]
    InvalidSectionId(String),
    #[error("invalid external tool id: {0}")]
    InvalidExternalToolId(String),
    #[error("invalid external tool name: {0}")]
    InvalidExternalToolName(String),
    #[error("invalid external tool config: {0}")]
    InvalidExternalToolConfig(String),
    #[error("invalid external tool placement: {0}")]
    InvalidExternalToolPlacement(String),
    #[error("invalid external tool placements: {0}")]
    InvalidExternalToolPlacements(String),
    #[error("invalid external tool update: {0}")]
    InvalidExternalToolUpdate(String),
    #[error("invalid calendar event title: {0}")]
    InvalidCalendarEventTitle(String),
    #[error("invalid conference title: {0}")]
    InvalidConferenceTitle(String),
    #[error("invalid collaboration title: {0}")]
    InvalidCollaborationTitle(String),
    #[error("invalid collaboration type: {0}")]
    InvalidCollaborationType(String),
    #[error("invalid collaborators: {0}")]
    InvalidCollaborators(String),
    #[error("invalid conference description: {0}")]
    InvalidConferenceDescription(String),
    #[error("invalid conference duration: {0}")]
    InvalidConferenceDuration(String),
    #[error("invalid user id: {0}")]
    InvalidUserId(String),
    #[error("invalid submission id: {0}")]
    InvalidSubmissionId(String),
    #[error("invalid page id: {0}")]
    InvalidPageId(String),
    #[error("invalid page title: {0}")]
    InvalidPageTitle(String),
    #[error("invalid page body: {0}")]
    InvalidPageBody(String),
    #[error("invalid module id: {0}")]
    InvalidModuleId(String),
    #[error("invalid module item id: {0}")]
    InvalidModuleItemId(String),
    #[error("invalid module name: {0}")]
    InvalidModuleName(String),
    #[error("invalid module requirement type: {0}")]
    InvalidModuleRequirementType(String),
    #[error("invalid module requirement: {0}")]
    InvalidModuleRequirement(String),
    #[error("invalid module prerequisites: {0}")]
    InvalidModulePrerequisites(String),
    #[error("invalid file id: {0}")]
    InvalidFileId(String),
    #[error("invalid folder id: {0}")]
    InvalidFolderId(String),
    #[error("invalid folder name: {0}")]
    InvalidFolderName(String),
    #[error("invalid discussion id: {0}")]
    InvalidDiscussionId(String),
    #[error("invalid group id: {0}")]
    InvalidGroupId(String),
    #[error("invalid group name: {0}")]
    InvalidGroupName(String),
    #[error("invalid user role: {0}")]
    InvalidUserRole(String),
    #[error("invalid message subject: {0}")]
    InvalidMessageSubject(String),
    #[error("invalid message body: {0}")]
    InvalidMessageBody(String),
    #[error("invalid recipients: {0}")]
    InvalidRecipients(String),
    #[error("invalid score: {0}")]
    InvalidScore(String),
    #[error("invalid points possible: {0}")]
    InvalidPointsPossible(String),
    #[error("invalid due date: {0}")]
    InvalidDueDate(String),
    #[error("invalid event timestamp: {0}")]
    InvalidEventDateTime(String),
    #[error("invalid all-day date: {0}")]
    InvalidAllDayDate(String),
    #[error("invalid quiz time limit: {0}")]
    InvalidQuizTimeLimit(String),
    #[error("invalid quiz access code: {0}")]
    InvalidQuizAccessCode(String),
    #[error("invalid publish state: {0}")]
    InvalidPublishState(String),
    #[error("invalid rubric selection: {0}")]
    InvalidRubricSelection(String),
    #[error("invalid rubric assessment: {0}")]
    InvalidRubricAssessment(String),
    #[error("invalid rubric criteria: {0}")]
    InvalidRubricCriteria(String),
    #[error("invalid course visibility: {0}")]
    InvalidCourseVisibility(String),
    #[error("invalid grading scheme id: {0}")]
    InvalidGradingSchemeId(String),
    #[error("invalid content migration id: {0}")]
    InvalidContentMigrationId(String),
    #[error("invalid content migration type: {0}")]
    InvalidContentMigrationType(String),
    #[error("invalid content migration create: {0}")]
    InvalidContentMigrationCreate(String),
    #[error("invalid report type: {0}")]
    InvalidReportType(String),
    #[error("invalid course dates: {0}")]
    InvalidCourseDates(String),
    #[error("invalid course update: {0}")]
    InvalidCourseUpdate(String),
    #[error("grading period not found: {0}")]
    GradingPeriodNotFound(u64),
    #[error("invalid assignment update: {0}")]
    InvalidAssignmentUpdate(String),
    #[error("invalid outcome update: {0}")]
    InvalidOutcomeUpdate(String),
    #[error("invalid quiz update: {0}")]
    InvalidQuizUpdate(String),
    #[error("invalid page update: {0}")]
    InvalidPageUpdate(String),
    #[error("invalid module update: {0}")]
    InvalidModuleUpdate(String),
    #[error("invalid module requirement update: {0}")]
    InvalidModuleRequirementUpdate(String),
    #[error("invalid module reorder: {0}")]
    InvalidModuleReorder(String),
    #[error("invalid rubric update: {0}")]
    InvalidRubricUpdate(String),
    #[error("invalid calendar event create: {0}")]
    InvalidCalendarEventCreate(String),
    #[error("invalid calendar event update: {0}")]
    InvalidCalendarEventUpdate(String),
    #[error("invalid calendar event context: {0}")]
    InvalidCalendarEventContext(String),
    #[error("invalid conference update: {0}")]
    InvalidConferenceUpdate(String),
    #[error("invalid rubric association target: {0}")]
    InvalidRubricAssociationTarget(String),
    #[error("missing assignment points for assignment id {0}")]
    MissingAssignmentPoints(u64),
    #[error("missing quiz points for quiz id {0}")]
    MissingQuizPoints(u64),
    #[error("invalid quiz availability: {0}")]
    InvalidQuizAvailability(String),
    #[error("invalid file upload: {0}")]
    InvalidFileUpload(String),
    #[error("content migration failed: {0}")]
    ContentMigrationFailed(String),
    #[error("content migration polling timed out: {0}")]
    ContentMigrationTimeout(String),
    #[error("report not ready: {0}")]
    ReportNotReady(String),
    #[error("report download failed: {0}")]
    ReportDownloadFailed(String),
    #[error("report polling timed out: {0}")]
    ReportTimeout(String),
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

pub fn parse_content_migration_id(
    raw: &str,
) -> Result<ContentMigrationId, CanvasError> {
    raw.parse::<ContentMigrationId>()
        .map_err(|_| CanvasError::InvalidContentMigrationId(raw.to_string()))
}

pub fn parse_content_migration_type(
    raw: &str,
) -> Result<ContentMigrationType, CanvasError> {
    raw.parse::<ContentMigrationType>()
        .map_err(|_| CanvasError::InvalidContentMigrationType(raw.to_string()))
}

pub fn parse_grading_period_id(
    raw: &str,
) -> Result<canvas_models::GradingPeriodId, CanvasError> {
    raw.parse::<canvas_models::GradingPeriodId>()
        .map_err(|_| CanvasError::InvalidGradingPeriodId(raw.to_string()))
}

pub fn parse_assignment_id(raw: &str) -> Result<AssignmentId, CanvasError> {    
    raw.parse::<AssignmentId>()
        .map_err(|_| CanvasError::InvalidAssignmentId(raw.to_string()))
}

pub fn parse_assignment_name(raw: &str) -> Result<AssignmentName, CanvasError> {
    raw.parse::<AssignmentName>()
        .map_err(|_| CanvasError::InvalidAssignmentName(raw.to_string()))
}

pub fn parse_outcome_id(raw: &str) -> Result<OutcomeId, CanvasError> {
    raw.parse::<OutcomeId>()
        .map_err(|_| CanvasError::InvalidOutcomeId(raw.to_string()))
}

pub fn parse_outcome_group_id(raw: &str) -> Result<OutcomeGroupId, CanvasError> {
    raw.parse::<OutcomeGroupId>()
        .map_err(|_| CanvasError::InvalidOutcomeGroupId(raw.to_string()))
}

pub fn parse_outcome_title(raw: &str) -> Result<OutcomeTitle, CanvasError> {
    raw.parse::<OutcomeTitle>()
        .map_err(|_| CanvasError::InvalidOutcomeTitle(raw.to_string()))
}

pub fn parse_outcome_description(
    raw: &str,
) -> Result<OutcomeDescription, CanvasError> {
    raw.parse::<OutcomeDescription>()
        .map_err(|_| CanvasError::InvalidOutcomeDescription(raw.to_string()))
}

pub fn parse_rubric_id(raw: &str) -> Result<RubricId, CanvasError> {
    raw.parse::<RubricId>()
        .map_err(|_| CanvasError::InvalidRubricId(raw.to_string()))
}

pub fn parse_rubric_association_id(
    raw: &str,
) -> Result<RubricAssociationId, CanvasError> {
    raw.parse::<RubricAssociationId>()
        .map_err(|_| CanvasError::InvalidRubricAssociationId(raw.to_string()))
}

pub fn parse_rubric_title(raw: &str) -> Result<RubricTitle, CanvasError> {
    raw.parse::<RubricTitle>()
        .map_err(|_| CanvasError::InvalidRubricTitle(raw.to_string()))
}

pub fn parse_quiz_id(raw: &str) -> Result<QuizId, CanvasError> {
    raw.parse::<QuizId>()
        .map_err(|_| CanvasError::InvalidQuizId(raw.to_string()))
}

pub fn parse_quiz_submission_id(
    raw: &str,
) -> Result<QuizSubmissionId, CanvasError> {
    raw.parse::<QuizSubmissionId>()
        .map_err(|_| CanvasError::InvalidQuizSubmissionId(raw.to_string()))
}

pub fn parse_quiz_title(raw: &str) -> Result<QuizTitle, CanvasError> {
    raw.parse::<QuizTitle>()
        .map_err(|_| CanvasError::InvalidQuizTitle(raw.to_string()))
}

pub fn parse_question_bank_id(raw: &str) -> Result<QuestionBankId, CanvasError> {
    raw.parse::<QuestionBankId>()
        .map_err(|_| CanvasError::InvalidQuestionBankId(raw.to_string()))
}

pub fn parse_question_id(raw: &str) -> Result<QuestionId, CanvasError> {
    raw.parse::<QuestionId>()
        .map_err(|_| CanvasError::InvalidQuestionId(raw.to_string()))
}

pub fn parse_question_bank_title(
    raw: &str,
) -> Result<QuestionBankTitle, CanvasError> {
    raw.parse::<QuestionBankTitle>()
        .map_err(|_| CanvasError::InvalidQuestionBankTitle(raw.to_string()))
}

pub fn parse_question_name(raw: &str) -> Result<QuestionName, CanvasError> {
    raw.parse::<QuestionName>()
        .map_err(|_| CanvasError::InvalidQuestionName(raw.to_string()))
}

pub fn parse_question_text(raw: &str) -> Result<QuestionText, CanvasError> {
    raw.parse::<QuestionText>()
        .map_err(|_| CanvasError::InvalidQuestionText(raw.to_string()))
}

pub fn parse_question_type(raw: &str) -> Result<QuestionType, CanvasError> {
    raw.parse::<QuestionType>()
        .map_err(|_| CanvasError::InvalidQuestionType(raw.to_string()))
}

pub fn parse_calendar_event_id(
    raw: &str,
) -> Result<CalendarEventId, CanvasError> {
    raw.parse::<CalendarEventId>()
        .map_err(|_| CanvasError::InvalidCalendarEventId(raw.to_string()))
}

pub fn parse_conference_id(raw: &str) -> Result<ConferenceId, CanvasError> {
    raw.parse::<ConferenceId>()
        .map_err(|_| CanvasError::InvalidConferenceId(raw.to_string()))
}

pub fn parse_collaboration_id(
    raw: &str,
) -> Result<CollaborationId, CanvasError> {
    raw.parse::<CollaborationId>()
        .map_err(|_| CanvasError::InvalidCollaborationId(raw.to_string()))
}

pub fn parse_section_id(raw: &str) -> Result<SectionId, CanvasError> {
    raw.parse::<SectionId>()
        .map_err(|_| CanvasError::InvalidSectionId(raw.to_string()))
}

pub fn parse_external_tool_id(raw: &str) -> Result<ExternalToolId, CanvasError> {
    raw.parse::<ExternalToolId>()
        .map_err(|_| CanvasError::InvalidExternalToolId(raw.to_string()))
}

pub fn parse_external_tool_name(raw: &str) -> Result<ExternalToolName, CanvasError> {
    raw.parse::<ExternalToolName>()
        .map_err(|_| CanvasError::InvalidExternalToolName(raw.to_string()))
}

pub fn parse_external_tool_config_url(
    raw: &str,
) -> Result<ExternalToolConfigUrl, CanvasError> {
    raw.parse::<ExternalToolConfigUrl>()
        .map_err(|_| CanvasError::InvalidExternalToolConfig(raw.to_string()))
}

pub fn parse_external_tool_placement(
    raw: &str,
) -> Result<ExternalToolPlacement, CanvasError> {
    raw.parse::<ExternalToolPlacement>()
        .map_err(|_| CanvasError::InvalidExternalToolPlacement(raw.to_string()))
}

pub fn parse_external_tool_config_xml(
    raw: String,
) -> Result<external_tools::ExternalToolConfigXml, CanvasError> {
    external_tools::ExternalToolConfigXml::new(raw)
        .map_err(|_| CanvasError::InvalidExternalToolConfig("empty".to_string()))
}

pub fn parse_external_tool_config_json(
    raw: &str,
) -> Result<external_tools::ExternalToolConfigJson, CanvasError> {
    let parsed: serde_json::Value = serde_json::from_str(raw).map_err(|_| {
        CanvasError::InvalidExternalToolConfig("invalid_json".to_string())
    })?;
    external_tools::ExternalToolConfigJson::new(parsed).map_err(|_| {
        CanvasError::InvalidExternalToolConfig("invalid_json".to_string())
    })
}

pub fn parse_external_tool_placement_list(
    placements: Vec<ExternalToolPlacement>,
) -> Result<external_tools::ExternalToolPlacementList, CanvasError> {
    external_tools::ExternalToolPlacementList::new(placements).map_err(|_| {
        CanvasError::InvalidExternalToolPlacements("empty".to_string())
    })
}

pub fn parse_external_tool_placement_settings(
    raw: &str,
) -> Result<external_tools::ExternalToolPlacementSettings, CanvasError> {
    let parsed: serde_json::Value = serde_json::from_str(raw).map_err(|_| {
        CanvasError::InvalidExternalToolPlacements("invalid_json".to_string())
    })?;
    external_tools::ExternalToolPlacementSettings::new(parsed).map_err(|_| {
        CanvasError::InvalidExternalToolPlacements("invalid_json".to_string())
    })
}

pub fn parse_calendar_event_title(
    raw: &str,
) -> Result<CalendarEventTitle, CanvasError> {
    raw.parse::<CalendarEventTitle>()
        .map_err(|_| CanvasError::InvalidCalendarEventTitle(raw.to_string()))
}

pub fn parse_conference_title(
    raw: &str,
) -> Result<ConferenceTitle, CanvasError> {
    raw.parse::<ConferenceTitle>()
        .map_err(|_| CanvasError::InvalidConferenceTitle(raw.to_string()))
}

pub fn parse_collaboration_title(
    raw: &str,
) -> Result<CollaborationTitle, CanvasError> {
    raw.parse::<CollaborationTitle>()
        .map_err(|_| CanvasError::InvalidCollaborationTitle(raw.to_string()))
}

pub fn parse_collaboration_type(
    raw: &str,
) -> Result<CollaborationType, CanvasError> {
    raw.parse::<CollaborationType>()
        .map_err(|_| CanvasError::InvalidCollaborationType(raw.to_string()))
}

pub fn parse_conference_description(
    raw: &str,
) -> Result<ConferenceDescription, CanvasError> {
    raw.parse::<ConferenceDescription>()
        .map_err(|_| CanvasError::InvalidConferenceDescription(raw.to_string()))
}

pub fn parse_conference_duration(
    raw: &str,
) -> Result<ConferenceDuration, CanvasError> {
    raw.parse::<ConferenceDuration>()
        .map_err(|_| CanvasError::InvalidConferenceDuration(raw.to_string()))
}

pub fn parse_user_id(raw: &str) -> Result<UserId, CanvasError> {
    raw.parse::<UserId>()
        .map_err(|_| CanvasError::InvalidUserId(raw.to_string()))
}

pub fn parse_submission_id(raw: &str) -> Result<SubmissionId, CanvasError> {
    raw.parse::<SubmissionId>()
        .map_err(|_| CanvasError::InvalidSubmissionId(raw.to_string()))
}

pub fn parse_page_id(raw: &str) -> Result<PageId, CanvasError> {
    raw.parse::<PageId>()
        .map_err(|_| CanvasError::InvalidPageId(raw.to_string()))
}

pub fn parse_page_title(raw: &str) -> Result<PageTitle, CanvasError> {
    raw.parse::<PageTitle>()
        .map_err(|_| CanvasError::InvalidPageTitle(raw.to_string()))
}

pub fn parse_page_body(raw: &str) -> Result<PageBody, CanvasError> {
    raw.parse::<PageBody>()
        .map_err(|_| CanvasError::InvalidPageBody(raw.to_string()))
}

pub fn parse_module_id(raw: &str) -> Result<ModuleId, CanvasError> {
    raw.parse::<ModuleId>()
        .map_err(|_| CanvasError::InvalidModuleId(raw.to_string()))
}

pub fn parse_module_item_id(raw: &str) -> Result<ModuleItemId, CanvasError> {
    raw.parse::<ModuleItemId>()
        .map_err(|_| CanvasError::InvalidModuleItemId(raw.to_string()))
}

pub fn parse_module_name(raw: &str) -> Result<ModuleName, CanvasError> {
    raw.parse::<ModuleName>()
        .map_err(|_| CanvasError::InvalidModuleName(raw.to_string()))
}

pub fn parse_module_requirement_type(
    raw: &str,
) -> Result<ModuleRequirementType, CanvasError> {
    raw.parse::<ModuleRequirementType>()
        .map_err(|_| CanvasError::InvalidModuleRequirementType(raw.to_string()))
}

pub fn parse_module_requirements(
    raw: &str,
) -> Result<ModuleRequirements, CanvasError> {
    let value: serde_json::Value =
        serde_json::from_str(raw).map_err(|_| {
            CanvasError::InvalidModuleRequirement("invalid_json".to_string())
        })?;
    let entries = value.as_array().ok_or_else(|| {
        CanvasError::InvalidModuleRequirement("expected_array".to_string())
    })?;
    let mut requirements = Vec::with_capacity(entries.len());
    for entry in entries {
        let object = entry.as_object().ok_or_else(|| {
            CanvasError::InvalidModuleRequirement("expected_object".to_string())
        })?;
        let item_id_value = object.get("item_id").ok_or_else(|| {
            CanvasError::InvalidModuleRequirement("missing_item_id".to_string())
        })?;
        let item_id_raw = match item_id_value {
            serde_json::Value::Number(value) => value.to_string(),
            serde_json::Value::String(value) => value.clone(),
            _ => {
                return Err(CanvasError::InvalidModuleRequirement(
                    "invalid_item_id".to_string(),
                ));
            }
        };
        let item_id = parse_module_item_id(&item_id_raw)?;
        let type_value = object.get("type").ok_or_else(|| {
            CanvasError::InvalidModuleRequirement("missing_type".to_string())
        })?;
        let type_raw = type_value
            .as_str()
            .ok_or_else(|| {
                CanvasError::InvalidModuleRequirement("invalid_type".to_string())
            })?;
        let requirement_type = parse_module_requirement_type(type_raw)?;
        let min_score_value = object.get("min_score");
        let requirement = match requirement_type {
            ModuleRequirementType::View => {
                if min_score_value.is_some() {
                    return Err(CanvasError::InvalidModuleRequirement(
                        "unexpected_min_score".to_string(),
                    ));
                }
                ModuleRequirement::View { item_id }
            }
            ModuleRequirementType::Submit => {
                if min_score_value.is_some() {
                    return Err(CanvasError::InvalidModuleRequirement(
                        "unexpected_min_score".to_string(),
                    ));
                }
                ModuleRequirement::Submit { item_id }
            }
            ModuleRequirementType::Contribute => {
                if min_score_value.is_some() {
                    return Err(CanvasError::InvalidModuleRequirement(
                        "unexpected_min_score".to_string(),
                    ));
                }
                ModuleRequirement::Contribute { item_id }
            }
            ModuleRequirementType::Score => {
                let min_score_value = min_score_value.ok_or_else(|| {
                    CanvasError::InvalidModuleRequirement(
                        "missing_min_score".to_string(),
                    )
                })?;
                let min_score_raw = match min_score_value {
                    serde_json::Value::Number(value) => value.to_string(),
                    serde_json::Value::String(value) => value.clone(),
                    _ => {
                        return Err(CanvasError::InvalidModuleRequirement(
                            "invalid_min_score".to_string(),
                        ));
                    }
                };
                let min_score = min_score_raw.parse::<f64>().map_err(|_| {
                    CanvasError::InvalidModuleRequirement(
                        "invalid_min_score".to_string(),
                    )
                })?;
                if !min_score.is_finite() || min_score < 0.0 {
                    return Err(CanvasError::InvalidModuleRequirement(
                        "invalid_min_score".to_string(),
                    ));
                }
                ModuleRequirement::Score { item_id, min_score }
            }
        };
        requirements.push(requirement);
    }
    Ok(ModuleRequirements::new(requirements))
}

pub fn parse_module_prerequisites(
    raw: &str,
) -> Result<ModulePrerequisites, CanvasError> {
    let value: serde_json::Value =
        serde_json::from_str(raw).map_err(|_| {
            CanvasError::InvalidModulePrerequisites("invalid_json".to_string())
        })?;
    let entries = value.as_array().ok_or_else(|| {
        CanvasError::InvalidModulePrerequisites("expected_array".to_string())
    })?;
    let mut prerequisites = Vec::with_capacity(entries.len());
    for entry in entries {
        let raw = match entry {
            serde_json::Value::Number(value) => value.to_string(),
            serde_json::Value::String(value) => value.clone(),
            _ => {
                return Err(CanvasError::InvalidModulePrerequisites(
                    "invalid_module_id".to_string(),
                ));
            }
        };
        let module_id = parse_module_id(&raw)?;
        prerequisites.push(module_id);
    }
    Ok(ModulePrerequisites::new(prerequisites))
}

pub fn parse_file_id(raw: &str) -> Result<FileId, CanvasError> {
    raw.parse::<FileId>()
        .map_err(|_| CanvasError::InvalidFileId(raw.to_string()))
}

pub fn parse_folder_id(raw: &str) -> Result<FolderId, CanvasError> {
    raw.parse::<FolderId>()
        .map_err(|_| CanvasError::InvalidFolderId(raw.to_string()))
}

pub fn parse_folder_name(raw: &str) -> Result<FolderName, CanvasError> {
    raw.parse::<FolderName>()
        .map_err(|_| CanvasError::InvalidFolderName(raw.to_string()))
}

pub fn parse_discussion_id(raw: &str) -> Result<DiscussionId, CanvasError> {
    raw.parse::<DiscussionId>()
        .map_err(|_| CanvasError::InvalidDiscussionId(raw.to_string()))
}

pub fn parse_group_id(raw: &str) -> Result<GroupId, CanvasError> {
    raw.parse::<GroupId>()
        .map_err(|_| CanvasError::InvalidGroupId(raw.to_string()))
}

pub fn parse_group_name(raw: &str) -> Result<GroupName, CanvasError> {
    raw.parse::<GroupName>()
        .map_err(|_| CanvasError::InvalidGroupName(raw.to_string()))
}

pub fn parse_user_role(raw: &str) -> Result<UserRole, CanvasError> {
    raw.parse::<UserRole>()
        .map_err(|_| CanvasError::InvalidUserRole(raw.to_string()))
}

pub fn parse_message_subject(raw: &str) -> Result<MessageSubject, CanvasError> {
    raw.parse::<MessageSubject>()
        .map_err(|_| CanvasError::InvalidMessageSubject(raw.to_string()))
}

pub fn parse_message_body(raw: &str) -> Result<MessageBody, CanvasError> {
    raw.parse::<MessageBody>()
        .map_err(|_| CanvasError::InvalidMessageBody(raw.to_string()))
}

pub fn parse_recipient_ids(
    ids: Vec<UserId>,
) -> Result<RecipientIds, CanvasError> {
    RecipientIds::new(ids)
        .map_err(|_| CanvasError::InvalidRecipients("empty".to_string()))
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

pub fn parse_group_category_id(
    raw: &str,
) -> Result<GroupCategoryId, CanvasError> {
    raw.parse::<GroupCategoryId>()
        .map_err(|_| CanvasError::InvalidGroupCategoryId(raw.to_string()))
}

pub fn parse_group_assignment_mode(
    raw: &str,
) -> Result<GroupAssignmentMode, CanvasError> {
    raw.parse::<GroupAssignmentMode>()
        .map_err(|_| CanvasError::InvalidGroupAssignmentMode(raw.to_string()))
}

pub fn parse_group_assignment_settings(
    mode: GroupAssignmentMode,
    category_id: Option<GroupCategoryId>,
) -> Result<GroupAssignmentSettings, CanvasError> {
    GroupAssignmentSettings::new(mode, category_id)
        .map_err(|_| CanvasError::InvalidGroupAssignmentSettings("invalid".to_string()))
}

pub fn parse_peer_review_mode(
    raw: &str,
) -> Result<PeerReviewMode, CanvasError> {
    raw.parse::<PeerReviewMode>()
        .map_err(|_| CanvasError::InvalidPeerReviewMode(raw.to_string()))
}

pub fn parse_peer_review_settings(
    mode: PeerReviewMode,
    assign_at: Option<DueDate>,
    due_at: Option<DueDate>,
) -> Result<PeerReviewSettings, CanvasError> {
    PeerReviewSettings::new(mode, assign_at, due_at)
        .map_err(|_| CanvasError::InvalidPeerReviewSettings("invalid".to_string()))
}

pub fn parse_grading_posting_policy(
    raw: &str,
) -> Result<GradingPostingPolicy, CanvasError> {
    raw.parse::<GradingPostingPolicy>()
        .map_err(|_| CanvasError::InvalidGradingPostingPolicy(raw.to_string()))
}

pub fn parse_muted_state(raw: &str) -> Result<MutedState, CanvasError> {
    raw.parse::<MutedState>()
        .map_err(|_| CanvasError::InvalidMutedState(raw.to_string()))
}

pub fn parse_assignment_override_dates(
    unlock_at: Option<DueDate>,
    due_at: Option<DueDate>,
    lock_at: Option<DueDate>,
) -> Result<AssignmentOverrideDates, CanvasError> {
    AssignmentOverrideDates::new(unlock_at, due_at, lock_at)
        .map_err(|_| CanvasError::InvalidAssignmentOverrideDates("invalid".to_string()))
}

pub fn parse_assignment_overrides(
    raw: &str,
) -> Result<AssignmentOverrides, CanvasError> {
    let value: serde_json::Value =
        serde_json::from_str(raw).map_err(|_| {
            CanvasError::InvalidAssignmentOverrides("invalid_json".to_string())
        })?;
    let entries = value.as_array().ok_or_else(|| {
        CanvasError::InvalidAssignmentOverrides("expected_array".to_string())
    })?;
    let mut overrides = Vec::with_capacity(entries.len());
    for entry in entries {
        let object = entry.as_object().ok_or_else(|| {
            CanvasError::InvalidAssignmentOverrides("expected_object".to_string())
        })?;
        let section_id_value = object.get("section_id");
        let student_ids_value = object.get("student_ids");
        let target = match (section_id_value, student_ids_value) {
            (Some(_), Some(_)) => {
                return Err(CanvasError::InvalidAssignmentOverrideTarget(
                    "conflicting_target".to_string(),
                ));
            }
            (Some(section_id_value), None) => {
                let raw = match section_id_value {
                    serde_json::Value::Number(value) => value.to_string(),
                    serde_json::Value::String(value) => value.clone(),
                    _ => {
                        return Err(CanvasError::InvalidAssignmentOverrideTarget(
                            "invalid_section_id".to_string(),
                        ));
                    }
                };
                let section_id = raw.parse::<SectionId>().map_err(|_| {
                    CanvasError::InvalidAssignmentOverrideTarget(
                        "invalid_section_id".to_string(),
                    )
                })?;
                AssignmentOverrideTarget::Section(section_id)
            }
            (None, Some(student_ids_value)) => {
                let ids = student_ids_value.as_array().ok_or_else(|| {
                    CanvasError::InvalidAssignmentOverrideTarget(
                        "invalid_student_ids".to_string(),
                    )
                })?;
                let mut parsed_ids = Vec::with_capacity(ids.len());
                for id in ids {
                    let raw = match id {
                        serde_json::Value::Number(value) => value.to_string(),
                        serde_json::Value::String(value) => value.clone(),
                        _ => {
                            return Err(
                                CanvasError::InvalidAssignmentOverrideTarget(
                                    "invalid_student_ids".to_string(),
                                ),
                            );
                        }
                    };
                    let user_id = raw.parse::<UserId>().map_err(|_| {
                        CanvasError::InvalidAssignmentOverrideTarget(
                            "invalid_student_ids".to_string(),
                        )
                    })?;
                    parsed_ids.push(user_id);
                }
                let parsed_ids = OverrideStudentIds::new(parsed_ids).map_err(|_| {
                    CanvasError::InvalidAssignmentOverrideTarget(
                        "invalid_student_ids".to_string(),
                    )
                })?;
                AssignmentOverrideTarget::Students(parsed_ids)
            }
            (None, None) => {
                return Err(CanvasError::InvalidAssignmentOverrideTarget(
                    "missing_target".to_string(),
                ));
            }
        };
        let unlock_at = match object.get("unlock_at") {
            Some(serde_json::Value::String(value)) => {
                Some(parse_due_date(value)?)
            }
            Some(serde_json::Value::Null) | None => None,
            _ => {
                return Err(CanvasError::InvalidAssignmentOverrideDates(
                    "invalid_unlock_at".to_string(),
                ));
            }
        };
        let due_at = match object.get("due_at") {
            Some(serde_json::Value::String(value)) => {
                Some(parse_due_date(value)?)
            }
            Some(serde_json::Value::Null) | None => None,
            _ => {
                return Err(CanvasError::InvalidAssignmentOverrideDates(
                    "invalid_due_at".to_string(),
                ));
            }
        };
        let lock_at = match object.get("lock_at") {
            Some(serde_json::Value::String(value)) => {
                Some(parse_due_date(value)?)
            }
            Some(serde_json::Value::Null) | None => None,
            _ => {
                return Err(CanvasError::InvalidAssignmentOverrideDates(
                    "invalid_lock_at".to_string(),
                ));
            }
        };
        let dates = parse_assignment_override_dates(unlock_at, due_at, lock_at)?;
        overrides.push(AssignmentOverride::new(target, dates));
    }
    AssignmentOverrides::new(overrides)
        .map_err(|_| CanvasError::InvalidAssignmentOverrides("empty".to_string()))
}

pub fn parse_event_date_time(
    raw: &str,
) -> Result<EventDateTime, CanvasError> {
    raw.parse::<EventDateTime>()
        .map_err(|_| CanvasError::InvalidEventDateTime(raw.to_string()))
}

pub fn parse_all_day_date(raw: &str) -> Result<AllDayDate, CanvasError> {
    raw.parse::<AllDayDate>()
        .map_err(|_| CanvasError::InvalidAllDayDate(raw.to_string()))
}

pub fn parse_calendar_event_context(
    course_id: Option<CourseId>,
    section_id: Option<SectionId>,
) -> Result<CalendarEventContext, CanvasError> {
    match (course_id, section_id) {
        (Some(course_id), None) => Ok(CalendarEventContext::Course(course_id)),
        (None, Some(section_id)) => Ok(CalendarEventContext::Section(section_id)),
        (None, None) => Err(CanvasError::InvalidCalendarEventContext(
            "missing_context".to_string(),
        )),
        _ => Err(CanvasError::InvalidCalendarEventContext(
            "conflicting_context".to_string(),
        )),
    }
}

pub fn parse_quiz_time_limit(raw: &str) -> Result<QuizTimeLimit, CanvasError> {
    raw.parse::<QuizTimeLimit>()
        .map_err(|_| CanvasError::InvalidQuizTimeLimit(raw.to_string()))
}

pub fn parse_quiz_access_code(
    raw: &str,
) -> Result<QuizAccessCode, CanvasError> {
    raw.parse::<QuizAccessCode>()
        .map_err(|_| CanvasError::InvalidQuizAccessCode(raw.to_string()))
}

pub fn parse_quiz_availability(
    unlock_at: Option<DueDate>,
    due_at: Option<DueDate>,
    lock_at: Option<DueDate>,
) -> Result<Option<QuizAvailability>, CanvasError> {
    if unlock_at.is_none() && due_at.is_none() && lock_at.is_none() {
        return Ok(None);
    }
    QuizAvailability::new(unlock_at, due_at, lock_at)
        .map(Some)
        .map_err(|_| CanvasError::InvalidQuizAvailability("invalid_window".to_string()))
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

pub fn parse_rubric_criteria(raw: &str) -> Result<rubrics::RubricCriteria, CanvasError> {
    rubrics::RubricCriteria::from_json(raw)
        .map_err(|_| CanvasError::InvalidRubricCriteria(raw.to_string()))
}

pub fn parse_rubric_association_target(
    assignment_id: Option<AssignmentId>,
    outcome_id: Option<OutcomeId>,
) -> Result<rubrics::RubricAssociationTarget, CanvasError> {
    match (assignment_id, outcome_id) {
        (Some(assignment_id), None) => Ok(rubrics::RubricAssociationTarget::Assignment(
            assignment_id,
        )),
        (None, Some(outcome_id)) => Ok(rubrics::RubricAssociationTarget::Outcome(outcome_id)),
        (None, None) => Err(CanvasError::InvalidRubricAssociationTarget(
            "missing_target".to_string(),
        )),
        _ => Err(CanvasError::InvalidRubricAssociationTarget(
            "conflicting_target".to_string(),
        )),
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

pub fn parse_report_type(raw: &str) -> Result<ReportType, CanvasError> {
    raw.parse::<ReportType>()
        .map_err(|_| CanvasError::InvalidReportType(raw.to_string()))
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

#[cfg(test)]
mod tests {
    use super::{
        parse_assignment_overrides, parse_course_dates, parse_external_tool_config_json,
        parse_external_tool_config_url, parse_external_tool_id, parse_external_tool_name,
        parse_external_tool_placement, parse_grading_period_id, parse_host,
        parse_module_prerequisites, parse_module_requirements, parse_recipient_ids,
        parse_rubric_assessment, parse_rubric_association_target, parse_score,
        parse_token,
    };
    use canvas_models::{AssignmentId, DueDate, OutcomeId};

    #[test]
    fn parse_score_rejects_out_of_range_values() {
        let parsed = parse_score("11", 10.0);
        assert!(parsed.is_err());
    }

    #[test]
    fn parse_score_rejects_non_numbers() {
        let parsed = parse_score("nope", 10.0);
        assert!(parsed.is_err());
    }

    #[test]
    fn parse_rubric_assessment_rejects_non_object_json() {
        let parsed = parse_rubric_assessment(r#"["one"]"#);
        assert!(parsed.is_err());
    }

    #[test]
    fn parse_recipient_ids_requires_non_empty() {
        let parsed = parse_recipient_ids(Vec::new());
        assert!(parsed.is_err());
    }

    #[test]
    fn parse_course_dates_rejects_start_after_end() {
        let start: DueDate = "2025-01-02T00:00:00Z".parse().expect("start");
        let end: DueDate = "2025-01-01T00:00:00Z".parse().expect("end");
        let parsed = parse_course_dates(Some(start), Some(end));
        assert!(parsed.is_err());
    }

    #[test]
    fn parse_host_rejects_missing_scheme() {
        let parsed = parse_host("example.instructure.com");
        assert!(parsed.is_err());
    }

    #[test]
    fn parse_token_rejects_whitespace() {
        let parsed = parse_token("tok en");
        assert!(parsed.is_err());
    }

    #[test]
    fn parse_rubric_association_target_requires_exactly_one() {
        let assignment: AssignmentId = "5".parse().expect("assignment");        
        let outcome: OutcomeId = "7".parse().expect("outcome");
        let parsed = parse_rubric_association_target(Some(assignment), None);   
        assert!(parsed.is_ok());
        let parsed = parse_rubric_association_target(Some(assignment), Some(outcome));
        assert!(parsed.is_err());
    }

    #[test]
    fn parse_grading_period_id_rejects_zero() {
        let parsed = parse_grading_period_id("0");
        assert!(parsed.is_err());
    }

    #[test]
    fn parse_assignment_overrides_accepts_section() {
        let raw = r#"[{"section_id":1,"due_at":"2025-01-01T00:00:00Z"}]"#;
        let parsed = parse_assignment_overrides(raw);
        assert!(parsed.is_ok());
    }

    #[test]
    fn parse_assignment_overrides_rejects_conflicting_targets() {
        let raw = r#"[{"section_id":1,"student_ids":[2]}]"#;
        let parsed = parse_assignment_overrides(raw);
        assert!(parsed.is_err());
    }

    #[test]
    fn parse_external_tool_id_rejects_zero() {
        let parsed = parse_external_tool_id("0");
        assert!(parsed.is_err());
    }

    #[test]
    fn parse_external_tool_name_rejects_empty() {
        let parsed = parse_external_tool_name(" ");
        assert!(parsed.is_err());
    }

    #[test]
    fn parse_external_tool_config_url_requires_http() {
        let parsed = parse_external_tool_config_url("ftp://example.com");
        assert!(parsed.is_err());
    }

    #[test]
    fn parse_external_tool_config_json_requires_object() {
        let parsed = parse_external_tool_config_json(r#"["tool"]"#);
        assert!(parsed.is_err());
    }

    #[test]
    fn parse_external_tool_placement_rejects_empty() {
        let parsed = parse_external_tool_placement(" ");
        assert!(parsed.is_err());
    }

    #[test]
    fn parse_module_requirements_accepts_score() {
        let raw = r#"[{"item_id":5,"type":"score","min_score":3.5}]"#;
        let parsed = parse_module_requirements(raw);
        assert!(parsed.is_ok());
    }

    #[test]
    fn parse_module_requirements_rejects_missing_min_score() {
        let raw = r#"[{"item_id":5,"type":"score"}]"#;
        let parsed = parse_module_requirements(raw);
        assert!(parsed.is_err());
    }

    #[test]
    fn parse_module_prerequisites_accepts_array() {
        let raw = r#"[1,"2"]"#;
        let parsed = parse_module_prerequisites(raw);
        assert!(parsed.is_ok());
    }
}
