use std::path::PathBuf;
use anyhow::{Context, Result};
use crate::auth;

const SYSTEM_PROMPT: &str = "\
You are Mueller, an expert AI project management agent. Your role is to work across
cross-functional teams, coordinate, propose, plan and execute projects while resolving resource
constraints to ensure optimal project performance for any brief you are given.

You approach every task with the following principles:
- Accuracy first: never speculate beyond what is mentioned in the brief, ticket, or meeting notes.
- If there is uncertainty, ask the user for clarification.
- Reference the Jira ticket whenever possible.
- Keep outputs clear, concise and task oriented-- no jargon.
- Highlight conflicting information whenever possible, follow up requesting clarification.

";

fn skills_dir() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let release_path = parent.join("skills");
            if release_path.exists() {
                return release_path;
            }
        }
    }
    PathBuf::from("skills")
}

pub fn load_skills(name: &str) -> Result<String> {
    let base = skills_dir();

    // Try flat file first: skills/NAME.md
    let flat = base.join(format!("{}.md", name));
    if flat.exists() {
        return std::fs::read_to_string(&flat)
            .with_context(|| format!("Could not load skill '{}'", name));
    }

    // Fall back to subdirectory: skills/NAME/SKILLS.md
    let nested = base.join(name).join("SKILLS.md");
    std::fs::read_to_string(&nested)
        .with_context(|| format!("Could not load skill '{}' from {}", name, nested.display()))
}

pub fn append_autopilot_directive(directive: &str) -> Result<()> {
    let path = skills_dir().join("autopilot").join("SKILLS.md");
    let mut content = std::fs::read_to_string(&path)
        .with_context(|| format!("Could not read autopilot skill from {}", path.display()))?;
    content.push_str(&format!("\n- {}", directive));
    std::fs::write(&path, content)
        .with_context(|| "Could not write autopilot directive")?;
    Ok(())
}

pub async fn run(prompt: &str, skill: Option<&str>) -> Result<()> {
    // Build system prompt: base + autopilot directives + optional skill context
    let mut system_prompt = SYSTEM_PROMPT.to_string();

    if let Ok(autopilot) = load_skills("autopilot") {
        system_prompt.push_str("\n\n");
        system_prompt.push_str(&autopilot);
    }

    if let Some(skill_name) = skill {
        if let Ok(skill_content) = load_skills(skill_name) {
            system_prompt.push_str("\n\n");
            system_prompt.push_str(&skill_content);
        }
    }

    let mut cmd = std::process::Command::new("claude");
    cmd.arg("-p").arg(prompt).arg("--system-prompt").arg(&system_prompt);

    if let Ok(creds) = auth::load_credentials() {
        cmd.env("ANTHROPIC_API_KEY", &creds.access_token);
    }

    let output = cmd.output().context("Could not run `claude`. Is Claude Code installed?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("claude failed: {}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("{}", stdout.trim());

    Ok(())
}
