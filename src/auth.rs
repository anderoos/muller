// `use` imports specific items so we can write `Result` instead of `anyhow::Result`.
use anyhow::{Context, Result};
// `Deserialize` and `Serialize` are traits from the `serde` crate.
// Implementing them lets a struct be converted to/from JSON, TOML, etc.
use serde::{Deserialize, Serialize};
// `PathBuf` is an owned, mutable file-system path — like `String` but for paths.
// It lives on the heap and you can push/pop segments onto it.
use std::path::PathBuf;

// `#[derive(Debug, Serialize, Deserialize, Clone)]` auto-generates four trait implementations:
//   Debug     — lets you print the struct with `{:?}` for debugging
//   Serialize — lets serde convert it to JSON (or other formats)
//   Deserialize — lets serde build it from JSON
//   Clone     — lets you call `.clone()` to make a deep copy
#[derive(Debug, Serialize, Deserialize, Clone)]
// `pub struct` — public so other modules can receive/return this type.
pub struct OAuthTokens {
    // `#[serde(rename = "...")]` tells serde to use "accessToken" in the JSON
    // even though the Rust field is named `access_token` (snake_case is idiomatic Rust).
    #[serde(rename = "accessToken")]
    pub access_token: String,
    // `Option<String>` — the refresh token might not exist; Rust forces you to handle that
    // explicitly rather than allowing a silent null.
    #[serde(rename = "refreshToken")]
    pub refresh_token: Option<String>,
    // `u64` is an unsigned 64-bit integer — appropriate for a Unix timestamp in seconds.
    #[serde(rename = "expiresAt")]
    pub expires_at: Option<u64>,
}

// This struct is private (no `pub`) — it only exists to match the exact JSON shape
// of the credentials file. We expose `OAuthTokens` to callers, not this wrapper.
#[derive(Debug, Serialize, Deserialize)]
struct CredentialsFile {
    #[serde(rename = "claudeAiOauth")]
    // `OAuthTokens` here is not wrapped in Option — the field is always required in the file.
    claude_ai_oauth: OAuthTokens,
}

// `pub fn` — public function, callable from other modules.
// Returns `PathBuf` (owned value, not a reference) so the caller owns the path.
pub fn credentials_path() -> PathBuf {
    dirs::home_dir() // Returns `Option<PathBuf>` — the user's home directory, if detectable.
        .expect("Could not find home directory") // `.expect()` panics with this message if the Option is None.
        .join(".mueller") // `PathBuf::join` appends a segment, returning a new PathBuf.
        .join("credentials.json") // Chained again — builds `~/.mueller/credentials.json`.
}

// `tokens: &OAuthTokens` — a shared (immutable) reference. We borrow the value
// without taking ownership, so the caller can still use `tokens` after this call.
pub fn save_credentials(tokens: &OAuthTokens) -> Result<PathBuf> {
    let path = credentials_path();
    // `.parent()` returns `Option<&Path>` — the directory portion of the path.
    // `.unwrap()` extracts the value, panicking if it's None (safe here — path always has a parent).
    std::fs::create_dir_all(path.parent().unwrap())?; // Creates `~/.mueller/` if it doesn't exist. `?` propagates errors.

    // Build a `CredentialsFile` wrapper so the JSON output matches the expected schema.
    // `.clone()` is needed because `tokens` is a reference — we need an owned copy for the struct.
    let file = CredentialsFile {
        claude_ai_oauth: tokens.clone(),
    };
    // `serde_json::to_string_pretty` serializes to JSON with indentation.
    // The `?` at the end propagates any serialization error up to the caller.
    let json = serde_json::to_string_pretty(&file)?;
    // Write the JSON string to disk. `?` propagates IO errors.
    std::fs::write(&path, json)?;

    // `#[cfg(unix)]` is a compile-time conditional — this block only compiles on Unix-like systems
    // (macOS, Linux). On Windows it's silently omitted.
    #[cfg(unix)]
    {
        // Bring the Unix-specific `PermissionsExt` trait into scope.
        // Traits must be in scope for their methods to be callable.
        use std::os::unix::fs::PermissionsExt;
        // `0o600` is an octal literal — file permissions: owner read+write, no access for others.
        // This protects the credentials file from other users on the system.
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    }

    // Return the path wrapped in `Ok(...)` — the `Result::Ok` variant signals success.
    Ok(path)
}

