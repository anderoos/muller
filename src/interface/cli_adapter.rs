use crate::cli::Command;
use super::normalizer;
use super::types::{InterfaceSource, PromptPayload, TaskType};

/// Convert a parsed CLI command into a PromptPayload.
///
/// This centralises all prompt-construction logic so main.rs becomes thin
/// routing and agent.rs never has to know which surface originated the request.
pub fn from_command(cmd: &Command) -> PromptPayload {
    let (raw_input, prompt, task_type, skill, goal) = build(cmd);
    let normalized = normalizer::normalize(&prompt);

    PromptPayload::new(
        InterfaceSource::Cli,
        raw_input,
        normalized,
        goal,
        None,
        task_type,
        skill,
    )
}

/// Build a free-text query from the CLI into a PromptPayload (no subcommand).
pub fn from_raw_query(query: &str) -> PromptPayload {
    let normalized = normalizer::normalize(query);
    let task_type = normalizer::infer_task_type(&normalized);
    let goal = normalizer::extract_goal(&normalized);

    PromptPayload::new(
        InterfaceSource::Cli,
        query,
        normalized,
        goal,
        None,
        task_type,
        None::<String>,
    )
}

// ── private helpers ───────────────────────────────────────────────────────────

type BuildResult = (String, String, TaskType, Option<String>, String);

fn build(cmd: &Command) -> BuildResult {
    match cmd {
        Command::Ask { query } => {
            let norm = normalizer::normalize(query);
            let goal = normalizer::extract_goal(&norm);
            (
                query.clone(),
                norm,
                TaskType::Get,
                Some("ASKS".to_string()),
                goal,
            )
        }

        Command::Init { brief } => {
            let norm = normalizer::normalize(brief);
            let prompt = format!(
                "Start a new project from the following brief. Break it down into epics and \
                smaller units of work, do not assign owners unless explicitly told and \
                produce a full Jira structure ready for sprint planning. Include descriptions \
                and deliverables for each ticket.\n\nBrief: {}",
                norm
            );
            (
                brief.clone(),
                prompt,
                TaskType::Insert,
                Some("pre-project/project-initiator".to_string()),
                format!("Initialize project: {}", normalizer::extract_goal(&norm)),
            )
        }

        Command::Update { ticket } => {
            let norm = normalizer::normalize(ticket);
            let prompt = format!(
                "Update the following ticket. Apply any status changes, append meeting notes, \
                adjust scope, or reassign as described.\n\nTicket: {}",
                norm
            );
            (
                ticket.clone(),
                prompt,
                TaskType::Update,
                Some("ASKS".to_string()),
                format!("Update ticket: {}", normalizer::extract_goal(&norm)),
            )
        }

        Command::Standup => (
            "standup".to_string(),
            "Trigger the daily standup relay. Prompt each team member for progress, \
            capture blockers, and update Jira ticket statuses based on responses."
                .to_string(),
            TaskType::Update,
            Some("active-sprint".to_string()),
            "Run daily standup relay".to_string(),
        ),

        Command::Health => (
            "health".to_string(),
            "Run a sprint health check. Analyse current ticket statuses, \
            cross-ticket dependencies, and velocity. Return an on-track / at-risk / \
            off-rails verdict per flagged item with a specific recommended action."
                .to_string(),
            TaskType::Get,
            Some("active-sprint/sprint-health-check".to_string()),
            "Sprint health check".to_string(),
        ),

        Command::Log { file } => {
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
            let raw = file.clone().unwrap_or_else(|| "log".to_string());
            (
                raw,
                prompt,
                TaskType::Insert,
                Some("meetings/meeting-note-transcriber".to_string()),
                "Log meeting and extract action items".to_string(),
            )
        }

        Command::Brief => (
            "brief".to_string(),
            "Generate a pre-meeting brief for the next calendar event. Surface \
            open blockers relevant to attendees, tickets last discussed by the group, \
            outstanding decisions from the prior session, and a suggested agenda."
                .to_string(),
            TaskType::Get,
            Some("meetings/pre-meeting-briefer".to_string()),
            "Generate pre-meeting brief".to_string(),
        ),

        Command::Scan => (
            "scan".to_string(),
            "Scan the active sprint for drift, stale tickets, slippage patterns, \
            and accountability gaps. Flag any tickets that are off-track with a severity \
            rating and recommended action."
                .to_string(),
            TaskType::Get,
            Some("risk-evaluation".to_string()),
            "Sprint drift and slippage scan".to_string(),
        ),

        Command::Summarize => (
            "summarize".to_string(),
            "Generate a plain-English summary of the current project. Cover the \
            timeline, key events, decisions made, and current status. Make it readable \
            for both technical and non-technical audiences."
                .to_string(),
            TaskType::Get,
            Some("communication/summarizer".to_string()),
            "Generate project summary".to_string(),
        ),

        Command::Onboard { member } => {
            let norm = normalizer::normalize(member);
            let prompt = format!(
                "Generate a detailed onboarding brief for a new team member joining the project \
                mid-flight. Cover project context, past key decisions, open blockers, their \
                assigned tickets, and the team roster. Deliver to Slack.\n\nNew member: {}",
                norm
            );
            (
                member.clone(),
                prompt,
                TaskType::Get,
                Some("communication/new-member-onboarder".to_string()),
                format!("Onboard new member: {}", norm),
            )
        }

        Command::Close => (
            "close".to_string(),
            "Close the project. Schedule the retrospective, notify stakeholders, \
            archive the Jira board, and trigger the knowledge transfer and estimation \
            feedback loop."
                .to_string(),
            TaskType::Delete,
            Some("project-close/project-terminator".to_string()),
            "Close and archive project".to_string(),
        ),

        // Non-agent commands — callers should not route these through the adapter.
        Command::Login | Command::Setup | Command::Dashboard | Command::Autopilot { .. } => {
            let label = cmd_label(cmd);
            (label.to_string(), label.to_string(), TaskType::Get, None, label.to_string())
        }
    }
}

fn cmd_label(cmd: &Command) -> &'static str {
    match cmd {
        Command::Login     => "login",
        Command::Setup     => "setup",
        Command::Dashboard => "dashboard",
        Command::Autopilot { .. } => "autopilot",
        _ => "unknown",
    }
}
