
mod agent;
mod auth;
mod cli;
mod config;
mod interface;
mod observability;

use anyhow::{Context, Result};
use clap::Parser;
use cli::{AutopilotCommand, Command};
use interface::cli_adapter;
use std::path::PathBuf;
use std::process;

/// If --dump-payload is set, serialise the payload as JSON and exit.
/// Used by the Python integration test suite to inspect the interface layer
/// without running the full agent pipeline.
macro_rules! maybe_dump_payload {
    ($flag:expr, $payload:expr) => {
        if $flag {
            println!("{}", serde_json::to_string_pretty(&$payload)?);
            return Ok(());
        }
    };
}

/// Blocks write commands in dev builds with a clear error.
/// Expands to a no-op in release builds (optimized away entirely).
macro_rules! write_only_guard {
    ($cmd:expr) => {
        if cfg!(debug_assertions) {
            eprintln!(
                "\x1b[31mError:\x1b[0m '{}' is a write command and is not available in \
                dev builds.\nBuild with \x1b[33mcargo build --release\x1b[0m to enable \
                Jira write operations.",
                $cmd
            );
            return Ok(());
        }
    };
}

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

    let output = process::Command::new("docker")
        .args([
            "run", "-d", "--rm",
            "--name", CONTAINER_NAME,
            "-p", "8000:8000",
            "-v", &format!("{}:/chroma/chroma", CHROMA_VOLUME),
            CHROMA_IMAGE,
        ])
        .output();

    match output {
        Ok(o) if o.status.success() => ChromaGuard { started_by_us: true },
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
        // ── housekeeping commands (no agent dispatch) ────────────────────────
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

        Some(Command::Dashboard) => {
            let script = scripts_dir().join("observability.py");
            if !script.exists() {
                anyhow::bail!("Observability script not found at {}", script.display());
            }
            let status = process::Command::new("python3")
                .arg(&script)
                .status()
                .context("Could not run observability.py — is python3 installed?")?;
            if let Some(code) = status.code() {
                if code != 0 {
                    anyhow::bail!(
                        "Observability server exited with code {}. If dependencies are missing, run:\n  \
                        pip install -r {}",
                        code,
                        scripts_dir().join("requirements.txt").display()
                    );
                }
            }
        }

        Some(Command::Autopilot { command }) => {
            let directive = match command {
                AutopilotCommand::Add { behavior }      => format!("add {}", behavior),
                AutopilotCommand::Override { behavior } => format!("override {}", behavior),
                AutopilotCommand::Less { behavior }     => format!("less {}", behavior),
            };
            agent::append_autopilot_directive(&directive)?;
            println!("Autopilot directive saved: {}", directive);
        }

        // ── Init: resolve brief path before handing to the interface layer ──
        Some(Command::Init { ref brief }) => {
            write_only_guard!("init");
            let resolved_brief = {
                let path = std::path::Path::new(brief.as_str());
                if path.exists() {
                    let abs = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
                    format!(
                        "Read the file at this absolute path and use its contents as the \
                        project brief: {}",
                        abs.display()
                    )
                } else {
                    brief.clone()
                }
            };
            let effective_cmd = Command::Init { brief: resolved_brief };
            let payload = cli_adapter::from_command(&effective_cmd);
            maybe_dump_payload!(cli.dump_payload, payload);
            agent::run_payload(&payload).await?;
        }

        // ── all remaining agent commands ─────────────────────────────────────
        Some(ref cmd) => {
            let payload = cli_adapter::from_command(cmd);
            maybe_dump_payload!(cli.dump_payload, payload);
            if payload.task_type.is_write() {
                write_only_guard!(payload.task_type.as_str());
            }
            agent::run_payload(&payload).await?;
        }

        None => match cli.query {
            Some(query) => {
                let payload = cli_adapter::from_raw_query(&query);
                maybe_dump_payload!(cli.dump_payload, payload);
                agent::run_payload(&payload).await?;
            }
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
