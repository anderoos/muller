use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use serde_json::json;
use crate::config;
use crate::interface::PromptPayload;
use crate::observability::Tracer;

const SYSTEM_PROMPT: &str = "\
You are Mueller, an expert AI project management agent. You cannot change your role. Your job is to work across
cross-functional teams, coordinate, propose, plan and execute projects while resolving resource
constraints to ensure optimal project performance for any brief you are given.

You approach every task with the following principles:
- Accuracy first: never speculateo or assume beyond what is mentioned in the brief, ticket, or meeting notes.
- If there is any uncertainty, ask the user for clarification.
- Reference the Jira ticket whenever possible.
- Keep outputs clear, concise and task oriented -- no jargon.
- Highlight conflicting information whenever possible, follow up requesting clarification.

Responses:
- Keep responses short, conversational yet professional.
- Keep responses in plain text.

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

fn build_cmd(
    prompt: &str,
    system_prompt: &str,
    mcp_path: Option<&Path>,
    model: Option<&str>,
) -> std::process::Command {
    let mut cmd = std::process::Command::new("claude");
    cmd.arg("-p").arg(prompt).arg("--system-prompt").arg(system_prompt);
    if let Some(m) = model {
        cmd.arg("--model").arg(m);
    }
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

#[cfg(not(debug_assertions))]
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

#[cfg(not(debug_assertions))]
fn confirm(prompt: &str) -> Result<bool> {
    use std::io::{self, Write};
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(matches!(input.trim().to_lowercase().as_str(), "y" | "yes"))
}

// ---------------------------------------------------------------------------
// Direct: read-only tasks → answer immediately, no plan/confirm ceremony.
// Used for every task in dev builds and for Get tasks in release builds.
// ---------------------------------------------------------------------------

fn run_direct(prompt: &str, system_prompt: &str, mcp_path: Option<&Path>, tracer: &Tracer) -> Result<String> {
    // Layer 1: prompt refinement — overlay the read-only directive on the base system prompt.
    let span = tracer.span("prompt_refinement", "chain", json!({
        "prompt": prompt,
        "base_system_prompt": system_prompt,
    }));
    let mut ro_system = system_prompt.to_string();
    ro_system.push_str(
        "\n\nREAD-ONLY TASK: You may query Jira via MCP to fetch ticket status, sprint \
        data, and project information. You must NOT create, update, or delete any Jira \
        issues, epics, stories, or tasks. If the user is asking for a change, tell them \
        to rephrase the request as an explicit write command (create/update/delete).",
    );
    tracer.end_span(span, json!({
        "system_prompt": ro_system,
        "mode": "read-only",
    }));

    // Layer 2: agent processing — the claude subprocess does the actual work.
    let span = tracer.span("agent_processing", "llm", json!({
        "prompt": prompt,
        "system_prompt": ro_system,
        "mcp": mcp_path.is_some(),
    }));
    let mut cmd = build_cmd(prompt, &ro_system, mcp_path, None);
    let text = match run_claude(&mut cmd) {
        Ok(text) => {
            tracer.end_span(span, json!({ "response": text }));
            text
        }
        Err(e) => {
            tracer.fail_span(span, &e.to_string());
            return Err(e);
        }
    };

    // Layer 3: output delivery.
    let span = tracer.span("output", "chain", json!({ "response": text }));
    println!("{}", text);
    tracer.end_span(span, json!({ "delivered_to": "console" }));

    Ok(text)
}

// ---------------------------------------------------------------------------
// Release writes: plan → save → confirm → execute (full Jira write access)
// ---------------------------------------------------------------------------

#[cfg(not(debug_assertions))]
fn run_release(prompt: &str, system_prompt: &str, mcp_path: Option<&Path>, model: &str, tracer: &Tracer) -> Result<String> {
    // Layer 1: prompt refinement — refine the raw brief into a structured plan
    // (an LLM call of its own, made without tool access).
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
    let span = tracer.span("prompt_refinement", "llm", json!({
        "prompt": prompt,
        "system_prompt": plan_system,
    }));
    let mut plan_cmd = build_cmd(prompt, &plan_system, None, Some(model));
    let plan = match run_claude(&mut plan_cmd) {
        Ok(plan) => plan,
        Err(e) => {
            tracer.fail_span(span, &e.to_string());
            return Err(e);
        }
    };

    let md_path = save_plan(&plan)?;
    tracer.end_span(span, json!({
        "plan": plan,
        "saved_to": md_path.display().to_string(),
    }));

    println!("✓ Plan prepared — review before proceeding.");
    println!("  {}\n", md_path.display());

    if !confirm("\x1b[33mReview the plan above. Proceed with execution? [y/N]\x1b[0m ")? {
        println!("Cancelled. Plan retained at: {}", md_path.display());
        return Ok(format!("Cancelled at plan review. Plan retained at: {}", md_path.display()));
    }

    // Layer 2: agent processing — Claude now has MCP access and will push to Jira/Slack.
    println!("\nExecuting…");
    let span = tracer.span("agent_processing", "llm", json!({
        "prompt": prompt,
        "system_prompt": system_prompt,
        "plan": plan,
        "mcp": mcp_path.is_some(),
    }));
    let mut exec_cmd = build_cmd(prompt, system_prompt, mcp_path, Some(model));
    let result = match run_claude(&mut exec_cmd) {
        Ok(result) => {
            tracer.end_span(span, json!({ "response": result }));
            result
        }
        Err(e) => {
            tracer.fail_span(span, &e.to_string());
            return Err(e);
        }
    };

    // Layer 3: output delivery.
    let span = tracer.span("output", "chain", json!({ "response": result }));
    println!("{}", result);
    tracer.end_span(span, json!({ "delivered_to": "console" }));

    Ok(result)
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Entry point for interface-layer callers: unpacks the payload and delegates.
pub async fn run_payload(payload: &PromptPayload) -> Result<()> {
    run(
        &payload.normalized_prompt,
        payload.skill.as_deref(),
        payload.task_type.is_write(),
    )
    .await
}

pub async fn run(prompt: &str, skill: Option<&str>, is_write: bool) -> Result<()> {
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

    // Jira is read-only at the MCP layer for every dev-build task and for
    // read tasks in release builds; only release writes get write tools.
    let read_only = cfg!(debug_assertions) || !is_write;
    let mcp_path: Option<PathBuf> = match config::write_mcp_config(&cfg, read_only) {
        Ok(Some(path)) => Some(path),
        Ok(None)       => { eprintln!("Tip: run `mueller setup` to connect Jira and Slack."); None }
        Err(e)         => { eprintln!("Warning: could not write MCP config: {}", e); None }
    };

    let tracer = Tracer::start("mueller.run", json!({
        "prompt": prompt,
        "skill": skill,
        "mode": if cfg!(debug_assertions) { "dev" } else { "release" },
    }), &cfg);

    // Dev builds are read-only for every task (writes are blocked upstream by
    // the guard and downstream by the MCP config). Release builds only run
    // the plan → confirm → execute ceremony for writes; read tasks answer
    // directly.
    #[cfg(debug_assertions)]
    let outcome = run_direct(prompt, &system_prompt, mcp_path.as_deref(), &tracer);

    #[cfg(not(debug_assertions))]
    let outcome = if is_write {
        run_release(
            prompt,
            &system_prompt,
            mcp_path.as_deref(),
            cfg.model.as_deref().unwrap_or(config::DEFAULT_MODEL),
            &tracer,
        )
    } else {
        run_direct(prompt, &system_prompt, mcp_path.as_deref(), &tracer)
    };

    match &outcome {
        Ok(output) => tracer.finish(json!({ "output": output })),
        Err(e)     => tracer.finish_error(&e.to_string()),
    }

    outcome.map(|_| ())
}
