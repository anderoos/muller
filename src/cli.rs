// `use` brings specific items from a crate (library) into scope so you can write
// `Parser` instead of the full path `clap::Parser` everywhere.
use clap::{Parser, Subcommand};

// `#[derive(...)]` is a procedural macro — it auto-generates trait implementations
// for the struct below. `Parser` gives `Cli` a `.parse()` method that reads
// command-line arguments from std::env::args() and fills the struct fields.
#[derive(Parser)]
// `#[command(...)]` attributes configure Clap's generated help text and binary name.
#[command(name = "mueller")]
#[command(about = "Your personal AI project management agent")]
// `pub struct` declares a public struct — `pub` means other modules (like main.rs) can use it.
// A struct is a named collection of fields, like a class without methods.
pub struct Cli {
    // `#[command(subcommand)]` tells Clap that this field should be parsed as a subcommand
    // (e.g. `mueller login`, `mueller standup`) rather than a flag or positional argument.
    #[command(subcommand)]
    // `Option<Command>` means this field is either `Some(command)` or `None` (absent).
    // Rust has no null — `Option` is the explicit way to say "this value might not exist".
    pub command: Option<Command>,

    /// Ask a quick question directly
    // `Option<String>` — if the user types `mueller "some text"` without a subcommand,
    // Clap captures it here. `String` is an owned, heap-allocated UTF-8 string.
    pub query: Option<String>,

    /// Re-embed changed methodology files into ChromaDB (incremental by default)
    #[arg(long)]
    pub refresh_embeddings: bool,
}

// `#[derive(Subcommand)]` generates the logic to parse one of these enum variants
// from a CLI argument string like "login", "standup", "health", etc.
#[derive(Subcommand)]
// `pub enum` declares a public enum — a type that can be exactly one of several named variants.
// Rust enums are much more powerful than in other languages: variants can carry data.
pub enum Command {
    /// Authenticate with Claude
    // A unit variant — no data attached. Matches the literal string "login" on the CLI.
    Login,

    /// Configure Jira credentials and project settings
    // Unit variant — matches the literal string "setup" on the CLI.
    Setup,

    /// Ask a question about current tickets or project status
    // A struct variant — carries a named field `query`. Clap treats it as a positional argument.
    Ask {
        query: String,
    },

    /// Start a new project from a brief
    Init {
        /// Project brief — purpose, goals, scope, team, timeline, and success metrics
        // `String` here is a required positional argument — the user must provide it.
        brief: String,
    },

    /// Update a ticket: status, meeting notes, scope changes, reassignment
    Update {
        /// Ticket ID or description of what to update
        ticket: String,
    },

    /// Trigger the daily standup relay
    Standup,

    /// Run a mid-sprint health check — returns on-track / at-risk / off-rails per ticket
    Health,

    /// Transcribe a meeting, extract action items, and push tasks to Jira
    Log {
        /// Path to transcript or audio file (omit to paste transcript interactively)
        // `#[arg(short, long)]` means this field is an optional flag: `-f` or `--file`.
        // `Option<String>` makes it optional — if omitted, the field is `None`.
        #[arg(short, long)]
        file: Option<String>,
    },

    /// Generate a pre-meeting brief for the next calendar event
    Brief,

    /// Scan for drift, stale tickets, and slippage patterns across the sprint
    Scan,

    /// Generate a plain-English project summary for any audience
    Summarize,

    /// Generate an onboarding brief for a new team member joining mid-project
    Onboard {
        /// Slack handle or name of the new team member
        member: String,
    },

    /// Close the project: schedule retro, archive board, trigger knowledge transfer
    Close,

    /// Manage autopilot behavioral directives
    Autopilot {
        // A nested subcommand — `mueller autopilot add "..."` etc.
        // Clap recurses into `AutopilotCommand` to parse the next word on the CLI.
        #[command(subcommand)]
        command: AutopilotCommand,
    },
}

// A second enum for the `autopilot` sub-subcommands.
// Nesting enums like this lets Clap build a multi-level command tree cleanly.
#[derive(Subcommand)]
pub enum AutopilotCommand {
    /// Add a new behavior on top of existing defaults
    Add {
        // `behavior: String` captures everything after `mueller autopilot add` as one string.
        behavior: String,
    },

    /// Replace a default behavior with an alternative
    Override {
        behavior: String,
    },

    /// Remove or suppress a default behavior
    Less {
        behavior: String,
    },
}
