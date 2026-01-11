#![recursion_limit = "256"]

use std::collections::BTreeSet;
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;

use canvas_models::{
    AssignmentId, CourseId, FileId, FolderId, ModuleId, PageId, QuizId,
    QuizSubmissionId, ReportType, UserId,
};
use clap::{Args, Parser, Subcommand};
use csv::ReaderBuilder;
use serde_json::{json, Value};
use thiserror::Error;
use tracing::info;

const SCHEMA_VERSION: &str = "v1";

#[derive(Debug, Parser)]
#[command(
    name = "canvas",
    version,
    about = "Canvas CLI",
    after_help = "Examples:\n  canvas auth check\n  canvas config init --confirm\n  canvas course list --course 42\n  canvas ask \"list assignments\" --json\n  canvas --schema"
)]
struct Cli {
    #[command(flatten)]
    global: GlobalArgs,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Args)]
struct GlobalArgs {
    /// Default course id for course-related commands
    #[arg(long, global = true)]
    course: Option<String>,
    /// Emit machine-readable JSON output
    #[arg(long, global = true)]
    json: bool,
    /// Suppress human-readable output
    #[arg(long, global = true)]
    quiet: bool,
    /// Confirm execution of destructive actions
    #[arg(long, global = true)]
    confirm: bool,
    /// Describe planned actions without executing
    #[arg(long, global = true)]
    explain: bool,
    /// Emit JSON schema for CLI outputs
    #[arg(long, global = true)]
    schema: bool,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Authenticate with Canvas
    #[command(after_help = "Example:\n  canvas auth check")]
    Auth {
        #[command(subcommand)]
        command: AuthCommand,
    },
    /// Manage local configuration
    #[command(after_help = "Example:\n  canvas config init --confirm")]
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    /// Course-related operations
    #[command(after_help = "Examples:\n  canvas course list\n  canvas course show --course 42\n  canvas course set --course 42\n  canvas course update --course 42 --start 2025-01-01T00:00:00Z --visibility institution --confirm")]
    Course {
        #[command(subcommand)]
        command: CourseCommand,
    },
    /// Assignment-related operations
    #[command(after_help = "Examples:\n  canvas assignment list --course 42\n  canvas assignment create --course 42 --name \"Essay 1\" --points 20 --due-at 2025-01-10T00:00:00Z\n  canvas assignment update --course 42 --assignment 7 --name \"Essay 1\" --publish-state published\n  canvas assignment delete --course 42 --assignment 7 --confirm")]
    Assignment {
        #[command(subcommand)]
        command: AssignmentCommand,
    },
    /// Submission-related operations
    #[command(after_help = "Examples:\n  canvas submission list --course 42 --assignment 7\n  canvas submission grade --course 42 --assignment 7 --user 99 --score 18 --confirm\n  canvas submission import --course 42 --assignment 7 --format csv --file grades.csv --confirm")]
    Submission {
        #[command(subcommand)]
        command: SubmissionCommand,
    },
    /// Quiz-related operations
    #[command(after_help = "Examples:\n  canvas quiz list --course 42\n  canvas quiz create --course 42 --title \"Quiz 1\" --points 10 --due-at 2025-01-10T00:00:00Z\n  canvas quiz update --course 42 --quiz 7 --title \"Quiz 1\" --time-limit 30\n  canvas quiz publish --course 42 --quiz 7 --publish-state published --confirm\n  canvas quiz submission list --course 42 --quiz 7\n  canvas quiz submission grade --course 42 --quiz 7 --submission 9 --score 8 --confirm")]
    Quiz {
        #[command(subcommand)]
        command: QuizCommand,
    },
    /// Page-related operations
    #[command(after_help = "Examples:\n  canvas page list --course 42\n  canvas page create --course 42 --title \"Week 1\" --body \"Welcome\" --publish-state published\n  canvas page update --course 42 --page \"week-1\" --body \"Updated\" --publish-state unpublished\n  canvas page publish --course 42 --page \"week-1\" --publish-state published")]
    Page {
        #[command(subcommand)]
        command: PageCommand,
    },
    /// Module-related operations
    #[command(after_help = "Examples:\n  canvas module list --course 42\n  canvas module create --course 42 --name \"Week 1\" --publish-state published\n  canvas module update --course 42 --module 5 --name \"Week 1\" --publish-state unpublished\n  canvas module reorder --course 42 --module 5 --module 9\n  canvas module publish --course 42 --module 5 --publish-state published")]
    Module {
        #[command(subcommand)]
        command: ModuleCommand,
    },
    /// File-related operations
    #[command(after_help = "Examples:\n  canvas file list --course 42\n  canvas file upload --course 42 --file syllabus.pdf\n  canvas file delete --file 100 --confirm")]
    File {
        #[command(subcommand)]
        command: FileCommand,
    },
    /// Folder-related operations
    #[command(after_help = "Examples:\n  canvas folder list --course 42\n  canvas folder create --course 42 --name \"Week 1\" --parent-folder 3")]
    Folder {
        #[command(subcommand)]
        command: FolderCommand,
    },
    /// Announcement-related operations
    #[command(after_help = "Examples:\n  canvas announcement list --course 42\n  canvas announcement create --course 42 --title \"Welcome\" --message \"Hello\"")]
    Announcement {
        #[command(subcommand)]
        command: AnnouncementCommand,
    },
    /// Discussion-related operations
    #[command(after_help = "Examples:\n  canvas discussion list --course 42\n  canvas discussion create --course 42 --title \"Topic\" --message \"Discuss\"")]
    Discussion {
        #[command(subcommand)]
        command: DiscussionCommand,
    },
    /// User-related operations
    #[command(after_help = "Examples:\n  canvas user list --course 42\n  canvas user list --course 42 --role student")]
    User {
        #[command(subcommand)]
        command: UserCommand,
    },
    /// Messaging operations
    #[command(after_help = "Example:\n  canvas message send --user 99 --subject \"Hello\" --body \"Welcome\" --confirm")]
    Message {
        #[command(subcommand)]
        command: MessageCommand,
    },
    /// Group-related operations
    #[command(after_help = "Examples:\n  canvas group list --course 42\n  canvas group create --course 42 --name \"Project Teams\"")]
    Group {
        #[command(subcommand)]
        command: GroupCommand,
    },
    /// Analytics and report operations
    #[command(after_help = "Examples:\n  canvas report gradebook-export --course 42 --format csv\n  canvas report submission-status --course 42 --format json\n  canvas report course-activity --course 42")]
    Report {
        #[command(subcommand)]
        command: ReportCommand,
    },
    /// Natural-language command planning
    #[command(after_help = "Example:\n  canvas ask \"list assignments\" --json")]
    Ask {
        /// Natural-language prompt
        prompt: String,
    },
}

#[derive(Debug, Subcommand)]
enum AuthCommand {
    /// Validate the configured Canvas credentials
    Check,
}

#[derive(Debug, Subcommand)]
enum ConfigCommand {
    /// Initialize local configuration
    Init,
}

