use std::io::{self, Write};
use std::path::PathBuf;

use canvas_models::{AssignmentId, CourseId, UserId};
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

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
enum ImportFormat {
    Csv,
    Json,
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

fn emit_schema() {
    let output = json!({
        "version": SCHEMA_VERSION,
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
    });
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
        CliError::Canvas(canvas_error) => match canvas_error {
            canvas_core::CanvasError::InvalidCourseId(_) => "invalid_course_id",
            canvas_core::CanvasError::InvalidAssignmentId(_) => "invalid_assignment_id",
            canvas_core::CanvasError::InvalidAssignmentName(_) => {
                "invalid_assignment_name"
            }
            canvas_core::CanvasError::InvalidUserId(_) => "invalid_user_id",
            canvas_core::CanvasError::InvalidSubmissionId(_) => "invalid_submission_id",
            canvas_core::CanvasError::InvalidScore(_) => "invalid_score",
            canvas_core::CanvasError::InvalidPointsPossible(_) => "invalid_points_possible",
            canvas_core::CanvasError::InvalidDueDate(_) => "invalid_due_date",
            canvas_core::CanvasError::InvalidPublishState(_) => "invalid_publish_state",
            canvas_core::CanvasError::InvalidRubricSelection(_) => "invalid_rubric_selection",
            canvas_core::CanvasError::InvalidRubricAssessment(_) => {
                "invalid_rubric_assessment"
            }
            canvas_core::CanvasError::InvalidCourseVisibility(_) => "invalid_course_visibility",
            canvas_core::CanvasError::InvalidGradingSchemeId(_) => "invalid_grading_scheme_id",
            canvas_core::CanvasError::InvalidCourseDates(_) => "invalid_course_dates",
            canvas_core::CanvasError::InvalidCourseUpdate(_) => "invalid_course_update",
            canvas_core::CanvasError::InvalidAssignmentUpdate(_) => {
                "invalid_assignment_update"
            }
            canvas_core::CanvasError::MissingAssignmentPoints(_) => {
                "missing_assignment_points"
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
        CliError::Canvas(canvas_error) => match canvas_error {
            canvas_core::CanvasError::InvalidCourseId(raw) => json!({ "input": raw }),
            canvas_core::CanvasError::InvalidAssignmentId(raw) => json!({ "input": raw }),
            canvas_core::CanvasError::InvalidAssignmentName(raw) => {
                json!({ "input": raw })
            }
            canvas_core::CanvasError::InvalidUserId(raw) => json!({ "input": raw }),
            canvas_core::CanvasError::InvalidSubmissionId(raw) => json!({ "input": raw }),
            canvas_core::CanvasError::InvalidScore(raw) => json!({ "input": raw }),
            canvas_core::CanvasError::InvalidPointsPossible(raw) => {
                json!({ "input": raw })
            }
            canvas_core::CanvasError::InvalidDueDate(raw) => json!({ "input": raw }),
            canvas_core::CanvasError::InvalidPublishState(raw) => json!({ "input": raw }),
            canvas_core::CanvasError::InvalidRubricSelection(raw) => json!({ "input": raw }),
            canvas_core::CanvasError::InvalidRubricAssessment(raw) => {
                json!({ "input": raw })
            }
            canvas_core::CanvasError::InvalidCourseVisibility(raw) => json!({ "input": raw }),
            canvas_core::CanvasError::InvalidGradingSchemeId(raw) => json!({ "input": raw }),
            canvas_core::CanvasError::InvalidCourseDates(detail) => json!({ "detail": detail }),
            canvas_core::CanvasError::InvalidCourseUpdate(detail) => json!({ "detail": detail }),
            canvas_core::CanvasError::InvalidAssignmentUpdate(detail) => {
                json!({ "detail": detail })
            }
            canvas_core::CanvasError::MissingAssignmentPoints(assignment_id) => {
                json!({ "assignment_id": assignment_id })
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
    use super::{parse_import_row, plan_from_prompt, ImportRow, PlannedCommand};

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
        let parsed = parse_import_row(row, 10.0).expect("parsed");
        assert_eq!(parsed.1.get(), 5);
        assert_eq!(parsed.2.value(), 9.0);
    }
}
