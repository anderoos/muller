
mod agent;
mod auth;
mod cli;
mod config;

use anyhow::{Context, Result};
use clap::Parser;
use cli::{AutopilotCommand, Command};
use std::path::PathBuf;
use std::process;

// ---------------------------------------------------------------------------
// ChromaDB Docker lifecycle
// ---------------------------------------------------------------------------

const CONTAINER_NAME: &str = "mueller-chromadb";
const CHROMA_IMAGE: &str = "chromadb/chroma:latest";
const CHROMA_VOLUME: &str = "mueller-chroma-data";

/// RAII guard: stops the ChromaDB container when dropped (only if we started it).
struct ChromaGuard {
    started_by_us: bool,
}

impl Drop for ChromaGuard {
    fn drop(&mut self) {
        if self.started_by_us {
            let _ = process::Command::new("docker")
                .args(["stop", CONTAINER_NAME])
                .output();
        }
    }
}

/// Checks whether the named container is currently running.
fn chroma_is_running() -> bool {
    process::Command::new("docker")
        .args(["ps", "-q", "-f", &format!("name={}", CONTAINER_NAME)])
        .output()
        .map(|o| !String::from_utf8_lossy(&o.stdout).trim().is_empty())
        .unwrap_or(false)
}

/// Starts the ChromaDB Docker container (no-op if already running).
/// Returns a guard that stops the container on drop.
fn start_chromadb() -> ChromaGuard {
    if chroma_is_running() {
        return ChromaGuard { started_by_us: false };
    }

    let status = process::Command::new("docker")
        .args([
            "run", "-d", "--rm",
            "--name", CONTAINER_NAME,
            "-p", "8000:8000",
            "-v", &format!("{}:/chroma/chroma", CHROMA_VOLUME),
            CHROMA_IMAGE,
        ])
        .status();

    match status {
        Ok(s) if s.success() => ChromaGuard { started_by_us: true },
        _ => {
            eprintln!("Warning: could not start ChromaDB container.");
            ChromaGuard { started_by_us: false }
        }
    }
}

// ---------------------------------------------------------------------------
// Embedding script helpers
// ---------------------------------------------------------------------------

fn scripts_dir() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let p = parent.join("scripts");
            if p.exists() {
                return p;
            }
        }
    }
    PathBuf::from("scripts")
}

fn methodology_cache_path() -> PathBuf {
    PathBuf::from("methodology/.embeddings_cache.json")
}

/// Calls the Python embedding script. Pass `force = true` to bypass hash checks.
fn run_embed_script(force: bool) -> Result<()> {
    let script = scripts_dir().join("embed_methodology.py");
    if !script.exists() {
        anyhow::bail!(
            "Embedding script not found at {}",
            script.display()
        );
    }

    let mut cmd = process::Command::new("python3");
    cmd.arg(&script);
    if force {
        cmd.arg("--force");
    }

    let cfg = config::load_config();
    match cfg.embedding {
        Some(ref emb) => {
            let env_var = match emb.provider {
                config::EmbeddingProvider::Anthropic  => "ANTHROPIC_API_KEY",
                config::EmbeddingProvider::OpenAI     => "OPENAI_API_KEY",
                config::EmbeddingProvider::OpenRouter => "OPENROUTER_API_KEY",
            };
            cmd.env(env_var, &emb.api_key);
            cmd.env("MUELLER_EMBEDDING_PROVIDER", match emb.provider {
                config::EmbeddingProvider::Anthropic  => "anthropic",
                config::EmbeddingProvider::OpenAI     => "openai",
                config::EmbeddingProvider::OpenRouter => "openrouter",
            });
        }
        None => {
            eprintln!("No embedding API configured. Run `mueller setup` to add one.");
            return Ok(());
        }
    }

    let output = cmd
        .output()
        .context("Could not run embed_methodology.py — is python3 installed?")?;

    print!("{}", String::from_utf8_lossy(&output.stdout));

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Embedding script failed: {}", stderr.trim());
    }

    Ok(())
}

