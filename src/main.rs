
mod agent;
mod auth;
mod cli;
mod config;

// `use` brings items into scope so you can write `Result` instead of `anyhow::Result<T, ...>`.
use anyhow::Result;
// `Parser` is the Clap trait that gives `Cli` its `.parse()` method.
use clap::Parser;
// Import both enum types from our `cli` module — needed to pattern-match on them below.
use cli::{AutopilotCommand, Command};

// `#[tokio::main]` is a procedural macro that wraps `main` in a Tokio async runtime.
// Without it you can't use `async fn main` or call `.await` at the top level.
#[tokio::main]
// `async fn` marks main as an async function — required when calling `.await` inside it.
// `-> Result<()>` means main either returns `Ok(())` (success) or an `anyhow::Error`.
// Returning a `Result` from main lets Rust print the error and exit with code 1 automatically.
async fn main() -> Result<()> {
    // `cli::Cli::parse()` reads `std::env::args()`, validates them against the struct definition,
    // and returns a populated `Cli`. If args are invalid, Clap prints help and exits.
    let cli = cli::Cli::parse();

    // `match` is Rust's exhaustive pattern matching. Every possible variant of `cli.command`
    // must be handled — the compiler enforces this, so you can't accidentally miss a case.
    match cli.command {
        // `Some(Command::Login)` — the user ran `mueller login`. The `Some(...)` unwraps the
        // `Option<Command>` and simultaneously matches the inner variant.
        Some(Command::Login) => {
            // `.await` suspends this async task until `login()` completes.
            // `?` unwraps the `Result`: if it's `Err`, main returns that error immediately.
            auth::login().await?;
            // `run_setup()` asks the user for their interaction preference (CLI or Slack),
            // then collects only the credentials that preference requires.
            let cfg = config::run_setup()?;
            config::save_config(&cfg)?;
            println!("✓ All set. Run any Mueller command to get started.");
        }

        Some(Command::Setup) => {
            // Standalone re-run — useful for changing interaction mode or rotating credentials
            // without re-authenticating with Claude.
            let cfg = config::run_setup()?;
            config::save_config(&cfg)?;
            println!("✓ Configuration updated.");
        }

        Some(Command::Ask { query }) => {
            // Destructuring in the pattern: `{ query }` binds the `query` field of the
            // `Ask` variant directly into a local variable named `query`.
            // `&query` passes a borrowed reference to avoid moving the String into `run()`.
            agent::run(&query, Some("ASKS")).await?;
        }

        Some(Command::Init { brief }) => {
            // `format!` builds a new owned `String` by interpolating variables into a template.
            // `\` at the end of the string literal continues onto the next line without a newline.
            let prompt = format!(
                "Start a new project from the following brief. Break it down into epics and \
                tickets, assign owners based on role and capacity, and produce a full Jira \
                structure ready for sprint planning.\n\nBrief: {}",
                brief
            );
            agent::run(&prompt, Some("pre-project/project-initiator")).await?;
        }

        Some(Command::Update { ticket }) => {
            let prompt = format!(
                "Update the following ticket. Apply any status changes, append meeting notes, \
                adjust scope, or reassign as described.\n\nTicket: {}",
                ticket
            );
            agent::run(&prompt, Some("ASKS")).await?;
        }

        Some(Command::Standup) => {
            // A string literal `"..."` is a `&'static str` — it lives in the binary's
            // read-only data segment for the entire lifetime of the program.
            let prompt = "Trigger the daily standup relay. Prompt each team member for progress, \
                capture blockers, and update Jira ticket statuses based on responses.";
            agent::run(prompt, Some("active-sprint")).await?;
        }

        Some(Command::Health) => {
            let prompt = "Run a sprint health check. Analyse current ticket statuses, \
                cross-ticket dependencies, and velocity. Return an on-track / at-risk / \
                off-rails verdict per flagged item with a specific recommended action.";
            agent::run(prompt, Some("active-sprint/sprint-health-check")).await?;
        }

        Some(Command::Log { file }) => {
            // `match file` on an `Option<String>` — handles both the Some and None cases.
            let prompt = match file {
                // `Some(path)` — user passed `--file /path/to/transcript`. `path` is a `String`.
                Some(path) => format!(
                    "Transcribe the meeting from the following file, extract action items, \
                    decisions, and takeaways, and push tasks to Jira.\n\nFile: {}",
                    path
                ),
                // `None` — no file flag; fall back to asking the user to paste a transcript.
                // `.to_string()` converts the `&str` literal into an owned `String` to match
                // the `String` type that `format!` produces in the `Some` arm.
                None => "Transcribe the meeting transcript provided, extract action items, \
                    decisions, and takeaways, and push tasks to Jira. Paste the transcript now."
                    .to_string(),
            };
            agent::run(&prompt, Some("meetings/meeting-note-transcriber")).await?;
        }

        Some(Command::Brief) => {
            let prompt = "Generate a pre-meeting brief for the next calendar event. Surface \
                open blockers relevant to attendees, tickets last discussed by the group, \
                outstanding decisions from the prior session, and a suggested agenda.";
            agent::run(prompt, Some("meetings/pre-meeting-briefer")).await?;
        }

        Some(Command::Scan) => {
            let prompt = "Scan the active sprint for drift, stale tickets, slippage patterns, \
                and accountability gaps. Flag any tickets that are off-track with a severity \
                rating and recommended action.";
            agent::run(prompt, Some("risk-evaluation")).await?;
        }

        Some(Command::Summarize) => {
            let prompt = "Generate a plain-English summary of the current project. Cover the \
                timeline, key events, decisions made, and current status. Make it readable \
                for both technical and non-technical audiences.";
            agent::run(prompt, Some("communication/summarizer")).await?;
        }

        Some(Command::Onboard { member }) => {
            let prompt = format!(
                "Generate a detailed onboarding brief for a new team member joining the project \
                mid-flight. Cover project context, past key decisions, open blockers, their \
                assigned tickets, and the team roster. Deliver to Slack.\n\nNew member: {}",
                member
            );
            agent::run(&prompt, Some("communication/new-member-onboarder")).await?;
        }

        Some(Command::Close) => {
            let prompt = "Close the project. Schedule the retrospective, notify stakeholders, \
                archive the Jira board, and trigger the knowledge transfer and estimation \
                feedback loop.";
            agent::run(prompt, Some("project-close/project-terminator")).await?;
        }

        Some(Command::Autopilot { command }) => {
            // Destructure the nested enum variant into its inner `AutopilotCommand`.
            // `match command` now switches on the sub-subcommand.
            let directive = match command {
                // Each arm extracts the `behavior: String` field and builds a directive string.
                AutopilotCommand::Add { behavior } => format!("add {}", behavior),
                AutopilotCommand::Override { behavior } => format!("override {}", behavior),
                AutopilotCommand::Less { behavior } => format!("less {}", behavior),
            };
            // `?` propagates any file IO error from appending to the autopilot skill file.
            agent::append_autopilot_directive(&directive)?;
            // `{}` calls the `Display` trait on `directive` — the default string formatting.
            println!("Autopilot directive saved: {}", directive);
        }

        // `None` — no subcommand was typed. Check whether the user passed a bare query string.
        None => match cli.query {
            // If a query was passed (e.g. `mueller "how many tickets are open?"`), run it.
            Some(query) => agent::run(&query, None).await?,
            // No subcommand and no query — print usage instructions.
            None => {
                println!("Mueller — Your personal AI project management agent\n");
                println!("Usage:");
                println!("  mueller \"quick question\"");
                println!("  mueller ask \"question\"");
                println!("  mueller login");
            }
        },
    }

    // Returning `Ok(())` from main signals a clean exit (exit code 0).
    Ok(())
}
