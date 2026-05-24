mod agent;
mod auth;
mod cli;

use anyhow::Result;
use clap::Parser;
use cli::{AutopilotCommand, Command};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    match cli.command {
        Some(Command::Login) => {
            auth::login().await?;
        }

        Some(Command::Ask { query }) => {
            agent::run(&query, Some("ASKS")).await?;
        }

        Some(Command::Init { brief }) => {
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
            let prompt = match file {
                Some(path) => format!(
                    "Transcribe the meeting from the following file, extract action items, \
                    decisions, and takeaways, and push tasks to Jira.\n\nFile: {}",
                    path
                ),
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
            let directive = match command {
                AutopilotCommand::Add { behavior } => format!("add {}", behavior),
                AutopilotCommand::Override { behavior } => format!("override {}", behavior),
                AutopilotCommand::Less { behavior } => format!("less {}", behavior),
            };
            agent::append_autopilot_directive(&directive)?;
            println!("Autopilot directive saved: {}", directive);
        }

        None => match cli.query {
            Some(query) => agent::run(&query, None).await?,
            None => {
                println!("Mueller — Your personal AI project management agent\n");
                println!("Usage:");
                println!("  mueller \"quick question\"");
                println!("  mueller ask \"question\"");
                println!("  mueller login");
            }
        },
    }

    Ok(())
}
