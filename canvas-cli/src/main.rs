use std::io::{self, Write};
use std::path::PathBuf;

use canvas_models::CourseId;
use clap::{Args, Parser, Subcommand};
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
    #[command(after_help = "Examples:\n  canvas course list\n  canvas course show --course 42")]
    Course {
        #[command(subcommand)]
        command: CourseCommand,
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
    /// Show a course summary (placeholder)
    Show {
        /// Course id override
        #[arg(long)]
        course: Option<String>,
    },
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
    #[error("confirmation required for {0}")]
    ConfirmationRequired(&'static str),
    #[error("unable to execute ask plan")]
    AskExecutionUnsupported,
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
        Command::Course { command } => handle_course(command, &global),
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

fn handle_course(command: CourseCommand, global: &GlobalOptions) -> Result<(), CliError> {
    match command {
        CourseCommand::List => {
            let planned = vec![PlannedAction {
                action: "list courses",
                risk: "none",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("course list", &planned, global);
                return Ok(());
            }
            emit_result(
                "course list",
                json!({"status": "todo", "courses": []}),
                global,
            );
            if !global.quiet && !global.json {
                println!("Course listing not implemented yet.");
            }
        }
        CourseCommand::Show { course } => {
            let course_id = match course {
                Some(raw) => Some(canvas_core::parse_course_id(&raw)?),
                None => global.course,
            };
            let planned = vec![PlannedAction {
                action: "show course summary",
                risk: "none",
                requires_confirmation: false,
            }];
            if global.explain {
                emit_plan("course show", &planned, global);
                return Ok(());
            }
            emit_result(
                "course show",
                json!({
                    "status": "todo",
                    "course_id": course_id.map(CourseId::get),
                }),
                global,
            );
            if !global.quiet && !global.json {
                match course_id {
                    Some(id) => println!("Course {} summary not implemented yet.", id.get()),
                    None => println!("Course summary not implemented yet."),
                }
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
                handle_course(CourseCommand::List, global)?;
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
        CliError::ConfirmationRequired(_) => "confirmation_required",
        CliError::AskExecutionUnsupported => "ask_execution_unsupported",
        CliError::Canvas(canvas_error) => match canvas_error {
            canvas_core::CanvasError::InvalidCourseId(_) => "invalid_course_id",
            canvas_core::CanvasError::InvalidHost(_) => "invalid_host",
            canvas_core::CanvasError::InvalidToken => "invalid_token",
            canvas_core::CanvasError::MissingConfig(_) => "missing_config",
            canvas_core::CanvasError::ConfigRead(_, _) => "config_read",
            canvas_core::CanvasError::ConfigParse(_, _) => "config_parse",
            canvas_core::CanvasError::ConfigWrite(_, _) => "config_write",
            canvas_core::CanvasError::AuthCheckFailed(_) => "auth_check_failed",
            canvas_core::CanvasError::Http(_) => "http_error",
        },
    }
}

fn error_details(error: &CliError) -> Value {
    match error {
        CliError::ConfirmationRequired(action) => {
            json!({ "action": action })
        }
        CliError::Canvas(canvas_error) => match canvas_error {
            canvas_core::CanvasError::InvalidCourseId(raw) => json!({ "input": raw }),
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

fn run_init(config_path: &PathBuf) -> Result<(), canvas_core::CanvasError> {
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

fn create_config_dir(path: &PathBuf) -> Result<(), canvas_core::CanvasError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|err| {
            canvas_core::CanvasError::ConfigWrite(parent.display().to_string(), err.to_string())
        })?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{plan_from_prompt, PlannedCommand};

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
}
