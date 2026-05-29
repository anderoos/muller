use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use crate::config;

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

// ---------------------------------------------------------------------------
// Skills
// ---------------------------------------------------------------------------

fn skills_dir() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let p = parent.join("skills");
            if p.exists() { return p; }
        }
    }
    PathBuf::from("skills")
}

pub fn load_skills(name: &str) -> Result<String> {
    let base = skills_dir();
    let flat = base.join(format!("{}.md", name));
    if flat.exists() {
        return std::fs::read_to_string(&flat)
            .with_context(|| format!("Could not load skill '{}'", name));
    }
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

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

fn build_cmd(prompt: &str, system_prompt: &str, mcp_path: Option<&Path>) -> std::process::Command {
    let mut cmd = std::process::Command::new("claude");
    cmd.arg("-p").arg(prompt).arg("--system-prompt").arg(system_prompt);
    if let Some(p) = mcp_path {
        cmd.arg("--mcp-config").arg(p);
    }
    cmd
}

fn run_claude(cmd: &mut std::process::Command) -> Result<String> {
    let output = cmd.output().context("Could not run `claude`. Is Claude Code installed?")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let detail = match (stderr.trim().is_empty(), stdout.trim().is_empty()) {
            (false, _)    => stderr.trim().to_string(),
            (true, false) => stdout.trim().to_string(),
            (true, true)  => format!("exit code {}", output.status),
        };
        anyhow::bail!("claude failed: {}", detail);
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

// ---------------------------------------------------------------------------
// Release-only helpers
// ---------------------------------------------------------------------------

fn save_plan(content: &str) -> Result<PathBuf> {
    let dir = dirs::home_dir()
        .context("Could not find home directory")?
        .join(".mueller")
        .join("plans");
    std::fs::create_dir_all(&dir)?;

    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let path = dir.join(format!("plan-{}.md", ts));
    std::fs::write(&path, content)?;
    Ok(path)
}

fn confirm(prompt: &str) -> Result<bool> {
    use std::io::{self, Write};
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(matches!(input.trim().to_lowercase().as_str(), "y" | "yes"))
}

// ---------------------------------------------------------------------------
// Execution: plan → save → confirm → execute
// ---------------------------------------------------------------------------

fn run_release(prompt: &str, system_prompt: &str, mcp_path: Option<&Path>) -> Result<()> {
    // Phase 1: generate a structured markdown plan without touching any tools.
    let plan_system = format!(
        "{}\n\n\
        IMPORTANT: Output ONLY a structured markdown plan — do not call any tools.\n\
        Use this hierarchy:\n\
        # <Project Name>\n\
        ## Epic: <name>\n\
        ### Story: <name>\n\
        - [ ] Task: <description>\n\
        Cover all epics, stories, and tasks derived from the brief.",
        system_prompt
    );
    let mut plan_cmd = build_cmd(prompt, &plan_system, None);
    let plan = run_claude(&mut plan_cmd)?;

    let md_path = save_plan(&plan)?;

    println!("✓ Plan prepared — review before proceeding.");
    println!("  {}\n", md_path.display());

    if !confirm("\x1b[33mReview the plan above. Proceed with execution? [y/N]\x1b[0m ")? {
        println!("Cancelled. Plan retained at: {}", md_path.display());
        return Ok(());
    }

    // Phase 2: execute — Claude now has MCP access and will push to Jira/Slack.
    println!("\nExecuting…");
    let mut exec_cmd = build_cmd(prompt, system_prompt, mcp_path);
    let result = run_claude(&mut exec_cmd)?;
    println!("{}", result);

    Ok(())
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

pub async fn run(prompt: &str, skill: Option<&str>) -> Result<()> {
    // Build the complete system prompt before touching the command builder.
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

    let cfg = config::load_config();

    if let Some(slack) = &cfg.slack {
        let destination = match &slack.delivery_mode {
            config::DeliveryMode::Channel => format!("the #{} channel", slack.destination),
            config::DeliveryMode::DirectMessage => "the user via direct message".to_string(),
        };
        system_prompt.push_str(&format!(
            "\n\nSlack relay: when you have a final answer or summary to deliver, \
            also post it to {} using the slack_post_message tool with channel \"{}\".",
            destination, slack.destination
        ));
    }

    let mcp_path: Option<PathBuf> = match config::write_mcp_config(&cfg) {
        Ok(Some(path)) => Some(path),
        Ok(None)       => { eprintln!("Tip: run `mueller setup` to connect Jira and Slack."); None }
        Err(e)         => { eprintln!("Warning: could not write MCP config: {}", e); None }
    };

    run_release(prompt, &system_prompt, mcp_path.as_deref())?;

    Ok(())
}
