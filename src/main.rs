# [allow(dead_code)]
mod auth;

use anyhow::{Context, Result};

async fn chat(message: &str) -> Result<()> {
    let output = std::process::Command::new("claude")
        .arg("-p")
        .arg(message)
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
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(String::as_str) {
        Some("login") => {
            auth::login().await?;
        }
        Some(message) => {
            chat(message).await?;
        }
        None => {
            println!("Usage:");
            println!("  mueller login          — authenticate with Claude");
            println!("  mueller \"your query\"   — ask Claude something");
        }
    }

    Ok(())
}