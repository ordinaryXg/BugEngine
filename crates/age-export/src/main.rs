use std::path::PathBuf;

use age_core::project::Project;
use age_export::{all_targets, ExportTargetId};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "age-export", about = "AgentGameEngine export pipeline")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Export project to HTML bundle
    Export {
        /// Project directory
        project: PathBuf,
        /// Output directory
        #[arg(short, long, default_value = "dist/html")]
        out: PathBuf,
        /// Export target: html, desktop, mobile
        #[arg(short, long, default_value = "html")]
        target: String,
    },
    /// List available export targets
    Targets,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Export {
            project,
            out,
            target,
        } => {
            let project = Project::load(&project)?;
            let target_id = match target.as_str() {
                "html" => ExportTargetId::Html,
                "desktop" => ExportTargetId::Desktop,
                "mobile" => ExportTargetId::Mobile,
                other => {
                    eprintln!("unknown target: {other}");
                    std::process::exit(1);
                }
            };

            let export_target = all_targets()
                .into_iter()
                .find(|t| t.id == target_id)
                .ok_or("target not found")?;

            let result = export_target.build(&project, &out)?;
            if result.ok {
                println!(
                    "Export succeeded: {}",
                    result.artifact_path.unwrap().display()
                );
            } else {
                eprintln!("Export failed: {}", result.error.unwrap_or_default());
                std::process::exit(1);
            }
        }
        Commands::Targets => {
            for target in all_targets() {
                println!("{} [{:?}] - {}", target.label, target.status, format!("{:?}", target.id));
            }
        }
    }
    Ok(())
}

// Library re-export for programmatic use
