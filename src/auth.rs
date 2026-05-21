use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OAuthTokens {
    #[serde(rename = "accessToken")]
    pub access_token: String,
    #[serde(rename = "refreshToken")]
    pub refresh_token: Option<String>,
    #[serde(rename = "expiresAt")]
    pub expires_at: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CredentialsFile {
    #[serde(rename = "claudeAiOauth")]
    claude_ai_oauth: OAuthTokens,
}

pub fn credentials_path() -> PathBuf {
    dirs::home_dir()
        .expect("Could not find home directory")
        .join(".mueller")
        .join("credentials.json")
}

pub fn save_credentials(tokens: &OAuthTokens) -> Result<PathBuf> {
    let path = credentials_path();
    std::fs::create_dir_all(path.parent().unwrap())?;

    let file = CredentialsFile {
        claude_ai_oauth: tokens.clone(),
    };
    let json = serde_json::to_string_pretty(&file)?;
    std::fs::write(&path, json)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    }

    Ok(path)
}

pub fn load_credentials() -> Result<OAuthTokens> {
    let path = credentials_path();
    let content = std::fs::read_to_string(&path)
        .context("Not logged in. Run `mueller login` first.")?;
    let file: CredentialsFile = serde_json::from_str(&content)?;
    Ok(file.claude_ai_oauth)
}

pub async fn login() -> Result<()> {
    println!("Logging in via Claude...");

    let output = std::process::Command::new("claude")
        .arg("setup-token")
        .output()
        .context("Could not run `claude`. Is Claude Code installed? Run: npm install -g @anthropic-ai/claude-code")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("claude setup-token failed: {}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    let token = stdout
        .lines()
        .find(|line| line.trim().starts_with("sk-ant-oat01-"))
        .context("Could not find token in claude setup-token output")?
        .trim()
        .to_string();

    let tokens = OAuthTokens {
        access_token: token,
        refresh_token: None,
        expires_at: None,
    };

    let path = save_credentials(&tokens)?;
    println!("✓ Logged in! Credentials saved to {}", path.display());

    Ok(())
}