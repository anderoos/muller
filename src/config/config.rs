use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct MuellerConfig {
    pub jira: Option<JiraConfig>,
    pub slack: Option<SlackConfig>,
    pub embedding: Option<EmbeddingConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum EmbeddingProvider {
    Anthropic,
    OpenAI,
    OpenRouter,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmbeddingConfig {
    pub provider: EmbeddingProvider,
    pub api_key: String,
}

// `pub enum` with two unit variants — the user either wants channel posts or DMs.
// `#[derive(Serialize, Deserialize)]` makes serde store this as a JSON string: "Channel" or "DirectMessage".
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DeliveryMode {
    Channel,
    DirectMessage,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SlackConfig {
    pub bot_token: String,
    pub team_id: String,
    // Holds a channel name (e.g. "pm-updates") for Channel mode,
    // or a Slack member ID (e.g. "U01234ABC") for DirectMessage mode.
    // Slack's API accepts both as the `channel` argument to `chat.postMessage`.
    pub destination: String,
    pub delivery_mode: DeliveryMode,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JiraConfig {
    pub url: String,
    pub email: String,
    pub api_token: String,
    pub project_key: String,
}

pub fn config_path() -> PathBuf {
    dirs::home_dir()
        .expect("Could not find home directory")
        .join(".mueller")
        .join("config.json")
}

pub fn load_config() -> MuellerConfig {
    let path = config_path();
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_config(config: &MuellerConfig) -> Result<()> {
    let path = config_path();
    std::fs::create_dir_all(path.parent().unwrap())?;
    let json = serde_json::to_string_pretty(config)?;
    std::fs::write(&path, &json)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    }

    Ok(())
}

pub fn mcp_config_path() -> PathBuf {
    dirs::home_dir()
        .expect("Could not find home directory")
        .join(".mueller")
        .join("mcp-config.json")
}

pub fn write_mcp_config(cfg: &MuellerConfig) -> Result<Option<PathBuf>> {
    let mut servers = serde_json::Map::new();

    if let Some(jira) = &cfg.jira {
        servers.insert("jira".to_string(), serde_json::json!({
            "command": "uvx",
            "args": ["mcp-atlassian"],
            "env": {
                "JIRA_URL": jira.url,
                "JIRA_USERNAME": jira.email,
                "JIRA_API_TOKEN": jira.api_token
            }
        }));
    }

    if let Some(slack) = &cfg.slack {
        servers.insert("slack".to_string(), serde_json::json!({
            "command": "npx",
            "args": ["-y", "@modelcontextprotocol/server-slack"],
            "env": {
                "SLACK_BOT_TOKEN": slack.bot_token,
                "SLACK_TEAM_ID": slack.team_id
            }
        }));
    }

    if servers.is_empty() {
        return Ok(None);
    }

    let path = mcp_config_path();
    let mcp = serde_json::json!({ "mcpServers": serde_json::Value::Object(servers) });
    std::fs::create_dir_all(path.parent().unwrap())?;
    std::fs::write(&path, serde_json::to_string_pretty(&mcp)?)?;
    Ok(Some(path))
}

// ── private input helpers ────────────────────────────────────────────────────

fn prompt(label: &str) -> Result<String> {
    print!("{}: ", label);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn prompt_masked(label: &str) -> Result<String> {
    let token = rpassword::prompt_password(format!("{}: ", label))
        .context("Could not read masked input")?;
    Ok(token.trim().to_string())
}

// Prints a numbered menu and loops until the user enters a valid number.
// `options: &[&str]` — a borrowed slice of string slices; no allocation needed.
// Returns the zero-based index of the chosen option.
fn choose(label: &str, options: &[&str]) -> Result<usize> {
    println!("\n{}", label);
    // `.iter().enumerate()` yields `(index, &item)` pairs — a standard iterator pattern.
    for (i, option) in options.iter().enumerate() {
        // `i + 1` shows 1-based numbers to the user even though our Vec is 0-based internally.
        println!("  [{}] {}", i + 1, option);
    }
    // `loop` runs forever until a `return` or `break` exits it — used here for retry on bad input.
    loop {
        print!("Choice [1-{}]: ", options.len());
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        // `.trim().parse::<usize>()` tries to parse the input as an unsigned integer.
        // `match` with a guard `if n >= 1 && n <= options.len()` validates the range.
        match input.trim().parse::<usize>() {
            Ok(n) if n >= 1 && n <= options.len() => return Ok(n - 1), // convert back to 0-based
            _ => println!("Please enter a number between 1 and {}.", options.len()),
        }
    }
}

// ── setup flows ──────────────────────────────────────────────────────────────

pub fn setup_jira() -> Result<JiraConfig> {
    println!("\nJira Cloud setup");
    println!("Get your API token at: https://id.atlassian.com/manage-profile/security/api-tokens\n");

    let url = prompt("Jira base URL (e.g. https://yourcompany.atlassian.net)")?;
    let email = prompt("Atlassian account email")?;
    let api_token = prompt_masked("API token")?;
    let project_key = prompt("Default project key (e.g. PROJ)")?;

    Ok(JiraConfig {
        url: url.trim_end_matches('/').to_string(),
        email,
        api_token,
        project_key: project_key.to_uppercase(),
    })
}

pub fn setup_slack() -> Result<SlackConfig> {
    println!("\nSlack setup");
    println!("Create a Slack app at https://api.slack.com/apps — required scopes: chat:write, channels:read\n");

    let bot_token = prompt_masked("Bot token (xoxb-...)")?;
    let team_id = prompt("Workspace ID (e.g. T01234ABC — Slack admin → Settings → Workspace ID)")?;

    // Ask how the user wants updates delivered.
    let delivery_choice = choose(
        "How would you like to receive Mueller's output?",
        &[
            "Post to a channel",
            "Direct message to me",
        ],
    )?;

    // `match delivery_choice` on a `usize` — each arm returns a `(String, DeliveryMode)` tuple.
    // Both arms must return the same type for the match expression to type-check.
    let (destination, delivery_mode) = match delivery_choice {
        0 => {
            let channel = prompt("Channel name (e.g. pm-updates)")?;
            // Strip a leading `#` — Slack's API wants the bare name.
            (channel.trim_start_matches('#').to_string(), DeliveryMode::Channel)
        }
        _ => {
            // For DMs, Slack's `chat.postMessage` accepts a member ID as the channel.
            let member_id = prompt("Your Slack member ID (right-click your name → Copy member ID, e.g. U01234ABC)")?;
            (member_id, DeliveryMode::DirectMessage)
        }
    };

    Ok(SlackConfig { bot_token, team_id, destination, delivery_mode })
}

pub fn setup_embedding() -> Result<Option<EmbeddingConfig>> {
    println!("\nEmbedding API (optional)");
    println!("Only required to run `mueller --refresh-embeddings`.");
    println!("This key is stored locally and never transmitted elsewhere.\n");

    let choice = choose(
        "Which provider would you like to use?",
        &[
            "Anthropic  (Claude Opus — recommended)",
            "OpenAI     (GPT-4o)",
            "OpenRouter (route to any model)",
            "Skip",
        ],
    )?;

    if choice == 3 {
        return Ok(None);
    }

    let (provider, hint) = match choice {
        0 => (EmbeddingProvider::Anthropic,  "sk-ant-api03-..."),
        1 => (EmbeddingProvider::OpenAI,     "sk-..."),
        _ => (EmbeddingProvider::OpenRouter, "sk-or-..."),
    };

    let api_key = prompt_masked(&format!("API key ({})", hint))?;
    Ok(Some(EmbeddingConfig { provider, api_key }))
}

// Top-level setup orchestrator called by both `mueller login` and `mueller setup`.
pub fn run_setup() -> Result<MuellerConfig> {
    let mode = choose(
        "How do you want to interact with Mueller?",
        &[
            "CLI — output in the terminal only",
            "Slack — relay output to Slack as well",
        ],
    )?;

    let jira = setup_jira()?;

    let slack = if mode == 1 {
        Some(setup_slack()?)
    } else {
        None
    };

    let embedding = setup_embedding()?;

    Ok(MuellerConfig { jira: Some(jira), slack, embedding })
}