/// Runs an initial embedding pass if no cache file exists yet (first-time setup).
fn maybe_bootstrap_embeddings() {
    if !methodology_cache_path().exists() {
        println!("First run: bootstrapping methodology embeddings…");
        if let Err(e) = run_embed_script(false) {
            eprintln!("Warning: initial embedding failed: {}", e);
        }
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    // Bring ChromaDB up; the guard tears it down when main returns.
    let _chroma = start_chromadb();

    // Auto-embed on first run (no-op afterwards unless --refresh-embeddings).
    maybe_bootstrap_embeddings();

    // Handle --refresh-embeddings before any subcommand.
    if cli.refresh_embeddings {
        println!("Refreshing methodology embeddings…");
        run_embed_script(false)?;
        println!("Done.");
        return Ok(());
    }

    match cli.command {
        Some(Command::Login) => {
            auth::login().await?;
            let cfg = config::run_setup()?;
            config::save_config(&cfg)?;
            println!("✓ All set. Run any Mueller command to get started.");
        }

        Some(Command::Setup) => {
            let cfg = config::run_setup()?;
            config::save_config(&cfg)?;
            println!("✓ Configuration updated.");
        }

        Some(Command::Ask { query }) => {
            agent::run(&query, Some("ASKS")).await?;
        }

        Some(Command::Init { brief }) => {
            let prompt = format!(
                "Start a new project from the following brief. Break it down into epics and \
                tickets, assign owners based on role and capacity, and produce a full Jira \
                structure ready for sprint planning.\n\nBrief: {}",
                brief
            );
            agent::run(&prompt, Some("pre-project/project-initiator")).await?;
        }

        Some(Command::Update { ticket }) => {
            let prompt = format!(
                "Update the following ticket. Apply any status changes, append meeting notes, \
                adjust scope, or reassign as described.\n\nTicket: {}",
                ticket
            );
            agent::run(&prompt, Some("ASKS")).await?;
        }

        Some(Command::Standup) => {
            let prompt = "Trigger the daily standup relay. Prompt each team member for progress, \
                capture blockers, and update Jira ticket statuses based on responses.";
            agent::run(prompt, Some("active-sprint")).await?;
        }

        Some(Command::Health) => {
            let prompt = "Run a sprint health check. Analyse current ticket statuses, \
                cross-ticket dependencies, and velocity. Return an on-track / at-risk / \
                off-rails verdict per flagged item with a specific recommended action.";
            agent::run(prompt, Some("active-sprint/sprint-health-check")).await?;
        }

        Some(Command::Log { file }) => {
            let prompt = match file {
                Some(path) => format!(
                    "Transcribe the meeting from the following file, extract action items, \
                    decisions, and takeaways, and push tasks to Jira.\n\nFile: {}",
                    path
                ),
                None => "Transcribe the meeting transcript provided, extract action items, \
                    decisions, and takeaways, and push tasks to Jira. Paste the transcript now."
                    .to_string(),
            };
            agent::run(&prompt, Some("meetings/meeting-note-transcriber")).await?;
        }

        Some(Command::Brief) => {
            let prompt = "Generate a pre-meeting brief for the next calendar event. Surface \
                open blockers relevant to attendees, tickets last discussed by the group, \
                outstanding decisions from the prior session, and a suggested agenda.";
            agent::run(prompt, Some("meetings/pre-meeting-briefer")).await?;
        }

        Some(Command::Scan) => {
            let prompt = "Scan the active sprint for drift, stale tickets, slippage patterns, \
                and accountability gaps. Flag any tickets that are off-track with a severity \
                rating and recommended action.";
            agent::run(prompt, Some("risk-evaluation")).await?;
        }

        Some(Command::Summarize) => {
            let prompt = "Generate a plain-English summary of the current project. Cover the \
                timeline, key events, decisions made, and current status. Make it readable \
                for both technical and non-technical audiences.";
            agent::run(prompt, Some("communication/summarizer")).await?;
        }

        Some(Command::Onboard { member }) => {
            let prompt = format!(
                "Generate a detailed onboarding brief for a new team member joining the project \
                mid-flight. Cover project context, past key decisions, open blockers, their \
                assigned tickets, and the team roster. Deliver to Slack.\n\nNew member: {}",
                member
            );
            agent::run(&prompt, Some("communication/new-member-onboarder")).await?;
        }

        Some(Command::Close) => {
            let prompt = "Close the project. Schedule the retrospective, notify stakeholders, \
                archive the Jira board, and trigger the knowledge transfer and estimation \
                feedback loop.";
            agent::run(prompt, Some("project-close/project-terminator")).await?;
        }

        Some(Command::Autopilot { command }) => {
            let directive = match command {
                AutopilotCommand::Add { behavior } => format!("add {}", behavior),
                AutopilotCommand::Override { behavior } => format!("override {}", behavior),
                AutopilotCommand::Less { behavior } => format!("less {}", behavior),
            };
            agent::append_autopilot_directive(&directive)?;
            println!("Autopilot directive saved: {}", directive);
        }

        None => match cli.query {
            Some(query) => agent::run(&query, None).await?,
            None => {
                println!("Mueller — Your personal AI project management agent\n");
                println!("Usage:");
                println!("  mueller \"quick question\"");
                println!("  mueller ask \"question\"");
                println!("  mueller login");
                println!("  mueller --refresh-embeddings");
            }
        },
    }

    Ok(())
}
