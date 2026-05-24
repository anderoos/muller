use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "mueller")]
#[command(about = "Your personal AI project management agent")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Ask a quick question directly
    pub query: Option<String>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Authenticate with Claude
    Login,

    /// Ask a question about current tickets or project status
    Ask {
        query: String,
    },

    /// Start a new project from a brief
    Init {
        /// Project brief — purpose, goals, scope, team, timeline, and success metrics
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
        #[command(subcommand)]
        command: AutopilotCommand,
    },
}

#[derive(Subcommand)]
pub enum AutopilotCommand {
    /// Add a new behavior on top of existing defaults
    Add {
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
