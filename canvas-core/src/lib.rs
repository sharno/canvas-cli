mod auth;
mod api;
mod assignments;
mod calendar;
mod config;
mod courses;
mod content;
mod external_tools;
mod people;
mod reports;
mod quizzes;
mod outcomes;
mod rubrics;
mod question_banks;

use canvas_models::{
    AllDayDate, AssignmentId, AssignmentName, CalendarEventContext,
    CalendarEventId, CalendarEventTitle, CanvasHost, CanvasToken, CourseDates,
    CourseId, CourseVisibility, DiscussionId, DueDate, EventDateTime,
    ExternalToolConfigUrl, ExternalToolId, ExternalToolName, ExternalToolPlacement,
    FileId, FolderId, FolderName, GradingSchemeId, GroupId, GroupName, MessageBody,
    MessageSubject, ModuleId, ModuleName, OutcomeDescription, OutcomeGroupId,
    OutcomeId, OutcomeTitle, PageBody, PageId, PageTitle, PointsPossible,
    PublishState, QuestionBankId, QuestionBankTitle, QuestionId, QuestionName,
    QuestionText, QuestionType, QuizAccessCode, QuizAvailability, QuizId,
    QuizSubmissionId, QuizTimeLimit, QuizTitle, RecipientIds, ReportType,
    RubricAssessment, RubricAssociationId, RubricId, RubricSelection, RubricTitle,
    Score, SectionId, SubmissionId, UserId, UserRole,
};
use thiserror::Error;

pub use auth::auth_check;
pub use api::{ApiError, ApiErrorCode, CanvasClient, RetryPolicy};
pub use assignments::{
    create_assignment, delete_assignment, ensure_assignment_points, get_assignment,
    grade_submission, list_assignments, list_submissions, update_assignment,
    AssignmentCreateInput, AssignmentSummary, AssignmentUpdateInput, SubmissionSummary,
};
pub use calendar::{
    create_calendar_event, delete_calendar_event, list_calendar_events,
    update_calendar_event, CalendarEventCreateInput, CalendarEventSummary,
    CalendarEventTiming, CalendarEventUpdateInput,
};
pub use config::{config_path, load_merged_config, write_config, AuthConfig, CanvasConfig, DefaultsConfig};
pub use courses::{get_course, list_courses, persist_default_course, update_course, CourseSettingsUpdate, CourseSummary};
pub use content::{
    create_announcement, create_discussion, create_folder, create_module,
    create_page, delete_file, list_announcements, list_discussions, list_files,
    list_folders, list_modules, list_pages, reorder_modules, update_module,
    update_page, upload_file, AnnouncementSummary, DiscussionSummary, FileSummary,
    FolderSummary, ModuleCreateInput, ModuleSummary, ModuleUpdateInput,
    PageCreateInput, PageSummary, PageUpdateInput, UploadFileInput,
};
pub use external_tools::{
    create_external_tool, delete_external_tool, list_external_tools,
    update_external_tool, ExternalToolConfig, ExternalToolConfigJson,
    ExternalToolConfigXml, ExternalToolCreateInput, ExternalToolPlacementList,
    ExternalToolPlacementSettings, ExternalToolSummary, ExternalToolUpdateInput,
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
    #[error("invalid assignment id: {0}")]
    InvalidAssignmentId(String),
    #[error("invalid assignment name: {0}")]
    InvalidAssignmentName(String),
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
    #[error("invalid module name: {0}")]
    InvalidModuleName(String),
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
    #[error("invalid report type: {0}")]
    InvalidReportType(String),
    #[error("invalid course dates: {0}")]
    InvalidCourseDates(String),
    #[error("invalid course update: {0}")]
    InvalidCourseUpdate(String),
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

pub fn parse_module_name(raw: &str) -> Result<ModuleName, CanvasError> {
    raw.parse::<ModuleName>()
        .map_err(|_| CanvasError::InvalidModuleName(raw.to_string()))
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
        parse_course_dates, parse_external_tool_config_json, parse_external_tool_config_url,
        parse_external_tool_id, parse_external_tool_name, parse_external_tool_placement,
        parse_host, parse_recipient_ids, parse_rubric_assessment,
        parse_rubric_association_target, parse_score, parse_token,
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
}
