#![recursion_limit = "512"]

use std::collections::BTreeSet;
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;

use canvas_models::{
    AssignmentId, CalendarEventContext, CalendarEventId, CollaborationId,
    ConferenceId, ContentMigrationId, CourseId, EnrollmentId, FileId, FolderId,
    GradingPeriodId, GradingPostingPolicy, ModuleId, OutcomeGroupId, OutcomeId,
    PageId, QuestionBankId, QuestionId, QuizId, QuizSubmissionId, ReportType,
    RubricAssociationId, RubricId, SectionId, UserId,
};
use clap::{Args, Parser, Subcommand};
use csv::ReaderBuilder;
use serde_json::{json, Value};
use thiserror::Error;
use tracing::info;

const SCHEMA_VERSION: &str = "v10";

#[derive(Debug, Parser)]
#[command(
    name = "canvas",
    bin_name = "canvas",
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
    /// Outcome-related operations
    #[command(after_help = "Examples:\n  canvas outcome list --course 42\n  canvas outcome create --course 42 --group 9 --title \"Outcome\" --description \"Readable\" --mastery-points 3\n  canvas outcome update --outcome 7 --title \"Outcome\" --mastery-points 4\n  canvas outcome delete --course 42 --group 9 --outcome 7 --confirm")]
    Outcome {
        #[command(subcommand)]
        command: OutcomeCommand,
    },
    /// Rubric-related operations
    #[command(after_help = "Examples:\n  canvas rubric list --course 42\n  canvas rubric create --course 42 --title \"Essay Rubric\" --criteria-json \"[{\\\"description\\\":\\\"Clarity\\\",\\\"points\\\":5}]\"\n  canvas rubric update --course 42 --rubric 7 --title \"Updated\"\n  canvas rubric delete --course 42 --rubric 7 --confirm\n  canvas rubric attach --course 42 --rubric 7 --assignment 10 --grading rubric --confirm\n  canvas rubric detach --course 42 --association 5 --confirm")]
    Rubric {
        #[command(subcommand)]
        command: RubricCommand,
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
    /// Question bank operations
    #[command(after_help = "Examples:\n  canvas question-bank list --course 42\n  canvas question-bank create --course 42 --title \"Chapter 1\"\n  canvas question-bank update --course 42 --bank 9 --title \"Updated Bank\"\n  canvas question-bank delete --course 42 --bank 9 --confirm")]
    QuestionBank {
        #[command(subcommand)]
        command: QuestionBankCommand,
    },
    /// Question operations
    #[command(after_help = "Examples:\n  canvas question list --bank 9\n  canvas question create --bank 9 --text \"What is 2+2?\" --type multiple_choice\n  canvas question create --bank 9 --question-file question.json\n  canvas question update --bank 9 --question 4 --text \"Updated\" --type essay\n  canvas question delete --bank 9 --question 4 --confirm")]
    Question {
        #[command(subcommand)]
        command: QuestionCommand,
    },
    /// Page-related operations
    #[command(after_help = "Examples:\n  canvas page list --course 42\n  canvas page create --course 42 --title \"Week 1\" --body \"Welcome\" --publish-state published\n  canvas page update --course 42 --page \"week-1\" --body \"Updated\" --publish-state unpublished\n  canvas page publish --course 42 --page \"week-1\" --publish-state published")]
    Page {
        #[command(subcommand)]
        command: PageCommand,
    },
    /// Module-related operations
    #[command(after_help = "Examples:\n  canvas module list --course 42\n  canvas module create --course 42 --name \"Week 1\" --publish-state published\n  canvas module update --course 42 --module 5 --name \"Week 1\" --publish-state unpublished\n  canvas module reorder --course 42 --module 5 --module 9\n  canvas module publish --course 42 --module 5 --publish-state published\n  canvas module requirement list --course 42 --module 5\n  canvas module requirement update --course 42 --module 5 --requirements-json \"[{\\\"item_id\\\":1,\\\"type\\\":\\\"view\\\"}]\"")]
    Module {
        #[command(subcommand)]
        command: ModuleCommand,
    },
    /// External tool operations
    #[command(after_help = "Examples:\n  canvas tool list --course 42\n  canvas tool create --course 42 --name \"Homework\" --config-url https://example.com/config.xml\n  canvas tool update --course 42 --tool 7 --name \"Updated Tool\" --config-file tool.json\n  canvas tool delete --course 42 --tool 7 --confirm")]
    Tool {
        #[command(subcommand)]
        command: ToolCommand,
    },
    /// Calendar event operations
    #[command(after_help = "Examples:\n  canvas calendar list --course 42\n  canvas calendar list --section 10\n  canvas calendar create --course 42 --title \"Office Hours\" --start-at 2025-01-10T15:00:00-05:00 --end-at 2025-01-10T16:00:00-05:00\n  canvas calendar create --section 10 --title \"Holiday\" --all-day-date 2025-01-15\n  canvas calendar update --event 7 --title \"New Time\" --start-at 2025-01-10T16:00:00-05:00\n  canvas calendar delete --event 7 --confirm")]
    Calendar {
        #[command(subcommand)]
        command: CalendarCommand,
    },
    /// Conference operations
    #[command(after_help = "Examples:\n  canvas conference list --course 42\n  canvas conference create --course 42 --title \"Weekly Sync\" --start-at 2025-02-01T10:00:00Z --duration 45 --recording enabled\n  canvas conference update --course 42 --conference 9 --title \"Updated\" --duration 60\n  canvas conference delete --course 42 --conference 9 --confirm")]
    Conference {
        #[command(subcommand)]
        command: ConferenceCommand,
    },
    /// Collaboration operations
    #[command(after_help = "Examples:\n  canvas collaboration list --course 42\n  canvas collaboration create --course 42 --title \"Project Doc\" --collaboration-type google_docs --user 99 --group 12\n  canvas collaboration delete --collaboration 7 --confirm")]
    Collaboration {
        #[command(subcommand)]
        command: CollaborationCommand,
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
    /// Content migration operations
    #[command(after_help = "Examples:\n  canvas content-migration list --course 42\n  canvas content-migration create --course 42 --type course_copy --source-course 99\n  canvas content-migration create --course 42 --type file_import --file course.zip\n  canvas content-migration show --course 42 --migration 5 --wait")]
    ContentMigration {
        #[command(subcommand)]
        command: ContentMigrationCommand,
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
    /// Section operations
    #[command(after_help = "Examples:\n  canvas section list --course 42\n  canvas section create --course 42 --name \"Section A\"\n  canvas section update --section 10 --name \"Section A1\"\n  canvas section delete --section 10 --confirm")]
    Section {
        #[command(subcommand)]
        command: SectionCommand,
    },
    /// Enrollment operations
    #[command(after_help = "Examples:\n  canvas enrollment list --section 10\n  canvas enrollment add --section 10 --user 99 --role student\n  canvas enrollment remove --section 10 --enrollment 5 --confirm")]
    Enrollment {
        #[command(subcommand)]
        command: EnrollmentCommand,
    },
    /// Analytics and report operations
    #[command(after_help = "Examples:\n  canvas report gradebook-export --course 42 --format csv\n  canvas report submission-status --course 42 --format json\n  canvas report course-activity --course 42\n  canvas report grade-change-log --course 42 --format csv")]
    Report {
        #[command(subcommand)]
        command: ReportCommand,
    },
    /// Gradebook policies and grading periods
    #[command(after_help = "Examples:\n  canvas gradebook grading-period list --course 42\n  canvas gradebook grading-period show --course 42 --period 9\n  canvas gradebook posting-policy get --course 42\n  canvas gradebook posting-policy set --course 42 --policy manual --confirm")]
    Gradebook {
        #[command(subcommand)]
        command: GradebookCommand,
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
        /// Peer review mode (automatic, manual)
        #[arg(long = "peer-review-mode")]
        peer_review_mode: Option<String>,
        /// Peer review assign date (RFC3339)
        #[arg(long = "peer-review-assign-at")]
        peer_review_assign_at: Option<String>,
        /// Peer review due date (RFC3339)
        #[arg(long = "peer-review-due-at")]
        peer_review_due_at: Option<String>,
        /// Group assignment mode (group, individual)
        #[arg(long = "group-assignment-mode")]
        group_assignment_mode: Option<String>,
        /// Group category id (required for group assignments)
        #[arg(long = "group-category-id")]
        group_category_id: Option<String>,
        /// Assignment overrides JSON (array)
        #[arg(long = "overrides-json")]
        overrides_json: Option<String>,
        /// Assignment overrides JSON file
        #[arg(long = "overrides-file")]
        overrides_file: Option<PathBuf>,
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
        /// Peer review mode (automatic, manual)
        #[arg(long = "peer-review-mode")]
        peer_review_mode: Option<String>,
        /// Peer review assign date (RFC3339)
        #[arg(long = "peer-review-assign-at")]
        peer_review_assign_at: Option<String>,
        /// Peer review due date (RFC3339)
        #[arg(long = "peer-review-due-at")]
        peer_review_due_at: Option<String>,
        /// Group assignment mode (group, individual)
        #[arg(long = "group-assignment-mode")]
        group_assignment_mode: Option<String>,
        /// Group category id (required for group assignments)
        #[arg(long = "group-category-id")]
        group_category_id: Option<String>,
        /// Assignment overrides JSON (array)
        #[arg(long = "overrides-json")]
        overrides_json: Option<String>,
        /// Assignment overrides JSON file
        #[arg(long = "overrides-file")]
        overrides_file: Option<PathBuf>,
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
enum OutcomeCommand {
    /// List outcomes for a course
    List {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
    },
    /// Create an outcome in an outcome group
    Create {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
        /// Outcome group id
        #[arg(long)]
        group: String,
        /// Outcome title
        #[arg(long)]
        title: String,
        /// Outcome description
        #[arg(long)]
        description: Option<String>,
        /// Points possible
        #[arg(long = "points-possible")]
        points_possible: Option<String>,
        /// Mastery points
        #[arg(long = "mastery-points")]
        mastery_points: Option<String>,
    },
    /// Update an outcome
    Update {
        /// Outcome id
        #[arg(long)]
        outcome: String,
        /// Outcome title
        #[arg(long)]
        title: Option<String>,
        /// Outcome description
        #[arg(long)]
        description: Option<String>,
        /// Points possible
        #[arg(long = "points-possible")]
        points_possible: Option<String>,
        /// Mastery points
        #[arg(long = "mastery-points")]
        mastery_points: Option<String>,
    },
    /// Delete an outcome
    Delete {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
        /// Outcome group id
        #[arg(long)]
        group: String,
        /// Outcome id
        #[arg(long)]
        outcome: String,
    },
}

#[derive(Debug, Subcommand)]
enum RubricCommand {
    /// List rubrics for a course
    List {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
    },
    /// Create a rubric
    Create {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
        /// Rubric title
        #[arg(long)]
        title: String,
        /// Rubric criteria JSON array
        #[arg(long = "criteria-json")]
        criteria_json: Option<String>,
        /// Rubric criteria JSON file path
        #[arg(long = "criteria-file")]
        criteria_file: Option<PathBuf>,
    },
    /// Update a rubric
    Update {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
        /// Rubric id
        #[arg(long)]
        rubric: String,
        /// Rubric title
        #[arg(long)]
        title: Option<String>,
        /// Rubric criteria JSON array
        #[arg(long = "criteria-json")]
        criteria_json: Option<String>,
        /// Rubric criteria JSON file path
        #[arg(long = "criteria-file")]
        criteria_file: Option<PathBuf>,
    },
    /// Delete a rubric
    Delete {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
        /// Rubric id
        #[arg(long)]
        rubric: String,
    },
    /// Attach a rubric to an assignment or outcome
    Attach {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
        /// Rubric id
        #[arg(long)]
        rubric: String,
        /// Assignment id
        #[arg(long)]
        assignment: Option<String>,
        /// Outcome id
        #[arg(long)]
        outcome: Option<String>,
        /// Rubric grading selection (rubric or none)
        #[arg(long)]
        grading: String,
        /// Association title override
        #[arg(long)]
        title: Option<String>,
    },
    /// Detach a rubric association
    Detach {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
        /// Rubric association id
        #[arg(long)]
        association: String,
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
enum QuestionBankCommand {
    /// List question banks for a course
    List {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
    },
    /// Create a question bank
    Create {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
        /// Question bank title
        #[arg(long)]
        title: Option<String>,
        /// Question bank JSON payload
        #[arg(long = "bank-json")]
        bank_json: Option<String>,
        /// Question bank JSON file path
        #[arg(long = "bank-file")]
        bank_file: Option<PathBuf>,
    },
    /// Update a question bank
    Update {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
        /// Question bank id
        #[arg(long = "bank")]
        bank: String,
        /// Question bank title
        #[arg(long)]
        title: Option<String>,
        /// Question bank JSON payload
        #[arg(long = "bank-json")]
        bank_json: Option<String>,
        /// Question bank JSON file path
        #[arg(long = "bank-file")]
        bank_file: Option<PathBuf>,
    },
    /// Delete a question bank
    Delete {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
        /// Question bank id
        #[arg(long = "bank")]
        bank: String,
    },
}

#[derive(Debug, Subcommand)]
enum QuestionCommand {
    /// List questions in a bank
    List {
        /// Question bank id
        #[arg(long = "bank")]
        bank: String,
    },
    /// Create a question in a bank
    Create {
        /// Question bank id
        #[arg(long = "bank")]
        bank: String,
        /// Question name
        #[arg(long)]
        name: Option<String>,
        /// Question text
        #[arg(long)]
        text: Option<String>,
        /// Question type (multiple_choice, essay, etc.)
        #[arg(long = "type")]
        question_type: Option<String>,
        /// Points possible
        #[arg(long)]
        points: Option<String>,
        /// Correct answer comments
        #[arg(long = "correct-comments")]
        correct_comments: Option<String>,
        /// Incorrect answer comments
        #[arg(long = "incorrect-comments")]
        incorrect_comments: Option<String>,
        /// Neutral answer comments
        #[arg(long = "neutral-comments")]
        neutral_comments: Option<String>,
        /// Question JSON payload
        #[arg(long = "question-json")]
        question_json: Option<String>,
        /// Question JSON file path
        #[arg(long = "question-file")]
        question_file: Option<PathBuf>,
    },
    /// Update a question in a bank
    Update {
        /// Question bank id
        #[arg(long = "bank")]
        bank: String,
        /// Question id
        #[arg(long = "question")]
        question: String,
        /// Question name
        #[arg(long)]
        name: Option<String>,
        /// Question text
        #[arg(long)]
        text: Option<String>,
        /// Question type (multiple_choice, essay, etc.)
        #[arg(long = "type")]
        question_type: Option<String>,
        /// Points possible
        #[arg(long)]
        points: Option<String>,
        /// Correct answer comments
        #[arg(long = "correct-comments")]
        correct_comments: Option<String>,
        /// Incorrect answer comments
        #[arg(long = "incorrect-comments")]
        incorrect_comments: Option<String>,
        /// Neutral answer comments
        #[arg(long = "neutral-comments")]
        neutral_comments: Option<String>,
        /// Question JSON payload
        #[arg(long = "question-json")]
        question_json: Option<String>,
        /// Question JSON file path
        #[arg(long = "question-file")]
        question_file: Option<PathBuf>,
    },
    /// Delete a question in a bank
    Delete {
        /// Question bank id
        #[arg(long = "bank")]
        bank: String,
        /// Question id
        #[arg(long = "question")]
        question: String,
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
    /// Manage module requirements
    Requirement {
        #[command(subcommand)]
        command: ModuleRequirementCommand,
    },
}

#[derive(Debug, Subcommand)]
enum ModuleRequirementCommand {
    /// List module requirements and prerequisites
    List {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
        /// Module id
        #[arg(long)]
        module: String,
    },
    /// Update module requirements, prerequisites, or unlock rules
    Update {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
        /// Module id
        #[arg(long)]
        module: String,
        /// Requirements JSON array
        #[arg(long = "requirements-json")]
        requirements_json: Option<String>,
        /// Requirements JSON file
        #[arg(long = "requirements-file")]
        requirements_file: Option<PathBuf>,
        /// Prerequisite module ids JSON array
        #[arg(long = "prerequisites-json")]
        prerequisites_json: Option<String>,
        /// Prerequisite module ids JSON file
        #[arg(long = "prerequisites-file")]
        prerequisites_file: Option<PathBuf>,
        /// Module unlock timestamp (RFC3339)
        #[arg(long = "unlock-at")]
        unlock_at: Option<String>,
        /// Sequential progress setting (enabled or disabled)
        #[arg(long = "sequential-progress")]
        sequential_progress: Option<SequentialProgressSetting>,
    },
}

#[derive(Debug, Subcommand)]
enum ToolCommand {
    /// List external tools for a course
    List {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
    },
    /// Create an external tool
    Create {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
        /// Tool name
        #[arg(long)]
        name: String,
        /// Tool config URL
        #[arg(long = "config-url")]
        config_url: Option<String>,
        /// Tool config JSON
        #[arg(long = "config-json")]
        config_json: Option<String>,
        /// Tool config XML
        #[arg(long = "config-xml")]
        config_xml: Option<String>,
        /// Tool config file path (.json or .xml)
        #[arg(long = "config-file")]
        config_file: Option<PathBuf>,
        /// Tool placement (repeatable)
        #[arg(long = "placement")]
        placement: Vec<String>,
        /// Placement settings JSON
        #[arg(long = "placements-json")]
        placements_json: Option<String>,
        /// Placement settings JSON file path
        #[arg(long = "placements-file")]
        placements_file: Option<PathBuf>,
    },
    /// Update an external tool
    Update {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
        /// Tool id
        #[arg(long)]
        tool: String,
        /// Tool name
        #[arg(long)]
        name: Option<String>,
        /// Tool config URL
        #[arg(long = "config-url")]
        config_url: Option<String>,
        /// Tool config JSON
        #[arg(long = "config-json")]
        config_json: Option<String>,
        /// Tool config XML
        #[arg(long = "config-xml")]
        config_xml: Option<String>,
        /// Tool config file path (.json or .xml)
        #[arg(long = "config-file")]
        config_file: Option<PathBuf>,
        /// Tool placement (repeatable)
        #[arg(long = "placement")]
        placement: Vec<String>,
        /// Placement settings JSON
        #[arg(long = "placements-json")]
        placements_json: Option<String>,
        /// Placement settings JSON file path
        #[arg(long = "placements-file")]
        placements_file: Option<PathBuf>,
    },
    /// Delete an external tool
    Delete {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
        /// Tool id
        #[arg(long)]
        tool: String,
    },
}

#[derive(Debug, Subcommand)]
enum CalendarCommand {
    /// List calendar events
    List {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
        /// Section id override
        #[arg(long)]
        section: Option<String>,
    },
    /// Create a calendar event
    Create {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
        /// Section id override
        #[arg(long)]
        section: Option<String>,
        /// Event title
        #[arg(long)]
        title: String,
        /// Event start time (RFC3339)
        #[arg(long = "start-at")]
        start_at: Option<String>,
        /// Event end time (RFC3339)
        #[arg(long = "end-at")]
        end_at: Option<String>,
        /// All-day date (YYYY-MM-DD)
        #[arg(long = "all-day-date")]
        all_day_date: Option<String>,
    },
    /// Update a calendar event
    Update {
        /// Event id
        #[arg(long = "event")]
        event_id: String,
        /// Event title
        #[arg(long)]
        title: Option<String>,
        /// Event start time (RFC3339)
        #[arg(long = "start-at")]
        start_at: Option<String>,
        /// Event end time (RFC3339)
        #[arg(long = "end-at")]
        end_at: Option<String>,
        /// All-day date (YYYY-MM-DD)
        #[arg(long = "all-day-date")]
        all_day_date: Option<String>,
    },
    /// Delete a calendar event
    Delete {
        /// Event id
        #[arg(long = "event")]
        event_id: String,
    },
}

#[derive(Debug, Subcommand)]
enum ConferenceCommand {
    /// List conferences
    List {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
    },
    /// Create a conference
    Create {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
        /// Conference title
        #[arg(long)]
        title: String,
        /// Conference description
        #[arg(long)]
        description: Option<String>,
        /// Conference start time (RFC3339)
        #[arg(long = "start-at")]
        start_at: Option<String>,
        /// Conference duration in minutes
        #[arg(long)]
        duration: Option<String>,
        /// Recording setting (enabled or disabled)
        #[arg(long)]
        recording: Option<RecordingSetting>,
    },
    /// Update a conference
    Update {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
        /// Conference id
        #[arg(long)]
        conference: String,
        /// Conference title
        #[arg(long)]
        title: Option<String>,
        /// Conference description
        #[arg(long)]
        description: Option<String>,
        /// Conference start time (RFC3339)
        #[arg(long = "start-at")]
        start_at: Option<String>,
        /// Conference duration in minutes
        #[arg(long)]
        duration: Option<String>,
        /// Recording setting (enabled or disabled)
        #[arg(long)]
        recording: Option<RecordingSetting>,
    },
    /// Delete a conference
    Delete {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
        /// Conference id
        #[arg(long)]
        conference: String,
    },
}

#[derive(Debug, Subcommand)]
enum CollaborationCommand {
    /// List collaborations for a course
    List {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
    },
    /// Create a collaboration
    Create {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
        /// Collaboration title
        #[arg(long)]
        title: String,
        /// Collaboration type (google_docs or office365)
        #[arg(long = "collaboration-type")]
        collaboration_type: String,
        /// Collaborator user ids
        #[arg(long = "user")]
        user_ids: Vec<String>,
        /// Collaborator group ids
        #[arg(long = "group")]
        group_ids: Vec<String>,
    },
    /// Delete a collaboration
    Delete {
        /// Collaboration id
        #[arg(long)]
        collaboration: String,
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
enum ContentMigrationCommand {
    /// List content migrations for a course
    List {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
    },
    /// Create a content migration
    Create {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
        /// Migration type (course_copy, file_import)
        #[arg(long = "type")]
        migration_type: String,
        /// Source course id (required for course_copy)
        #[arg(long = "source-course")]
        source_course: Option<String>,
        /// Zip file to import (required for file_import)
        #[arg(long)]
        file: Option<PathBuf>,
        /// Wait for completion
        #[arg(long)]
        wait: bool,
    },
    /// Show a content migration
    Show {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
        /// Content migration id
        #[arg(long)]
        migration: String,
        /// Wait for completion
        #[arg(long)]
        wait: bool,
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
enum SectionCommand {
    /// List sections in a course
    List {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
    },
    /// Create a section in a course
    Create {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
        /// Section name
        #[arg(long)]
        name: String,
    },
    /// Update a section
    Update {
        /// Section id
        #[arg(long)]
        section: String,
        /// Section name
        #[arg(long)]
        name: Option<String>,
    },
    /// Delete a section
    Delete {
        /// Section id
        #[arg(long)]
        section: String,
    },
}

#[derive(Debug, Subcommand)]
enum EnrollmentCommand {
    /// List enrollments in a section
    List {
        /// Section id
        #[arg(long)]
        section: String,
        /// Role filter (student, ta, teacher)
        #[arg(long)]
        role: Option<String>,
    },
    /// Add enrollment to a section
    Add {
        /// Section id
        #[arg(long)]
        section: String,
        /// User id
        #[arg(long)]
        user: String,
        /// Enrollment role (student, ta, teacher)
        #[arg(long)]
        role: String,
        /// Limit privileges to the section
        #[arg(long = "limit-to-section")]
        limit_to_section: bool,
    },
    /// Remove enrollment from a section
    Remove {
        /// Section id
        #[arg(long)]
        section: String,
        /// Enrollment id
        #[arg(long)]
        enrollment: String,
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
    /// Export grade change log if available
    GradeChangeLog {
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
}

#[derive(Debug, Subcommand)]
enum GradebookCommand {
    /// Manage grading periods
    GradingPeriod {
        #[command(subcommand)]
        command: GradingPeriodCommand,
    },
    /// Manage gradebook posting policy
    PostingPolicy {
        #[command(subcommand)]
        command: PostingPolicyCommand,
    },
}

#[derive(Debug, Subcommand)]
enum GradingPeriodCommand {
    /// List grading periods for a course
    List {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
    },
    /// Show a grading period
    Show {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
        /// Grading period id
        #[arg(long)]
        period: String,
    },
}

#[derive(Debug, Subcommand)]
enum PostingPolicyCommand {
    /// Get the gradebook posting policy
    Get {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
    },
    /// Update the gradebook posting policy
    Set {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
        /// Posting policy (automatic or manual)
        #[arg(long)]
        policy: String,
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

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
enum RecordingSetting {
    Enabled,
    Disabled,
}

impl RecordingSetting {
    fn as_bool(self) -> bool {
        matches!(self, RecordingSetting::Enabled)
    }
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
enum SequentialProgressSetting {
    Enabled,
    Disabled,
}

impl SequentialProgressSetting {
    fn as_bool(self) -> bool {
        matches!(self, SequentialProgressSetting::Enabled)
    }
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
struct OutcomeListRequest {
    course_id: CourseId,
}

#[derive(Debug)]
struct OutcomeCreateRequest {
    course_id: CourseId,
    outcome_group_id: OutcomeGroupId,
    input: canvas_core::OutcomeCreateInput,
}

#[derive(Debug)]
struct OutcomeUpdateRequest {
    outcome_id: OutcomeId,
    input: canvas_core::OutcomeUpdateInput,
}

#[derive(Debug)]
struct OutcomeDeleteRequest {
    course_id: CourseId,
    outcome_group_id: OutcomeGroupId,
    outcome_id: OutcomeId,
}

#[derive(Debug)]
enum OutcomeRequest {
    List(OutcomeListRequest),
    Create(OutcomeCreateRequest),
    Update(OutcomeUpdateRequest),
    Delete(OutcomeDeleteRequest),
}

#[derive(Debug)]
struct RubricListRequest {
    course_id: CourseId,
}

#[derive(Debug)]
struct RubricCreateRequest {
    course_id: CourseId,
    input: canvas_core::RubricCreateInput,
}

#[derive(Debug)]
struct RubricUpdateRequest {
    course_id: CourseId,
    rubric_id: RubricId,
    input: canvas_core::RubricUpdateInput,
}

#[derive(Debug)]
struct RubricDeleteRequest {
    course_id: CourseId,
    rubric_id: RubricId,
}

#[derive(Debug)]
struct RubricAttachRequest {
    course_id: CourseId,
    input: canvas_core::RubricAssociationInput,
}

#[derive(Debug)]
struct RubricDetachRequest {
    course_id: CourseId,
    rubric_association_id: RubricAssociationId,
}

#[derive(Debug)]
enum RubricRequest {
    List(RubricListRequest),
    Create(RubricCreateRequest),
    Update(RubricUpdateRequest),
    Delete(RubricDeleteRequest),
    Attach(RubricAttachRequest),
    Detach(RubricDetachRequest),
}

#[derive(Debug)]
struct CalendarEventListRequest {
    context: CalendarEventContext,
}

#[derive(Debug)]
struct CalendarEventCreateRequest {
    input: canvas_core::CalendarEventCreateInput,
}

#[derive(Debug)]
struct CalendarEventUpdateRequest {
    event_id: CalendarEventId,
    input: canvas_core::CalendarEventUpdateInput,
}

#[derive(Debug)]
struct CalendarEventDeleteRequest {
    event_id: CalendarEventId,
}

#[derive(Debug)]
enum CalendarEventRequest {
    List(CalendarEventListRequest),
    Create(CalendarEventCreateRequest),
    Update(CalendarEventUpdateRequest),
    Delete(CalendarEventDeleteRequest),
}

#[derive(Debug)]
struct ConferenceListRequest {
    course_id: CourseId,
}

#[derive(Debug)]
struct ConferenceCreateRequest {
    course_id: CourseId,
    input: canvas_core::ConferenceCreateInput,
}

#[derive(Debug)]
struct ConferenceUpdateRequest {
    course_id: CourseId,
    conference_id: ConferenceId,
    input: canvas_core::ConferenceUpdateInput,
}

#[derive(Debug)]
struct ConferenceDeleteRequest {
    course_id: CourseId,
    conference_id: ConferenceId,
}

#[derive(Debug)]
enum ConferenceRequest {
    List(ConferenceListRequest),
    Create(ConferenceCreateRequest),
    Update(ConferenceUpdateRequest),
    Delete(ConferenceDeleteRequest),
}

#[derive(Debug)]
struct CollaborationListRequest {
    course_id: CourseId,
}

#[derive(Debug)]
struct CollaborationCreateRequest {
    course_id: CourseId,
    input: canvas_core::CollaborationCreateInput,
}

#[derive(Debug)]
struct CollaborationDeleteRequest {
    collaboration_id: CollaborationId,
}

#[derive(Debug)]
enum CollaborationRequest {
    List(CollaborationListRequest),
    Create(CollaborationCreateRequest),
    Delete(CollaborationDeleteRequest),
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

struct QuestionBankListRequest {
    course_id: CourseId,
}

struct QuestionBankCreateRequest {
    course_id: CourseId,
    input: canvas_core::QuestionBankCreateInput,
}

struct QuestionBankUpdateRequest {
    course_id: CourseId,
    bank_id: QuestionBankId,
    input: canvas_core::QuestionBankUpdateInput,
}

struct QuestionBankDeleteRequest {
    course_id: CourseId,
    bank_id: QuestionBankId,
}

enum QuestionBankRequest {
    List(QuestionBankListRequest),
    Create(QuestionBankCreateRequest),
    Update(QuestionBankUpdateRequest),
    Delete(QuestionBankDeleteRequest),
}

struct QuestionListRequest {
    bank_id: QuestionBankId,
}

struct QuestionCreateRequest {
    bank_id: QuestionBankId,
    input: canvas_core::QuestionCreateInput,
}

struct QuestionUpdateRequest {
    bank_id: QuestionBankId,
    question_id: QuestionId,
    input: canvas_core::QuestionUpdateInput,
}

struct QuestionDeleteRequest {
    bank_id: QuestionBankId,
    question_id: QuestionId,
}

enum QuestionRequest {
    List(QuestionListRequest),
    Create(QuestionCreateRequest),
    Update(QuestionUpdateRequest),
    Delete(QuestionDeleteRequest),
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
struct ModuleRequirementListRequest {
    course_id: CourseId,
    module_id: ModuleId,
}

#[derive(Debug)]
struct ModuleRequirementUpdateRequest {
    course_id: CourseId,
    module_id: ModuleId,
    input: canvas_core::ModuleRequirementsUpdateInput,
}

#[derive(Debug)]
enum ModuleRequest {
    List(ModuleListRequest),
    Create(ModuleCreateRequest),
    Update(ModuleUpdateRequest),
    Reorder(ModuleReorderRequest),
    Publish(ModulePublishRequest),
    RequirementList(ModuleRequirementListRequest),
    RequirementUpdate(ModuleRequirementUpdateRequest),
}

#[derive(Debug)]
struct ToolListRequest {
    course_id: CourseId,
}

#[derive(Debug)]
struct ToolCreateRequest {
    course_id: CourseId,
    name: canvas_models::ExternalToolName,
    config: canvas_core::ExternalToolConfig,
    placements: Option<canvas_core::ExternalToolPlacementList>,
    placement_settings: Option<canvas_core::ExternalToolPlacementSettings>,
}

#[derive(Debug)]
struct ToolUpdateRequest {
    course_id: CourseId,
    tool_id: canvas_models::ExternalToolId,
    name: Option<canvas_models::ExternalToolName>,
    config: Option<canvas_core::ExternalToolConfig>,
    placements: Option<canvas_core::ExternalToolPlacementList>,
    placement_settings: Option<canvas_core::ExternalToolPlacementSettings>,
}

#[derive(Debug)]
struct ToolDeleteRequest {
    course_id: CourseId,
    tool_id: canvas_models::ExternalToolId,
}

#[derive(Debug)]
enum ToolRequest {
    List(ToolListRequest),
    Create(ToolCreateRequest),
    Update(ToolUpdateRequest),
    Delete(ToolDeleteRequest),
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
struct ContentMigrationListRequest {
    course_id: CourseId,
}

#[derive(Debug)]
struct ContentMigrationCreateRequest {
    course_id: CourseId,
    input: canvas_core::ContentMigrationCreateInput,
    wait: bool,
}

#[derive(Debug)]
struct ContentMigrationShowRequest {
    course_id: CourseId,
    migration_id: ContentMigrationId,
    wait: bool,
}

#[derive(Debug)]
enum ContentMigrationRequest {
    List(ContentMigrationListRequest),
    Create(ContentMigrationCreateRequest),
    Show(ContentMigrationShowRequest),
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
struct SectionListRequest {
    course_id: CourseId,
}

#[derive(Debug)]
struct SectionCreateRequest {
    course_id: CourseId,
    input: canvas_core::SectionCreateInput,
}

#[derive(Debug)]
struct SectionUpdateRequest {
    section_id: SectionId,
    input: canvas_core::SectionUpdateInput,
}

#[derive(Debug)]
struct SectionDeleteRequest {
    section_id: SectionId,
}

#[derive(Debug)]
enum SectionRequest {
    List(SectionListRequest),
    Create(SectionCreateRequest),
    Update(SectionUpdateRequest),
    Delete(SectionDeleteRequest),
}

#[derive(Debug)]
struct EnrollmentListRequest {
    section_id: SectionId,
    role: Option<canvas_models::UserRole>,
}

#[derive(Debug)]
struct EnrollmentAddRequest {
    section_id: SectionId,
    input: canvas_core::EnrollmentCreateInput,
}

#[derive(Debug)]
struct EnrollmentRemoveRequest {
    section_id: SectionId,
    enrollment_id: EnrollmentId,
}

#[derive(Debug)]
enum EnrollmentRequest {
    List(EnrollmentListRequest),
    Add(EnrollmentAddRequest),
    Remove(EnrollmentRemoveRequest),
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
    GradeChangeLog(ReportExportRequest),
}

#[derive(Debug)]
enum GradebookRequest {
    GradingPeriodList(GradingPeriodListRequest),
    GradingPeriodShow(GradingPeriodShowRequest),
    PostingPolicyGet(PostingPolicyRequest),
    PostingPolicySet(PostingPolicyUpdateRequest),
}

#[derive(Debug)]
struct GradingPeriodListRequest {
    course_id: CourseId,
}

#[derive(Debug)]
struct GradingPeriodShowRequest {
    course_id: CourseId,
    grading_period_id: GradingPeriodId,
}

#[derive(Debug)]
struct PostingPolicyRequest {
    course_id: CourseId,
}

#[derive(Debug)]
struct PostingPolicyUpdateRequest {
    course_id: CourseId,
    policy: GradingPostingPolicy,
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
        Command::Outcome { command } => {
            let request = OutcomeRequest::try_from((command, &global))?;
            handle_outcome(request, &global)
        }
        Command::Rubric { command } => {
            let request = RubricRequest::try_from((command, &global))?;
            handle_rubric(request, &global)
        }
        Command::Submission { command } => {
            let request = SubmissionRequest::try_from((command, &global))?;
            handle_submission(request, &global)
        }
        Command::Quiz { command } => {
            let request = QuizRequest::try_from((command, &global))?;
            handle_quiz(request, &global)
        }
        Command::QuestionBank { command } => {
            let request = QuestionBankRequest::try_from((command, &global))?;
            handle_question_bank(request, &global)
        }
        Command::Question { command } => {
            let request = QuestionRequest::try_from((command, &global))?;
            handle_question(request, &global)
        }
        Command::Page { command } => {
            let request = PageRequest::try_from((command, &global))?;
            handle_page(request, &global)
        }
        Command::Module { command } => {
            let request = ModuleRequest::try_from((command, &global))?;
            handle_module(request, &global)
        }
        Command::Tool { command } => {
            let request = ToolRequest::try_from((command, &global))?;
            handle_tool(request, &global)
        }
        Command::Calendar { command } => {
            let request = CalendarEventRequest::try_from((command, &global))?;
            handle_calendar_event(request, &global)
        }
        Command::Conference { command } => {
            let request = ConferenceRequest::try_from((command, &global))?;
            handle_conference(request, &global)
        }
        Command::Collaboration { command } => {
            let request = CollaborationRequest::try_from((command, &global))?;
            handle_collaboration(request, &global)
        }
        Command::File { command } => {
            let request = FileRequest::try_from((command, &global))?;
            handle_file(request, &global)
        }
        Command::Folder { command } => {
            let request = FolderRequest::try_from((command, &global))?;
            handle_folder(request, &global)
        }
        Command::ContentMigration { command } => {
            let request =
                ContentMigrationRequest::try_from((command, &global))?;
            handle_content_migration(request, &global)
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
        Command::Section { command } => {
            let request = SectionRequest::try_from((command, &global))?;
            handle_section(request, &global)
        }
        Command::Enrollment { command } => {
            let request = EnrollmentRequest::try_from((command, &global))?;
            handle_enrollment(request, &global)
        }
        Command::Report { command } => {
            let request = ReportRequest::try_from((command, &global))?;
            handle_report(request, &global)
        }
        Command::Gradebook { command } => {
            let request = GradebookRequest::try_from((command, &global))?;
            handle_gradebook(request, &global)
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

fn handle_outcome(
    request: OutcomeRequest,
    global: &GlobalOptions,
) -> Result<(), CliError> {
    match request {
        OutcomeRequest::List(request) => {
            let planned = vec![PlannedAction {
                action: "list outcomes",
                risk: "none",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("outcome list", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let outcomes = canvas_core::list_outcomes(&config, request.course_id)?;
            let data = json!({
                "status": "ok",
                "course_id": request.course_id.get(),
                "outcomes": outcomes.iter().map(outcome_summary_json).collect::<Vec<_>>(),
            });
            emit_result("outcome list", data, global);
            if !global.quiet && !global.json {
                println!("Found {} outcomes.", outcomes.len());
            }
        }
        OutcomeRequest::Create(request) => {
            let planned = vec![PlannedAction {
                action: "create outcome",
                risk: "creates remote outcome",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("outcome create", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let outcome = canvas_core::create_outcome(
                &config,
                request.course_id,
                request.outcome_group_id,
                &request.input,
            )?;
            emit_result(
                "outcome create",
                json!({
                    "status": "created",
                    "course_id": request.course_id.get(),
                    "outcome_group_id": request.outcome_group_id.get(),
                    "outcome": outcome_summary_json(&outcome),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("Outcome {} created.", outcome.id);
            }
        }
        OutcomeRequest::Update(request) => {
            let planned = vec![PlannedAction {
                action: "update outcome",
                risk: "updates remote outcome",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("outcome update", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let outcome = canvas_core::update_outcome(
                &config,
                request.outcome_id,
                &request.input,
            )?;
            emit_result(
                "outcome update",
                json!({
                    "status": "updated",
                    "outcome": outcome_summary_json(&outcome),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("Outcome {} updated.", outcome.id);
            }
        }
        OutcomeRequest::Delete(request) => {
            let planned = vec![PlannedAction {
                action: "delete outcome",
                risk: "deletes remote outcome",
                requires_confirmation: true,
            }];
            if global.explain {
                emit_plan("outcome delete", &planned, global);
                return Ok(());
            }
            require_confirmation(global, "outcome delete")?;
            let config = canvas_core::load_merged_config()?;
            let outcome = canvas_core::delete_outcome(
                &config,
                request.course_id,
                request.outcome_group_id,
                request.outcome_id,
            )?;
            emit_result(
                "outcome delete",
                json!({
                    "status": "deleted",
                    "course_id": request.course_id.get(),
                    "outcome_group_id": request.outcome_group_id.get(),
                    "outcome": outcome_summary_json(&outcome),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("Outcome {} deleted.", outcome.id);
            }
        }
    }
    Ok(())
}

fn handle_rubric(
    request: RubricRequest,
    global: &GlobalOptions,
) -> Result<(), CliError> {
    match request {
        RubricRequest::List(request) => {
            let planned = vec![PlannedAction {
                action: "list rubrics",
                risk: "none",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("rubric list", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let rubrics = canvas_core::list_rubrics(&config, request.course_id)?;
            let data = json!({
                "status": "ok",
                "course_id": request.course_id.get(),
                "rubrics": rubrics.iter().map(rubric_summary_json).collect::<Vec<_>>(),
            });
            emit_result("rubric list", data, global);
            if !global.quiet && !global.json {
                println!("Found {} rubrics.", rubrics.len());
            }
        }
        RubricRequest::Create(request) => {
            let planned = vec![PlannedAction {
                action: "create rubric",
                risk: "creates remote rubric",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("rubric create", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let rubric =
                canvas_core::create_rubric(&config, request.course_id, &request.input)?;
            emit_result(
                "rubric create",
                json!({
                    "status": "created",
                    "course_id": request.course_id.get(),
                    "rubric": rubric_summary_json(&rubric),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("Rubric {} created.", rubric.id);
            }
        }
        RubricRequest::Update(request) => {
            let planned = vec![PlannedAction {
                action: "update rubric",
                risk: "updates remote rubric",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("rubric update", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let rubric = canvas_core::update_rubric(
                &config,
                request.course_id,
                request.rubric_id,
                &request.input,
            )?;
            emit_result(
                "rubric update",
                json!({
                    "status": "updated",
                    "course_id": request.course_id.get(),
                    "rubric": rubric_summary_json(&rubric),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("Rubric {} updated.", rubric.id);
            }
        }
        RubricRequest::Delete(request) => {
            let planned = vec![PlannedAction {
                action: "delete rubric",
                risk: "deletes remote rubric",
                requires_confirmation: true,
            }];
            if global.explain {
                emit_plan("rubric delete", &planned, global);
                return Ok(());
            }
            require_confirmation(global, "rubric delete")?;
            let config = canvas_core::load_merged_config()?;
            let rubric = canvas_core::delete_rubric(
                &config,
                request.course_id,
                request.rubric_id,
            )?;
            emit_result(
                "rubric delete",
                json!({
                    "status": "deleted",
                    "course_id": request.course_id.get(),
                    "rubric": rubric_summary_json(&rubric),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("Rubric {} deleted.", rubric.id);
            }
        }
        RubricRequest::Attach(request) => {
            let planned = vec![PlannedAction {
                action: "attach rubric",
                risk: "attaches rubric to target",
                requires_confirmation: true,
            }];
            if global.explain {
                emit_plan("rubric attach", &planned, global);
                return Ok(());
            }
            require_confirmation(global, "rubric attach")?;
            let config = canvas_core::load_merged_config()?;
            let association =
                canvas_core::attach_rubric(&config, request.course_id, &request.input)?;
            emit_result(
                "rubric attach",
                json!({
                    "status": "attached",
                    "course_id": request.course_id.get(),
                    "rubric_association": rubric_association_summary_json(&association),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("Rubric association {} attached.", association.id);
            }
        }
        RubricRequest::Detach(request) => {
            let planned = vec![PlannedAction {
                action: "detach rubric",
                risk: "detaches rubric association",
                requires_confirmation: true,
            }];
            if global.explain {
                emit_plan("rubric detach", &planned, global);
                return Ok(());
            }
            require_confirmation(global, "rubric detach")?;
            let config = canvas_core::load_merged_config()?;
            let association = canvas_core::detach_rubric(
                &config,
                request.course_id,
                request.rubric_association_id,
            )?;
            emit_result(
                "rubric detach",
                json!({
                    "status": "detached",
                    "course_id": request.course_id.get(),
                    "rubric_association": rubric_association_summary_json(&association),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("Rubric association {} detached.", association.id);
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

fn handle_question_bank(
    request: QuestionBankRequest,
    global: &GlobalOptions,
) -> Result<(), CliError> {
    match request {
        QuestionBankRequest::List(request) => {
            let planned = vec![PlannedAction {
                action: "list question banks",
                risk: "none",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("question bank list", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let banks = canvas_core::list_question_banks(&config, request.course_id)?;
            let data = json!({
                "status": "ok",
                "course_id": request.course_id.get(),
                "question_banks": banks
                    .iter()
                    .map(question_bank_summary_json)
                    .collect::<Vec<_>>(),
            });
            emit_result("question bank list", data, global);
            if !global.quiet && !global.json {
                println!("Found {} question banks.", banks.len());
            }
        }
        QuestionBankRequest::Create(request) => {
            let planned = vec![PlannedAction {
                action: "create question bank",
                risk: "creates remote question bank",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("question bank create", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let bank =
                canvas_core::create_question_bank(&config, request.course_id, &request.input)?;
            emit_result(
                "question bank create",
                json!({
                    "status": "ok",
                    "course_id": request.course_id.get(),
                    "question_bank": question_bank_summary_json(&bank),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("Question bank {} created.", bank.id);
            }
        }
        QuestionBankRequest::Update(request) => {
            let planned = vec![PlannedAction {
                action: "update question bank",
                risk: "updates remote question bank",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("question bank update", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let bank = canvas_core::update_question_bank(
                &config,
                request.course_id,
                request.bank_id,
                &request.input,
            )?;
            emit_result(
                "question bank update",
                json!({
                    "status": "ok",
                    "course_id": request.course_id.get(),
                    "question_bank": question_bank_summary_json(&bank),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("Question bank {} updated.", bank.id);
            }
        }
        QuestionBankRequest::Delete(request) => {
            let planned = vec![PlannedAction {
                action: "delete question bank",
                risk: "deletes remote question bank",
                requires_confirmation: true,
            }];
            if global.explain {
                emit_plan("question bank delete", &planned, global);
                return Ok(());
            }
            require_confirmation(global, "question bank delete")?;
            let config = canvas_core::load_merged_config()?;
            let bank = canvas_core::delete_question_bank(
                &config,
                request.course_id,
                request.bank_id,
            )?;
            emit_result(
                "question bank delete",
                json!({
                    "status": "ok",
                    "course_id": request.course_id.get(),
                    "question_bank": question_bank_summary_json(&bank),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("Question bank {} deleted.", bank.id);
            }
        }
    }
    Ok(())
}

fn handle_question(request: QuestionRequest, global: &GlobalOptions) -> Result<(), CliError> {
    match request {
        QuestionRequest::List(request) => {
            let planned = vec![PlannedAction {
                action: "list questions",
                risk: "none",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("question list", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let questions = canvas_core::list_questions(&config, request.bank_id)?;
            let data = json!({
                "status": "ok",
                "bank_id": request.bank_id.get(),
                "questions": questions
                    .iter()
                    .map(question_summary_json)
                    .collect::<Vec<_>>(),
            });
            emit_result("question list", data, global);
            if !global.quiet && !global.json {
                println!("Found {} questions.", questions.len());
            }
        }
        QuestionRequest::Create(request) => {
            let planned = vec![PlannedAction {
                action: "create question",
                risk: "creates remote question",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("question create", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let question =
                canvas_core::create_question(&config, request.bank_id, &request.input)?;
            emit_result(
                "question create",
                json!({
                    "status": "ok",
                    "bank_id": request.bank_id.get(),
                    "question": question_summary_json(&question),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("Question {} created.", question.id);
            }
        }
        QuestionRequest::Update(request) => {
            let planned = vec![PlannedAction {
                action: "update question",
                risk: "updates remote question",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("question update", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let question = canvas_core::update_question(
                &config,
                request.bank_id,
                request.question_id,
                &request.input,
            )?;
            emit_result(
                "question update",
                json!({
                    "status": "ok",
                    "bank_id": request.bank_id.get(),
                    "question": question_summary_json(&question),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("Question {} updated.", question.id);
            }
        }
        QuestionRequest::Delete(request) => {
            let planned = vec![PlannedAction {
                action: "delete question",
                risk: "deletes remote question",
                requires_confirmation: true,
            }];
            if global.explain {
                emit_plan("question delete", &planned, global);
                return Ok(());
            }
            require_confirmation(global, "question delete")?;
            let config = canvas_core::load_merged_config()?;
            let question = canvas_core::delete_question(
                &config,
                request.bank_id,
                request.question_id,
            )?;
            emit_result(
                "question delete",
                json!({
                    "status": "ok",
                    "bank_id": request.bank_id.get(),
                    "question": question_summary_json(&question),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("Question {} deleted.", question.id);
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
        ModuleRequest::RequirementList(request) => {
            let planned = vec![PlannedAction {
                action: "list module requirements",
                risk: "reads module requirements",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("module requirement list", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let module = canvas_core::get_module(
                &config,
                request.course_id,
                request.module_id,
            )?;
            let data = json!({
                "status": "ok",
                "course_id": request.course_id.get(),
                "module": module_summary_json(&module),
            });
            emit_result("module requirement list", data, global);
            if !global.quiet && !global.json {
                println!("Module {} requirements loaded.", module.id);
            }
        }
        ModuleRequest::RequirementUpdate(request) => {
            let planned = vec![PlannedAction {
                action: "update module requirements",
                risk: "updates module requirements",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("module requirement update", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let module = canvas_core::update_module_requirements(
                &config,
                request.course_id,
                request.module_id,
                &request.input,
            )?;
            let data = json!({
                "status": "updated",
                "course_id": request.course_id.get(),
                "module": module_summary_json(&module),
            });
            emit_result("module requirement update", data, global);
            if !global.quiet && !global.json {
                println!("Module {} requirements updated.", module.id);
            }
        }
    }
    Ok(())
}

fn handle_tool(request: ToolRequest, global: &GlobalOptions) -> Result<(), CliError> {
    match request {
        ToolRequest::List(request) => {
            let planned = vec![PlannedAction {
                action: "list external tools",
                risk: "none",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("tool list", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let tools = canvas_core::list_external_tools(&config, request.course_id)?;
            let data = json!({
                "status": "ok",
                "course_id": request.course_id.get(),
                "tools": tools.iter().map(external_tool_summary_json).collect::<Vec<_>>(),
            });
            emit_result("tool list", data, global);
            if !global.quiet && !global.json {
                println!("Found {} tools.", tools.len());
            }
        }
        ToolRequest::Create(request) => {
            let planned = vec![PlannedAction {
                action: "create external tool",
                risk: "creates remote external tool",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("tool create", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let input = canvas_core::ExternalToolCreateInput {
                name: request.name,
                config: request.config,
                placements: request.placements,
                placement_settings: request.placement_settings,
            };
            let tool =
                canvas_core::create_external_tool(&config, request.course_id, &input)?;
            emit_result(
                "tool create",
                json!({
                    "status": "created",
                    "course_id": request.course_id.get(),
                    "tool": external_tool_summary_json(&tool),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("External tool created.");
            }
        }
        ToolRequest::Update(request) => {
            let planned = vec![PlannedAction {
                action: "update external tool",
                risk: "updates remote external tool",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("tool update", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let input = canvas_core::ExternalToolUpdateInput::new(
                request.name,
                request.config,
                request.placements,
                request.placement_settings,
            )
            .map_err(|_| {
                CliError::Canvas(canvas_core::CanvasError::InvalidExternalToolUpdate(
                    "missing_fields".to_string(),
                ))
            })?;
            let tool = canvas_core::update_external_tool(
                &config,
                request.course_id,
                request.tool_id,
                &input,
            )?;
            emit_result(
                "tool update",
                json!({
                    "status": "updated",
                    "course_id": request.course_id.get(),
                    "tool": external_tool_summary_json(&tool),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("External tool updated.");
            }
        }
        ToolRequest::Delete(request) => {
            let planned = vec![PlannedAction {
                action: "delete external tool",
                risk: "deletes remote external tool",
                requires_confirmation: true,
            }];
            if global.explain {
                emit_plan("tool delete", &planned, global);
                return Ok(());
            }
            require_confirmation(global, "tool delete")?;
            let config = canvas_core::load_merged_config()?;
            let tool = canvas_core::delete_external_tool(
                &config,
                request.course_id,
                request.tool_id,
            )?;
            emit_result(
                "tool delete",
                json!({
                    "status": "deleted",
                    "course_id": request.course_id.get(),
                    "tool": external_tool_summary_json(&tool),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("External tool deleted.");
            }
        }
    }
    Ok(())
}

fn handle_calendar_event(
    request: CalendarEventRequest,
    global: &GlobalOptions,
) -> Result<(), CliError> {
    match request {
        CalendarEventRequest::List(request) => {
            let planned = vec![PlannedAction {
                action: "list calendar events",
                risk: "none",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("calendar list", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let events =
                canvas_core::list_calendar_events(&config, request.context)?;
            let data = json!({
                "status": "ok",
                "context_type": request.context.context_type(),
                "context_id": request.context.context_id(),
                "events": events
                    .iter()
                    .map(calendar_event_summary_json)
                    .collect::<Vec<_>>(),
            });
            emit_result("calendar list", data, global);
            if !global.quiet && !global.json {
                println!("Found {} calendar events.", events.len());
            }
        }
        CalendarEventRequest::Create(request) => {
            let planned = vec![PlannedAction {
                action: "create calendar event",
                risk: "creates remote calendar event",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("calendar create", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let event =
                canvas_core::create_calendar_event(&config, &request.input)?;
            emit_result(
                "calendar create",
                json!({
                    "status": "created",
                    "event": calendar_event_summary_json(&event),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("Calendar event created.");
            }
        }
        CalendarEventRequest::Update(request) => {
            let planned = vec![PlannedAction {
                action: "update calendar event",
                risk: "updates remote calendar event",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("calendar update", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let event = canvas_core::update_calendar_event(
                &config,
                request.event_id,
                &request.input,
            )?;
            emit_result(
                "calendar update",
                json!({
                    "status": "updated",
                    "event": calendar_event_summary_json(&event),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("Calendar event updated.");
            }
        }
        CalendarEventRequest::Delete(request) => {
            let planned = vec![PlannedAction {
                action: "delete calendar event",
                risk: "deletes remote calendar event",
                requires_confirmation: true,
            }];
            if global.explain {
                emit_plan("calendar delete", &planned, global);
                return Ok(());
            }
            require_confirmation(global, "calendar delete")?;
            let config = canvas_core::load_merged_config()?;
            let event = canvas_core::delete_calendar_event(
                &config,
                request.event_id,
            )?;
            emit_result(
                "calendar delete",
                json!({
                    "status": "deleted",
                    "event": calendar_event_summary_json(&event),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("Calendar event deleted.");
            }
        }
    }
    Ok(())
}

fn handle_conference(
    request: ConferenceRequest,
    global: &GlobalOptions,
) -> Result<(), CliError> {
    let config = canvas_core::load_merged_config().map_err(CliError::Canvas)?;
    match request {
        ConferenceRequest::List(request) => {
            let planned = vec![PlannedAction {
                action: "list conferences",
                risk: "low",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("conference list", &planned, global);
                return Ok(());
            }
            let conferences =
                canvas_core::list_conferences(&config, request.course_id)?;
            let data = json!({
                "conferences": conferences
                    .iter()
                    .map(conference_summary_json)
                    .collect::<Vec<_>>(),
            });
            emit_result("conference list", data, global);
            if !global.json && !global.quiet {
                println!("Found {} conferences.", conferences.len());
            }
        }
        ConferenceRequest::Create(request) => {
            let planned = vec![PlannedAction {
                action: "create conference",
                risk: "creates remote conference",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("conference create", &planned, global);
                return Ok(());
            }
            let conference = canvas_core::create_conference(
                &config,
                request.course_id,
                &request.input,
            )?;
            let data = json!({
                "conference": conference_summary_json(&conference),
            });
            emit_result("conference create", data, global);
            if !global.json && !global.quiet {
                println!("Conference created.");
            }
        }
        ConferenceRequest::Update(request) => {
            let planned = vec![PlannedAction {
                action: "update conference",
                risk: "updates remote conference",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("conference update", &planned, global);
                return Ok(());
            }
            let conference = canvas_core::update_conference(
                &config,
                request.course_id,
                request.conference_id,
                &request.input,
            )?;
            let data = json!({
                "conference": conference_summary_json(&conference),
            });
            emit_result("conference update", data, global);
            if !global.json && !global.quiet {
                println!("Conference updated.");
            }
        }
        ConferenceRequest::Delete(request) => {
            require_confirmation(global, "conference delete")?;
            let planned = vec![PlannedAction {
                action: "delete conference",
                risk: "deletes remote conference",
                requires_confirmation: true,
            }];
            if global.explain {
                emit_plan("conference delete", &planned, global);
                return Ok(());
            }
            let conference = canvas_core::delete_conference(
                &config,
                request.course_id,
                request.conference_id,
            )?;
            let data = json!({
                "conference": conference_summary_json(&conference),
            });
            emit_result("conference delete", data, global);
            if !global.json && !global.quiet {
                println!("Conference deleted.");
            }
        }
    }
    Ok(())
}

fn handle_collaboration(
    request: CollaborationRequest,
    global: &GlobalOptions,
) -> Result<(), CliError> {
    match request {
        CollaborationRequest::List(request) => {
            let planned = vec![PlannedAction {
                action: "list collaborations",
                risk: "none",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("collaboration list", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let collaborations =
                canvas_core::list_collaborations(&config, request.course_id)?;
            emit_result(
                "collaboration list",
                json!({
                    "status": "ok",
                    "course_id": request.course_id.get(),
                    "collaborations": collaborations
                        .iter()
                        .map(collaboration_summary_json)
                        .collect::<Vec<_>>(),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("Found {} collaborations.", collaborations.len());
            }
        }
        CollaborationRequest::Create(request) => {
            let planned = vec![PlannedAction {
                action: "create collaboration",
                risk: "creates remote collaboration",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("collaboration create", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let collaboration = canvas_core::create_collaboration(
                &config,
                request.course_id,
                &request.input,
            )?;
            emit_result(
                "collaboration create",
                json!({
                    "status": "created",
                    "course_id": request.course_id.get(),
                    "collaboration": collaboration_summary_json(&collaboration),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("Collaboration created.");
            }
        }
        CollaborationRequest::Delete(request) => {
            let planned = vec![PlannedAction {
                action: "delete collaboration",
                risk: "deletes remote collaboration",
                requires_confirmation: true,
            }];
            if global.explain {
                emit_plan("collaboration delete", &planned, global);
                return Ok(());
            }
            require_confirmation(global, "collaboration delete")?;
            let config = canvas_core::load_merged_config()?;
            let collaboration =
                canvas_core::delete_collaboration(&config, request.collaboration_id)?;
            emit_result(
                "collaboration delete",
                json!({
                    "status": "deleted",
                    "collaboration": collaboration_summary_json(&collaboration),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("Collaboration deleted.");
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

fn handle_content_migration(
    request: ContentMigrationRequest,
    global: &GlobalOptions,
) -> Result<(), CliError> {
    match request {
        ContentMigrationRequest::List(request) => {
            let planned = vec![PlannedAction {
                action: "list content migrations",
                risk: "none",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("content-migration list", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let migrations =
                canvas_core::list_content_migrations(&config, request.course_id)?;
            let data = json!({
                "status": "ok",
                "course_id": request.course_id.get(),
                "migrations": migrations
                    .iter()
                    .map(|migration| content_migration_summary_json(migration, None))
                    .collect::<Vec<_>>(),
            });
            emit_result("content-migration list", data, global);
            if !global.quiet && !global.json {
                println!("Found {} content migrations.", migrations.len());
            }
        }
        ContentMigrationRequest::Create(request) => {
            let planned = vec![PlannedAction {
                action: "create content migration",
                risk: "imports remote content",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("content-migration create", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let mut migration = canvas_core::create_content_migration(
                &config,
                request.course_id,
                &request.input,
            )?;
            if request.wait {
                let migration_id = canvas_core::parse_content_migration_id(
                    &migration.id.to_string(),
                )?;
                migration = canvas_core::wait_for_content_migration(
                    &config,
                    request.course_id,
                    migration_id,
                    canvas_core::ContentMigrationWaitOptions::default(),
                )?;
            }
            let progress = match migration.progress_url.as_deref() {
                Some(url) => {
                    Some(canvas_core::get_content_migration_progress(&config, url)?)
                }
                None => None,
            };
            emit_result(
                "content-migration create",
                json!({
                    "status": "created",
                    "course_id": request.course_id.get(),
                    "migration": content_migration_summary_json(
                        &migration,
                        progress.as_ref(),
                    ),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("Content migration created.");
            }
        }
        ContentMigrationRequest::Show(request) => {
            let planned = vec![PlannedAction {
                action: "show content migration",
                risk: "none",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("content-migration show", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let mut migration = canvas_core::get_content_migration(
                &config,
                request.course_id,
                request.migration_id,
            )?;
            if request.wait {
                migration = canvas_core::wait_for_content_migration(
                    &config,
                    request.course_id,
                    request.migration_id,
                    canvas_core::ContentMigrationWaitOptions::default(),
                )?;
            }
            let progress = match migration.progress_url.as_deref() {
                Some(url) => {
                    Some(canvas_core::get_content_migration_progress(&config, url)?)
                }
                None => None,
            };
            emit_result(
                "content-migration show",
                json!({
                    "status": "ok",
                    "course_id": request.course_id.get(),
                    "migration": content_migration_summary_json(
                        &migration,
                        progress.as_ref(),
                    ),
                }),
                global,
            );
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

fn handle_section(
    request: SectionRequest,
    global: &GlobalOptions,
) -> Result<(), CliError> {
    match request {
        SectionRequest::List(request) => {
            let planned = vec![PlannedAction {
                action: "list sections",
                risk: "none",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("section list", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let sections =
                canvas_core::list_sections(&config, request.course_id)?;
            emit_result(
                "section list",
                json!({
                    "status": "ok",
                    "course_id": request.course_id.get(),
                    "sections": sections
                        .iter()
                        .map(section_summary_json)
                        .collect::<Vec<_>>(),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("Found {} sections.", sections.len());
            }
        }
        SectionRequest::Create(request) => {
            let planned = vec![PlannedAction {
                action: "create section",
                risk: "creates remote section",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("section create", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let section = canvas_core::create_section(
                &config,
                request.course_id,
                &request.input,
            )?;
            emit_result(
                "section create",
                json!({
                    "status": "created",
                    "course_id": request.course_id.get(),
                    "section": section_summary_json(&section),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("Section created.");
            }
        }
        SectionRequest::Update(request) => {
            let planned = vec![PlannedAction {
                action: "update section",
                risk: "updates remote section",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("section update", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let section = canvas_core::update_section(
                &config,
                request.section_id,
                &request.input,
            )?;
            emit_result(
                "section update",
                json!({
                    "status": "updated",
                    "section": section_summary_json(&section),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("Section updated.");
            }
        }
        SectionRequest::Delete(request) => {
            let planned = vec![PlannedAction {
                action: "delete section",
                risk: "deletes remote section",
                requires_confirmation: true,
            }];
            if global.explain {
                emit_plan("section delete", &planned, global);
                return Ok(());
            }
            require_confirmation(global, "section delete")?;
            let config = canvas_core::load_merged_config()?;
            let section =
                canvas_core::delete_section(&config, request.section_id)?;
            emit_result(
                "section delete",
                json!({
                    "status": "deleted",
                    "section": section_summary_json(&section),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("Section deleted.");
            }
        }
    }
    Ok(())
}

fn handle_enrollment(
    request: EnrollmentRequest,
    global: &GlobalOptions,
) -> Result<(), CliError> {
    match request {
        EnrollmentRequest::List(request) => {
            let planned = vec![PlannedAction {
                action: "list enrollments",
                risk: "none",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("enrollment list", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let enrollments = canvas_core::list_enrollments(
                &config,
                request.section_id,
                request.role,
            )?;
            emit_result(
                "enrollment list",
                json!({
                    "status": "ok",
                    "section_id": request.section_id.get(),
                    "enrollments": enrollments
                        .iter()
                        .map(enrollment_summary_json)
                        .collect::<Vec<_>>(),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("Found {} enrollments.", enrollments.len());
            }
        }
        EnrollmentRequest::Add(request) => {
            let planned = vec![PlannedAction {
                action: "add enrollment",
                risk: "creates remote enrollment",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("enrollment add", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let enrollment = canvas_core::add_enrollment(
                &config,
                request.section_id,
                &request.input,
            )?;
            emit_result(
                "enrollment add",
                json!({
                    "status": "created",
                    "section_id": request.section_id.get(),
                    "enrollment": enrollment_summary_json(&enrollment),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("Enrollment added.");
            }
        }
        EnrollmentRequest::Remove(request) => {
            let planned = vec![PlannedAction {
                action: "remove enrollment",
                risk: "deletes remote enrollment",
                requires_confirmation: true,
            }];
            if global.explain {
                emit_plan("enrollment remove", &planned, global);
                return Ok(());
            }
            require_confirmation(global, "enrollment remove")?;
            let config = canvas_core::load_merged_config()?;
            let enrollment = canvas_core::remove_enrollment(
                &config,
                request.section_id,
                request.enrollment_id,
            )?;
            emit_result(
                "enrollment remove",
                json!({
                    "status": "deleted",
                    "section_id": request.section_id.get(),
                    "enrollment": enrollment_summary_json(&enrollment),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("Enrollment removed.");
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
        ReportRequest::GradeChangeLog(request) => {
            handle_report_export("report grade-change-log", request, global)
        }
    }
}

fn handle_gradebook(
    request: GradebookRequest,
    global: &GlobalOptions,
) -> Result<(), CliError> {
    match request {
        GradebookRequest::GradingPeriodList(request) => {
            let planned = vec![PlannedAction {
                action: "list grading periods",
                risk: "none",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("gradebook grading-period list", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let periods =
                canvas_core::list_grading_periods(&config, request.course_id)?;
            emit_result(
                "gradebook grading-period list",
                json!({
                    "status": "ok",
                    "course_id": request.course_id.get(),
                    "grading_periods": periods
                        .iter()
                        .map(grading_period_summary_json)
                        .collect::<Vec<_>>(),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("Found {} grading periods.", periods.len());
            }
            Ok(())
        }
        GradebookRequest::GradingPeriodShow(request) => {
            let planned = vec![PlannedAction {
                action: "show grading period",
                risk: "none",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("gradebook grading-period show", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let period = canvas_core::get_grading_period(
                &config,
                request.course_id,
                request.grading_period_id,
            )?;
            emit_result(
                "gradebook grading-period show",
                json!({
                    "status": "ok",
                    "course_id": request.course_id.get(),
                    "grading_period": grading_period_summary_json(&period),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("Grading period {} retrieved.", period.id);
            }
            Ok(())
        }
        GradebookRequest::PostingPolicyGet(request) => {
            let planned = vec![PlannedAction {
                action: "get posting policy",
                risk: "none",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("gradebook posting-policy get", &planned, global);
                return Ok(());
            }
            let config = canvas_core::load_merged_config()?;
            let policy =
                canvas_core::get_posting_policy(&config, request.course_id)?;
            emit_result(
                "gradebook posting-policy get",
                json!({
                    "status": "ok",
                    "course_id": request.course_id.get(),
                    "posting_policy": posting_policy_summary_json(&policy),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!("Posting policy: {}.", policy.policy.as_str());
            }
            Ok(())
        }
        GradebookRequest::PostingPolicySet(request) => {
            let planned = vec![PlannedAction {
                action: "update posting policy",
                risk: "updates gradebook posting policy",
                requires_confirmation: true,
            }];
            if global.explain {
                emit_plan("gradebook posting-policy set", &planned, global);
                return Ok(());
            }
            require_confirmation(global, "gradebook posting-policy set")?;
            let config = canvas_core::load_merged_config()?;
            let policy = canvas_core::update_posting_policy(
                &config,
                request.course_id,
                request.policy,
            )?;
            emit_result(
                "gradebook posting-policy set",
                json!({
                    "status": "updated",
                    "course_id": request.course_id.get(),
                    "posting_policy": posting_policy_summary_json(&policy),
                }),
                global,
            );
            if !global.quiet && !global.json {
                println!(
                    "Posting policy updated to {}.",
                    policy.policy.as_str()
                );
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

fn parse_calendar_timing_for_create(
    start_at: Option<String>,
    end_at: Option<String>,
    all_day_date: Option<String>,
) -> Result<canvas_core::CalendarEventTiming, CliError> {
    if all_day_date.is_some() && (start_at.is_some() || end_at.is_some()) {
        return Err(CliError::Canvas(
            canvas_core::CanvasError::InvalidCalendarEventCreate(
                "conflicting_timing".to_string(),
            ),
        ));
    }
    if let Some(date) = all_day_date {
        let date = canvas_core::parse_all_day_date(&date)?;
        return Ok(canvas_core::CalendarEventTiming::AllDay(date));
    }
    let start_at = match start_at {
        Some(value) => value,
        None => {
            return Err(CliError::Canvas(
                canvas_core::CanvasError::InvalidCalendarEventCreate(
                    "missing_start".to_string(),
                ),
            ));
        }
    };
    let start_at = canvas_core::parse_event_date_time(&start_at)?;
    let end_at = match end_at {
        Some(value) => Some(canvas_core::parse_event_date_time(&value)?),
        None => None,
    };
    Ok(canvas_core::CalendarEventTiming::Timed { start_at, end_at })
}

fn parse_calendar_timing_for_update(
    start_at: Option<String>,
    end_at: Option<String>,
    all_day_date: Option<String>,
) -> Result<Option<canvas_core::CalendarEventTiming>, CliError> {
    if all_day_date.is_some() && (start_at.is_some() || end_at.is_some()) {
        return Err(CliError::Canvas(
            canvas_core::CanvasError::InvalidCalendarEventUpdate(
                "conflicting_timing".to_string(),
            ),
        ));
    }
    if let Some(date) = all_day_date {
        let date = canvas_core::parse_all_day_date(&date)?;
        return Ok(Some(canvas_core::CalendarEventTiming::AllDay(date)));
    }
    if start_at.is_none() && end_at.is_none() {
        return Ok(None);
    }
    let start_at = match start_at {
        Some(value) => canvas_core::parse_event_date_time(&value)?,
        None => {
            return Err(CliError::Canvas(
                canvas_core::CanvasError::InvalidCalendarEventUpdate(
                    "missing_start".to_string(),
                ),
            ));
        }
    };
    let end_at = match end_at {
        Some(value) => Some(canvas_core::parse_event_date_time(&value)?),
        None => None,
    };
    Ok(Some(canvas_core::CalendarEventTiming::Timed { start_at, end_at }))
}

fn parse_question_bank_create_input(
    title: Option<String>,
    bank_json: Option<String>,
    bank_file: Option<PathBuf>,
) -> Result<canvas_core::QuestionBankCreateInput, CliError> {
    let sources = [
        title.is_some(),
        bank_json.is_some(),
        bank_file.is_some(),
    ]
    .into_iter()
    .filter(|value| *value)
    .count();
    if sources > 1 {
        return Err(CliError::Canvas(
            canvas_core::CanvasError::InvalidQuestionBankJson(
                "multiple_sources".to_string(),
            ),
        ));
    }
    if let Some(raw) = bank_json {
        return canvas_core::QuestionBankCreateInput::from_json(&raw)
            .map_err(CliError::Canvas);
    }
    if let Some(path) = bank_file {
        let raw = read_file_to_string(&path)?;
        return canvas_core::QuestionBankCreateInput::from_json(&raw)
            .map_err(CliError::Canvas);
    }
    let title = title.ok_or_else(|| {
        CliError::Canvas(canvas_core::CanvasError::InvalidQuestionBankTitle(
            "missing".to_string(),
        ))
    })?;
    let title = canvas_core::parse_question_bank_title(&title)?;
    Ok(canvas_core::QuestionBankCreateInput::new(title))
}

fn parse_question_bank_update_input(
    title: Option<String>,
    bank_json: Option<String>,
    bank_file: Option<PathBuf>,
) -> Result<canvas_core::QuestionBankUpdateInput, CliError> {
    let sources = [
        title.is_some(),
        bank_json.is_some(),
        bank_file.is_some(),
    ]
    .into_iter()
    .filter(|value| *value)
    .count();
    if sources > 1 {
        return Err(CliError::Canvas(
            canvas_core::CanvasError::InvalidQuestionBankJson(
                "multiple_sources".to_string(),
            ),
        ));
    }
    if let Some(raw) = bank_json {
        return canvas_core::QuestionBankUpdateInput::from_json(&raw)
            .map_err(CliError::Canvas);
    }
    if let Some(path) = bank_file {
        let raw = read_file_to_string(&path)?;
        return canvas_core::QuestionBankUpdateInput::from_json(&raw)
            .map_err(CliError::Canvas);
    }
    let title = match title {
        Some(raw) => Some(canvas_core::parse_question_bank_title(&raw)?),
        None => None,
    };
    canvas_core::QuestionBankUpdateInput::new(title).map_err(CliError::Canvas)
}

fn parse_question_create_input(
    name: Option<String>,
    text: Option<String>,
    question_type: Option<String>,
    points: Option<String>,
    correct_comments: Option<String>,
    incorrect_comments: Option<String>,
    neutral_comments: Option<String>,
    question_json: Option<String>,
    question_file: Option<PathBuf>,
) -> Result<canvas_core::QuestionCreateInput, CliError> {
    let json_sources = [question_json.is_some(), question_file.is_some()]
        .into_iter()
        .filter(|value| *value)
        .count();
    let has_fields = name.is_some()
        || text.is_some()
        || question_type.is_some()
        || points.is_some()
        || correct_comments.is_some()
        || incorrect_comments.is_some()
        || neutral_comments.is_some();
    if json_sources > 1 || (json_sources == 1 && has_fields) {
        return Err(CliError::Canvas(
            canvas_core::CanvasError::InvalidQuestionJson(
                "multiple_sources".to_string(),
            ),
        ));
    }
    if let Some(raw) = question_json {
        return canvas_core::QuestionCreateInput::from_json(&raw)
            .map_err(CliError::Canvas);
    }
    if let Some(path) = question_file {
        let raw = read_file_to_string(&path)?;
        return canvas_core::QuestionCreateInput::from_json(&raw)
            .map_err(CliError::Canvas);
    }
    let text = text.ok_or_else(|| {
        CliError::Canvas(canvas_core::CanvasError::InvalidQuestionText(
            "missing".to_string(),
        ))
    })?;
    let question_type = question_type.ok_or_else(|| {
        CliError::Canvas(canvas_core::CanvasError::InvalidQuestionType(
            "missing".to_string(),
        ))
    })?;
    let question_name = match name {
        Some(raw) => Some(canvas_core::parse_question_name(&raw)?),
        None => None,
    };
    let question_text = canvas_core::parse_question_text(&text)?;
    let question_type = canvas_core::parse_question_type(&question_type)?;
    let points_possible = match points {
        Some(raw) => Some(canvas_core::parse_points_possible(&raw)?),
        None => None,
    };
    Ok(canvas_core::QuestionCreateInput::new(
        question_name,
        question_text,
        question_type,
        points_possible,
        correct_comments,
        incorrect_comments,
        neutral_comments,
        None,
    ))
}

fn parse_question_update_input(
    name: Option<String>,
    text: Option<String>,
    question_type: Option<String>,
    points: Option<String>,
    correct_comments: Option<String>,
    incorrect_comments: Option<String>,
    neutral_comments: Option<String>,
    question_json: Option<String>,
    question_file: Option<PathBuf>,
) -> Result<canvas_core::QuestionUpdateInput, CliError> {
    let json_sources = [question_json.is_some(), question_file.is_some()]
        .into_iter()
        .filter(|value| *value)
        .count();
    let has_fields = name.is_some()
        || text.is_some()
        || question_type.is_some()
        || points.is_some()
        || correct_comments.is_some()
        || incorrect_comments.is_some()
        || neutral_comments.is_some();
    if json_sources > 1 || (json_sources == 1 && has_fields) {
        return Err(CliError::Canvas(
            canvas_core::CanvasError::InvalidQuestionJson(
                "multiple_sources".to_string(),
            ),
        ));
    }
    if let Some(raw) = question_json {
        return canvas_core::QuestionUpdateInput::from_json(&raw)
            .map_err(CliError::Canvas);
    }
    if let Some(path) = question_file {
        let raw = read_file_to_string(&path)?;
        return canvas_core::QuestionUpdateInput::from_json(&raw)
            .map_err(CliError::Canvas);
    }
    let question_name = match name {
        Some(raw) => Some(canvas_core::parse_question_name(&raw)?),
        None => None,
    };
    let question_text = match text {
        Some(raw) => Some(canvas_core::parse_question_text(&raw)?),
        None => None,
    };
    let question_type = match question_type {
        Some(raw) => Some(canvas_core::parse_question_type(&raw)?),
        None => None,
    };
    let points_possible = match points {
        Some(raw) => Some(canvas_core::parse_points_possible(&raw)?),
        None => None,
    };
    canvas_core::QuestionUpdateInput::new(
        question_name,
        question_text,
        question_type,
        points_possible,
        correct_comments,
        incorrect_comments,
        neutral_comments,
        None,
    )
    .map_err(CliError::Canvas)
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

fn parse_rubric_criteria_input(
    criteria_json: Option<String>,
    criteria_file: Option<PathBuf>,
) -> Result<Option<canvas_core::RubricCriteria>, CliError> {
    if criteria_json.is_some() && criteria_file.is_some() {
        return Err(CliError::Canvas(
            canvas_core::CanvasError::InvalidRubricCriteria(
                "multiple_sources".to_string(),
            ),
        ));
    }
    let raw = if let Some(raw) = criteria_json {
        raw
    } else if let Some(path) = criteria_file {
        read_file_to_string(&path)?
    } else {
        return Ok(None);
    };
    let parsed = canvas_core::parse_rubric_criteria(&raw)?;
    Ok(Some(parsed))
}

fn parse_assignment_overrides_input(
    overrides_json: Option<String>,
    overrides_file: Option<PathBuf>,
) -> Result<Option<canvas_models::AssignmentOverrides>, CliError> {
    if overrides_json.is_some() && overrides_file.is_some() {
        return Err(CliError::Canvas(
            canvas_core::CanvasError::InvalidAssignmentOverrides(
                "multiple_sources".to_string(),
            ),
        ));
    }
    let raw = if let Some(raw) = overrides_json {
        raw
    } else if let Some(path) = overrides_file {
        read_file_to_string(&path)?
    } else {
        return Ok(None);
    };
    let parsed = canvas_core::parse_assignment_overrides(&raw)?;
    Ok(Some(parsed))
}

fn parse_module_requirements_input(
    requirements_json: Option<String>,
    requirements_file: Option<PathBuf>,
) -> Result<Option<canvas_models::ModuleRequirements>, CliError> {
    if requirements_json.is_some() && requirements_file.is_some() {
        return Err(CliError::Canvas(
            canvas_core::CanvasError::InvalidModuleRequirement(
                "multiple_sources".to_string(),
            ),
        ));
    }
    let raw = if let Some(raw) = requirements_json {
        raw
    } else if let Some(path) = requirements_file {
        read_file_to_string(&path)?
    } else {
        return Ok(None);
    };
    let parsed = canvas_core::parse_module_requirements(&raw)?;
    Ok(Some(parsed))
}

fn parse_module_prerequisites_input(
    prerequisites_json: Option<String>,
    prerequisites_file: Option<PathBuf>,
) -> Result<Option<canvas_models::ModulePrerequisites>, CliError> {
    if prerequisites_json.is_some() && prerequisites_file.is_some() {
        return Err(CliError::Canvas(
            canvas_core::CanvasError::InvalidModulePrerequisites(
                "multiple_sources".to_string(),
            ),
        ));
    }
    let raw = if let Some(raw) = prerequisites_json {
        raw
    } else if let Some(path) = prerequisites_file {
        read_file_to_string(&path)?
    } else {
        return Ok(None);
    };
    let parsed = canvas_core::parse_module_prerequisites(&raw)?;
    Ok(Some(parsed))
}

fn parse_peer_review_settings_input(
    mode: Option<String>,
    assign_at: Option<String>,
    due_at: Option<String>,
) -> Result<Option<canvas_models::PeerReviewSettings>, CliError> {
    if mode.is_none() && assign_at.is_none() && due_at.is_none() {
        return Ok(None);
    }
    let mode = mode.ok_or_else(|| {
        CliError::Canvas(canvas_core::CanvasError::InvalidPeerReviewSettings(
            "missing_mode".to_string(),
        ))
    })?;
    let mode = canvas_core::parse_peer_review_mode(&mode)?;
    let assign_at = match assign_at {
        Some(raw) => Some(canvas_core::parse_due_date(&raw)?),
        None => None,
    };
    let due_at = match due_at {
        Some(raw) => Some(canvas_core::parse_due_date(&raw)?),
        None => None,
    };
    let parsed =
        canvas_core::parse_peer_review_settings(mode, assign_at, due_at)?;
    Ok(Some(parsed))
}

fn parse_group_assignment_settings_input(
    mode: Option<String>,
    category_id: Option<String>,
) -> Result<Option<canvas_models::GroupAssignmentSettings>, CliError> {
    if mode.is_none() && category_id.is_none() {
        return Ok(None);
    }
    let mode = mode.ok_or_else(|| {
        CliError::Canvas(canvas_core::CanvasError::InvalidGroupAssignmentSettings(
            "missing_mode".to_string(),
        ))
    })?;
    let mode = canvas_core::parse_group_assignment_mode(&mode)?;
    let category_id = match category_id {
        Some(raw) => Some(canvas_core::parse_group_category_id(&raw)?),
        None => None,
    };
    let parsed = canvas_core::parse_group_assignment_settings(mode, category_id)?;
    Ok(Some(parsed))
}

fn parse_external_tool_config_input(
    config_url: Option<String>,
    config_json: Option<String>,
    config_xml: Option<String>,
    config_file: Option<PathBuf>,
    required: bool,
) -> Result<Option<canvas_core::ExternalToolConfig>, CliError> {
    let sources = [
        config_url.is_some(),
        config_json.is_some(),
        config_xml.is_some(),
        config_file.is_some(),
    ]
    .into_iter()
    .filter(|value| *value)
    .count();
    if sources > 1 {
        return Err(CliError::Canvas(
            canvas_core::CanvasError::InvalidExternalToolConfig(
                "multiple_sources".to_string(),
            ),
        ));
    }
    if sources == 0 {
        if required {
            return Err(CliError::Canvas(
                canvas_core::CanvasError::InvalidExternalToolConfig(
                    "missing_config".to_string(),
                ),
            ));
        }
        return Ok(None);
    }
    if let Some(url) = config_url {
        let parsed = canvas_core::parse_external_tool_config_url(&url)?;
        return Ok(Some(canvas_core::ExternalToolConfig::Url(parsed)));
    }
    if let Some(raw) = config_json {
        let parsed = canvas_core::parse_external_tool_config_json(&raw)?;
        return Ok(Some(canvas_core::ExternalToolConfig::Json(parsed)));
    }
    if let Some(raw) = config_xml {
        let parsed = canvas_core::parse_external_tool_config_xml(raw)?;
        return Ok(Some(canvas_core::ExternalToolConfig::Xml(parsed)));
    }
    let path = config_file.expect("config file path");
    let raw = read_file_to_string(&path)?;
    let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
    if extension.eq_ignore_ascii_case("json") {
        let parsed = canvas_core::parse_external_tool_config_json(&raw)?;
        return Ok(Some(canvas_core::ExternalToolConfig::Json(parsed)));
    }
    if extension.eq_ignore_ascii_case("xml") {
        let parsed = canvas_core::parse_external_tool_config_xml(raw)?;
        return Ok(Some(canvas_core::ExternalToolConfig::Xml(parsed)));
    }
    if let Ok(parsed) = canvas_core::parse_external_tool_config_json(&raw) {
        return Ok(Some(canvas_core::ExternalToolConfig::Json(parsed)));
    }
    let parsed = canvas_core::parse_external_tool_config_xml(raw)?;
    Ok(Some(canvas_core::ExternalToolConfig::Xml(parsed)))
}

fn parse_external_tool_placements_input(
    placements: Vec<String>,
    placements_json: Option<String>,
    placements_file: Option<PathBuf>,
) -> Result<
    (
        Option<canvas_core::ExternalToolPlacementList>,
        Option<canvas_core::ExternalToolPlacementSettings>,
    ),
    CliError,
> {
    let placements = if placements.is_empty() {
        None
    } else {
        let parsed = placements
            .into_iter()
            .map(|raw| canvas_core::parse_external_tool_placement(&raw))
            .collect::<Result<Vec<_>, _>>()?;
        Some(canvas_core::parse_external_tool_placement_list(parsed)?)
    };
    let settings = match (placements_json, placements_file) {
        (Some(_), Some(_)) => {
            return Err(CliError::Canvas(
                canvas_core::CanvasError::InvalidExternalToolPlacements(
                    "multiple_sources".to_string(),
                ),
            ));
        }
        (Some(raw), None) => {
            Some(canvas_core::parse_external_tool_placement_settings(&raw)?)
        }
        (None, Some(path)) => {
            let raw = read_file_to_string(&path)?;
            Some(canvas_core::parse_external_tool_placement_settings(&raw)?)
        }
        (None, None) => None,
    };
    Ok((placements, settings))
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
        "override_count": assignment.override_count,
        "peer_reviews": assignment.peer_reviews.as_ref().map(|settings| {
            json!({
                "mode": settings.mode.as_str(),
                "assign_at": settings.assign_at,
                "due_at": settings.due_at,
            })
        }),
        "group_settings": assignment.group_assignment.as_ref().map(|settings| {
            json!({
                "mode": settings.mode.as_str(),
                "category_id": settings.category_id,
                "grade_individually": settings.grade_individually,
            })
        }),
        "grading_posting_policy": assignment
            .grading_posting_policy
            .map(|policy| policy.as_str()),
        "muted_state": assignment.muted.map(|state| state.as_str()),
    })
}

fn outcome_summary_json(outcome: &canvas_core::OutcomeSummary) -> Value {
    json!({
        "id": outcome.id,
        "title": outcome.title,
        "description": outcome.description,
        "points_possible": outcome.points_possible,
        "mastery_points": outcome.mastery_points,
        "ratings": outcome
            .ratings
            .iter()
            .map(outcome_rating_summary_json)
            .collect::<Vec<_>>(),
    })
}

fn outcome_rating_summary_json(
    rating: &canvas_core::OutcomeRatingSummary,
) -> Value {
    json!({
        "description": rating.description,
        "points": rating.points,
    })
}

fn calendar_event_summary_json(
    event: &canvas_core::CalendarEventSummary,
) -> Value {
    json!({
        "id": event.id,
        "title": event.title,
        "start_at": event.start_at,
        "end_at": event.end_at,
        "all_day": event.all_day,
        "all_day_date": event.all_day_date,
        "context_type": event.context_type,
        "context_id": event.context_id,
        "context_code": event.context_code,
    })
}

fn conference_summary_json(
    conference: &canvas_core::ConferenceSummary,
) -> Value {
    json!({
        "id": conference.id,
        "title": conference.title,
        "start_at": conference.start_at,
        "duration": conference.duration,
        "recording_enabled": conference.recording_enabled,
    })
}

fn collaboration_summary_json(
    collaboration: &canvas_core::CollaborationSummary,
) -> Value {
    json!({
        "id": collaboration.id,
        "collaboration_type": collaboration.collaboration_type,
        "document_id": collaboration.document_id,
        "document_url": collaboration.document_url,
        "user_id": collaboration.user_id,
        "context_id": collaboration.context_id,
        "context_type": collaboration.context_type,
        "created_at": collaboration.created_at,
        "updated_at": collaboration.updated_at,
        "description": collaboration.description,
        "title": collaboration.title,
        "update_url": collaboration.update_url,
        "user_name": collaboration.user_name,
        "collaborator_ids": collaboration.collaborator_ids,
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

fn rubric_summary_json(rubric: &canvas_core::RubricSummary) -> Value {
    json!({
        "id": rubric.id,
        "title": rubric.title,
        "points_possible": rubric.points_possible,
        "criteria": rubric
            .criteria
            .iter()
            .map(rubric_criterion_summary_json)
            .collect::<Vec<_>>(),
    })
}

fn rubric_criterion_summary_json(
    criterion: &canvas_core::RubricCriterionSummary,
) -> Value {
    json!({
        "id": criterion.id,
        "description": criterion.description,
        "points": criterion.points,
    })
}

fn rubric_association_summary_json(
    association: &canvas_core::RubricAssociationSummary,
) -> Value {
    json!({
        "id": association.id,
        "rubric_id": association.rubric_id,
        "association_id": association.association_id,
        "association_type": association.association_type,
        "use_for_grading": association.use_for_grading,
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

fn question_bank_summary_json(bank: &canvas_core::QuestionBankSummary) -> Value {
    json!({
        "id": bank.id,
        "title": bank.title,
        "question_count": bank.question_count,
        "context_type": bank.context_type,
        "context_id": bank.context_id,
        "created_at": bank.created_at,
        "updated_at": bank.updated_at,
    })
}

fn question_answer_summary_json(answer: &canvas_core::QuestionAnswerSummary) -> Value {
    json!({
        "id": answer.id,
        "text": answer.text,
        "weight": answer.weight,
        "comments": answer.comments,
        "html": answer.html,
    })
}

fn question_summary_json(question: &canvas_core::QuestionSummary) -> Value {
    json!({
        "id": question.id,
        "question_name": question.question_name,
        "question_text": question.question_text,
        "question_type": question.question_type,
        "points_possible": question.points_possible,
        "position": question.position,
        "correct_comments": question.correct_comments,
        "incorrect_comments": question.incorrect_comments,
        "neutral_comments": question.neutral_comments,
        "answers": question.answers.iter().map(question_answer_summary_json).collect::<Vec<_>>(),
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
        "requirements": module
            .completion_requirements
            .as_ref()
            .map(|requirements| {
                requirements
                    .iter()
                    .map(module_requirement_summary_json)
                    .collect::<Vec<_>>()
            }),
        "prerequisites": module
            .prerequisites
            .as_ref()
            .map(|prerequisites| {
                prerequisites
                    .iter()
                    .map(module_prerequisite_summary_json)
                    .collect::<Vec<_>>()
            }),
        "unlock_at": module.unlock_at,
        "require_sequential_progress": module.require_sequential_progress,
    })
}

fn module_requirement_summary_json(
    requirement: &canvas_core::ModuleRequirementSummary,
) -> Value {
    json!({
        "item_id": requirement.item_id,
        "type": requirement.requirement_type,
        "min_score": requirement.min_score,
    })
}

fn module_prerequisite_summary_json(
    prerequisite: &canvas_core::ModulePrerequisiteSummary,
) -> Value {
    json!({
        "id": prerequisite.id,
        "name": prerequisite.name,
        "type": prerequisite.prerequisite_type,
    })
}

fn external_tool_summary_json(tool: &canvas_core::ExternalToolSummary) -> Value {
    json!({
        "id": tool.id,
        "name": tool.name,
        "placements": tool.placements,
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

fn content_migration_progress_json(
    progress: &canvas_core::ContentMigrationProgress,
) -> Value {
    json!({
        "completion": progress.completion,
        "workflow_state": progress.workflow_state,
        "message": progress.message,
    })
}

fn content_migration_summary_json(
    migration: &canvas_core::ContentMigrationSummary,
    progress: Option<&canvas_core::ContentMigrationProgress>,
) -> Value {
    json!({
        "migration_id": migration.id,
        "migration_type": migration
            .migration_type
            .as_ref()
            .map(|kind| kind.as_str()),
        "migration_type_title": migration.migration_type_title,
        "workflow_state": migration.workflow_state.as_str(),
        "progress_url": migration.progress_url,
        "started_at": migration.started_at,
        "finished_at": migration.finished_at,
        "user_id": migration.user_id,
        "progress": progress.map(content_migration_progress_json),
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

fn section_summary_json(section: &canvas_core::SectionSummary) -> Value {
    json!({
        "id": section.id,
        "course_id": section.course_id,
        "name": section.name,
        "enrollment_count": section.enrollment_count,
    })
}

fn enrollment_summary_json(enrollment: &canvas_core::EnrollmentSummary) -> Value {
    json!({
        "id": enrollment.id,
        "user_id": enrollment.user_id,
        "section_id": enrollment.section_id,
        "role": enrollment.role,
        "state": enrollment.enrollment_state,
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

fn grading_period_summary_json(
    period: &canvas_core::GradingPeriodSummary,
) -> Value {
    json!({
        "id": period.id,
        "title": period.title,
        "start_date": period.start_date,
        "end_date": period.end_date,
        "close_date": period.close_date,
        "weight": period.weight,
        "is_closed": period.is_closed,
    })
}

fn posting_policy_summary_json(
    policy: &canvas_core::PostingPolicySummary,
) -> Value {
    json!({
        "state": policy.policy.as_str(),
        "scope": posting_policy_scope_json(policy.scope),
    })
}

fn posting_policy_scope_json(scope: canvas_core::PostingPolicyScope) -> Value {
    json!({
        "type": scope.as_str(),
        "course_id": scope.course_id().get(),
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
            "outcome_list": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "outcome_mutation": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "rubric_list": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "rubric_mutation": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "rubric_association": {
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
            "question_bank_list": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "question_bank_mutation": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "question_list": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "question_mutation": {
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
            "module_requirement_list": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "module_requirement_update": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "tool_list": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "tool_mutation": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "calendar_list": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "calendar_mutation": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],        
            },
            "conference_list": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "conference_mutation": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "collaboration_list": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "collaboration_mutation": {
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
            "content_migration_list": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "content_migration_mutation": {
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
            "section_list": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "section_create": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "section_update": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "section_delete": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "enrollment_list": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "enrollment_add": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "enrollment_remove": {
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
            "gradebook_grading_period_list": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "gradebook_grading_period_show": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "gradebook_posting_policy_get": {
                "type": "object",
                "properties": {
                    "ok": { "type": "boolean" },
                    "schema_version": { "type": "string" },
                    "command": { "type": "string" },
                    "data": { "type": "object" },
                },
                "required": ["ok", "schema_version", "command", "data"],
            },
            "gradebook_posting_policy_set": {
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
            canvas_core::CanvasError::InvalidGradingPeriodId(_) => {
                "invalid_grading_period_id"
            }
            canvas_core::CanvasError::InvalidAssignmentId(_) => "invalid_assignment_id",
            canvas_core::CanvasError::InvalidAssignmentName(_) => {
                "invalid_assignment_name"
            }
            canvas_core::CanvasError::InvalidGroupCategoryId(_) => {
                "invalid_group_category_id"
            }
            canvas_core::CanvasError::InvalidGroupAssignmentMode(_) => {
                "invalid_group_assignment_mode"
            }
            canvas_core::CanvasError::InvalidGroupAssignmentSettings(_) => {
                "invalid_group_assignment_settings"
            }
            canvas_core::CanvasError::InvalidPeerReviewMode(_) => {
                "invalid_peer_review_mode"
            }
            canvas_core::CanvasError::InvalidPeerReviewSettings(_) => {
                "invalid_peer_review_settings"
            }
            canvas_core::CanvasError::InvalidAssignmentOverrides(_) => {
                "invalid_assignment_overrides"
            }
            canvas_core::CanvasError::InvalidAssignmentOverrideTarget(_) => {
                "invalid_assignment_override_target"
            }
            canvas_core::CanvasError::InvalidAssignmentOverrideDates(_) => {
                "invalid_assignment_override_dates"
            }
            canvas_core::CanvasError::InvalidGradingPostingPolicy(_) => {
                "invalid_grading_posting_policy"
            }
            canvas_core::CanvasError::InvalidMutedState(_) => {
                "invalid_muted_state"
            }
            canvas_core::CanvasError::InvalidOutcomeId(_) => "invalid_outcome_id",
            canvas_core::CanvasError::InvalidOutcomeGroupId(_) => {
                "invalid_outcome_group_id"
            }
            canvas_core::CanvasError::InvalidOutcomeTitle(_) => "invalid_outcome_title",
            canvas_core::CanvasError::InvalidOutcomeDescription(_) => {
                "invalid_outcome_description"
            }
            canvas_core::CanvasError::InvalidRubricId(_) => "invalid_rubric_id",
            canvas_core::CanvasError::InvalidRubricAssociationId(_) => {
                "invalid_rubric_association_id"
            }
            canvas_core::CanvasError::InvalidRubricTitle(_) => "invalid_rubric_title",
            canvas_core::CanvasError::InvalidQuizId(_) => "invalid_quiz_id",
            canvas_core::CanvasError::InvalidQuizSubmissionId(_) => {
                "invalid_quiz_submission_id"
            }
            canvas_core::CanvasError::InvalidQuizTitle(_) => "invalid_quiz_title",
            canvas_core::CanvasError::InvalidQuestionBankId(_) => {
                "invalid_question_bank_id"
            }
            canvas_core::CanvasError::InvalidQuestionId(_) => "invalid_question_id",
            canvas_core::CanvasError::InvalidQuestionBankTitle(_) => {
                "invalid_question_bank_title"
            }
            canvas_core::CanvasError::InvalidQuestionName(_) => "invalid_question_name",
            canvas_core::CanvasError::InvalidQuestionText(_) => "invalid_question_text",
            canvas_core::CanvasError::InvalidQuestionType(_) => "invalid_question_type",
            canvas_core::CanvasError::InvalidQuestionBankUpdate(_) => {
                "invalid_question_bank_update"
            }
            canvas_core::CanvasError::InvalidQuestionUpdate(_) => {
                "invalid_question_update"
            }
            canvas_core::CanvasError::InvalidQuestionBankJson(_) => {
                "invalid_question_bank_json"
            }
            canvas_core::CanvasError::InvalidQuestionJson(_) => "invalid_question_json",
            canvas_core::CanvasError::InvalidCalendarEventId(_) => {
                "invalid_calendar_event_id"
            }
            canvas_core::CanvasError::InvalidConferenceId(_) => {
                "invalid_conference_id"
            }
            canvas_core::CanvasError::InvalidCollaborationId(_) => {
                "invalid_collaboration_id"
            }
            canvas_core::CanvasError::InvalidSectionId(_) => "invalid_section_id",
            canvas_core::CanvasError::InvalidSectionName(_) => {
                "invalid_section_name"
            }
            canvas_core::CanvasError::InvalidEnrollmentId(_) => {
                "invalid_enrollment_id"
            }
            canvas_core::CanvasError::InvalidExternalToolId(_) => "invalid_external_tool_id",
            canvas_core::CanvasError::InvalidExternalToolName(_) => {
                "invalid_external_tool_name"
            }
            canvas_core::CanvasError::InvalidExternalToolConfig(_) => {
                "invalid_external_tool_config"
            }
            canvas_core::CanvasError::InvalidExternalToolPlacement(_) => {
                "invalid_external_tool_placement"
            }
            canvas_core::CanvasError::InvalidExternalToolPlacements(_) => {
                "invalid_external_tool_placements"
            }
            canvas_core::CanvasError::InvalidExternalToolUpdate(_) => {
                "invalid_external_tool_update"
            }
            canvas_core::CanvasError::InvalidCalendarEventTitle(_) => {
                "invalid_calendar_event_title"
            }
            canvas_core::CanvasError::InvalidConferenceTitle(_) => {
                "invalid_conference_title"
            }
            canvas_core::CanvasError::InvalidCollaborationTitle(_) => {
                "invalid_collaboration_title"
            }
            canvas_core::CanvasError::InvalidCollaborationType(_) => {
                "invalid_collaboration_type"
            }
            canvas_core::CanvasError::InvalidCollaborators(_) => {
                "invalid_collaborators"
            }
            canvas_core::CanvasError::InvalidConferenceDescription(_) => {
                "invalid_conference_description"
            }
            canvas_core::CanvasError::InvalidConferenceDuration(_) => {
                "invalid_conference_duration"
            }
            canvas_core::CanvasError::InvalidUserId(_) => "invalid_user_id",
            canvas_core::CanvasError::InvalidSubmissionId(_) => "invalid_submission_id",
            canvas_core::CanvasError::InvalidPageId(_) => "invalid_page_id",
            canvas_core::CanvasError::InvalidPageTitle(_) => "invalid_page_title",
            canvas_core::CanvasError::InvalidPageBody(_) => "invalid_page_body",
            canvas_core::CanvasError::InvalidModuleId(_) => "invalid_module_id",
            canvas_core::CanvasError::InvalidModuleItemId(_) => {
                "invalid_module_item_id"
            }
            canvas_core::CanvasError::InvalidModuleName(_) => "invalid_module_name",
            canvas_core::CanvasError::InvalidModuleRequirementType(_) => {
                "invalid_module_requirement_type"
            }
            canvas_core::CanvasError::InvalidModuleRequirement(_) => {
                "invalid_module_requirement"
            }
            canvas_core::CanvasError::InvalidModulePrerequisites(_) => {
                "invalid_module_prerequisites"
            }
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
            canvas_core::CanvasError::InvalidEventDateTime(_) => {
                "invalid_event_date_time"
            }
            canvas_core::CanvasError::InvalidAllDayDate(_) => "invalid_all_day_date",
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
            canvas_core::CanvasError::InvalidRubricCriteria(_) => {
                "invalid_rubric_criteria"
            }
            canvas_core::CanvasError::InvalidCourseVisibility(_) => "invalid_course_visibility",
            canvas_core::CanvasError::InvalidGradingSchemeId(_) => "invalid_grading_scheme_id",
            canvas_core::CanvasError::InvalidContentMigrationId(_) => {
                "invalid_content_migration_id"
            }
            canvas_core::CanvasError::InvalidContentMigrationType(_) => {
                "invalid_content_migration_type"
            }
            canvas_core::CanvasError::InvalidContentMigrationCreate(_) => {
                "invalid_content_migration_create"
            }
            canvas_core::CanvasError::InvalidReportType(_) => "invalid_report_type",
            canvas_core::CanvasError::InvalidCourseDates(_) => "invalid_course_dates",
            canvas_core::CanvasError::InvalidCourseUpdate(_) => "invalid_course_update",
            canvas_core::CanvasError::GradingPeriodNotFound(_) => {
                "grading_period_not_found"
            }
            canvas_core::CanvasError::InvalidAssignmentUpdate(_) => {
                "invalid_assignment_update"
            }
            canvas_core::CanvasError::InvalidOutcomeUpdate(_) => "invalid_outcome_update",
            canvas_core::CanvasError::InvalidQuizUpdate(_) => "invalid_quiz_update",
            canvas_core::CanvasError::InvalidPageUpdate(_) => "invalid_page_update",
            canvas_core::CanvasError::InvalidModuleUpdate(_) => "invalid_module_update",
            canvas_core::CanvasError::InvalidSectionUpdate(_) => {
                "invalid_section_update"
            }
            canvas_core::CanvasError::InvalidModuleRequirementUpdate(_) => {
                "invalid_module_requirement_update"
            }
            canvas_core::CanvasError::InvalidModuleReorder(_) => "invalid_module_reorder",
            canvas_core::CanvasError::InvalidRubricUpdate(_) => "invalid_rubric_update",
            canvas_core::CanvasError::InvalidCalendarEventCreate(_) => {
                "invalid_calendar_event_create"
            }
            canvas_core::CanvasError::MissingAssignmentPoints(_) => {
                "missing_assignment_points"
            }
            canvas_core::CanvasError::MissingQuizPoints(_) => "missing_quiz_points",
            canvas_core::CanvasError::InvalidQuizAvailability(_) => {
                "invalid_quiz_availability"
            }
            canvas_core::CanvasError::InvalidFileUpload(_) => "invalid_file_upload",
            canvas_core::CanvasError::ContentMigrationFailed(_) => {
                "content_migration_failed"
            }
            canvas_core::CanvasError::ContentMigrationTimeout(_) => {
                "content_migration_timeout"
            }
            canvas_core::CanvasError::ReportNotReady(_) => "report_not_ready",
            canvas_core::CanvasError::ReportDownloadFailed(_) => "report_download_failed",
            canvas_core::CanvasError::ReportTimeout(_) => "report_timeout",
            canvas_core::CanvasError::InvalidCalendarEventUpdate(_) => {
                "invalid_calendar_event_update"
            }
            canvas_core::CanvasError::InvalidCalendarEventContext(_) => {
                "invalid_calendar_event_context"
            }
            canvas_core::CanvasError::InvalidConferenceUpdate(_) => {
                "invalid_conference_update"
            }
            canvas_core::CanvasError::InvalidRubricAssociationTarget(_) => {
                "invalid_rubric_association_target"
            }
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
            canvas_core::CanvasError::InvalidGradingPeriodId(raw) => {
                json!({ "input": raw })
            }
            canvas_core::CanvasError::InvalidAssignmentId(raw) => json!({ "input": raw }),
            canvas_core::CanvasError::InvalidAssignmentName(raw) => {
                json!({ "input": raw })
            }
            canvas_core::CanvasError::InvalidGroupCategoryId(raw) => {
                json!({ "input": raw })
            }
            canvas_core::CanvasError::InvalidGroupAssignmentMode(raw) => {
                json!({ "input": raw })
            }
            canvas_core::CanvasError::InvalidGroupAssignmentSettings(detail) => {
                json!({ "detail": detail })
            }
            canvas_core::CanvasError::InvalidPeerReviewMode(raw) => {
                json!({ "input": raw })
            }
            canvas_core::CanvasError::InvalidPeerReviewSettings(detail) => {
                json!({ "detail": detail })
            }
            canvas_core::CanvasError::InvalidAssignmentOverrides(detail) => {
                json!({ "detail": detail })
            }
            canvas_core::CanvasError::InvalidAssignmentOverrideTarget(detail) => {
                json!({ "detail": detail })
            }
            canvas_core::CanvasError::InvalidAssignmentOverrideDates(detail) => {
                json!({ "detail": detail })
            }
            canvas_core::CanvasError::InvalidGradingPostingPolicy(raw) => {
                json!({ "input": raw })
            }
            canvas_core::CanvasError::InvalidMutedState(raw) => {
                json!({ "input": raw })
            }
            canvas_core::CanvasError::InvalidOutcomeId(raw) => json!({ "input": raw }),
            canvas_core::CanvasError::InvalidOutcomeGroupId(raw) => {
                json!({ "input": raw })
            }
            canvas_core::CanvasError::InvalidOutcomeTitle(raw) => {
                json!({ "input": raw })
            }
            canvas_core::CanvasError::InvalidOutcomeDescription(raw) => {
                json!({ "input": raw })
            }
            canvas_core::CanvasError::InvalidRubricId(raw) => json!({ "input": raw }),
            canvas_core::CanvasError::InvalidRubricAssociationId(raw) => {
                json!({ "input": raw })
            }
            canvas_core::CanvasError::InvalidRubricTitle(raw) => json!({ "input": raw }),
            canvas_core::CanvasError::InvalidQuizId(raw) => json!({ "input": raw }),
            canvas_core::CanvasError::InvalidQuizSubmissionId(raw) => {
                json!({ "input": raw })
            }
            canvas_core::CanvasError::InvalidQuizTitle(raw) => json!({ "input": raw }),
            canvas_core::CanvasError::InvalidQuestionBankId(raw) => {
                json!({ "input": raw })
            }
            canvas_core::CanvasError::InvalidQuestionId(raw) => json!({ "input": raw }),
            canvas_core::CanvasError::InvalidQuestionBankTitle(raw) => {
                json!({ "input": raw })
            }
            canvas_core::CanvasError::InvalidQuestionName(raw) => json!({ "input": raw }),
            canvas_core::CanvasError::InvalidQuestionText(raw) => json!({ "input": raw }),
            canvas_core::CanvasError::InvalidQuestionType(raw) => json!({ "input": raw }),
            canvas_core::CanvasError::InvalidQuestionBankUpdate(detail) => {
                json!({ "detail": detail })
            }
            canvas_core::CanvasError::InvalidQuestionUpdate(detail) => {
                json!({ "detail": detail })
            }
            canvas_core::CanvasError::InvalidQuestionBankJson(detail) => {
                json!({ "detail": detail })
            }
            canvas_core::CanvasError::InvalidQuestionJson(detail) => {
                json!({ "detail": detail })
            }
            canvas_core::CanvasError::InvalidCalendarEventId(raw) => {
                json!({ "input": raw })
            }
            canvas_core::CanvasError::InvalidConferenceId(raw) => {
                json!({ "input": raw })
            }
            canvas_core::CanvasError::InvalidCollaborationId(raw) => {
                json!({ "input": raw })
            }
            canvas_core::CanvasError::InvalidSectionId(raw) => json!({ "input": raw }),
            canvas_core::CanvasError::InvalidSectionName(raw) => {
                json!({ "input": raw })
            }
            canvas_core::CanvasError::InvalidEnrollmentId(raw) => {
                json!({ "input": raw })
            }
            canvas_core::CanvasError::InvalidExternalToolId(raw) => {
                json!({ "input": raw })
            }
            canvas_core::CanvasError::InvalidExternalToolName(raw) => {
                json!({ "input": raw })
            }
            canvas_core::CanvasError::InvalidExternalToolConfig(detail) => {
                json!({ "detail": detail })
            }
            canvas_core::CanvasError::InvalidExternalToolPlacement(raw) => {
                json!({ "input": raw })
            }
            canvas_core::CanvasError::InvalidExternalToolPlacements(detail) => {
                json!({ "detail": detail })
            }
            canvas_core::CanvasError::InvalidExternalToolUpdate(detail) => {
                json!({ "detail": detail })
            }
            canvas_core::CanvasError::InvalidCalendarEventTitle(raw) => {
                json!({ "input": raw })
            }
            canvas_core::CanvasError::InvalidConferenceTitle(raw) => {
                json!({ "input": raw })
            }
            canvas_core::CanvasError::InvalidCollaborationTitle(raw) => {
                json!({ "input": raw })
            }
            canvas_core::CanvasError::InvalidCollaborationType(raw) => {
                json!({ "input": raw })
            }
            canvas_core::CanvasError::InvalidCollaborators(detail) => {
                json!({ "detail": detail })
            }
            canvas_core::CanvasError::InvalidConferenceDescription(raw) => {
                json!({ "input": raw })
            }
            canvas_core::CanvasError::InvalidConferenceDuration(raw) => {
                json!({ "input": raw })
            }
            canvas_core::CanvasError::InvalidUserId(raw) => json!({ "input": raw }),
            canvas_core::CanvasError::InvalidSubmissionId(raw) => json!({ "input": raw }),
            canvas_core::CanvasError::InvalidPageId(raw) => json!({ "input": raw }),
            canvas_core::CanvasError::InvalidPageTitle(raw) => json!({ "input": raw }),
            canvas_core::CanvasError::InvalidPageBody(raw) => json!({ "input": raw }),
            canvas_core::CanvasError::InvalidModuleId(raw) => json!({ "input": raw }),
            canvas_core::CanvasError::InvalidModuleItemId(raw) => {
                json!({ "input": raw })
            }
            canvas_core::CanvasError::InvalidModuleName(raw) => json!({ "input": raw }),
            canvas_core::CanvasError::InvalidModuleRequirementType(raw) => {
                json!({ "input": raw })
            }
            canvas_core::CanvasError::InvalidModuleRequirement(detail) => {
                json!({ "detail": detail })
            }
            canvas_core::CanvasError::InvalidModulePrerequisites(detail) => {
                json!({ "detail": detail })
            }
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
            canvas_core::CanvasError::InvalidEventDateTime(raw) => {
                json!({ "input": raw })
            }
            canvas_core::CanvasError::InvalidAllDayDate(raw) => {
                json!({ "input": raw })
            }
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
            canvas_core::CanvasError::InvalidRubricCriteria(raw) => {
                json!({ "input": raw })
            }
            canvas_core::CanvasError::InvalidCourseVisibility(raw) => json!({ "input": raw }),
            canvas_core::CanvasError::InvalidGradingSchemeId(raw) => json!({ "input": raw }),
            canvas_core::CanvasError::InvalidContentMigrationId(raw) => {
                json!({ "input": raw })
            }
            canvas_core::CanvasError::InvalidContentMigrationType(raw) => {
                json!({ "input": raw })
            }
            canvas_core::CanvasError::InvalidContentMigrationCreate(detail) => {
                json!({ "detail": detail })
            }
            canvas_core::CanvasError::InvalidReportType(raw) => json!({ "input": raw }),
            canvas_core::CanvasError::InvalidCourseDates(detail) => json!({ "detail": detail }),
            canvas_core::CanvasError::InvalidCourseUpdate(detail) => json!({ "detail": detail }),
            canvas_core::CanvasError::GradingPeriodNotFound(id) => {
                json!({ "grading_period_id": id })
            }
            canvas_core::CanvasError::InvalidAssignmentUpdate(detail) => {
                json!({ "detail": detail })
            }
            canvas_core::CanvasError::InvalidOutcomeUpdate(detail) => {
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
            canvas_core::CanvasError::InvalidSectionUpdate(detail) => {
                json!({ "detail": detail })
            }
            canvas_core::CanvasError::InvalidModuleRequirementUpdate(detail) => {
                json!({ "detail": detail })
            }
            canvas_core::CanvasError::InvalidModuleReorder(detail) => {
                json!({ "detail": detail })
            }
            canvas_core::CanvasError::InvalidRubricUpdate(detail) => {
                json!({ "detail": detail })
            }
            canvas_core::CanvasError::InvalidCalendarEventCreate(detail) => {
                json!({ "detail": detail })
            }
            canvas_core::CanvasError::InvalidCalendarEventUpdate(detail) => {
                json!({ "detail": detail })
            }
            canvas_core::CanvasError::InvalidCalendarEventContext(detail) => {
                json!({ "detail": detail })
            }
            canvas_core::CanvasError::InvalidConferenceUpdate(detail) => {
                json!({ "detail": detail })
            }
            canvas_core::CanvasError::InvalidRubricAssociationTarget(detail) => {
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
            canvas_core::CanvasError::ContentMigrationFailed(detail) => {
                json!({ "detail": detail })
            }
            canvas_core::CanvasError::ContentMigrationTimeout(detail) => {
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
                peer_review_mode,
                peer_review_assign_at,
                peer_review_due_at,
                group_assignment_mode,
                group_category_id,
                overrides_json,
                overrides_file,
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
                let peer_reviews = parse_peer_review_settings_input(
                    peer_review_mode,
                    peer_review_assign_at,
                    peer_review_due_at,
                )?;
                let group_assignment = parse_group_assignment_settings_input(
                    group_assignment_mode,
                    group_category_id,
                )?;
                let overrides = parse_assignment_overrides_input(
                    overrides_json,
                    overrides_file,
                )?;
                let input = canvas_core::AssignmentCreateInput::new(
                    name,
                    points,
                    due_at,
                    publish_state,
                    peer_reviews,
                    group_assignment,
                    overrides,
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
                peer_review_mode,
                peer_review_assign_at,
                peer_review_due_at,
                group_assignment_mode,
                group_category_id,
                overrides_json,
                overrides_file,
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
                let peer_reviews = parse_peer_review_settings_input(
                    peer_review_mode,
                    peer_review_assign_at,
                    peer_review_due_at,
                )?;
                let group_assignment = parse_group_assignment_settings_input(
                    group_assignment_mode,
                    group_category_id,
                )?;
                let overrides = parse_assignment_overrides_input(
                    overrides_json,
                    overrides_file,
                )?;
                let input = canvas_core::AssignmentUpdateInput::new(
                    name,
                    points,
                    due_at,
                    publish_state,
                    peer_reviews,
                    group_assignment,
                    overrides,
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

impl TryFrom<(OutcomeCommand, &GlobalOptions)> for OutcomeRequest {
    type Error = CliError;

    fn try_from(
        input: (OutcomeCommand, &GlobalOptions),
    ) -> Result<Self, Self::Error> {
        let (command, global) = input;
        match command {
            OutcomeCommand::List { course } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                Ok(OutcomeRequest::List(OutcomeListRequest { course_id }))
            }
            OutcomeCommand::Create {
                course,
                group,
                title,
                description,
                points_possible,
                mastery_points,
            } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                let outcome_group_id =
                    canvas_core::parse_outcome_group_id(&group)?;
                let title = canvas_core::parse_outcome_title(&title)?;
                let description = match description {
                    Some(raw) => Some(canvas_core::parse_outcome_description(&raw)?),
                    None => None,
                };
                let points_possible = match points_possible {
                    Some(raw) => Some(canvas_core::parse_points_possible(&raw)?),
                    None => None,
                };
                let mastery_points = match mastery_points {
                    Some(raw) => Some(canvas_core::parse_points_possible(&raw)?),
                    None => None,
                };
                let input = canvas_core::OutcomeCreateInput::new(
                    title,
                    description,
                    points_possible,
                    mastery_points,
                )?;
                Ok(OutcomeRequest::Create(OutcomeCreateRequest {
                    course_id,
                    outcome_group_id,
                    input,
                }))
            }
            OutcomeCommand::Update {
                outcome,
                title,
                description,
                points_possible,
                mastery_points,
            } => {
                let outcome_id = canvas_core::parse_outcome_id(&outcome)?;
                let title = match title {
                    Some(raw) => Some(canvas_core::parse_outcome_title(&raw)?),
                    None => None,
                };
                let description = match description {
                    Some(raw) => Some(canvas_core::parse_outcome_description(&raw)?),
                    None => None,
                };
                let points_possible = match points_possible {
                    Some(raw) => Some(canvas_core::parse_points_possible(&raw)?),
                    None => None,
                };
                let mastery_points = match mastery_points {
                    Some(raw) => Some(canvas_core::parse_points_possible(&raw)?),
                    None => None,
                };
                let input = canvas_core::OutcomeUpdateInput::new(
                    title,
                    description,
                    points_possible,
                    mastery_points,
                )?;
                Ok(OutcomeRequest::Update(OutcomeUpdateRequest {
                    outcome_id,
                    input,
                }))
            }
            OutcomeCommand::Delete {
                course,
                group,
                outcome,
            } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                let outcome_group_id =
                    canvas_core::parse_outcome_group_id(&group)?;
                let outcome_id = canvas_core::parse_outcome_id(&outcome)?;
                Ok(OutcomeRequest::Delete(OutcomeDeleteRequest {
                    course_id,
                    outcome_group_id,
                    outcome_id,
                }))
            }
        }
    }
}

impl TryFrom<(RubricCommand, &GlobalOptions)> for RubricRequest {
    type Error = CliError;

    fn try_from(
        input: (RubricCommand, &GlobalOptions),
    ) -> Result<Self, Self::Error> {
        let (command, global) = input;
        match command {
            RubricCommand::List { course } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                Ok(RubricRequest::List(RubricListRequest { course_id }))
            }
            RubricCommand::Create {
                course,
                title,
                criteria_json,
                criteria_file,
            } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                let title = canvas_core::parse_rubric_title(&title)?;
                let criteria = parse_rubric_criteria_input(criteria_json, criteria_file)?
                    .ok_or_else(|| {
                        CliError::Canvas(canvas_core::CanvasError::InvalidRubricCriteria(
                            "missing_criteria".to_string(),
                        ))
                    })?;
                let input = canvas_core::RubricCreateInput::new(title, criteria);
                Ok(RubricRequest::Create(RubricCreateRequest { course_id, input }))
            }
            RubricCommand::Update {
                course,
                rubric,
                title,
                criteria_json,
                criteria_file,
            } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                let rubric_id = canvas_core::parse_rubric_id(&rubric)?;
                let title = match title {
                    Some(raw) => Some(canvas_core::parse_rubric_title(&raw)?),
                    None => None,
                };
                let criteria =
                    parse_rubric_criteria_input(criteria_json, criteria_file)?;
                let input = canvas_core::RubricUpdateInput::new(title, criteria)?;
                Ok(RubricRequest::Update(RubricUpdateRequest {
                    course_id,
                    rubric_id,
                    input,
                }))
            }
            RubricCommand::Delete { course, rubric } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                let rubric_id = canvas_core::parse_rubric_id(&rubric)?;
                Ok(RubricRequest::Delete(RubricDeleteRequest {
                    course_id,
                    rubric_id,
                }))
            }
            RubricCommand::Attach {
                course,
                rubric,
                assignment,
                outcome,
                grading,
                title,
            } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                let rubric_id = canvas_core::parse_rubric_id(&rubric)?;
                let assignment_id = match assignment {
                    Some(raw) => Some(canvas_core::parse_assignment_id(&raw)?),
                    None => None,
                };
                let outcome_id = match outcome {
                    Some(raw) => Some(canvas_core::parse_outcome_id(&raw)?),
                    None => None,
                };
                let target =
                    canvas_core::parse_rubric_association_target(assignment_id, outcome_id)?;
                let selection = canvas_core::parse_rubric_selection(&grading)?;
                let input =
                    canvas_core::RubricAssociationInput::new(rubric_id, target, selection, title);
                Ok(RubricRequest::Attach(RubricAttachRequest {
                    course_id,
                    input,
                }))
            }
            RubricCommand::Detach {
                course,
                association,
            } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                let rubric_association_id =
                    canvas_core::parse_rubric_association_id(&association)?;
                Ok(RubricRequest::Detach(RubricDetachRequest {
                    course_id,
                    rubric_association_id,
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

impl TryFrom<(QuestionBankCommand, &GlobalOptions)> for QuestionBankRequest {
    type Error = CliError;

    fn try_from(
        input: (QuestionBankCommand, &GlobalOptions),
    ) -> Result<Self, Self::Error> {
        let (command, global) = input;
        match command {
            QuestionBankCommand::List { course } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                Ok(QuestionBankRequest::List(QuestionBankListRequest { course_id }))
            }
            QuestionBankCommand::Create {
                course,
                title,
                bank_json,
                bank_file,
            } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                let input = parse_question_bank_create_input(title, bank_json, bank_file)?;
                Ok(QuestionBankRequest::Create(QuestionBankCreateRequest { course_id, input }))
            }
            QuestionBankCommand::Update {
                course,
                bank,
                title,
                bank_json,
                bank_file,
            } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                let bank_id = canvas_core::parse_question_bank_id(&bank)?;
                let input = parse_question_bank_update_input(title, bank_json, bank_file)?;
                Ok(QuestionBankRequest::Update(QuestionBankUpdateRequest {
                    course_id,
                    bank_id,
                    input,
                }))
            }
            QuestionBankCommand::Delete { course, bank } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                let bank_id = canvas_core::parse_question_bank_id(&bank)?;
                Ok(QuestionBankRequest::Delete(QuestionBankDeleteRequest {
                    course_id,
                    bank_id,
                }))
            }
        }
    }
}

impl TryFrom<(QuestionCommand, &GlobalOptions)> for QuestionRequest {
    type Error = CliError;

    fn try_from(input: (QuestionCommand, &GlobalOptions)) -> Result<Self, Self::Error> {
        let (command, _global) = input;
        match command {
            QuestionCommand::List { bank } => {
                let bank_id = canvas_core::parse_question_bank_id(&bank)?;
                Ok(QuestionRequest::List(QuestionListRequest { bank_id }))
            }
            QuestionCommand::Create {
                bank,
                name,
                text,
                question_type,
                points,
                correct_comments,
                incorrect_comments,
                neutral_comments,
                question_json,
                question_file,
            } => {
                let bank_id = canvas_core::parse_question_bank_id(&bank)?;
                let input = parse_question_create_input(
                    name,
                    text,
                    question_type,
                    points,
                    correct_comments,
                    incorrect_comments,
                    neutral_comments,
                    question_json,
                    question_file,
                )?;
                Ok(QuestionRequest::Create(QuestionCreateRequest { bank_id, input }))
            }
            QuestionCommand::Update {
                bank,
                question,
                name,
                text,
                question_type,
                points,
                correct_comments,
                incorrect_comments,
                neutral_comments,
                question_json,
                question_file,
            } => {
                let bank_id = canvas_core::parse_question_bank_id(&bank)?;
                let question_id = canvas_core::parse_question_id(&question)?;
                let input = parse_question_update_input(
                    name,
                    text,
                    question_type,
                    points,
                    correct_comments,
                    incorrect_comments,
                    neutral_comments,
                    question_json,
                    question_file,
                )?;
                Ok(QuestionRequest::Update(QuestionUpdateRequest {
                    bank_id,
                    question_id,
                    input,
                }))
            }
            QuestionCommand::Delete { bank, question } => {
                let bank_id = canvas_core::parse_question_bank_id(&bank)?;
                let question_id = canvas_core::parse_question_id(&question)?;
                Ok(QuestionRequest::Delete(QuestionDeleteRequest {
                    bank_id,
                    question_id,
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
            ModuleCommand::Requirement { command } => match command {
                ModuleRequirementCommand::List { course, module } => {
                    let course_id = match course {
                        Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                        None => global.course,
                    }
                    .ok_or(CliError::MissingCourseId)?;
                    let module_id = canvas_core::parse_module_id(&module)?;
                    Ok(ModuleRequest::RequirementList(ModuleRequirementListRequest {
                        course_id,
                        module_id,
                    }))
                }
                ModuleRequirementCommand::Update {
                    course,
                    module,
                    requirements_json,
                    requirements_file,
                    prerequisites_json,
                    prerequisites_file,
                    unlock_at,
                    sequential_progress,
                } => {
                    let course_id = match course {
                        Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                        None => global.course,
                    }
                    .ok_or(CliError::MissingCourseId)?;
                    let module_id = canvas_core::parse_module_id(&module)?;
                    let requirements = parse_module_requirements_input(
                        requirements_json,
                        requirements_file,
                    )?;
                    let prerequisites = parse_module_prerequisites_input(
                        prerequisites_json,
                        prerequisites_file,
                    )?;
                    let unlock_at = match unlock_at {
                        Some(raw) => Some(canvas_core::parse_due_date(&raw)?),
                        None => None,
                    };
                    let sequential_progress =
                        sequential_progress.map(SequentialProgressSetting::as_bool);
                    let unlock_rules = if unlock_at.is_some()
                        || sequential_progress.is_some()
                    {
                        Some(canvas_models::ModuleUnlockRules::new(
                            unlock_at,
                            sequential_progress,
                        ))
                    } else {
                        None
                    };
                    let input = canvas_core::ModuleRequirementsUpdateInput::new(
                        requirements,
                        prerequisites,
                        unlock_rules,
                    )?;
                    Ok(ModuleRequest::RequirementUpdate(
                        ModuleRequirementUpdateRequest {
                            course_id,
                            module_id,
                            input,
                        },
                    ))
                }
            },
        }
    }
}

impl TryFrom<(ToolCommand, &GlobalOptions)> for ToolRequest {
    type Error = CliError;

    fn try_from(input: (ToolCommand, &GlobalOptions)) -> Result<Self, Self::Error> {
        let (command, global) = input;
        match command {
            ToolCommand::List { course } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                Ok(ToolRequest::List(ToolListRequest { course_id }))
            }
            ToolCommand::Create {
                course,
                name,
                config_url,
                config_json,
                config_xml,
                config_file,
                placement,
                placements_json,
                placements_file,
            } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                let name = canvas_core::parse_external_tool_name(&name)?;
                let config = parse_external_tool_config_input(
                    config_url,
                    config_json,
                    config_xml,
                    config_file,
                    true,
                )?
                .ok_or_else(|| {
                    CliError::Canvas(canvas_core::CanvasError::InvalidExternalToolConfig(
                        "missing_config".to_string(),
                    ))
                })?;
                let (placements, placement_settings) = parse_external_tool_placements_input(
                    placement,
                    placements_json,
                    placements_file,
                )?;
                Ok(ToolRequest::Create(ToolCreateRequest {
                    course_id,
                    name,
                    config,
                    placements,
                    placement_settings,
                }))
            }
            ToolCommand::Update {
                course,
                tool,
                name,
                config_url,
                config_json,
                config_xml,
                config_file,
                placement,
                placements_json,
                placements_file,
            } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                let tool_id = canvas_core::parse_external_tool_id(&tool)?;
                let name = match name {
                    Some(raw) => Some(canvas_core::parse_external_tool_name(&raw)?),
                    None => None,
                };
                let config = parse_external_tool_config_input(
                    config_url,
                    config_json,
                    config_xml,
                    config_file,
                    false,
                )?;
                let (placements, placement_settings) = parse_external_tool_placements_input(
                    placement,
                    placements_json,
                    placements_file,
                )?;
                Ok(ToolRequest::Update(ToolUpdateRequest {
                    course_id,
                    tool_id,
                    name,
                    config,
                    placements,
                    placement_settings,
                }))
            }
            ToolCommand::Delete { course, tool } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                let tool_id = canvas_core::parse_external_tool_id(&tool)?;
                Ok(ToolRequest::Delete(ToolDeleteRequest { course_id, tool_id }))
            }
        }
    }
}

impl TryFrom<(CalendarCommand, &GlobalOptions)> for CalendarEventRequest {
    type Error = CliError;

    fn try_from(
        input: (CalendarCommand, &GlobalOptions),
    ) -> Result<Self, Self::Error> {
        let (command, global) = input;
        match command {
            CalendarCommand::List { course, section } => {
                let section_id = match section {
                    Some(raw) => Some(canvas_core::parse_section_id(&raw)?),
                    None => None,
                };
                if section_id.is_some() && course.is_some() {
                    return Err(CliError::Canvas(
                        canvas_core::CanvasError::InvalidCalendarEventContext(
                            "conflicting_context".to_string(),
                        ),
                    ));
                }
                let course_id = if section_id.is_some() {
                    None
                } else {
                    match course {
                        Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                        None => global.course,
                    }
                };
                let context =
                    canvas_core::parse_calendar_event_context(course_id, section_id)?;
                Ok(CalendarEventRequest::List(CalendarEventListRequest {
                    context,
                }))
            }
            CalendarCommand::Create {
                course,
                section,
                title,
                start_at,
                end_at,
                all_day_date,
            } => {
                let section_id = match section {
                    Some(raw) => Some(canvas_core::parse_section_id(&raw)?),
                    None => None,
                };
                if section_id.is_some() && course.is_some() {
                    return Err(CliError::Canvas(
                        canvas_core::CanvasError::InvalidCalendarEventContext(
                            "conflicting_context".to_string(),
                        ),
                    ));
                }
                let course_id = if section_id.is_some() {
                    None
                } else {
                    match course {
                        Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                        None => global.course,
                    }
                };
                let context =
                    canvas_core::parse_calendar_event_context(course_id, section_id)?;
                let title = canvas_core::parse_calendar_event_title(&title)?;
                let timing = parse_calendar_timing_for_create(
                    start_at,
                    end_at,
                    all_day_date,
                )?;
                let input =
                    canvas_core::CalendarEventCreateInput::new(context, title, timing);
                Ok(CalendarEventRequest::Create(CalendarEventCreateRequest {
                    input,
                }))
            }
            CalendarCommand::Update {
                event_id,
                title,
                start_at,
                end_at,
                all_day_date,
            } => {
                let event_id = canvas_core::parse_calendar_event_id(&event_id)?;
                let title = match title {
                    Some(raw) => Some(canvas_core::parse_calendar_event_title(&raw)?),
                    None => None,
                };
                let timing = parse_calendar_timing_for_update(
                    start_at,
                    end_at,
                    all_day_date,
                )?;
                let input = canvas_core::CalendarEventUpdateInput::new(title, timing)?;
                Ok(CalendarEventRequest::Update(CalendarEventUpdateRequest {
                    event_id,
                    input,
                }))
            }
            CalendarCommand::Delete { event_id } => {
                let event_id = canvas_core::parse_calendar_event_id(&event_id)?;
                Ok(CalendarEventRequest::Delete(CalendarEventDeleteRequest {
                    event_id,
                }))
            }
        }
    }
}

impl TryFrom<(ConferenceCommand, &GlobalOptions)> for ConferenceRequest {
    type Error = CliError;

    fn try_from(
        input: (ConferenceCommand, &GlobalOptions),
    ) -> Result<Self, Self::Error> {
        let (command, global) = input;
        match command {
            ConferenceCommand::List { course } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                Ok(ConferenceRequest::List(ConferenceListRequest { course_id }))
            }
            ConferenceCommand::Create {
                course,
                title,
                description,
                start_at,
                duration,
                recording,
            } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                let title = canvas_core::parse_conference_title(&title)?;
                let description = match description {
                    Some(raw) => Some(canvas_core::parse_conference_description(
                        &raw,
                    )?),
                    None => None,
                };
                let start_at = match start_at {
                    Some(raw) => {
                        Some(canvas_core::parse_event_date_time(&raw)?)
                    }
                    None => None,
                };
                let duration = match duration {
                    Some(raw) => {
                        Some(canvas_core::parse_conference_duration(&raw)?)
                    }
                    None => None,
                };
                let schedule = if start_at.is_some() || duration.is_some() {
                    Some(canvas_core::ConferenceSchedule::new(
                        start_at, duration,
                    ))
                } else {
                    None
                };
                let recording_enabled = recording.map(RecordingSetting::as_bool);
                Ok(ConferenceRequest::Create(ConferenceCreateRequest {
                    course_id,
                    input: canvas_core::ConferenceCreateInput::new(
                        title,
                        description,
                        schedule,
                        recording_enabled,
                    ),
                }))
            }
            ConferenceCommand::Update {
                course,
                conference,
                title,
                description,
                start_at,
                duration,
                recording,
            } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                let conference_id =
                    canvas_core::parse_conference_id(&conference)?;
                let title = match title {
                    Some(raw) => Some(canvas_core::parse_conference_title(&raw)?),
                    None => None,
                };
                let description = match description {
                    Some(raw) => Some(canvas_core::parse_conference_description(
                        &raw,
                    )?),
                    None => None,
                };
                let start_at = match start_at {
                    Some(raw) => {
                        Some(canvas_core::parse_event_date_time(&raw)?)
                    }
                    None => None,
                };
                let duration = match duration {
                    Some(raw) => {
                        Some(canvas_core::parse_conference_duration(&raw)?)
                    }
                    None => None,
                };
                let schedule = if start_at.is_some() || duration.is_some() {
                    Some(canvas_core::ConferenceSchedule::new(
                        start_at, duration,
                    ))
                } else {
                    None
                };
                let recording_enabled = recording.map(RecordingSetting::as_bool);
                let input = canvas_core::ConferenceUpdateInput::new(
                    title,
                    description,
                    schedule,
                    recording_enabled,
                )?;
                Ok(ConferenceRequest::Update(ConferenceUpdateRequest {
                    course_id,
                    conference_id,
                    input,
                }))
            }
            ConferenceCommand::Delete { course, conference } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                let conference_id =
                    canvas_core::parse_conference_id(&conference)?;
                Ok(ConferenceRequest::Delete(ConferenceDeleteRequest {
                    course_id,
                    conference_id,
                }))
            }
        }
    }
}

impl TryFrom<(CollaborationCommand, &GlobalOptions)> for CollaborationRequest {
    type Error = CliError;

    fn try_from(
        input: (CollaborationCommand, &GlobalOptions),
    ) -> Result<Self, Self::Error> {
        let (command, global) = input;
        match command {
            CollaborationCommand::List { course } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                Ok(CollaborationRequest::List(CollaborationListRequest {
                    course_id,
                }))
            }
            CollaborationCommand::Create {
                course,
                title,
                collaboration_type,
                user_ids,
                group_ids,
            } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                let title = canvas_core::parse_collaboration_title(&title)?;
                let collaboration_type =
                    canvas_core::parse_collaboration_type(&collaboration_type)?;
                let mut collaborators = Vec::new();
                for raw in user_ids {
                    let user_id = canvas_core::parse_user_id(&raw)?;
                    collaborators.push(canvas_core::Collaborator::user(user_id));
                }
                for raw in group_ids {
                    let group_id = canvas_core::parse_group_id(&raw)?;
                    collaborators.push(canvas_core::Collaborator::group(group_id));
                }
                let collaborators =
                    canvas_core::Collaborators::new(collaborators)?;
                let input = canvas_core::CollaborationCreateInput::new(
                    title,
                    collaboration_type,
                    collaborators,
                );
                Ok(CollaborationRequest::Create(
                    CollaborationCreateRequest { course_id, input },
                ))
            }
            CollaborationCommand::Delete { collaboration } => {
                let collaboration_id =
                    canvas_core::parse_collaboration_id(&collaboration)?;
                Ok(CollaborationRequest::Delete(
                    CollaborationDeleteRequest { collaboration_id },
                ))
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

impl TryFrom<(ContentMigrationCommand, &GlobalOptions)> for ContentMigrationRequest {
    type Error = CliError;

    fn try_from(
        input: (ContentMigrationCommand, &GlobalOptions),
    ) -> Result<Self, Self::Error> {
        let (command, global) = input;
        match command {
            ContentMigrationCommand::List { course } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                Ok(ContentMigrationRequest::List(ContentMigrationListRequest {
                    course_id,
                }))
            }
            ContentMigrationCommand::Create {
                course,
                migration_type,
                source_course,
                file,
                wait,
            } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                let migration_type =
                    canvas_core::parse_content_migration_type(&migration_type)?;
                let input = match migration_type {
                    canvas_models::ContentMigrationType::CourseCopy => {
                        let source_course = source_course.ok_or_else(|| {
                            CliError::Canvas(
                                canvas_core::CanvasError::InvalidContentMigrationCreate(
                                    "missing_source_course".to_string(),
                                ),
                            )
                        })?;
                        let source_course_id =
                            canvas_core::parse_course_id(&source_course)?;
                        canvas_core::ContentMigrationCreateInput::CourseCopy {
                            source_course_id,
                        }
                    }
                    canvas_models::ContentMigrationType::FileImport => {
                        let file = file.ok_or_else(|| {
                            CliError::Canvas(
                                canvas_core::CanvasError::InvalidContentMigrationCreate(
                                    "missing_file".to_string(),
                                ),
                            )
                        })?;
                        canvas_core::ContentMigrationCreateInput::FileImport {
                            file,
                        }
                    }
                };
                Ok(ContentMigrationRequest::Create(
                    ContentMigrationCreateRequest {
                        course_id,
                        input,
                        wait,
                    },
                ))
            }
            ContentMigrationCommand::Show {
                course,
                migration,
                wait,
            } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                let migration_id =
                    canvas_core::parse_content_migration_id(&migration)?;
                Ok(ContentMigrationRequest::Show(ContentMigrationShowRequest {
                    course_id,
                    migration_id,
                    wait,
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

impl TryFrom<(SectionCommand, &GlobalOptions)> for SectionRequest {
    type Error = CliError;

    fn try_from(
        input: (SectionCommand, &GlobalOptions),
    ) -> Result<Self, Self::Error> {
        let (command, global) = input;
        match command {
            SectionCommand::List { course } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                Ok(SectionRequest::List(SectionListRequest { course_id }))
            }
            SectionCommand::Create { course, name } => {
                let course_id = match course {
                    Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                    None => global.course,
                }
                .ok_or(CliError::MissingCourseId)?;
                let name = canvas_core::parse_section_name(&name)?;
                let input = canvas_core::SectionCreateInput::new(name);
                Ok(SectionRequest::Create(SectionCreateRequest {
                    course_id,
                    input,
                }))
            }
            SectionCommand::Update { section, name } => {
                let section_id = canvas_core::parse_section_id(&section)?;
                let name = match name {
                    Some(raw) => Some(canvas_core::parse_section_name(&raw)?),
                    None => None,
                };
                let input = canvas_core::SectionUpdateInput::new(name)?;
                Ok(SectionRequest::Update(SectionUpdateRequest {
                    section_id,
                    input,
                }))
            }
            SectionCommand::Delete { section } => {
                let section_id = canvas_core::parse_section_id(&section)?;
                Ok(SectionRequest::Delete(SectionDeleteRequest { section_id }))
            }
        }
    }
}

impl TryFrom<(EnrollmentCommand, &GlobalOptions)> for EnrollmentRequest {
    type Error = CliError;

    fn try_from(
        input: (EnrollmentCommand, &GlobalOptions),
    ) -> Result<Self, Self::Error> {
        let (command, _global) = input;
        match command {
            EnrollmentCommand::List { section, role } => {
                let section_id = canvas_core::parse_section_id(&section)?;
                let role = match role {
                    Some(raw) => Some(canvas_core::parse_user_role(&raw)?),
                    None => None,
                };
                Ok(EnrollmentRequest::List(EnrollmentListRequest {
                    section_id,
                    role,
                }))
            }
            EnrollmentCommand::Add {
                section,
                user,
                role,
                limit_to_section,
            } => {
                let section_id = canvas_core::parse_section_id(&section)?;
                let user_id = canvas_core::parse_user_id(&user)?;
                let role = canvas_core::parse_user_role(&role)?;
                let input = canvas_core::EnrollmentCreateInput::new(
                    user_id,
                    role,
                    limit_to_section,
                );
                Ok(EnrollmentRequest::Add(EnrollmentAddRequest {
                    section_id,
                    input,
                }))
            }
            EnrollmentCommand::Remove {
                section,
                enrollment,
            } => {
                let section_id = canvas_core::parse_section_id(&section)?;
                let enrollment_id =
                    canvas_core::parse_enrollment_id(&enrollment)?;
                Ok(EnrollmentRequest::Remove(EnrollmentRemoveRequest {
                    section_id,
                    enrollment_id,
                }))
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
            ReportCommand::GradeChangeLog {
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
                Ok(ReportRequest::GradeChangeLog(ReportExportRequest {
                    course_id,
                    report_type: ReportType::GradeChangeLog,
                    format,
                    output,
                }))
            }
        }
    }
}

impl TryFrom<(GradebookCommand, &GlobalOptions)> for GradebookRequest {
    type Error = CliError;

    fn try_from(
        input: (GradebookCommand, &GlobalOptions),
    ) -> Result<Self, Self::Error> {
        let (command, global) = input;
        match command {
            GradebookCommand::GradingPeriod { command } => match command {
                GradingPeriodCommand::List { course } => {
                    let course_id = match course {
                        Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                        None => global.course,
                    }
                    .ok_or(CliError::MissingCourseId)?;
                    Ok(GradebookRequest::GradingPeriodList(
                        GradingPeriodListRequest { course_id },
                    ))
                }
                GradingPeriodCommand::Show { course, period } => {
                    let course_id = match course {
                        Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                        None => global.course,
                    }
                    .ok_or(CliError::MissingCourseId)?;
                    let grading_period_id =
                        canvas_core::parse_grading_period_id(&period)?;
                    Ok(GradebookRequest::GradingPeriodShow(
                        GradingPeriodShowRequest {
                            course_id,
                            grading_period_id,
                        },
                    ))
                }
            },
            GradebookCommand::PostingPolicy { command } => match command {
                PostingPolicyCommand::Get { course } => {
                    let course_id = match course {
                        Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                        None => global.course,
                    }
                    .ok_or(CliError::MissingCourseId)?;
                    Ok(GradebookRequest::PostingPolicyGet(
                        PostingPolicyRequest { course_id },
                    ))
                }
                PostingPolicyCommand::Set { course, policy } => {
                    let course_id = match course {
                        Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                        None => global.course,
                    }
                    .ok_or(CliError::MissingCourseId)?;
                    let policy =
                        canvas_core::parse_grading_posting_policy(&policy)?;
                    Ok(GradebookRequest::PostingPolicySet(
                        PostingPolicyUpdateRequest { course_id, policy },
                    ))
                }
            },
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
        let help = Cli::command().render_help().to_string();
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