// Returns `Result<OAuthTokens>` — either the loaded tokens or an error describing what went wrong.
pub fn load_credentials() -> Result<OAuthTokens> {
    let path = credentials_path();
    // `read_to_string` loads the file into a `String`. `.context(...)` wraps any IO error
    // with a human-readable message — from the `anyhow::Context` trait.
    let content = std::fs::read_to_string(&path)
        .context("Not logged in. Run `mueller login` first.")?;
    // Parse the JSON string into our `CredentialsFile` struct.
    // If the JSON is malformed or fields are missing, serde returns an Err — `?` propagates it.
    let file: CredentialsFile = serde_json::from_str(&content)?;
    // Return the inner `OAuthTokens`, discarding the file wrapper struct.
    Ok(file.claude_ai_oauth)
}

// `async fn` — this function can perform non-blocking operations (like spawning processes).
// The Tokio runtime (set up in main.rs) drives it to completion.
pub async fn login() -> Result<()> {
    // `println!` is a macro — the `!` distinguishes macros from regular functions in Rust.
    println!("Logging in via Claude...");

    // `std::process::Command` is a builder for spawning child processes.
    // `.arg()` appends command-line arguments one at a time.
    let output = std::process::Command::new("claude")
        .arg("setup-token") // Equivalent to running: claude setup-token
        .output() // Spawns the process and waits for it to finish, capturing stdout/stderr.
        .context("Could not run `claude`. Is Claude Code installed? Run: npm install -g @anthropic-ai/claude-code")?;

    // `.status` is the exit code. `.success()` returns true if the exit code was 0.
    if !output.status.success() {
        // `String::from_utf8_lossy` converts raw bytes to a string, replacing invalid UTF-8.
        // `&output.stderr` borrows the Vec<u8> — no copying needed.
        let stderr = String::from_utf8_lossy(&output.stderr);
        // `anyhow::bail!` is a macro that constructs an error and immediately returns it.
        // `{}` in the format string is replaced by `stderr` at runtime.
        anyhow::bail!("claude setup-token failed: {}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // `.lines()` returns an iterator over each line of the string — lazy, no allocation.
    // `.find(|line| ...)` walks the iterator until the closure returns true.
    // The `|line|` syntax defines a closure (anonymous function) that takes one argument.
    let token = stdout
        .lines()
        .find(|line| line.trim().starts_with("sk-ant-oat01-"))
        // `.context(...)` attaches a message if the Option is None, turning it into a Result.
        .context("Could not find token in claude setup-token output")?
        .trim() // Removes leading/trailing whitespace — returns a `&str` (borrowed slice).
        .to_string(); // Converts the `&str` to an owned `String` stored on the heap.

    // Construct the OAuthTokens struct with field init syntax.
    // Fields not set explicitly use their given values; no implicit defaults in Rust.
    let tokens = OAuthTokens {
        access_token: token,
        refresh_token: None, // Explicitly set to None — no refresh token from this flow.
        expires_at: None,    // Explicitly set to None — no expiry info from this flow.
    };

    // Pass a reference `&tokens` — save_credentials borrows it without taking ownership.
    let path = save_credentials(&tokens)?;
    // Format the path using `{}` — this calls the `Display` trait implementation on `path`.
    println!("✓ Logged in! Credentials saved to {}", path.display());

    // `Ok(())` returns a successful Result wrapping the unit type `()` — Rust's "void".
    Ok(())
}
