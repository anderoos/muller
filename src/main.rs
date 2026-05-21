mod auth;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "mueller")]
#[command(about = "An AI research agent")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// Ask a quick question directly
    query: Option<String>,
}

#[derive(Subcommand)]
enum Command {
    /// Authenticate with Claude
    Login,

    /// Ask a quick question
    Ask {
        query: String,
    },

    /// Deep research on a topic
    Research {
        topic: String,
    },

    /// Literature review on a topic
    Lit {
        topic: String,
    },
}

async fn run_claude(prompt: &str) -> Result<()> {
    let output = std::process::Command::new("claude")
        .arg("-p")
        .arg(prompt)
        .output()
        .context("Could not run `claude`. Is Claude Code installed?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("claude failed: {}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("{}", stdout.trim());

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Login) => {
            auth::login().await?;
        }

        Some(Command::Ask { query }) => {
            run_claude(&query).await?;
        }

        Some(Command::Research { topic }) => {
            let prompt = format!(
                "You are a research agent. Conduct a thorough investigation of the following topic. \
                Search for key findings, cite sources, summarize consensus and open questions.\n\nTopic: {}",
                topic
            );
            run_claude(&prompt).await?;
        }

        Some(Command::Lit { topic }) => {
            let prompt = format!(
                "You are a research agent conducting a literature review. For the following topic, \
                identify key papers, summarize their contributions, note areas of consensus and disagreement, \
                and highlight open questions.\n\nTopic: {}",
                topic
            );
            run_claude(&prompt).await?;
        }

        // No subcommand — treat bare query as a quick ask
        None => match cli.query {
            Some(query) => run_claude(&query).await?,
            None => {
                println!("Mueller — AI Research Agent\n");
                println!("Usage:");
                println!("  mueller \"quick question\"");
                println!("  mueller ask \"question\"");
                println!("  mueller research \"topic\"");
                println!("  mueller lit \"topic\"");
                println!("  mueller login");
            }
        },
    }

    Ok(())
}