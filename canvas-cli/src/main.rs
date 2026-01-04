use std::io::{self, Write};
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use tracing::info;

#[derive(Debug, Parser)]
#[command(name = "canvas", version, about = "Canvas CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Authenticate with Canvas
    Auth {
        #[command(subcommand)]
        command: AuthCommand,
    },
    /// Initialize local configuration
    Init,
    /// Course-related operations
    Courses {
        /// Optional course id to demonstrate parsing at the edge
        #[arg(long)]
        course_id: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
enum AuthCommand {
    /// Validate the configured Canvas credentials
    Check,
}

fn main() -> Result<(), canvas_core::CanvasError> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Command::Auth { command } => match command {
            AuthCommand::Check => {
                let config = canvas_core::load_merged_config()?;
                canvas_core::auth_check(&config)?;
                info!("auth check succeeded");
            }
        },
        Command::Init => {
            run_init()?;
        }
        Command::Courses { course_id } => {
            if let Some(raw) = course_id {
                let parsed = canvas_core::parse_course_id(&raw)?;
                info!(course_id = parsed.get(), "parsed course id");
            } else {
                info!("courses placeholder");
            }
        }
    }

    Ok(())
}

fn run_init() -> Result<(), canvas_core::CanvasError> {
    let config_path = canvas_core::config_path();
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

    create_config_dir(&config_path)?;
    canvas_core::write_config(&config_path, &config)?;
    info!("wrote config to {}", config_path.display());
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
