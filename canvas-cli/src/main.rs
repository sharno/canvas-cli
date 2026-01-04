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
    Auth,
    /// Course-related operations
    Courses {
        /// Optional course id to demonstrate parsing at the edge
        #[arg(long)]
        course_id: Option<String>,
    },
}

fn main() -> Result<(), canvas_core::CanvasError> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Command::Auth => {
            info!("auth placeholder");
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