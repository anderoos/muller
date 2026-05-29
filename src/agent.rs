use std::path::PathBuf;
use anyhow::{Context, Result};
use crate::auth;
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
    let base = skills_dir(); // Get the base skills directory path.

    // Try flat file first: skills/NAME.md
    let flat = base.join(format!("{}.md", name));
    // Check if the flat file path exists before trying to read it.
    if flat.exists() {
        // `std::fs::read_to_string` reads the entire file into a `String`.
        // `with_context` attaches a lazy error message (the closure only runs on error).
        return std::fs::read_to_string(&flat)
            .with_context(|| format!("Could not load skill '{}'", name));
    }

    // Fall back to a nested path: skills/NAME/SKILLS.md
    let nested = base.join(name).join("SKILLS.md");
    // If this also fails, the `?` in the caller will propagate the error upward.
    std::fs::read_to_string(&nested)
        .with_context(|| format!("Could not load skill '{}' from {}", name, nested.display()))
    // Note: no `return` or `;` — in Rust, the last expression in a function is its return value.
    // The semicolon would make it a statement returning `()`, which would be a type error here.
}

// Appends a new directive line to the autopilot skill file so it persists across runs.
pub fn append_autopilot_directive(directive: &str) -> Result<()> {
    let path = skills_dir().join("autopilot").join("SKILLS.md");
    // Read the existing file into an owned, mutable `String`.
    let mut content = std::fs::read_to_string(&path)
        .with_context(|| format!("Could not read autopilot skill from {}", path.display()))?;
    // `push_str` mutates the `String` in place — appends a `&str` without reallocating if there's capacity.
    content.push_str(&format!("\n- {}", directive));
    // Write the modified string back to disk, overwriting the file.
    std::fs::write(&path, content)
        .with_context(|| "Could not write autopilot directive")?;
    Ok(())
}

// `async fn` — this function is non-blocking. It must be `.await`-ed by the caller.
// `prompt: &str` — borrowed string slice; the caller keeps ownership of the prompt text.
// `skill: Option<&str>` — optionally load a named skill file to inject into the system prompt.
pub async fn run(prompt: &str, skill: Option<&str>) -> Result<()> {
    // `to_string()` copies the `&str` constant into a new heap-allocated `String`
    // so we can push more text onto it below.
    let mut system_prompt = SYSTEM_PROMPT.to_string();

    // Always load and prepend the autopilot directives if the file exists.
    // `if let Ok(autopilot) = ...` — pattern match on Result; skip silently on error.
    if let Ok(autopilot) = load_skills("autopilot") {
        system_prompt.push_str("\n\n"); // Separate sections with blank lines.
        system_prompt.push_str(&autopilot); // Append the autopilot skill file content.
    }

    // If a skill name was provided, load and append that skill's context too.
    // `if let Some(skill_name) = skill` — unwraps the Option; `skill_name` is a `&str`.
    if let Some(skill_name) = skill {
        if let Ok(skill_content) = load_skills(skill_name) {
            system_prompt.push_str("\n\n");
            system_prompt.push_str(&skill_content);
        }
    }

    // `std::process::Command` is a builder for spawning child processes.
    // Start building a command that will run the `claude` CLI binary.
    let mut cmd = std::process::Command::new("claude");
    // `.arg()` appends one CLI argument. Each call returns `&mut Command` so we can chain.
    // `-p` is the "print" (non-interactive) mode flag for the Claude CLI.
    cmd.arg("-p").arg(prompt).arg("--system-prompt").arg(&system_prompt);

    // If the user has logged in, inject their API key as an environment variable.
    // `if let Ok(creds) = ...` — only runs if credentials exist; silently skips if not logged in.
    if let Ok(creds) = auth::load_credentials() {
        // `.env(key, value)` sets an environment variable for the child process only.
        cmd.env("ANTHROPIC_API_KEY", &creds.access_token);
    }

    // Load the full config — both Jira and Slack settings live here.
    let cfg = config::load_config();

    // When Slack is configured, append a standing instruction so Claude knows it can
    // post its output there. The `slack_post_message` tool becomes available via MCP.
    if let Some(slack) = &cfg.slack {
        // Build a human-readable destination string based on the delivery mode.
        // `match` on an enum reference — `&config::DeliveryMode::Channel` vs `&config::DeliveryMode::DirectMessage`.
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

    // `write_mcp_config` now returns `Result<Option<PathBuf>>`:
    //   Ok(Some(path)) — at least one server configured, file written
    //   Ok(None)       — nothing configured, no file written
    //   Err(e)         — IO error writing the file
    match config::write_mcp_config(&cfg) {
        Ok(Some(path)) => { cmd.arg("--mcp-config").arg(&path); }
        Ok(None) => eprintln!("Tip: run `mueller setup` to connect Jira and Slack."),
        Err(e) => eprintln!("Warning: could not write MCP config: {}", e),
    }

    // `.output()` spawns the child process, waits for it to finish, and captures
    // stdout + stderr as `Vec<u8>` byte buffers. This blocks the async task.
    let output = cmd.output().context("Could not run `claude`. Is Claude Code installed?")?;

    // Check the child process exit code. A non-zero code signals failure.
    if !output.status.success() {
        // `String::from_utf8_lossy` converts raw bytes to a string view, replacing
        // any invalid UTF-8 sequences with the replacement character `\u{FFFD}`.
        let stderr = String::from_utf8_lossy(&output.stderr);
        // `anyhow::bail!` is shorthand for `return Err(anyhow::anyhow!(...))`.
        anyhow::bail!("claude failed: {}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    // `.trim()` on a `Cow<str>` (what `from_utf8_lossy` returns) removes surrounding whitespace.
    println!("{}", stdout.trim());

    Ok(())
}