#[derive(Debug, Subcommand)]
enum CourseCommand {
    /// List courses (placeholder)
    List,
    /// Show a course summary
    Show {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
    },
    /// Set the default course id
    Set {
        /// Course id to set as default
        #[arg(long)]
        course: String,
    },
    /// Update course settings
    Update {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
        /// Course start date (RFC3339)
        #[arg(long)]
        start: Option<String>,
        /// Course end date (RFC3339)
        #[arg(long)]
        end: Option<String>,
        /// Course visibility (public, institution, course)
        #[arg(long)]
        visibility: Option<String>,
        /// Grading scheme id
        #[arg(long = "grading-scheme-id")]
        grading_scheme_id: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
enum AssignmentCommand {
    /// List assignments for a course
    List {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
    },
    /// Create an assignment
    Create {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
        /// Assignment name
        #[arg(long)]
        name: String,
        /// Assignment points possible
        #[arg(long)]
        points: Option<String>,
        /// Assignment due date (RFC3339)
        #[arg(long = "due-at")]
        due_at: Option<String>,
        /// Publish state (published, unpublished)
        #[arg(long = "publish-state")]
        publish_state: Option<String>,
    },
    /// Update an assignment
    Update {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
        /// Assignment id
        #[arg(long)]
        assignment: String,
        /// Assignment name
        #[arg(long)]
        name: Option<String>,
        /// Assignment points possible
        #[arg(long)]
        points: Option<String>,
        /// Assignment due date (RFC3339)
        #[arg(long = "due-at")]
        due_at: Option<String>,
        /// Publish state (published, unpublished)
        #[arg(long = "publish-state")]
        publish_state: Option<String>,
    },
    /// Delete an assignment
    Delete {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
        /// Assignment id
        #[arg(long)]
        assignment: String,
    },
}

#[derive(Debug, Subcommand)]
enum SubmissionCommand {
    /// List submissions for an assignment
    List {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
        /// Assignment id
        #[arg(long)]
        assignment: String,
    },
    /// Grade a submission
    Grade {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
        /// Assignment id
        #[arg(long)]
        assignment: String,
        /// User id
        #[arg(long)]
        user: String,
        /// Score to apply
        #[arg(long)]
        score: String,
        /// Rubric assessment JSON
        #[arg(long = "rubric-json")]
        rubric_json: Option<String>,
        /// Rubric assessment JSON file path
        #[arg(long = "rubric-file")]
        rubric_file: Option<PathBuf>,
    },
    /// Import grades from CSV or JSON
    Import {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
        /// Assignment id
        #[arg(long)]
        assignment: String,
        /// Import format (csv or json)
        #[arg(long)]
        format: ImportFormat,
        /// File path to import
        #[arg(long)]
        file: PathBuf,
    },
}

#[derive(Debug, Subcommand)]
enum QuizCommand {
    /// List quizzes for a course
    List {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
    },
    /// Create a quiz
    Create {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
        /// Quiz title
        #[arg(long)]
        title: String,
        /// Quiz points possible
        #[arg(long)]
        points: Option<String>,
        /// Quiz time limit in minutes
        #[arg(long = "time-limit")]
        time_limit: Option<String>,
        /// Quiz access code
        #[arg(long = "access-code")]
        access_code: Option<String>,
        /// Quiz unlock date (RFC3339)
        #[arg(long = "unlock-at")]
        unlock_at: Option<String>,
        /// Quiz due date (RFC3339)
        #[arg(long = "due-at")]
        due_at: Option<String>,
        /// Quiz lock date (RFC3339)
        #[arg(long = "lock-at")]
        lock_at: Option<String>,
        /// Publish state (published, unpublished)
        #[arg(long = "publish-state")]
        publish_state: Option<String>,
    },
    /// Update a quiz
    Update {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
        /// Quiz id
        #[arg(long)]
        quiz: String,
        /// Quiz title
        #[arg(long)]
        title: Option<String>,
        /// Quiz points possible
        #[arg(long)]
        points: Option<String>,
        /// Quiz time limit in minutes
        #[arg(long = "time-limit")]
        time_limit: Option<String>,
        /// Quiz access code
        #[arg(long = "access-code")]
        access_code: Option<String>,
        /// Quiz unlock date (RFC3339)
        #[arg(long = "unlock-at")]
        unlock_at: Option<String>,
        /// Quiz due date (RFC3339)
        #[arg(long = "due-at")]
        due_at: Option<String>,
        /// Quiz lock date (RFC3339)
        #[arg(long = "lock-at")]
        lock_at: Option<String>,
        /// Publish state (published, unpublished)
        #[arg(long = "publish-state")]
        publish_state: Option<String>,
    },
    /// Delete a quiz
    Delete {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
        /// Quiz id
        #[arg(long)]
        quiz: String,
    },
    /// Publish or unpublish a quiz
    Publish {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
        /// Quiz id
        #[arg(long)]
        quiz: String,
        /// Publish state (published, unpublished)
        #[arg(long = "publish-state")]
        publish_state: String,
    },
    /// Quiz submission operations
    Submission {
        #[command(subcommand)]
        command: QuizSubmissionCommand,
    },
}

#[derive(Debug, Subcommand)]
enum QuizSubmissionCommand {
    /// List quiz submissions
    List {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
        /// Quiz id
        #[arg(long)]
        quiz: String,
    },
    /// Grade a quiz submission
    Grade {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
        /// Quiz id
        #[arg(long)]
        quiz: String,
        /// Quiz submission id
        #[arg(long = "submission")]
        submission: String,
        /// Score to apply
        #[arg(long)]
        score: String,
    },
}

#[derive(Debug, Subcommand)]
enum PageCommand {
    /// List pages for a course
    List {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
    },
    /// Create a page
    Create {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
        /// Page title
        #[arg(long)]
        title: String,
        /// Page body
        #[arg(long)]
        body: String,
        /// Publish state (published, unpublished)
        #[arg(long = "publish-state")]
        publish_state: Option<String>,
    },
    /// Update a page
    Update {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
        /// Page id (URL slug)
        #[arg(long)]
        page: String,
        /// Page title
        #[arg(long)]
        title: Option<String>,
        /// Page body
        #[arg(long)]
        body: Option<String>,
        /// Publish state (published, unpublished)
        #[arg(long = "publish-state")]
        publish_state: Option<String>,
    },
    /// Publish or unpublish a page
    Publish {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
        /// Page id (URL slug)
        #[arg(long)]
        page: String,
        /// Publish state (published, unpublished)
        #[arg(long = "publish-state")]
        publish_state: String,
    },
}

#[derive(Debug, Subcommand)]
enum ModuleCommand {
    /// List modules for a course
    List {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
    },
    /// Create a module
    Create {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
        /// Module name
        #[arg(long)]
        name: String,
        /// Publish state (published, unpublished)
        #[arg(long = "publish-state")]
        publish_state: Option<String>,
    },
    /// Update a module
    Update {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
        /// Module id
        #[arg(long)]
        module: String,
        /// Module name
        #[arg(long)]
        name: Option<String>,
        /// Publish state (published, unpublished)
        #[arg(long = "publish-state")]
        publish_state: Option<String>,
    },
    /// Reorder modules
    Reorder {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
        /// Module ids in desired order
        #[arg(long = "module")]
        module_ids: Vec<String>,
    },
    /// Publish or unpublish a module
    Publish {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
        /// Module id
        #[arg(long)]
        module: String,
        /// Publish state (published, unpublished)
        #[arg(long = "publish-state")]
        publish_state: String,
    },
}

#[derive(Debug, Subcommand)]
enum FileCommand {
    /// List files for a course
    List {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
        /// Folder id
        #[arg(long = "folder")]
        folder: Option<String>,
    },
    /// Upload a file
    Upload {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
        /// File path to upload
        #[arg(long)]
        file: PathBuf,
        /// Parent folder id
        #[arg(long = "parent-folder")]
        parent_folder: Option<String>,
    },
    /// Delete a file
    Delete {
        /// File id
        #[arg(long)]
        file: String,
    },
}

#[derive(Debug, Subcommand)]
enum FolderCommand {
    /// List folders for a course
    List {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
    },
    /// Create a folder
    Create {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
        /// Folder name
        #[arg(long)]
        name: String,
        /// Parent folder id
        #[arg(long = "parent-folder")]
        parent_folder: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
enum AnnouncementCommand {
    /// List announcements for a course
    List {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
    },
    /// Create an announcement
    #[command(visible_alias = "send")]
    Create {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
        /// Announcement title
        #[arg(long)]
        title: String,
        /// Announcement message
        #[arg(long)]
        message: String,
    },
}

#[derive(Debug, Subcommand)]
enum DiscussionCommand {
    /// List discussions for a course
    List {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
    },
    /// Create a discussion
    Create {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
        /// Discussion title
        #[arg(long)]
        title: String,
        /// Discussion message
        #[arg(long)]
        message: String,
    },
}

#[derive(Debug, Subcommand)]
enum UserCommand {
    /// List users in a course
    List {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
        /// Role filter (student, ta, teacher)
        #[arg(long)]
        role: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
enum MessageCommand {
    /// Send a message to users
    Send {
        /// Course id override (required when using role filter)
        #[arg(long)]
        course: Option<String>,
        /// Message subject
        #[arg(long)]
        subject: String,
        /// Message body
        #[arg(long)]
        body: String,
        /// Recipient user id (repeatable)
        #[arg(long = "user")]
        user_ids: Vec<String>,
        /// Role filter (student, ta, teacher)
        #[arg(long)]
        role: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
enum GroupCommand {
    /// List groups in a course
    List {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
    },
    /// Create a group in a course
    Create {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
        /// Group name
        #[arg(long)]
        name: String,
    },
}

#[derive(Debug, Subcommand)]
enum ReportCommand {
    /// Export the course gradebook
    GradebookExport {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
        /// Output format (json or csv)
        #[arg(long, value_enum, default_value = "json")]
        format: ReportFormat,
        /// Output file path for CSV
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Export missing/late submission status
    SubmissionStatus {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
        /// Output format (json or csv)
        #[arg(long, value_enum, default_value = "json")]
        format: ReportFormat,
        /// Output file path for CSV
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Fetch course activity summary if available
    CourseActivity {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
    },
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
enum ImportFormat {
    Csv,
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
enum ReportFormat {
    Json,
    Csv,
}

#[derive(Debug)]
struct CourseListRequest;

#[derive(Debug)]
struct CourseShowRequest {
    course_id: CourseId,
}

#[derive(Debug)]
struct CourseSetRequest {
    course_id: CourseId,
}

#[derive(Debug)]
struct CourseUpdateRequest {
    course_id: CourseId,
    settings: canvas_core::CourseSettingsUpdate,
}

#[derive(Debug)]
enum CourseRequest {
    List(CourseListRequest),
    Show(CourseShowRequest),
    Set(CourseSetRequest),
    Update(CourseUpdateRequest),
}

#[derive(Debug)]
struct AssignmentListRequest {
    course_id: CourseId,
}

#[derive(Debug)]
struct AssignmentCreateRequest {
    course_id: CourseId,
    input: canvas_core::AssignmentCreateInput,
}

#[derive(Debug)]
struct AssignmentUpdateRequest {
    course_id: CourseId,
    assignment_id: AssignmentId,
    input: canvas_core::AssignmentUpdateInput,
}

#[derive(Debug)]
struct AssignmentDeleteRequest {
    course_id: CourseId,
    assignment_id: AssignmentId,
}

#[derive(Debug)]
enum AssignmentRequest {
    List(AssignmentListRequest),
    Create(AssignmentCreateRequest),
    Update(AssignmentUpdateRequest),
    Delete(AssignmentDeleteRequest),
}

#[derive(Debug)]
struct SubmissionListRequest {
    course_id: CourseId,
    assignment_id: AssignmentId,
}

#[derive(Debug)]
struct SubmissionGradeRequest {
    course_id: CourseId,
    assignment_id: AssignmentId,
    user_id: UserId,
    score: String,
    rubric_assessment: Option<canvas_models::RubricAssessment>,
}

#[derive(Debug)]
struct SubmissionImportRequest {
    course_id: CourseId,
    assignment_id: AssignmentId,
    format: ImportFormat,
    file: PathBuf,
}

#[derive(Debug)]
enum SubmissionRequest {
    List(SubmissionListRequest),
    Grade(SubmissionGradeRequest),
    Import(SubmissionImportRequest),
}

#[derive(Debug)]
struct QuizListRequest {
    course_id: CourseId,
}

#[derive(Debug)]
struct QuizCreateRequest {
    course_id: CourseId,
    input: canvas_core::QuizCreateInput,
}

#[derive(Debug)]
struct QuizUpdateRequest {
    course_id: CourseId,
    quiz_id: QuizId,
    input: canvas_core::QuizUpdateInput,
}

#[derive(Debug)]
struct QuizDeleteRequest {
    course_id: CourseId,
    quiz_id: QuizId,
}

#[derive(Debug)]
struct QuizPublishRequest {
    course_id: CourseId,
    quiz_id: QuizId,
    publish_state: canvas_models::PublishState,
}

#[derive(Debug)]
enum QuizRequest {
    List(QuizListRequest),
    Create(QuizCreateRequest),
    Update(QuizUpdateRequest),
    Delete(QuizDeleteRequest),
    Publish(QuizPublishRequest),
    Submission(QuizSubmissionRequest),
}

#[derive(Debug)]
struct QuizSubmissionListRequest {
    course_id: CourseId,
    quiz_id: QuizId,
}

#[derive(Debug)]
struct QuizSubmissionGradeRequest {
    course_id: CourseId,
    quiz_id: QuizId,
    submission_id: QuizSubmissionId,
    score: String,
}

#[derive(Debug)]
enum QuizSubmissionRequest {
    List(QuizSubmissionListRequest),
    Grade(QuizSubmissionGradeRequest),
}

#[derive(Debug)]
struct PageListRequest {
    course_id: CourseId,
}

#[derive(Debug)]
struct PageCreateRequest {
    course_id: CourseId,
    input: canvas_core::PageCreateInput,
}

#[derive(Debug)]
struct PageUpdateRequest {
    course_id: CourseId,
    page_id: PageId,
    input: canvas_core::PageUpdateInput,
}

#[derive(Debug)]
struct PagePublishRequest {
    course_id: CourseId,
    page_id: PageId,
    publish_state: canvas_models::PublishState,
}

#[derive(Debug)]
enum PageRequest {
    List(PageListRequest),
    Create(PageCreateRequest),
    Update(PageUpdateRequest),
    Publish(PagePublishRequest),
}

#[derive(Debug)]
struct ModuleListRequest {
    course_id: CourseId,
}

#[derive(Debug)]
struct ModuleCreateRequest {
    course_id: CourseId,
    input: canvas_core::ModuleCreateInput,
}

#[derive(Debug)]
struct ModuleUpdateRequest {
    course_id: CourseId,
    module_id: ModuleId,
    input: canvas_core::ModuleUpdateInput,
}

#[derive(Debug)]
struct ModuleReorderRequest {
    course_id: CourseId,
    module_ids: Vec<ModuleId>,
}

#[derive(Debug)]
struct ModulePublishRequest {
    course_id: CourseId,
    module_id: ModuleId,
    publish_state: canvas_models::PublishState,
}

#[derive(Debug)]
enum ModuleRequest {
    List(ModuleListRequest),
    Create(ModuleCreateRequest),
    Update(ModuleUpdateRequest),
    Reorder(ModuleReorderRequest),
    Publish(ModulePublishRequest),
}

#[derive(Debug)]
struct FileListRequest {
    course_id: CourseId,
    folder_id: Option<FolderId>,
}

#[derive(Debug)]
struct FileUploadRequest {
    course_id: CourseId,
    input: canvas_core::UploadFileInput,
}

#[derive(Debug)]
struct FileDeleteRequest {
    file_id: FileId,
}

#[derive(Debug)]
enum FileRequest {
    List(FileListRequest),
    Upload(FileUploadRequest),
    Delete(FileDeleteRequest),
}

#[derive(Debug)]
struct FolderListRequest {
    course_id: CourseId,
}

#[derive(Debug)]
struct FolderCreateRequest {
    course_id: CourseId,
    name: canvas_models::FolderName,
    parent_folder_id: Option<FolderId>,
}

#[derive(Debug)]
enum FolderRequest {
    List(FolderListRequest),
    Create(FolderCreateRequest),
}

#[derive(Debug)]
struct AnnouncementListRequest {
    course_id: CourseId,
}

#[derive(Debug)]
struct AnnouncementCreateRequest {
    course_id: CourseId,
    title: String,
    message: String,
}

#[derive(Debug)]
enum AnnouncementRequest {
    List(AnnouncementListRequest),
    Create(AnnouncementCreateRequest),
}

#[derive(Debug)]
struct DiscussionListRequest {
    course_id: CourseId,
}

#[derive(Debug)]
struct DiscussionCreateRequest {
    course_id: CourseId,
    title: String,
    message: String,
}

#[derive(Debug)]
enum DiscussionRequest {
    List(DiscussionListRequest),
    Create(DiscussionCreateRequest),
}

#[derive(Debug)]
struct UserListRequest {
    course_id: CourseId,
    role: Option<canvas_models::UserRole>,
}

#[derive(Debug)]
enum UserRequest {
    List(UserListRequest),
}

#[derive(Debug)]
struct MessageSendRequest {
    course_id: Option<CourseId>,
    subject: canvas_models::MessageSubject,
    body: canvas_models::MessageBody,
    user_ids: Vec<UserId>,
    role: Option<canvas_models::UserRole>,
}

#[derive(Debug)]
enum MessageRequest {
    Send(MessageSendRequest),
}

#[derive(Debug)]
struct GroupListRequest {
    course_id: CourseId,
}

#[derive(Debug)]
struct GroupCreateRequest {
    course_id: CourseId,
    name: canvas_models::GroupName,
}

#[derive(Debug)]
enum GroupRequest {
    List(GroupListRequest),
    Create(GroupCreateRequest),
}

#[derive(Debug)]
struct ReportExportRequest {
    course_id: CourseId,
    report_type: ReportType,
    format: ReportFormat,
    output: Option<PathBuf>,
}

#[derive(Debug)]
struct CourseActivitySummaryRequest {
    course_id: CourseId,
}

#[derive(Debug)]
enum ReportRequest {
    GradebookExport(ReportExportRequest),
    SubmissionStatus(ReportExportRequest),
    CourseActivitySummary(CourseActivitySummaryRequest),
}

#[derive(Debug)]
struct GlobalOptions {
    course: Option<CourseId>,
    json: bool,
    quiet: bool,
    confirm: bool,
    explain: bool,
}

#[derive(Debug)]
struct PlannedAction {
    action: &'static str,
    risk: &'static str,
    requires_confirmation: bool,
}

#[derive(Debug)]
struct AskPlan {
    prompt: String,
    rationale: String,
    commands: Vec<PlannedCommand>,
}

#[derive(Debug)]
struct PlannedCommand {
    id: &'static str,
    command: &'static str,
    params: Vec<(&'static str, String)>,
    risk: &'static str,
    destructive: bool,
}

#[derive(Debug, Error)]
enum CliError {
    #[error("missing command")]
    MissingCommand,
    #[error("missing course id")]
    MissingCourseId,
    #[error("confirmation required for {0}")]
    ConfirmationRequired(&'static str),
    #[error("unable to execute ask plan")]
    AskExecutionUnsupported,
    #[error("failed to read file at {0}: {1}")]
    FileRead(String, String),
    #[error("failed to write file at {0}: {1}")]
    FileWrite(String, String),
    #[error("invalid report format: {0}")]
    InvalidReportFormat(String),
    #[error(transparent)]
    Canvas(#[from] canvas_core::CanvasError),
}

fn main() {
    let cli = Cli::parse();
    let json_output = cli.global.json;
    if let Err(err) = run(cli) {
        emit_error(&err, json_output);
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), CliError> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let global = GlobalOptions::try_from(&cli.global)?;
    if cli.global.schema {
        emit_schema();
        return Ok(());
    }

    let command = cli.command.ok_or(CliError::MissingCommand)?;
    match command {
        Command::Auth { command } => handle_auth(command, &global),
        Command::Config { command } => handle_config(command, &global),
        Command::Course { command } => {
            let request = CourseRequest::try_from((command, &global))?;
            handle_course(request, &global)
        }
        Command::Assignment { command } => {
            let request = AssignmentRequest::try_from((command, &global))?;
            handle_assignment(request, &global)
        }
        Command::Submission { command } => {
            let request = SubmissionRequest::try_from((command, &global))?;
            handle_submission(request, &global)
        }
        Command::Quiz { command } => {
            let request = QuizRequest::try_from((command, &global))?;
            handle_quiz(request, &global)
        }
        Command::Page { command } => {
            let request = PageRequest::try_from((command, &global))?;
            handle_page(request, &global)
        }
        Command::Module { command } => {
            let request = ModuleRequest::try_from((command, &global))?;
            handle_module(request, &global)
        }
        Command::File { command } => {
            let request = FileRequest::try_from((command, &global))?;
            handle_file(request, &global)
        }
        Command::Folder { command } => {
            let request = FolderRequest::try_from((command, &global))?;
            handle_folder(request, &global)
        }
        Command::Announcement { command } => {
            let request = AnnouncementRequest::try_from((command, &global))?;
            handle_announcement(request, &global)
        }
        Command::Discussion { command } => {
            let request = DiscussionRequest::try_from((command, &global))?;
            handle_discussion(request, &global)
        }
        Command::User { command } => {
            let request = UserRequest::try_from((command, &global))?;
            handle_user(request, &global)
        }
        Command::Message { command } => {
            let request = MessageRequest::try_from((command, &global))?;
            handle_message(request, &global)
        }
        Command::Group { command } => {
            let request = GroupRequest::try_from((command, &global))?;
            handle_group(request, &global)
        }
        Command::Report { command } => {
            let request = ReportRequest::try_from((command, &global))?;
            handle_report(request, &global)
        }
        Command::Ask { prompt } => handle_ask(&prompt, &global),
    }
}

fn handle_auth(command: AuthCommand, global: &GlobalOptions) -> Result<(), CliError> {
    match command {
        AuthCommand::Check => {
            let planned = vec![PlannedAction {
                action: "validate stored Canvas credentials",
                risk: "none",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("auth check", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            canvas_core::auth_check(&config)?;
            emit_result(
                "auth check",
                json!({"status": "ok", "detail": "auth check succeeded"}),
                global,
            );
            info!("auth check succeeded");
        }
    }
    Ok(())
}

fn handle_config(command: ConfigCommand, global: &GlobalOptions) -> Result<(), CliError> {
    match command {
        ConfigCommand::Init => {
            let config_path = canvas_core::config_path();
            let planned = vec![PlannedAction {
                action: "prompt for Canvas host/token and write config",
                risk: "writes local config",
                requires_confirmation: true,
            }];
            if global.explain {
                emit_plan("config init", &planned, global);
                return Ok(());
            }
            require_confirmation(global, "config init")?;
            run_init(&config_path)?;
            emit_result(
                "config init",
                json!({"status": "ok", "config_path": config_path.display().to_string()}),
                global,
            );
            info!("wrote config to {}", config_path.display());
        }
    }
    Ok(())
}

fn handle_course(request: CourseRequest, global: &GlobalOptions) -> Result<(), CliError> {
    match request {
        CourseRequest::List(_) => {
            let planned = vec![PlannedAction {
                action: "list courses",
                risk: "none",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("course list", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let courses = canvas_core::list_courses(&config)?;
            let data = json!({
                "courses": courses.iter().map(course_summary_json).collect::<Vec<_>>(),
            });
            emit_result("course list", data, global);
            if !global.quiet && !global.json {
                println!("Found {} courses.", courses.len());
            }
        }
        CourseRequest::Show(request) => {
            let course_id = request.course_id;
            let planned = vec![PlannedAction {
                action: "show course summary",
                risk: "none",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("course show", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let course = canvas_core::get_course(&config, course_id)?;
            emit_result(
                "course show",
                json!({
                    "course": course_summary_json(&course),
                }),
                global,
            );
            if !global.quiet && !global.json {
                match &course.name {
                    Some(name) => println!("Course {}: {}", course.id, name),
                    None => println!("Course {}", course.id),
                }
            }
        }
        CourseRequest::Set(request) => {
            let planned = vec![PlannedAction {
                action: "set default course id",
                risk: "writes local config",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("course set", &planned, global);
                return Ok(());
            }
            let config_path = canvas_core::persist_default_course(request.course_id)?;
            emit_result(
                "course set",
                json!({
                    "default_course_id": request.course_id.get(),
                    "config_path": config_path.display().to_string(),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!(
                    "Default course set to {}.",
                    request.course_id.get()
                );
            }
        }
        CourseRequest::Update(request) => {
            let planned = vec![PlannedAction {
                action: "update course settings",
                risk: "updates remote course settings",
                requires_confirmation: true,
            }];
            if global.explain {
                emit_plan("course update", &planned, global);
                return Ok(());
            }
            require_confirmation(global, "course update")?;
            let config = canvas_core::load_merged_config()?;
            let course = canvas_core::update_course(&config, request.course_id, &request.settings)?;
            emit_result(
                "course update",
                json!({
                    "course": course_summary_json(&course),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("Course {} updated.", course.id);
            }
        }
    }
    Ok(())
}

fn handle_assignment(
    request: AssignmentRequest,
    global: &GlobalOptions,
) -> Result<(), CliError> {
    match request {
        AssignmentRequest::List(request) => {
            let planned = vec![PlannedAction {
                action: "list assignments",
                risk: "none",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("assignment list", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let assignments =
                canvas_core::list_assignments(&config, request.course_id)?;
            let data = json!({
                "status": "ok",
                "course_id": request.course_id.get(),
                "assignments": assignments
                    .iter()
                    .map(assignment_summary_json)
                    .collect::<Vec<_>>(),
            });
            emit_result("assignment list", data, global);
            if !global.quiet && !global.json {
                println!("Found {} assignments.", assignments.len());
            }
        }
        AssignmentRequest::Create(request) => {
            let planned = vec![PlannedAction {
                action: "create assignment",
                risk: "creates remote assignment",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("assignment create", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let assignment = canvas_core::create_assignment(
                &config,
                request.course_id,
                &request.input,
            )?;
            emit_result(
                "assignment create",
                json!({
                    "status": "created",
                    "course_id": request.course_id.get(),
                    "assignment": assignment_summary_json(&assignment),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("Assignment {} created.", assignment.id);
            }
        }
        AssignmentRequest::Update(request) => {
            let planned = vec![PlannedAction {
                action: "update assignment",
                risk: "updates remote assignment",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("assignment update", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let assignment = canvas_core::update_assignment(
                &config,
                request.course_id,
                request.assignment_id,
                &request.input,
            )?;
            emit_result(
                "assignment update",
                json!({
                    "status": "updated",
                    "course_id": request.course_id.get(),
                    "assignment": assignment_summary_json(&assignment),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("Assignment {} updated.", assignment.id);
            }
        }
        AssignmentRequest::Delete(request) => {
            let planned = vec![PlannedAction {
                action: "delete assignment",
                risk: "deletes remote assignment",
                requires_confirmation: true,
            }];
            if global.explain {
                emit_plan("assignment delete", &planned, global);
                return Ok(());
            }
            require_confirmation(global, "assignment delete")?;
            let config = canvas_core::load_merged_config()?;
            let assignment = canvas_core::delete_assignment(
                &config,
                request.course_id,
                request.assignment_id,
            )?;
            emit_result(
                "assignment delete",
                json!({
                    "status": "deleted",
                    "course_id": request.course_id.get(),
                    "assignment": assignment_summary_json(&assignment),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("Assignment {} deleted.", assignment.id);
            }
        }
    }
    Ok(())
}

fn handle_submission(
    request: SubmissionRequest,
    global: &GlobalOptions,
) -> Result<(), CliError> {
    match request {
        SubmissionRequest::List(request) => {
            let planned = vec![PlannedAction {
                action: "list submissions",
                risk: "none",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("submission list", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let submissions = canvas_core::list_submissions(
                &config,
                request.course_id,
                request.assignment_id,
            )?;
            emit_result(
                "submission list",
                json!({
                    "status": "ok",
                    "course_id": request.course_id.get(),
                    "assignment_id": request.assignment_id.get(),
                    "submissions": submissions
                        .iter()
                        .map(submission_summary_json)
                        .collect::<Vec<_>>(),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("Found {} submissions.", submissions.len());
            }
        }
        SubmissionRequest::Grade(request) => {
            let planned = vec![PlannedAction {
                action: "grade submission",
                risk: "updates remote submission grade",
                requires_confirmation: true,
            }];
            if global.explain {
                emit_plan("submission grade", &planned, global);
                return Ok(());
            }
            require_confirmation(global, "submission grade")?;
            let config = canvas_core::load_merged_config()?;
            let assignment = canvas_core::get_assignment(
                &config,
                request.course_id,
                request.assignment_id,
            )?;
            let max_points = canvas_core::ensure_assignment_points(
                request.assignment_id,
                assignment.points_possible,
            )?;
            let score = canvas_core::parse_score(&request.score, max_points)?;
            let submission = canvas_core::grade_submission(
                &config,
                request.course_id,
                request.assignment_id,
                request.user_id,
                score,
                request.rubric_assessment,
            )?;
            emit_result(
                "submission grade",
                json!({
                    "status": "graded",
                    "course_id": request.course_id.get(),
                    "assignment_id": request.assignment_id.get(),
                    "user_id": request.user_id.get(),
                    "submission": submission_summary_json(&submission),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!(
                    "Submission graded for user {} on assignment {}.",
                    request.user_id.get(),
                    request.assignment_id.get()
                );
            }
        }
        SubmissionRequest::Import(request) => {
            let planned = vec![PlannedAction {
                action: "bulk import grades",
                risk: "updates remote submission grades",
                requires_confirmation: true,
            }];
            if global.explain {
                emit_plan("submission import", &planned, global);
                return Ok(());
            }
            require_confirmation(global, "submission import")?;
            let config = canvas_core::load_merged_config()?;
            let assignment = canvas_core::get_assignment(
                &config,
                request.course_id,
                request.assignment_id,
            )?;
            let max_points = canvas_core::ensure_assignment_points(
                request.assignment_id,
                assignment.points_possible,
            )?;
            let rows = match request.format {
                ImportFormat::Csv => read_csv_import_rows(&request.file)?,
                ImportFormat::Json => read_json_import_rows(&request.file)?,
            };
            let mut results = Vec::new();
            let mut success = 0;
            let mut failed = 0;
            for row in rows {
                match parse_import_row(row, max_points) {
                    Ok((row_index, user_id, score)) => {
                        match canvas_core::grade_submission(
                            &config,
                            request.course_id,
                            request.assignment_id,
                            user_id,
                            score,
                            None,
                        ) {
                            Ok(submission) => {
                                success += 1;
                                results.push(json!({
                                    "row": row_index,
                                    "user_id": user_id.get(),
                                    "status": "success",
                                    "submission": submission_summary_json(&submission),
                                }));
                            }
                            Err(err) => {
                                failed += 1;
                                results.push(json!({
                                    "row": row_index,
                                    "user_id": user_id.get(),
                                    "status": "failure",
                                    "error": err.to_string(),
                                }));
                            }
                        }
                    }
                    Err((row_index, error)) => {
                        failed += 1;
                        results.push(json!({
                            "row": row_index,
                            "status": "failure",
                            "error": error,
                        }));
                    }
                }
            }
            let status = if failed == 0 {
                "imported"
            } else {
                "imported_with_errors"
            };
            emit_result(
                "submission import",
                json!({
                    "status": status,
                    "course_id": request.course_id.get(),
                    "assignment_id": request.assignment_id.get(),
                    "results": results,
                    "totals": {
                        "success": success,
                        "failed": failed,
                    },
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!(
                    "Imported grades: {} succeeded, {} failed.",
                    success, failed
                );
            }
        }
    }
    Ok(())
}

fn handle_quiz(
    request: QuizRequest,
    global: &GlobalOptions,
) -> Result<(), CliError> {
    match request {
        QuizRequest::List(request) => {
            let planned = vec![PlannedAction {
                action: "list quizzes",
                risk: "none",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("quiz list", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let quizzes =
                canvas_core::list_quizzes(&config, request.course_id)?;
            emit_result(
                "quiz list",
                json!({
                    "status": "ok",
                    "course_id": request.course_id.get(),
                    "quizzes": quizzes.iter().map(quiz_summary_json).collect::<Vec<_>>(),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("Found {} quizzes.", quizzes.len());
            }
        }
        QuizRequest::Create(request) => {
            let planned = vec![PlannedAction {
                action: "create quiz",
                risk: "creates remote quiz",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("quiz create", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let quiz =
                canvas_core::create_quiz(&config, request.course_id, &request.input)?;
            emit_result(
                "quiz create",
                json!({
                    "status": "created",
                    "course_id": request.course_id.get(),
                    "quiz": quiz_summary_json(&quiz),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("Quiz {} created.", quiz.id);
            }
        }
        QuizRequest::Update(request) => {
            let planned = vec![PlannedAction {
                action: "update quiz",
                risk: "updates remote quiz",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("quiz update", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let quiz = canvas_core::update_quiz(
                &config,
                request.course_id,
                request.quiz_id,
                &request.input,
            )?;
            emit_result(
                "quiz update",
                json!({
                    "status": "updated",
                    "course_id": request.course_id.get(),
                    "quiz": quiz_summary_json(&quiz),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("Quiz {} updated.", quiz.id);
            }
        }
        QuizRequest::Delete(request) => {
            let planned = vec![PlannedAction {
                action: "delete quiz",
                risk: "deletes remote quiz",
                requires_confirmation: true,
            }];
            if global.explain {
                emit_plan("quiz delete", &planned, global);
                return Ok(());
            }
            require_confirmation(global, "quiz delete")?;
            let config = canvas_core::load_merged_config()?;
            let quiz = canvas_core::delete_quiz(
                &config,
                request.course_id,
                request.quiz_id,
            )?;
            emit_result(
                "quiz delete",
                json!({
                    "status": "deleted",
                    "course_id": request.course_id.get(),
                    "quiz": quiz_summary_json(&quiz),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("Quiz {} deleted.", quiz.id);
            }
        }
        QuizRequest::Publish(request) => {
            let planned = vec![PlannedAction {
                action: "publish quiz",
                risk: "updates remote quiz publish state",
                requires_confirmation: true,
            }];
            if global.explain {
                emit_plan("quiz publish", &planned, global);
                return Ok(());
            }
            require_confirmation(global, "quiz publish")?;
            let config = canvas_core::load_merged_config()?;
            let input = canvas_core::QuizUpdateInput::new(
                None,
                None,
                Some(request.publish_state),
                None,
                None,
                None,
            )?;
            let quiz = canvas_core::update_quiz(
                &config,
                request.course_id,
                request.quiz_id,
                &input,
            )?;
            emit_result(
                "quiz publish",
                json!({
                    "status": request.publish_state.as_str(),
                    "course_id": request.course_id.get(),
                    "quiz": quiz_summary_json(&quiz),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("Quiz {} {}.", quiz.id, request.publish_state.as_str());
            }
        }
        QuizRequest::Submission(request) => handle_quiz_submission(request, global)?,
    }
    Ok(())
}

fn handle_quiz_submission(
    request: QuizSubmissionRequest,
    global: &GlobalOptions,
) -> Result<(), CliError> {
    match request {
        QuizSubmissionRequest::List(request) => {
            let planned = vec![PlannedAction {
                action: "list quiz submissions",
                risk: "none",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("quiz submission list", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let submissions = canvas_core::list_quiz_submissions(
                &config,
                request.course_id,
                request.quiz_id,
            )?;
            emit_result(
                "quiz submission list",
                json!({
                    "status": "ok",
                    "course_id": request.course_id.get(),
                    "quiz_id": request.quiz_id.get(),
                    "submissions": submissions
                        .iter()
                        .map(quiz_submission_summary_json)
                        .collect::<Vec<_>>(),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("Found {} quiz submissions.", submissions.len());
            }
        }
        QuizSubmissionRequest::Grade(request) => {
            let planned = vec![PlannedAction {
                action: "grade quiz submission",
                risk: "updates remote quiz submission grade",
                requires_confirmation: true,
            }];
            if global.explain {
                emit_plan("quiz submission grade", &planned, global);
                return Ok(());
            }
            require_confirmation(global, "quiz submission grade")?;
            let config = canvas_core::load_merged_config()?;
            let quiz =
                canvas_core::get_quiz(&config, request.course_id, request.quiz_id)?;
            let max_points =
                canvas_core::ensure_quiz_points(request.quiz_id, quiz.points_possible)?;
            let score = canvas_core::parse_score(&request.score, max_points)?;
            let submission = canvas_core::grade_quiz_submission(
                &config,
                request.course_id,
                request.quiz_id,
                request.submission_id,
                score,
            )?;
            emit_result(
                "quiz submission grade",
                json!({
                    "status": "graded",
                    "course_id": request.course_id.get(),
                    "quiz_id": request.quiz_id.get(),
                    "submission_id": request.submission_id.get(),
                    "submission": quiz_submission_summary_json(&submission),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!(
                    "Quiz submission {} graded.",
                    request.submission_id.get()
                );
            }
        }
    }
    Ok(())
}

fn handle_page(request: PageRequest, global: &GlobalOptions) -> Result<(), CliError> {
    match request {
        PageRequest::List(request) => {
            let planned = vec![PlannedAction {
                action: "list pages",
                risk: "none",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("page list", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let pages = canvas_core::list_pages(&config, request.course_id)?;
            let data = json!({
                "status": "ok",
                "course_id": request.course_id.get(),
                "pages": pages.iter().map(page_summary_json).collect::<Vec<_>>(),
            });
            emit_result("page list", data, global);
            if !global.quiet && !global.json {
                println!("Found {} pages.", pages.len());
            }
        }
        PageRequest::Create(request) => {
            let planned = vec![PlannedAction {
                action: "create page",
                risk: "creates remote page",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("page create", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let page =
                canvas_core::create_page(&config, request.course_id, &request.input)?;
            emit_result(
                "page create",
                json!({
                    "status": "created",
                    "course_id": request.course_id.get(),
                    "page": page_summary_json(&page),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("Page created.");
            }
        }
        PageRequest::Update(request) => {
            let planned = vec![PlannedAction {
                action: "update page",
                risk: "updates remote page",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("page update", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let page = canvas_core::update_page(
                &config,
                request.course_id,
                request.page_id,
                &request.input,
            )?;
            emit_result(
                "page update",
                json!({
                    "status": "updated",
                    "course_id": request.course_id.get(),
                    "page": page_summary_json(&page),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("Page updated.");
            }
        }
        PageRequest::Publish(request) => {
            let planned = vec![PlannedAction {
                action: "update page publish state",
                risk: "updates remote page",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("page publish", &planned, global);
                return Ok(());
            }
            let input = canvas_core::PageUpdateInput::new(
                None,
                None,
                Some(request.publish_state),
            )?;
            let config = canvas_core::load_merged_config()?;
            let page = canvas_core::update_page(
                &config,
                request.course_id,
                request.page_id,
                &input,
            )?;
            emit_result(
                "page publish",
                json!({
                    "status": "updated",
                    "course_id": request.course_id.get(),
                    "page": page_summary_json(&page),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("Page publish state updated.");
            }
        }
    }
    Ok(())
}

fn handle_module(request: ModuleRequest, global: &GlobalOptions) -> Result<(), CliError> {
    match request {
        ModuleRequest::List(request) => {
            let planned = vec![PlannedAction {
                action: "list modules",
                risk: "none",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("module list", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let modules = canvas_core::list_modules(&config, request.course_id)?;
            let data = json!({
                "status": "ok",
                "course_id": request.course_id.get(),
                "modules": modules.iter().map(module_summary_json).collect::<Vec<_>>(),
            });
            emit_result("module list", data, global);
            if !global.quiet && !global.json {
                println!("Found {} modules.", modules.len());
            }
        }
        ModuleRequest::Create(request) => {
            let planned = vec![PlannedAction {
                action: "create module",
                risk: "creates remote module",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("module create", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let module =
                canvas_core::create_module(&config, request.course_id, &request.input)?;
            emit_result(
                "module create",
                json!({
                    "status": "created",
                    "course_id": request.course_id.get(),
                    "module": module_summary_json(&module),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("Module created.");
            }
        }
        ModuleRequest::Update(request) => {
            let planned = vec![PlannedAction {
                action: "update module",
                risk: "updates remote module",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("module update", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let module = canvas_core::update_module(
                &config,
                request.course_id,
                request.module_id,
                &request.input,
            )?;
            emit_result(
                "module update",
                json!({
                    "status": "updated",
                    "course_id": request.course_id.get(),
                    "module": module_summary_json(&module),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("Module updated.");
            }
        }
        ModuleRequest::Reorder(request) => {
            let planned = vec![PlannedAction {
                action: "reorder modules",
                risk: "updates remote module order",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("module reorder", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let modules =
                canvas_core::reorder_modules(&config, request.course_id, &request.module_ids)?;
            emit_result(
                "module reorder",
                json!({
                    "status": "updated",
                    "course_id": request.course_id.get(),
                    "modules": modules.iter().map(module_summary_json).collect::<Vec<_>>(),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("Modules reordered.");
            }
        }
        ModuleRequest::Publish(request) => {
            let planned = vec![PlannedAction {
                action: "update module publish state",
                risk: "updates remote module",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("module publish", &planned, global);
                return Ok(());
            }
            let input = canvas_core::ModuleUpdateInput::new(
                None,
                Some(request.publish_state),
            )?;
            let config = canvas_core::load_merged_config()?;
            let module = canvas_core::update_module(
                &config,
                request.course_id,
                request.module_id,
                &input,
            )?;
            emit_result(
                "module publish",
                json!({
                    "status": "updated",
                    "course_id": request.course_id.get(),
                    "module": module_summary_json(&module),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("Module publish state updated.");
            }
        }
    }
    Ok(())
}

fn handle_file(request: FileRequest, global: &GlobalOptions) -> Result<(), CliError> {
    match request {
        FileRequest::List(request) => {
            let planned = vec![PlannedAction {
                action: "list files",
                risk: "none",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("file list", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let files = canvas_core::list_files(
                &config,
                request.course_id,
                request.folder_id,
            )?;
            let data = json!({
                "status": "ok",
                "course_id": request.course_id.get(),
                "files": files.iter().map(file_summary_json).collect::<Vec<_>>(),
            });
            emit_result("file list", data, global);
            if !global.quiet && !global.json {
                println!("Found {} files.", files.len());
            }
        }
        FileRequest::Upload(request) => {
            let planned = vec![PlannedAction {
                action: "upload file",
                risk: "uploads file to Canvas",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("file upload", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let file =
                canvas_core::upload_file(&config, request.course_id, &request.input)?;
            emit_result(
                "file upload",
                json!({
                    "status": "uploaded",
                    "course_id": request.course_id.get(),
                    "file": file_summary_json(&file),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("File uploaded.");
            }
        }
        FileRequest::Delete(request) => {
            let planned = vec![PlannedAction {
                action: "delete file",
                risk: "deletes remote file",
                requires_confirmation: true,
            }];
            if global.explain {
                emit_plan("file delete", &planned, global);
                return Ok(());
            }
            require_confirmation(global, "file delete")?;
            let config = canvas_core::load_merged_config()?;
            let file = canvas_core::delete_file(&config, request.file_id)?;
            emit_result(
                "file delete",
                json!({
                    "status": "deleted",
                    "file": file_summary_json(&file),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("File deleted.");
            }
        }
    }
    Ok(())
}

fn handle_folder(request: FolderRequest, global: &GlobalOptions) -> Result<(), CliError> {
    match request {
        FolderRequest::List(request) => {
            let planned = vec![PlannedAction {
                action: "list folders",
                risk: "none",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("folder list", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let folders = canvas_core::list_folders(&config, request.course_id)?;
            let data = json!({
                "status": "ok",
                "course_id": request.course_id.get(),
                "folders": folders.iter().map(folder_summary_json).collect::<Vec<_>>(),
            });
            emit_result("folder list", data, global);
            if !global.quiet && !global.json {
                println!("Found {} folders.", folders.len());
            }
        }
        FolderRequest::Create(request) => {
            let planned = vec![PlannedAction {
                action: "create folder",
                risk: "creates remote folder",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("folder create", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let folder = canvas_core::create_folder(
                &config,
                request.course_id,
                request.name,
                request.parent_folder_id,
            )?;
            emit_result(
                "folder create",
                json!({
                    "status": "created",
                    "course_id": request.course_id.get(),
                    "folder": folder_summary_json(&folder),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("Folder created.");
            }
        }
    }
    Ok(())
}

fn handle_announcement(
    request: AnnouncementRequest,
    global: &GlobalOptions,
) -> Result<(), CliError> {
    match request {
        AnnouncementRequest::List(request) => {
            let planned = vec![PlannedAction {
                action: "list announcements",
                risk: "none",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("announcement list", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let announcements =
                canvas_core::list_announcements(&config, request.course_id)?;
            let data = json!({
                "status": "ok",
                "course_id": request.course_id.get(),
                "announcements": announcements.iter().map(announcement_summary_json).collect::<Vec<_>>(),
            });
            emit_result("announcement list", data, global);
            if !global.quiet && !global.json {
                println!("Found {} announcements.", announcements.len());
            }
        }
        AnnouncementRequest::Create(request) => {
            let planned = vec![PlannedAction {
                action: "create announcement",
                risk: "creates remote announcement",
                requires_confirmation: true,
            }];
            if global.explain {
                emit_plan("announcement create", &planned, global);
                return Ok(());
            }
            require_confirmation(global, "announcement create")?;
            let config = canvas_core::load_merged_config()?;
            let announcement = canvas_core::create_announcement(
                &config,
                request.course_id,
                request.title,
                request.message,
            )?;
            emit_result(
                "announcement create",
                json!({
                    "status": "created",
                    "course_id": request.course_id.get(),
                    "announcement": announcement_summary_json(&announcement),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("Announcement created.");
            }
        }
    }
    Ok(())
}

fn handle_discussion(
    request: DiscussionRequest,
    global: &GlobalOptions,
) -> Result<(), CliError> {
    match request {
        DiscussionRequest::List(request) => {
            let planned = vec![PlannedAction {
                action: "list discussions",
                risk: "none",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("discussion list", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let discussions = canvas_core::list_discussions(&config, request.course_id)?;
            let data = json!({
                "status": "ok",
                "course_id": request.course_id.get(),
                "discussions": discussions.iter().map(discussion_summary_json).collect::<Vec<_>>(),
            });
            emit_result("discussion list", data, global);
            if !global.quiet && !global.json {
                println!("Found {} discussions.", discussions.len());
            }
        }
        DiscussionRequest::Create(request) => {
            let planned = vec![PlannedAction {
                action: "create discussion",
                risk: "creates remote discussion",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("discussion create", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let discussion = canvas_core::create_discussion(
                &config,
                request.course_id,
                request.title,
                request.message,
            )?;
            emit_result(
                "discussion create",
                json!({
                    "status": "created",
                    "course_id": request.course_id.get(),
                    "discussion": discussion_summary_json(&discussion),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("Discussion created.");
            }
        }
    }
    Ok(())
}

fn handle_user(request: UserRequest, global: &GlobalOptions) -> Result<(), CliError> {
    match request {
        UserRequest::List(request) => {
            let planned = vec![PlannedAction {
                action: "list users",
                risk: "none",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("user list", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let users =
                canvas_core::list_users(&config, request.course_id, request.role)?;
            let data = json!({
                "status": "ok",
                "course_id": request.course_id.get(),
                "users": users.iter().map(user_summary_json).collect::<Vec<_>>(),
            });
            emit_result("user list", data, global);
            if !global.quiet && !global.json {
                println!("Found {} users.", users.len());
            }
        }
    }
    Ok(())
}

fn handle_message(
    request: MessageRequest,
    global: &GlobalOptions,
) -> Result<(), CliError> {
    match request {
        MessageRequest::Send(request) => {
            let planned = vec![PlannedAction {
                action: "send message",
                risk: "sends message to recipients",
                requires_confirmation: true,
            }];
            if global.explain {
                emit_plan("message send", &planned, global);
                return Ok(());
            }
            require_confirmation(global, "message send")?;
            let config = canvas_core::load_merged_config()?;
            let mut recipient_ids = BTreeSet::new();
            for user_id in request.user_ids {
                recipient_ids.insert(user_id.get());
            }
            if let Some(role) = request.role {
                let course_id = request.course_id.ok_or(CliError::MissingCourseId)?;
                let users = canvas_core::list_users(&config, course_id, Some(role))?;
                for user in users {
                    recipient_ids.insert(user.id);
                }
            }
            let parsed_ids = recipient_ids
                .iter()
                .map(|id| canvas_core::parse_user_id(&id.to_string()))
                .collect::<Result<Vec<_>, _>>()?;
            let recipients = canvas_core::parse_recipient_ids(parsed_ids)?;
            let input = canvas_core::MessageSendInput::new(
                request.subject,
                request.body,
                recipients,
            );
            let result = canvas_core::send_message(&config, &input)?;
            emit_result(
                "message send",
                json!({
                    "status": "sent",
                    "recipient_count": recipient_ids.len(),
                    "recipient_ids": recipient_ids.iter().copied().collect::<Vec<_>>(),
                    "conversation_ids": result.conversation_ids,
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("Message sent to {} recipients.", recipient_ids.len());
            }
        }
    }
    Ok(())
}

fn handle_group(request: GroupRequest, global: &GlobalOptions) -> Result<(), CliError> {
    match request {
        GroupRequest::List(request) => {
            let planned = vec![PlannedAction {
                action: "list groups",
                risk: "none",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("group list", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let groups = canvas_core::list_groups(&config, request.course_id)?;
            let data = json!({
                "status": "ok",
                "course_id": request.course_id.get(),
                "groups": groups.iter().map(group_summary_json).collect::<Vec<_>>(),
            });
            emit_result("group list", data, global);
            if !global.quiet && !global.json {
                println!("Found {} groups.", groups.len());
            }
        }
        GroupRequest::Create(request) => {
            let planned = vec![PlannedAction {
                action: "create group",
                risk: "creates remote group",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("group create", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let group = canvas_core::create_group(
                &config,
                request.course_id,
                request.name,
            )?;
            emit_result(
                "group create",
                json!({
                    "status": "created",
                    "course_id": request.course_id.get(),
                    "group": group_summary_json(&group),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("Group created.");
            }
        }
    }
    Ok(())
}

fn handle_report(request: ReportRequest, global: &GlobalOptions) -> Result<(), CliError> {
    match request {
        ReportRequest::GradebookExport(request) => {
            handle_report_export("report gradebook-export", request, global)
        }
        ReportRequest::SubmissionStatus(request) => {
            handle_report_export("report submission-status", request, global)
        }
        ReportRequest::CourseActivitySummary(request) => {
            let planned = vec![PlannedAction {
                action: "fetch course activity summary",
                risk: "none",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("report course-activity", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let summary =
                canvas_core::get_course_activity_summary(&config, request.course_id)?;
            emit_result(
                "report course-activity",
                json!({
                    "status": "ok",
                    "course_id": request.course_id.get(),
                    "summary": course_activity_summary_json(&summary),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("Course activity summary retrieved.");
            }
            Ok(())
        }
    }
}

fn handle_report_export(
    command: &str,
    request: ReportExportRequest,
    global: &GlobalOptions,
) -> Result<(), CliError> {
    let planned = vec![PlannedAction {
        action: "request and download report",
        risk: "reads report data",
        requires_confirmation: false,
    }];
    if global.explain {
        emit_plan(command, &planned, global);
        return Ok(());
    }
    if global.json && request.format == ReportFormat::Csv {
        return Err(CliError::InvalidReportFormat(
            "json_output_with_csv".to_string(),
        ));
    }
    let config = canvas_core::load_merged_config()?;
    let report =
        canvas_core::request_report(&config, request.course_id, request.report_type)?;
    let report = canvas_core::wait_for_report(
        &config,
        request.course_id,
        request.report_type,
        report.id,
        canvas_core::ReportWaitOptions::default(),
    )?;
    match request.format {
        ReportFormat::Json => {
            emit_result(
                command,
                json!({
                    "status": "ok",
                    "course_id": request.course_id.get(),
                    "report": report_summary_json(&report),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("Report {} is ready.", report.id);
            }
            Ok(())
        }
        ReportFormat::Csv => {
            let file_url = report.file_url.ok_or_else(|| {
                CliError::Canvas(canvas_core::CanvasError::ReportNotReady(
                    "missing_file_url".to_string(),
                ))
            })?;
            if let Some(path) = request.output {
                let mut file = File::create(&path)
                    .map_err(|err| CliError::FileWrite(path.display().to_string(), err.to_string()))?;
                canvas_core::download_report_to_writer(&config, &file_url, &mut file)?;
            } else {
                let stdout = io::stdout();
                let mut handle = stdout.lock();
                canvas_core::download_report_to_writer(&config, &file_url, &mut handle)?;
            }
            Ok(())
        }
    }
}

fn handle_ask(prompt: &str, global: &GlobalOptions) -> Result<(), CliError> {
    let plan = plan_from_prompt(prompt);
    if global.explain {
        emit_ask_plan(&plan, global);
        return Ok(());
    }
    emit_ask_plan(&plan, global);
    if !global.confirm {
        return Ok(());
    }
    if plan.commands.is_empty() {
        return Ok(());
    }
    execute_ask_plan(&plan, global)?;
    Ok(())
}

fn execute_ask_plan(plan: &AskPlan, global: &GlobalOptions) -> Result<(), CliError> {
    for command in &plan.commands {
        match command.command {
            "auth check" => {
                handle_auth(AuthCommand::Check, global)?;
            }
            "course list" => {
                handle_course(CourseRequest::List(CourseListRequest), global)?;
            }
            _ => return Err(CliError::AskExecutionUnsupported),
        }
    }
    Ok(())
}

fn plan_from_prompt(prompt: &str) -> AskPlan {
    let normalized = prompt.to_lowercase();
    if normalized.contains("auth") && normalized.contains("check") {
        AskPlan {
            prompt: prompt.to_string(),
            rationale: "Detected intent to validate credentials.".to_string(),
            commands: vec![PlannedCommand {
                id: "auth-check",
                command: "auth check",
                params: Vec::new(),
                risk: "none",
                destructive: false,
            }],
        }
    } else if normalized.contains("list") && normalized.contains("course") {
        AskPlan {
            prompt: prompt.to_string(),
            rationale: "Detected intent to list courses.".to_string(),
            commands: vec![PlannedCommand {
                id: "course-list",
                command: "course list",
                params: Vec::new(),
                risk: "none",
                destructive: false,
            }],
        }
    } else {
        AskPlan {
            prompt: prompt.to_string(),
            rationale: "No matching command yet.".to_string(),
            commands: Vec::new(),
        }
    }
}

fn require_confirmation(global: &GlobalOptions, action: &'static str) -> Result<(), CliError> {
    if global.confirm {
        Ok(())
    } else {
        Err(CliError::ConfirmationRequired(action))
    }
}

fn parse_rubric_input(
    rubric_json: Option<String>,
    rubric_file: Option<PathBuf>,
) -> Result<Option<canvas_models::RubricAssessment>, CliError> {
    if rubric_json.is_some() && rubric_file.is_some() {
        return Err(CliError::Canvas(
            canvas_core::CanvasError::InvalidRubricAssessment(
                "multiple_sources".to_string(),
            ),
        ));
    }
    let raw = if let Some(raw) = rubric_json {
        raw
    } else if let Some(path) = rubric_file {
        read_file_to_string(&path)?
    } else {
        return Ok(None);
    };
    let parsed = canvas_core::parse_rubric_assessment(&raw)?;
    Ok(Some(parsed))
}

fn emit_result(command: &str, data: Value, global: &GlobalOptions) {
    if global.json {
        let output = json!({
            "ok": true,
            "schema_version": SCHEMA_VERSION,
            "command": command,
            "data": data,
        });
        println!("{output}");
        return;
    }
    if global.quiet {
        return;
    }
    if let Some(message) = data.get("detail").and_then(Value::as_str) {
        println!("{message}");
    }
}

fn emit_plan(command: &str, actions: &[PlannedAction], global: &GlobalOptions) {
    if global.json {
        let output = json!({
            "ok": true,
            "schema_version": SCHEMA_VERSION,
            "command": command,
            "planned_actions": actions
                .iter()
                .map(|action| {
                    json!({
                        "action": action.action,
                        "risk": action.risk,
                        "requires_confirmation": action.requires_confirmation,
                    })
                })
                .collect::<Vec<_>>(),
        });
        println!("{output}");
        return;
    }
    if global.quiet {
        return;
    }
    println!("Planned actions for {command}:");
    for action in actions {
        println!(
            "- {} (risk: {}, confirm: {})",
            action.action, action.risk, action.requires_confirmation
        );
    }
}

fn emit_ask_plan(plan: &AskPlan, global: &GlobalOptions) {
    if global.json {
        let output = json!({
            "ok": true,
            "schema_version": SCHEMA_VERSION,
            "command": "ask",
            "data": {
                "prompt": plan.prompt,
                "rationale": plan.rationale,
                "commands": plan.commands.iter().map(|command| {
                    json!({
                        "id": command.id,
                        "command": command.command,
                        "params": command.params,
                        "risk": command.risk,
                        "destructive": command.destructive,
                    })
                }).collect::<Vec<_>>(),
            },
        });
        println!("{output}");
        return;
    }
    if global.quiet {
        return;
    }
    println!("Ask plan:");
    println!("Prompt: {}", plan.prompt);
    if !plan.rationale.is_empty() {
        println!("Rationale: {}", plan.rationale);
    }
    if plan.commands.is_empty() {
        println!("No matching commands.");
        return;
    }
    for command in &plan.commands {
        println!("- {} (risk: {})", command.command, command.risk);
    }
}

fn course_summary_json(course: &canvas_core::CourseSummary) -> Value {
    json!({
        "id": course.id,
        "name": course.name,
        "code": course.code,
        "start_at": course.start_at,
        "end_at": course.end_at,
        "workflow_state": course.workflow_state,
    })
}

fn assignment_summary_json(assignment: &canvas_core::AssignmentSummary) -> Value {
    json!({
        "id": assignment.id,
        "name": assignment.name,
        "points_possible": assignment.points_possible,
        "due_at": assignment.due_at,
        "published": assignment.published,
        "workflow_state": assignment.workflow_state,
    })
}

fn submission_summary_json(submission: &canvas_core::SubmissionSummary) -> Value {
    json!({
        "id": submission.id,
        "user_id": submission.user_id,
        "score": submission.score,
        "submitted_at": submission.submitted_at,
        "graded_at": submission.graded_at,
        "workflow_state": submission.workflow_state,
    })
}

fn quiz_summary_json(quiz: &canvas_core::QuizSummary) -> Value {
    json!({
        "id": quiz.id,
        "title": quiz.title,
        "points_possible": quiz.points_possible,
        "due_at": quiz.due_at,
        "published": quiz.published,
        "workflow_state": quiz.workflow_state,
        "time_limit": quiz.time_limit,
        "access_code": quiz.access_code,
        "unlock_at": quiz.unlock_at,
        "lock_at": quiz.lock_at,
    })
}

fn quiz_submission_summary_json(
    submission: &canvas_core::QuizSubmissionSummary,
) -> Value {
    json!({
        "id": submission.id,
        "user_id": submission.user_id,
        "score": submission.score,
        "submitted_at": submission.submitted_at,
        "workflow_state": submission.workflow_state,
    })
}

fn page_summary_json(page: &canvas_core::PageSummary) -> Value {
    json!({
        "page_id": page.page_id,
        "url": page.url,
        "title": page.title,
        "body": page.body,
        "published": page.published,
        "workflow_state": page.workflow_state,
        "html_url": page.html_url,
    })
}

fn module_summary_json(module: &canvas_core::ModuleSummary) -> Value {
    json!({
        "id": module.id,
        "name": module.name,
        "published": module.published,
        "position": module.position,
        "workflow_state": module.workflow_state,
        "items_count": module.items_count,
        "html_url": module.html_url,
    })
}

fn file_summary_json(file: &canvas_core::FileSummary) -> Value {
    json!({
        "id": file.id,
        "display_name": file.display_name,
        "filename": file.filename,
        "size": file.size,
        "content_type": file.content_type,
        "url": file.url,
        "thumbnail_url": file.thumbnail_url,
        "updated_at": file.updated_at,
        "created_at": file.created_at,
        "folder_id": file.folder_id,
    })
}

fn folder_summary_json(folder: &canvas_core::FolderSummary) -> Value {
    json!({
        "id": folder.id,
        "name": folder.name,
        "full_name": folder.full_name,
        "context_id": folder.context_id,
        "parent_folder_id": folder.parent_folder_id,
        "created_at": folder.created_at,
        "updated_at": folder.updated_at,
        "files_count": folder.files_count,
    })
}

fn discussion_summary_json(discussion: &canvas_core::DiscussionSummary) -> Value {
    json!({
        "id": discussion.id,
        "title": discussion.title,
        "message": discussion.message,
        "posted_at": discussion.posted_at,
        "discussion_type": discussion.discussion_type,
        "html_url": discussion.html_url,
    })
}

fn announcement_summary_json(
    announcement: &canvas_core::AnnouncementSummary,
) -> Value {
    json!({
        "id": announcement.id,
        "title": announcement.title,
        "message": announcement.message,
        "posted_at": announcement.posted_at,
        "html_url": announcement.html_url,
    })
}

fn user_summary_json(user: &canvas_core::UserSummary) -> Value {
    json!({
        "id": user.id,
        "name": user.name,
        "sortable_name": user.sortable_name,
        "short_name": user.short_name,
        "login_id": user.login_id,
        "email": user.email,
    })
}

fn group_summary_json(group: &canvas_core::GroupSummary) -> Value {
    json!({
        "id": group.id,
        "name": group.name,
        "description": group.description,
        "members_count": group.members_count,
        "group_category_id": group.group_category_id,
    })
}

fn report_summary_json(report: &canvas_core::ReportSummary) -> Value {
    json!({
        "id": report.id,
        "report_type": report.report_type.as_str(),
        "status": report.status.as_str(),
        "progress": report.progress,
        "file_url": report.file_url,
        "created_at": report.created_at,
        "started_at": report.started_at,
        "ended_at": report.ended_at,
        "updated_at": report.updated_at,
    })
}

fn course_activity_summary_json(summary: &canvas_core::CourseActivitySummary) -> Value {
    json!({
        "page_views": summary.page_views,
        "participations": summary.participations,
        "start_at": summary.start_at,
        "end_at": summary.end_at,
        "extra": summary.extra,
    })
}

#[derive(Debug)]
struct ImportRow {
    row: usize,
    user_id: String,
    score: String,
}

fn read_csv_import_rows(path: &PathBuf) -> Result<Vec<ImportRow>, CliError> {
    let mut reader = ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_path(path)
        .map_err(|err| CliError::FileRead(path.display().to_string(), err.to_string()))?;
    let headers = reader
        .headers()
        .map_err(|err| CliError::FileRead(path.display().to_string(), err.to_string()))?
        .clone();
    let user_index = headers
        .iter()
        .position(|header| header.eq_ignore_ascii_case("user_id"))
        .ok_or_else(|| {
            CliError::FileRead(
                path.display().to_string(),
                "missing user_id header".to_string(),
            )
        })?;
    let score_index = headers
        .iter()
        .position(|header| header.eq_ignore_ascii_case("score"))
        .ok_or_else(|| {
            CliError::FileRead(
                path.display().to_string(),
                "missing score header".to_string(),
            )
        })?;
    let mut rows = Vec::new();
    for (idx, record) in reader.records().enumerate() {
        let record = record
            .map_err(|err| CliError::FileRead(path.display().to_string(), err.to_string()))?;
        rows.push(ImportRow {
            row: idx + 1,
            user_id: record.get(user_index).unwrap_or_default().to_string(),
            score: record.get(score_index).unwrap_or_default().to_string(),
        });
    }
    Ok(rows)
}

fn read_json_import_rows(path: &PathBuf) -> Result<Vec<ImportRow>, CliError> {
    let raw = read_file_to_string(path)?;
    let parsed: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|err| CliError::FileRead(path.display().to_string(), err.to_string()))?;
    let array = parsed.as_array().ok_or_else(|| {
        CliError::FileRead(
            path.display().to_string(),
            "expected JSON array".to_string(),
        )
    })?;
    let mut rows = Vec::new();
    for (idx, item) in array.iter().enumerate() {
        let (user_id, score) = match item.as_object() {
            Some(object) => (
                json_value_to_string(object.get("user_id")).unwrap_or_default(),
                json_value_to_string(object.get("score")).unwrap_or_default(),
            ),
            None => (String::new(), String::new()),
        };
        rows.push(ImportRow {
            row: idx + 1,
            user_id,
            score,
        });
    }
    Ok(rows)
}

fn json_value_to_string(value: Option<&serde_json::Value>) -> Option<String> {
    match value? {
        serde_json::Value::String(value) => Some(value.clone()),
        serde_json::Value::Number(value) => Some(value.to_string()),
        _ => None,
    }
}

fn parse_import_row(
    row: ImportRow,
    max_points: f64,
) -> Result<(usize, UserId, canvas_models::Score), (usize, String)> {
    let row_index = row.row;
    let user_raw = row.user_id.trim();
    let score_raw = row.score.trim();
    if user_raw.is_empty() && score_raw.is_empty() {
        return Err((
            row_index,
            "missing user_id and score".to_string(),
        ));
    }
    if user_raw.is_empty() {
        return Err((row_index, "missing user_id".to_string()));
    }
    if score_raw.is_empty() {
        return Err((row_index, "missing score".to_string()));
    }
    let user_id = canvas_core::parse_user_id(user_raw)
        .map_err(|err| (row_index, err.to_string()))?;
    let score = canvas_core::parse_score(score_raw, max_points)
        .map_err(|err| (row_index, err.to_string()))?;
    Ok((row_index, user_id, score))
}

fn read_file_to_string(path: &PathBuf) -> Result<String, CliError> {
    std::fs::read_to_string(path)
        .map_err(|err| CliError::FileRead(path.display().to_string(), err.to_string()))
}

fn schema_definition() -> Value {
    json!({
        "version": SCHEMA_VERSION,
        "schema_version": SCHEMA_VERSION,
        "schemas": {
            "command_result": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "planned_actions": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "planned_actions": { "type": "array" },
                },
                "required": ["ok", "schema_version", "command", "planned_actions"],
            },
            "ask_plan": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": {
                        "type": "object",
                        "properties": {
                            "prompt": { "type": "string" },
                            "rationale": { "type": "string" },
                            "commands": { "type": "array" },
                        },
                        "required": ["prompt", "rationale", "commands"],
                    },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "auth_check": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "config_init": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "course_list": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "course_show": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "course_set": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "course_update": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "assignment_list": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "assignment_mutation": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "submission_list": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "submission_grade": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "submission_import": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "quiz_list": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "quiz_mutation": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "quiz_submission_list": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "quiz_submission_grade": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "page_list": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "page_mutation": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "module_list": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "module_mutation": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "file_list": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "file_upload": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "file_delete": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "folder_list": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "folder_create": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "announcement_list": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "announcement_create": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "discussion_list": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "discussion_create": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "user_list": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "message_send": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "group_list": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "group_create": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "report_export": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "course_activity_summary": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "error": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "error": {
                        "type": "object",
                        "properties": {
                            "code": { "type": "string" },
                            "message": { "type": "string" },
                            "details": { "type": "object" },
                        },
                        "required": ["code", "message", "details"],
                    },
                },
                "required": ["ok", "schema_version", "error"],
            },
        },
    })
}

fn emit_schema() {
    let output = schema_definition();
    println!("{output}");
}

fn emit_error(error: &CliError, json_output: bool) {
    if json_output {
        let output = json!({
            "ok": false,
            "schema_version": SCHEMA_VERSION,
            "error": {
                "code": error_code(error),
                "message": error.to_string(),
                "details": error_details(error),
            },
        });
        println!("{output}");
        return;
    }
    eprintln!("{error}");
}

fn error_code(error: &CliError) -> &'static str {
    match error {
        CliError::MissingCommand => "missing_command",
        CliError::MissingCourseId => "missing_course_id",
        CliError::ConfirmationRequired(_) => "confirmation_required",
        CliError::AskExecutionUnsupported => "ask_execution_unsupported",
        CliError::FileRead(_, _) => "file_read",
        CliError::FileWrite(_, _) => "file_write",
        CliError::InvalidReportFormat(_) => "invalid_report_format",
        CliError::Canvas(canvas_error) => match canvas_error {
            canvas_core::CanvasError::InvalidCourseId(_) => "invalid_course_id",
            canvas_core::CanvasError::InvalidAssignmentId(_) => "invalid_assignment_id",
            canvas_core::CanvasError::InvalidAssignmentName(_) => {
                "invalid_assignment_name"
            }
            canvas_core::CanvasError::InvalidQuizId(_) => "invalid_quiz_id",
            canvas_core::CanvasError::InvalidQuizSubmissionId(_) => {
                "invalid_quiz_submission_id"
            }
            canvas_core::CanvasError::InvalidQuizTitle(_) => "invalid_quiz_title",
            canvas_core::CanvasError::InvalidUserId(_) => "invalid_user_id",
            canvas_core::CanvasError::InvalidSubmissionId(_) => "invalid_submission_id",
            canvas_core::CanvasError::InvalidPageId(_) => "invalid_page_id",
            canvas_core::CanvasError::InvalidPageTitle(_) => "invalid_page_title",
            canvas_core::CanvasError::InvalidPageBody(_) => "invalid_page_body",
            canvas_core::CanvasError::InvalidModuleId(_) => "invalid_module_id",
            canvas_core::CanvasError::InvalidModuleName(_) => "invalid_module_name",
            canvas_core::CanvasError::InvalidFileId(_) => "invalid_file_id",
            canvas_core::CanvasError::InvalidFolderId(_) => "invalid_folder_id",
            canvas_core::CanvasError::InvalidFolderName(_) => "invalid_folder_name",
            canvas_core::CanvasError::InvalidDiscussionId(_) => "invalid_discussion_id",
            canvas_core::CanvasError::InvalidGroupId(_) => "invalid_group_id",
            canvas_core::CanvasError::InvalidGroupName(_) => "invalid_group_name",
            canvas_core::CanvasError::InvalidUserRole(_) => "invalid_user_role",
            canvas_core::CanvasError::InvalidMessageSubject(_) => {
                "invalid_message_subject"
            }
            canvas_core::CanvasError::InvalidMessageBody(_) => "invalid_message_body",
            canvas_core::CanvasError::InvalidRecipients(_) => "invalid_recipients",
            canvas_core::CanvasError::InvalidScore(_) => "invalid_score",
            canvas_core::CanvasError::InvalidPointsPossible(_) => "invalid_points_possible",
            canvas_core::CanvasError::InvalidDueDate(_) => "invalid_due_date",
            canvas_core::CanvasError::InvalidQuizTimeLimit(_) => {
                "invalid_quiz_time_limit"
            }
            canvas_core::CanvasError::InvalidQuizAccessCode(_) => {
                "invalid_quiz_access_code"
            }
            canvas_core::CanvasError::InvalidPublishState(_) => "invalid_publish_state",
            canvas_core::CanvasError::InvalidRubricSelection(_) => "invalid_rubric_selection",
            canvas_core::CanvasError::InvalidRubricAssessment(_) => {
                "invalid_rubric_assessment"
            }
            canvas_core::CanvasError::InvalidCourseVisibility(_) => "invalid_course_visibility",
            canvas_core::CanvasError::InvalidGradingSchemeId(_) => "invalid_grading_scheme_id",
            canvas_core::CanvasError::InvalidReportType(_) => "invalid_report_type",
            canvas_core::CanvasError::InvalidCourseDates(_) => "invalid_course_dates",
            canvas_core::CanvasError::InvalidCourseUpdate(_) => "invalid_course_update",
            canvas_core::CanvasError::InvalidAssignmentUpdate(_) => {
                "invalid_assignment_update"
            }
            canvas_core::CanvasError::InvalidQuizUpdate(_) => "invalid_quiz_update",
            canvas_core::CanvasError::InvalidPageUpdate(_) => "invalid_page_update",
            canvas_core::CanvasError::InvalidModuleUpdate(_) => "invalid_module_update",
            canvas_core::CanvasError::InvalidModuleReorder(_) => "invalid_module_reorder",
            canvas_core::CanvasError::MissingAssignmentPoints(_) => {
                "missing_assignment_points"
            }
            canvas_core::CanvasError::MissingQuizPoints(_) => "missing_quiz_points",
            canvas_core::CanvasError::InvalidQuizAvailability(_) => {
                "invalid_quiz_availability"
            }
            canvas_core::CanvasError::InvalidFileUpload(_) => "invalid_file_upload",
            canvas_core::CanvasError::ReportNotReady(_) => "report_not_ready",
            canvas_core::CanvasError::ReportDownloadFailed(_) => "report_download_failed",
            canvas_core::CanvasError::ReportTimeout(_) => "report_timeout",
            canvas_core::CanvasError::InvalidHost(_) => "invalid_host",
            canvas_core::CanvasError::InvalidToken => "invalid_token",
            canvas_core::CanvasError::MissingConfig(_) => "missing_config",
            canvas_core::CanvasError::ConfigRead(_, _) => "config_read",
            canvas_core::CanvasError::ConfigParse(_, _) => "config_parse",
            canvas_core::CanvasError::ConfigWrite(_, _) => "config_write",
            canvas_core::CanvasError::AuthCheckFailed(_) => "auth_check_failed",
            canvas_core::CanvasError::Http(_) => "http_error",
            canvas_core::CanvasError::Api(api_error) => api_error.code().as_str(),
        },
    }
}

fn error_details(error: &CliError) -> Value {
    match error {
        CliError::ConfirmationRequired(action) => {
            json!({ "action": action })
        }
        CliError::MissingCourseId => json!({ "field": "course" }),
        CliError::FileRead(path, detail) => json!({ "path": path, "detail": detail }),
        CliError::FileWrite(path, detail) => json!({ "path": path, "detail": detail }),
        CliError::InvalidReportFormat(detail) => json!({ "detail": detail }),
        CliError::Canvas(canvas_error) => match canvas_error {
            canvas_core::CanvasError::InvalidCourseId(raw) => json!({ "input": raw }),
            canvas_core::CanvasError::InvalidAssignmentId(raw) => json!({ "input": raw }),
            canvas_core::CanvasError::InvalidAssignmentName(raw) => {
                json!({ "input": raw })
            }
            canvas_core::CanvasError::InvalidQuizId(raw) => json!({ "input": raw }),
            canvas_core::CanvasError::InvalidQuizSubmissionId(raw) => {
                json!({ "input": raw })
            }
            canvas_core::CanvasError::InvalidQuizTitle(raw) => json!({ "input": raw }),
            canvas_core::CanvasError::InvalidUserId(raw) => json!({ "input": raw }),
            canvas_core::CanvasError::InvalidSubmissionId(raw) => json!({ "input": raw }),
            canvas_core::CanvasError::InvalidPageId(raw) => json!({ "input": raw }),
            canvas_core::CanvasError::InvalidPageTitle(raw) => json!({ "input": raw }),
            canvas_core::CanvasError::InvalidPageBody(raw) => json!({ "input": raw }),
            canvas_core::CanvasError::InvalidModuleId(raw) => json!({ "input": raw }),
            canvas_core::CanvasError::InvalidModuleName(raw) => json!({ "input": raw }),
            canvas_core::CanvasError::InvalidFileId(raw) => json!({ "input": raw }),
            canvas_core::CanvasError::InvalidFolderId(raw) => json!({ "input": raw }),
            canvas_core::CanvasError::InvalidFolderName(raw) => json!({ "input": raw }),
            canvas_core::CanvasError::InvalidDiscussionId(raw) => json!({ "input": raw }),
            canvas_core::CanvasError::InvalidGroupId(raw) => json!({ "input": raw }),
            canvas_core::CanvasError::InvalidGroupName(raw) => json!({ "input": raw }),
            canvas_core::CanvasError::InvalidUserRole(raw) => json!({ "input": raw }),
            canvas_core::CanvasError::InvalidMessageSubject(raw) => {
                json!({ "input": raw })
            }
            canvas_core::CanvasError::InvalidMessageBody(raw) => {
                json!({ "input": raw })
            }
            canvas_core::CanvasError::InvalidRecipients(raw) => json!({ "input": raw }),
            canvas_core::CanvasError::InvalidScore(raw) => json!({ "input": raw }),
            canvas_core::CanvasError::InvalidPointsPossible(raw) => {
                json!({ "input": raw })
            }
            canvas_core::CanvasError::InvalidDueDate(raw) => json!({ "input": raw }),
            canvas_core::CanvasError::InvalidQuizTimeLimit(raw) => {
                json!({ "input": raw })
            }
            canvas_core::CanvasError::InvalidQuizAccessCode(raw) => {
                json!({ "input": raw })
            }
            canvas_core::CanvasError::InvalidPublishState(raw) => json!({ "input": raw }),
            canvas_core::CanvasError::InvalidRubricSelection(raw) => json!({ "input": raw }),
            canvas_core::CanvasError::InvalidRubricAssessment(raw) => {
                json!({ "input": raw })
            }
            canvas_core::CanvasError::InvalidCourseVisibility(raw) => json!({ "input": raw }),
            canvas_core::CanvasError::InvalidGradingSchemeId(raw) => json!({ "input": raw }),
            canvas_core::CanvasError::InvalidReportType(raw) => json!({ "input": raw }),
            canvas_core::CanvasError::InvalidCourseDates(detail) => json!({ "detail": detail }),
            canvas_core::CanvasError::InvalidCourseUpdate(detail) => json!({ "detail": detail }),
            canvas_core::CanvasError::InvalidAssignmentUpdate(detail) => {
                json!({ "detail": detail })
            }
            canvas_core::CanvasError::InvalidQuizUpdate(detail) => {
                json!({ "detail": detail })
            }
            canvas_core::CanvasError::InvalidPageUpdate(detail) => {
                json!({ "detail": detail })
            }
            canvas_core::CanvasError::InvalidModuleUpdate(detail) => {
                json!({ "detail": detail })
            }
            canvas_core::CanvasError::InvalidModuleReorder(detail) => {
                json!({ "detail": detail })
            }
            canvas_core::CanvasError::MissingAssignmentPoints(assignment_id) => {
                json!({ "assignment_id": assignment_id })
            }
            canvas_core::CanvasError::MissingQuizPoints(quiz_id) => {
                json!({ "quiz_id": quiz_id })
            }
            canvas_core::CanvasError::InvalidQuizAvailability(detail) => {
                json!({ "detail": detail })
            }
            canvas_core::CanvasError::InvalidFileUpload(detail) => {
                json!({ "detail": detail })
            }
            canvas_core::CanvasError::ReportNotReady(detail) => {
                json!({ "detail": detail })
            }
            canvas_core::CanvasError::ReportDownloadFailed(detail) => {
                json!({ "detail": detail })
            }
            canvas_core::CanvasError::ReportTimeout(detail) => {
                json!({ "detail": detail })
            }
            canvas_core::CanvasError::InvalidHost(raw) => json!({ "input": raw }),
            canvas_core::CanvasError::MissingConfig(path) => json!({ "path": path }),
            canvas_core::CanvasError::ConfigRead(path, detail) => {
                json!({ "path": path, "detail": detail })
            }
            canvas_core::CanvasError::ConfigParse(path, detail) => {
                json!({ "path": path, "detail": detail })
            }
            canvas_core::CanvasError::ConfigWrite(path, detail) => {
                json!({ "path": path, "detail": detail })
            }
            canvas_core::CanvasError::AuthCheckFailed(detail) => json!({ "detail": detail }),
            canvas_core::CanvasError::Http(detail) => json!({ "detail": detail }),
            canvas_core::CanvasError::Api(api_error) => json!({
                "status": api_error.status(),
                "request_id": api_error.request_id(),
                "body": api_error.body(),
            }),
            canvas_core::CanvasError::InvalidToken => json!({}),
        },
        _ => json!({}),
    }
}

impl TryFrom<&GlobalArgs> for GlobalOptions {
    type Error = CliError;

    fn try_from(args: &GlobalArgs) -> Result<Self, Self::Error> {
        let course = match &args.course {
            Some(raw) => Some(canvas_core::parse_course_id(raw)?),
            None => None,
        };
        Ok(Self {
            course,
            json: args.json,
            quiet: args.quiet,
            confirm: args.confirm,
            explain: args.explain,
        })
    }
}

impl TryFrom<(CourseCommand, &GlobalOptions)> for CourseRequest {
    type Error = CliError;

    fn try_from(input: (CourseCommand, &GlobalOptions)) -> Result<Self, Self::Error> {
        let (command, global) = input;
        match command {
            CourseCommand::List => Ok(CourseRequest::List(CourseListRequest)),
            CourseCommand::Show { course } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                Ok(CourseRequest::Show(CourseShowRequest { course_id }))
            }
            CourseCommand::Set { course } => {
                let course_id = canvas_core::parse_course_id(&course)?;
                Ok(CourseRequest::Set(CourseSetRequest { course_id }))
            }
            CourseCommand::Update {
                course,
                start,
                end,
                visibility,
                grading_scheme_id,
            } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;

                let start = match start {
                    Some(raw) => Some(canvas_core::parse_due_date(&raw)?),
                    None => None,
                };
                let end = match end {
                    Some(raw) => Some(canvas_core::parse_due_date(&raw)?),
                    None => None,
                };
                let dates = match (start, end) {
                    (None, None) => None,
                    (start, end) => Some(canvas_core::parse_course_dates(start, end)?),
                };
                let visibility = match visibility {
                    Some(raw) => Some(canvas_core::parse_course_visibility(&raw)?),
                    None => None,
                };
                let grading_scheme_id = match grading_scheme_id {
                    Some(raw) => Some(canvas_core::parse_grading_scheme_id(&raw)?),
                    None => None,
                };
                let settings =
                    canvas_core::CourseSettingsUpdate::new(dates, visibility, grading_scheme_id)?;
                Ok(CourseRequest::Update(CourseUpdateRequest {
                    course_id,
                    settings,
                }))
            }
        }
    }
}

impl TryFrom<(AssignmentCommand, &GlobalOptions)> for AssignmentRequest {
    type Error = CliError;

    fn try_from(
        input: (AssignmentCommand, &GlobalOptions),
    ) -> Result<Self, Self::Error> {
        let (command, global) = input;
        match command {
            AssignmentCommand::List { course } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                Ok(AssignmentRequest::List(AssignmentListRequest {
                    course_id,
                }))
            }
            AssignmentCommand::Create {
                course,
                name,
                points,
                due_at,
                publish_state,
            } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                let name = canvas_core::parse_assignment_name(&name)?;
                let points = match points {
                    Some(raw) => Some(canvas_core::parse_points_possible(&raw)?),
                    None => None,
                };
                let due_at = match due_at {
                    Some(raw) => Some(canvas_core::parse_due_date(&raw)?),
                    None => None,
                };
                let publish_state = match publish_state {
                    Some(raw) => Some(canvas_core::parse_publish_state(&raw)?),
                    None => None,
                };
                let input = canvas_core::AssignmentCreateInput::new(
                    name,
                    points,
                    due_at,
                    publish_state,
                );
                Ok(AssignmentRequest::Create(AssignmentCreateRequest {
                    course_id,
                    input,
                }))
            }
            AssignmentCommand::Update {
                course,
                assignment,
                name,
                points,
                due_at,
                publish_state,
            } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                let assignment_id = canvas_core::parse_assignment_id(&assignment)?;
                let name = match name {
                    Some(raw) => Some(canvas_core::parse_assignment_name(&raw)?),
                    None => None,
                };
                let points = match points {
                    Some(raw) => Some(canvas_core::parse_points_possible(&raw)?),
                    None => None,
                };
                let due_at = match due_at {
                    Some(raw) => Some(canvas_core::parse_due_date(&raw)?),
                    None => None,
                };
                let publish_state = match publish_state {
                    Some(raw) => Some(canvas_core::parse_publish_state(&raw)?),
                    None => None,
                };
                let input = canvas_core::AssignmentUpdateInput::new(
                    name,
                    points,
                    due_at,
                    publish_state,
                )?;
                Ok(AssignmentRequest::Update(AssignmentUpdateRequest {
                    course_id,
                    assignment_id,
                    input,
                }))
            }
            AssignmentCommand::Delete { course, assignment } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                let assignment_id = canvas_core::parse_assignment_id(&assignment)?;
                Ok(AssignmentRequest::Delete(AssignmentDeleteRequest {
                    course_id,
                    assignment_id,
                }))
            }
        }
    }
}

impl TryFrom<(SubmissionCommand, &GlobalOptions)> for SubmissionRequest {
    type Error = CliError;

    fn try_from(
        input: (SubmissionCommand, &GlobalOptions),
    ) -> Result<Self, Self::Error> {
        let (command, global) = input;
        match command {
            SubmissionCommand::List { course, assignment } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                let assignment_id = canvas_core::parse_assignment_id(&assignment)?;
                Ok(SubmissionRequest::List(SubmissionListRequest {
                    course_id,
                    assignment_id,
                }))
            }
            SubmissionCommand::Grade {
                course,
                assignment,
                user,
                score,
                rubric_json,
                rubric_file,
            } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                let assignment_id = canvas_core::parse_assignment_id(&assignment)?;
                let user_id = canvas_core::parse_user_id(&user)?;
                let rubric_assessment =
                    parse_rubric_input(rubric_json, rubric_file)?;
                Ok(SubmissionRequest::Grade(SubmissionGradeRequest {
                    course_id,
                    assignment_id,
                    user_id,
                    score,
                    rubric_assessment,
                }))
            }
            SubmissionCommand::Import {
                course,
                assignment,
                format,
                file,
            } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                let assignment_id = canvas_core::parse_assignment_id(&assignment)?;
                Ok(SubmissionRequest::Import(SubmissionImportRequest {
                    course_id,
                    assignment_id,
                    format,
                    file,
                }))
            }
        }
    }
}

impl TryFrom<(QuizCommand, &GlobalOptions)> for QuizRequest {
    type Error = CliError;

    fn try_from(input: (QuizCommand, &GlobalOptions)) -> Result<Self, Self::Error> {
        let (command, global) = input;
        match command {
            QuizCommand::List { course } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                Ok(QuizRequest::List(QuizListRequest { course_id }))
            }
            QuizCommand::Create {
                course,
                title,
                points,
                time_limit,
                access_code,
                unlock_at,
                due_at,
                lock_at,
                publish_state,
            } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                let title = canvas_core::parse_quiz_title(&title)?;
                let points = match points {
                    Some(raw) => Some(canvas_core::parse_points_possible(&raw)?),
                    None => None,
                };
                let time_limit = match time_limit {
                    Some(raw) => Some(canvas_core::parse_quiz_time_limit(&raw)?),
                    None => None,
                };
                let access_code = match access_code {
                    Some(raw) => Some(canvas_core::parse_quiz_access_code(&raw)?),
                    None => None,
                };
                let unlock_at = match unlock_at {
                    Some(raw) => Some(canvas_core::parse_due_date(&raw)?),
                    None => None,
                };
                let due_at = match due_at {
                    Some(raw) => Some(canvas_core::parse_due_date(&raw)?),
                    None => None,
                };
                let lock_at = match lock_at {
                    Some(raw) => Some(canvas_core::parse_due_date(&raw)?),
                    None => None,
                };
                let availability = canvas_core::parse_quiz_availability(
                    unlock_at,
                    due_at,
                    lock_at,
                )?;
                let publish_state = match publish_state {
                    Some(raw) => Some(canvas_core::parse_publish_state(&raw)?),
                    None => None,
                };
                let input = canvas_core::QuizCreateInput::new(
                    title,
                    points,
                    publish_state,
                    time_limit,
                    access_code,
                    availability,
                );
                Ok(QuizRequest::Create(QuizCreateRequest { course_id, input }))
            }
            QuizCommand::Update {
                course,
                quiz,
                title,
                points,
                time_limit,
                access_code,
                unlock_at,
                due_at,
                lock_at,
                publish_state,
            } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                let quiz_id = canvas_core::parse_quiz_id(&quiz)?;
                let title = match title {
                    Some(raw) => Some(canvas_core::parse_quiz_title(&raw)?),
                    None => None,
                };
                let points = match points {
                    Some(raw) => Some(canvas_core::parse_points_possible(&raw)?),
                    None => None,
                };
                let time_limit = match time_limit {
                    Some(raw) => Some(canvas_core::parse_quiz_time_limit(&raw)?),
                    None => None,
                };
                let access_code = match access_code {
                    Some(raw) => Some(canvas_core::parse_quiz_access_code(&raw)?),
                    None => None,
                };
                let unlock_at = match unlock_at {
                    Some(raw) => Some(canvas_core::parse_due_date(&raw)?),
                    None => None,
                };
                let due_at = match due_at {
                    Some(raw) => Some(canvas_core::parse_due_date(&raw)?),
                    None => None,
                };
                let lock_at = match lock_at {
                    Some(raw) => Some(canvas_core::parse_due_date(&raw)?),
                    None => None,
                };
                let availability = canvas_core::parse_quiz_availability(
                    unlock_at,
                    due_at,
                    lock_at,
                )?;
                let publish_state = match publish_state {
                    Some(raw) => Some(canvas_core::parse_publish_state(&raw)?),
                    None => None,
                };
                let input = canvas_core::QuizUpdateInput::new(
                    title,
                    points,
                    publish_state,
                    time_limit,
                    access_code,
                    availability,
                )?;
                Ok(QuizRequest::Update(QuizUpdateRequest {
                    course_id,
                    quiz_id,
                    input,
                }))
            }
            QuizCommand::Delete { course, quiz } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                let quiz_id = canvas_core::parse_quiz_id(&quiz)?;
                Ok(QuizRequest::Delete(QuizDeleteRequest {
                    course_id,
                    quiz_id,
                }))
            }
            QuizCommand::Publish {
                course,
                quiz,
                publish_state,
            } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                let quiz_id = canvas_core::parse_quiz_id(&quiz)?;
                let publish_state = canvas_core::parse_publish_state(&publish_state)?;
                Ok(QuizRequest::Publish(QuizPublishRequest {
                    course_id,
                    quiz_id,
                    publish_state,
                }))
            }
            QuizCommand::Submission { command } => {
                let request = QuizSubmissionRequest::try_from((command, global))?;
                Ok(QuizRequest::Submission(request))
            }
        }
    }
}

impl TryFrom<(QuizSubmissionCommand, &GlobalOptions)> for QuizSubmissionRequest {
    type Error = CliError;

    fn try_from(
        input: (QuizSubmissionCommand, &GlobalOptions),
    ) -> Result<Self, Self::Error> {
        let (command, global) = input;
        match command {
            QuizSubmissionCommand::List { course, quiz } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                let quiz_id = canvas_core::parse_quiz_id(&quiz)?;
                Ok(QuizSubmissionRequest::List(QuizSubmissionListRequest {
                    course_id,
                    quiz_id,
                }))
            }
            QuizSubmissionCommand::Grade {
                course,
                quiz,
                submission,
                score,
            } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                let quiz_id = canvas_core::parse_quiz_id(&quiz)?;
                let submission_id =
                    canvas_core::parse_quiz_submission_id(&submission)?;
                Ok(QuizSubmissionRequest::Grade(QuizSubmissionGradeRequest {
                    course_id,
                    quiz_id,
                    submission_id,
                    score,
                }))
            }
        }
    }
}

impl TryFrom<(PageCommand, &GlobalOptions)> for PageRequest {
    type Error = CliError;

    fn try_from(
        input: (PageCommand, &GlobalOptions),
    ) -> Result<Self, Self::Error> {
        let (command, global) = input;
        match command {
            PageCommand::List { course } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                Ok(PageRequest::List(PageListRequest { course_id }))
            }
            PageCommand::Create {
                course,
                title,
                body,
                publish_state,
            } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                let title = canvas_core::parse_page_title(&title)?;
                let body = canvas_core::parse_page_body(&body)?;
                let publish_state = match publish_state {
                    Some(raw) => Some(canvas_core::parse_publish_state(&raw)?),
                    None => None,
                };
                let input =
                    canvas_core::PageCreateInput::new(title, body, publish_state);
                Ok(PageRequest::Create(PageCreateRequest { course_id, input }))
            }
            PageCommand::Update {
                course,
                page,
                title,
                body,
                publish_state,
            } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                let page_id = canvas_core::parse_page_id(&page)?;
                let title = match title {
                    Some(raw) => Some(canvas_core::parse_page_title(&raw)?),
                    None => None,
                };
                let body = match body {
                    Some(raw) => Some(canvas_core::parse_page_body(&raw)?),
                    None => None,
                };
                let publish_state = match publish_state {
                    Some(raw) => Some(canvas_core::parse_publish_state(&raw)?),
                    None => None,
                };
                let input =
                    canvas_core::PageUpdateInput::new(title, body, publish_state)?;
                Ok(PageRequest::Update(PageUpdateRequest {
                    course_id,
                    page_id,
                    input,
                }))
            }
            PageCommand::Publish {
                course,
                page,
                publish_state,
            } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                let page_id = canvas_core::parse_page_id(&page)?;
                let publish_state = canvas_core::parse_publish_state(&publish_state)?;
                Ok(PageRequest::Publish(PagePublishRequest {
                    course_id,
                    page_id,
                    publish_state,
                }))
            }
        }
    }
}

impl TryFrom<(ModuleCommand, &GlobalOptions)> for ModuleRequest {
    type Error = CliError;

    fn try_from(
        input: (ModuleCommand, &GlobalOptions),
    ) -> Result<Self, Self::Error> {
        let (command, global) = input;
        match command {
            ModuleCommand::List { course } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                Ok(ModuleRequest::List(ModuleListRequest { course_id }))
            }
            ModuleCommand::Create {
                course,
                name,
                publish_state,
            } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                let name = canvas_core::parse_module_name(&name)?;
                let publish_state = match publish_state {
                    Some(raw) => Some(canvas_core::parse_publish_state(&raw)?),
                    None => None,
                };
                let input =
                    canvas_core::ModuleCreateInput::new(name, publish_state);
                Ok(ModuleRequest::Create(ModuleCreateRequest { course_id, input }))
            }
            ModuleCommand::Update {
                course,
                module,
                name,
                publish_state,
            } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                let module_id = canvas_core::parse_module_id(&module)?;
                let name = match name {
                    Some(raw) => Some(canvas_core::parse_module_name(&raw)?),
                    None => None,
                };
                let publish_state = match publish_state {
                    Some(raw) => Some(canvas_core::parse_publish_state(&raw)?),
                    None => None,
                };
                let input =
                    canvas_core::ModuleUpdateInput::new(name, publish_state)?;
                Ok(ModuleRequest::Update(ModuleUpdateRequest {
                    course_id,
                    module_id,
                    input,
                }))
            }
            ModuleCommand::Reorder { course, module_ids } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                let parsed_ids = module_ids
                    .iter()
                    .map(|raw| canvas_core::parse_module_id(raw))
                    .collect::<Result<Vec<_>, _>>()?;
                if parsed_ids.is_empty() {
                    return Err(CliError::Canvas(canvas_core::CanvasError::InvalidModuleReorder(
                        "empty_order".to_string(),
                    )));
                }
                Ok(ModuleRequest::Reorder(ModuleReorderRequest {
                    course_id,
                    module_ids: parsed_ids,
                }))
            }
            ModuleCommand::Publish {
                course,
                module,
                publish_state,
            } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                let module_id = canvas_core::parse_module_id(&module)?;
                let publish_state = canvas_core::parse_publish_state(&publish_state)?;
                Ok(ModuleRequest::Publish(ModulePublishRequest {
                    course_id,
                    module_id,
                    publish_state,
                }))
            }
        }
    }
}

impl TryFrom<(FileCommand, &GlobalOptions)> for FileRequest {
    type Error = CliError;

    fn try_from(
        input: (FileCommand, &GlobalOptions),
    ) -> Result<Self, Self::Error> {
        let (command, global) = input;
        match command {
            FileCommand::List { course, folder } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                let folder_id = match folder {
                    Some(raw) => Some(canvas_core::parse_folder_id(&raw)?),
                    None => None,
                };
                Ok(FileRequest::List(FileListRequest {
                    course_id,
                    folder_id,
                }))
            }
            FileCommand::Upload {
                course,
                file,
                parent_folder,
            } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                let parent_folder_id = match parent_folder {
                    Some(raw) => Some(canvas_core::parse_folder_id(&raw)?),
                    None => None,
                };
                let input = canvas_core::UploadFileInput::from_path(
                    file,
                    parent_folder_id,
                )?;
                Ok(FileRequest::Upload(FileUploadRequest { course_id, input }))
            }
            FileCommand::Delete { file } => {
                let file_id = canvas_core::parse_file_id(&file)?;
                Ok(FileRequest::Delete(FileDeleteRequest { file_id }))
            }
        }
    }
}

impl TryFrom<(FolderCommand, &GlobalOptions)> for FolderRequest {
    type Error = CliError;

    fn try_from(
        input: (FolderCommand, &GlobalOptions),
    ) -> Result<Self, Self::Error> {
        let (command, global) = input;
        match command {
            FolderCommand::List { course } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                Ok(FolderRequest::List(FolderListRequest { course_id }))
            }
            FolderCommand::Create {
                course,
                name,
                parent_folder,
            } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                let name = canvas_core::parse_folder_name(&name)?;
                let parent_folder_id = match parent_folder {
                    Some(raw) => Some(canvas_core::parse_folder_id(&raw)?),
                    None => None,
                };
                Ok(FolderRequest::Create(FolderCreateRequest {
                    course_id,
                    name,
                    parent_folder_id,
                }))
            }
        }
    }
}

impl TryFrom<(AnnouncementCommand, &GlobalOptions)> for AnnouncementRequest {
    type Error = CliError;

    fn try_from(
        input: (AnnouncementCommand, &GlobalOptions),
    ) -> Result<Self, Self::Error> {
        let (command, global) = input;
        match command {
            AnnouncementCommand::List { course } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                Ok(AnnouncementRequest::List(AnnouncementListRequest {
                    course_id,
                }))
            }
            AnnouncementCommand::Create {
                course,
                title,
                message,
            } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                Ok(AnnouncementRequest::Create(AnnouncementCreateRequest {
                    course_id,
                    title,
                    message,
                }))
            }
        }
    }
}

impl TryFrom<(DiscussionCommand, &GlobalOptions)> for DiscussionRequest {
    type Error = CliError;

    fn try_from(
        input: (DiscussionCommand, &GlobalOptions),
    ) -> Result<Self, Self::Error> {
        let (command, global) = input;
        match command {
            DiscussionCommand::List { course } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                Ok(DiscussionRequest::List(DiscussionListRequest { course_id }))
            }
            DiscussionCommand::Create {
                course,
                title,
                message,
            } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                Ok(DiscussionRequest::Create(DiscussionCreateRequest {
                    course_id,
                    title,
                    message,
                }))
            }
        }
    }
}

impl TryFrom<(UserCommand, &GlobalOptions)> for UserRequest {
    type Error = CliError;

    fn try_from(
        input: (UserCommand, &GlobalOptions),
    ) -> Result<Self, Self::Error> {
        let (command, global) = input;
        match command {
            UserCommand::List { course, role } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                let role = match role {
                    Some(raw) => Some(canvas_core::parse_user_role(&raw)?),
                    None => None,
                };
                Ok(UserRequest::List(UserListRequest { course_id, role }))
            }
        }
    }
}

impl TryFrom<(MessageCommand, &GlobalOptions)> for MessageRequest {
    type Error = CliError;

    fn try_from(
        input: (MessageCommand, &GlobalOptions),
    ) -> Result<Self, Self::Error> {
        let (command, global) = input;
        match command {
            MessageCommand::Send {
                course,
                subject,
                body,
                user_ids,
                role,
            } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                };
                let subject = canvas_core::parse_message_subject(&subject)?;
                let body = canvas_core::parse_message_body(&body)?;
                let parsed_user_ids = user_ids
                    .iter()
                    .map(|raw| canvas_core::parse_user_id(raw))
                    .collect::<Result<Vec<_>, _>>()?;
                let role = match role {
                    Some(raw) => Some(canvas_core::parse_user_role(&raw)?),
                    None => None,
                };
                if parsed_user_ids.is_empty() && role.is_none() {
                    return Err(CliError::Canvas(
                        canvas_core::CanvasError::InvalidRecipients(
                            "empty".to_string(),
                        ),
                    ));
                }
                if role.is_some() && course_id.is_none() {
                    return Err(CliError::MissingCourseId);
                }
                Ok(MessageRequest::Send(MessageSendRequest {
                    course_id,
                    subject,
                    body,
                    user_ids: parsed_user_ids,
                    role,
                }))
            }
        }
    }
}

impl TryFrom<(GroupCommand, &GlobalOptions)> for GroupRequest {
    type Error = CliError;

    fn try_from(
        input: (GroupCommand, &GlobalOptions),
    ) -> Result<Self, Self::Error> {
        let (command, global) = input;
        match command {
            GroupCommand::List { course } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                Ok(GroupRequest::List(GroupListRequest { course_id }))
            }
            GroupCommand::Create { course, name } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                let name = canvas_core::parse_group_name(&name)?;
                Ok(GroupRequest::Create(GroupCreateRequest { course_id, name }))
            }
        }
    }
}

impl TryFrom<(ReportCommand, &GlobalOptions)> for ReportRequest {
    type Error = CliError;

    fn try_from(
        input: (ReportCommand, &GlobalOptions),
    ) -> Result<Self, Self::Error> {
        let (command, global) = input;
        match command {
            ReportCommand::GradebookExport {
                course,
                format,
                output,
            } => {
                if format == ReportFormat::Json && output.is_some() {
                    return Err(CliError::InvalidReportFormat(
                        "output_requires_csv".to_string(),
                    ));
                }
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                Ok(ReportRequest::GradebookExport(ReportExportRequest {
                    course_id,
                    report_type: ReportType::GradebookExport,
                    format,
                    output,
                }))
            }
            ReportCommand::SubmissionStatus {
                course,
                format,
                output,
            } => {
                if format == ReportFormat::Json && output.is_some() {
                    return Err(CliError::InvalidReportFormat(
                        "output_requires_csv".to_string(),
                    ));
                }
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                Ok(ReportRequest::SubmissionStatus(ReportExportRequest {
                    course_id,
                    report_type: ReportType::MissingSubmissions,
                    format,
                    output,
                }))
            }
            ReportCommand::CourseActivity { course } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                Ok(ReportRequest::CourseActivitySummary(
                    CourseActivitySummaryRequest { course_id },
                ))
            }
        }
    }
}

fn run_init(config_path: &std::path::Path) -> Result<(), canvas_core::CanvasError> {
    let host_input = prompt("Canvas host (e.g., https://school.instructure.com): ")?;
    let host = canvas_core::parse_host(&host_input)?;
    let token_input = prompt("Canvas token: ")?;
    let token = canvas_core::parse_token(&token_input)?;
    let course_input = prompt("Default course id (optional, press enter to skip): ")?;
    let default_course_id = if course_input.trim().is_empty() {
        None
    } else {
        Some(canvas_core::parse_course_id(course_input.trim())?)
    };

    let config = canvas_core::CanvasConfig {
        auth: canvas_core::AuthConfig { host, token },
        defaults: canvas_core::DefaultsConfig {
            course_id: default_course_id,
        },
    };

    create_config_dir(config_path)?;
    canvas_core::write_config(config_path, &config)?;
    Ok(())
}

fn prompt(message: &str) -> Result<String, canvas_core::CanvasError> {
    print!("{message}");
    io::stdout()
        .flush()
        .map_err(|err| canvas_core::CanvasError::ConfigWrite("stdout".to_string(), err.to_string()))?;
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|err| canvas_core::CanvasError::ConfigRead("stdin".to_string(), err.to_string()))?;
    Ok(input.trim().to_string())
}

fn create_config_dir(path: &std::path::Path) -> Result<(), canvas_core::CanvasError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|err| {
            canvas_core::CanvasError::ConfigWrite(parent.display().to_string(), err.to_string())
        })?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        parse_import_row, plan_from_prompt, schema_definition, Cli, ImportRow,
        PlannedCommand, SCHEMA_VERSION,
    };
    use clap::CommandFactory;
    use serde_json::Value;
    use std::path::PathBuf;

    #[test]
    fn ask_plan_detects_auth_check() {
        let plan = plan_from_prompt("Please check auth status");
        assert_eq!(plan.commands.len(), 1);
        assert_eq!(plan.commands[0].command, "auth check");
    }

    #[test]
    fn ask_plan_detects_course_list() {
        let plan = plan_from_prompt("List courses for me");
        assert_eq!(plan.commands.len(), 1);
        assert_eq!(plan.commands[0].command, "course list");
    }

    #[test]
    fn ask_plan_returns_empty_for_unknown() {
        let plan = plan_from_prompt("Generate a grade report");
        assert!(plan.commands.is_empty());
    }

    #[test]
    fn planned_command_params_are_stable() {
        let command = PlannedCommand {
            id: "course-list",
            command: "course list",
            params: Vec::new(),
            risk: "none",
            destructive: false,
        };
        assert!(command.params.is_empty());
    }

    #[test]
    fn import_row_rejects_missing_user_id() {
        let row = ImportRow {
            row: 1,
            user_id: "".to_string(),
            score: "10".to_string(),
        };
        let parsed = parse_import_row(row, 20.0);
        assert!(parsed.is_err());
    }

    #[test]
    fn import_row_parses_score() {
        let row = ImportRow {
            row: 1,
            user_id: "5".to_string(),
            score: "9".to_string(),
        };
        let parsed =
            parse_import_row(row, 10.0).unwrap_or_else(|err| panic!("{err:?}"));
        assert_eq!(parsed.1.get(), 5);
        assert_eq!(parsed.2.value(), 9.0);
    }

    #[test]
    fn schema_includes_core_groups() {
        let schema = schema_definition();
        let schemas = schema
            .get("schemas")
            .and_then(Value::as_object)
            .unwrap_or_else(|| panic!("schemas map"));
        for key in [
            "auth_check",
            "config_init",
            "course_list",
            "course_show",
            "course_set",
            "course_update",
        ] {
            assert!(schemas.contains_key(key), "missing schema {key}");
        }
        assert_eq!(
            schema.get("schema_version").and_then(Value::as_str),
            Some(SCHEMA_VERSION)
        );
    }

    #[test]
    fn schema_snapshot_matches() {
        let schema = schema_definition();
        let actual = serde_json::to_string(&schema)
            .unwrap_or_else(|err| panic!("schema json: {err}"));
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("snapshots")
            .join("schema.json");
        let expected =
            std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("{path:?}"));
        assert_eq!(actual.trim(), expected.trim());
    }

    #[test]
    fn cli_help_snapshot_matches() {
        let help = Cli::command().render_long_help().to_string();
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("snapshots")
            .join("cli-help.txt");
        let expected =
            std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("{path:?}"));
        assert_eq!(normalize_newlines(&help).trim(), normalize_newlines(&expected).trim());
    }

    fn normalize_newlines(value: &str) -> String {
        value.replace("\r\n", "\n")
    }
}
