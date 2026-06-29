#![cfg(test)]

use super::cli_adapter;
use super::normalizer;
use super::slack_adapter::{from_message, SlackMessage};
use super::types::{InterfaceSource, TaskType};
use crate::cli::{AutopilotCommand, Command};

// ── normalizer ────────────────────────────────────────────────────────────────

#[test]
fn normalize_collapses_internal_whitespace() {
    assert_eq!(normalizer::normalize("hello   world"), "hello world");
}

#[test]
fn normalize_trims_leading_trailing_space() {
    assert_eq!(normalizer::normalize("  hello "), "hello");
}

#[test]
fn normalize_collapses_tabs_and_newlines() {
    assert_eq!(normalizer::normalize("hello\t\nworld"), "hello world");
}

#[test]
fn normalize_empty_string() {
    assert_eq!(normalizer::normalize(""), "");
}

// ── extract_goal ──────────────────────────────────────────────────────────────

#[test]
fn extract_goal_single_sentence_returns_full_text() {
    let goal = normalizer::extract_goal("What is the current sprint status");
    assert_eq!(goal, "What is the current sprint status");
}

#[test]
fn extract_goal_stops_at_period() {
    let goal = normalizer::extract_goal("Update ticket ABC-123. Add meeting notes.");
    assert_eq!(goal, "Update ticket ABC-123");
}

#[test]
fn extract_goal_stops_at_question_mark() {
    let goal = normalizer::extract_goal("Who owns this ticket? Please check.");
    assert_eq!(goal, "Who owns this ticket");
}

#[test]
fn extract_goal_empty_input_returns_empty() {
    assert_eq!(normalizer::extract_goal(""), "");
}

// ── infer_task_type ───────────────────────────────────────────────────────────

#[test]
fn infer_get_for_neutral_query() {
    assert_eq!(normalizer::infer_task_type("What is the sprint velocity?"), TaskType::Get);
}

#[test]
fn infer_insert_for_create_keyword() {
    assert_eq!(normalizer::infer_task_type("Create a new epic for Q3"), TaskType::Insert);
}

#[test]
fn infer_insert_for_add_keyword() {
    assert_eq!(normalizer::infer_task_type("Add a story to the backlog"), TaskType::Insert);
}

#[test]
fn infer_update_for_update_keyword() {
    assert_eq!(normalizer::infer_task_type("Update ticket ABC-99 status"), TaskType::Update);
}

#[test]
fn infer_update_for_reassign_keyword() {
    assert_eq!(normalizer::infer_task_type("Reassign the blocker to Alice"), TaskType::Update);
}

#[test]
fn infer_delete_wins_over_update_when_both_present() {
    // "archive" is Delete; "change" is Update — Delete should win because it
    // is checked first in the priority order.
    assert_eq!(normalizer::infer_task_type("archive and change the project"), TaskType::Delete);
}

#[test]
fn infer_delete_for_remove_keyword() {
    assert_eq!(normalizer::infer_task_type("Remove the stale ticket"), TaskType::Delete);
}

// ── cli_adapter: task types ───────────────────────────────────────────────────

#[test]
fn ask_command_is_get() {
    let p = cli_adapter::from_command(&Command::Ask { query: "sprint health".into() });
    assert_eq!(p.task_type, TaskType::Get);
}

#[test]
fn health_command_is_get() {
    let p = cli_adapter::from_command(&Command::Health);
    assert_eq!(p.task_type, TaskType::Get);
}

#[test]
fn brief_command_is_get() {
    let p = cli_adapter::from_command(&Command::Brief);
    assert_eq!(p.task_type, TaskType::Get);
}

#[test]
fn scan_command_is_get() {
    let p = cli_adapter::from_command(&Command::Scan);
    assert_eq!(p.task_type, TaskType::Get);
}

#[test]
fn summarize_command_is_get() {
    let p = cli_adapter::from_command(&Command::Summarize);
    assert_eq!(p.task_type, TaskType::Get);
}

#[test]
fn onboard_command_is_get() {
    let p = cli_adapter::from_command(&Command::Onboard { member: "alice".into() });
    assert_eq!(p.task_type, TaskType::Get);
}

#[test]
fn init_command_is_insert() {
    let p = cli_adapter::from_command(&Command::Init { brief: "Build a portal".into() });
    assert_eq!(p.task_type, TaskType::Insert);
}

#[test]
fn log_command_is_insert() {
    let p = cli_adapter::from_command(&Command::Log { file: None });
    assert_eq!(p.task_type, TaskType::Insert);
}

#[test]
fn log_command_with_file_is_insert() {
    let p = cli_adapter::from_command(&Command::Log { file: Some("meeting.txt".into()) });
    assert_eq!(p.task_type, TaskType::Insert);
}

#[test]
fn update_command_is_update() {
    let p = cli_adapter::from_command(&Command::Update { ticket: "ABC-1".into() });
    assert_eq!(p.task_type, TaskType::Update);
}

#[test]
fn standup_command_is_update() {
    let p = cli_adapter::from_command(&Command::Standup);
    assert_eq!(p.task_type, TaskType::Update);
}

#[test]
fn close_command_is_delete() {
    let p = cli_adapter::from_command(&Command::Close);
    assert_eq!(p.task_type, TaskType::Delete);
}

// ── cli_adapter: write guard helper ──────────────────────────────────────────

#[test]
fn write_commands_report_is_write_true() {
    let write_commands = [
        Command::Init { brief: "test".into() },
        Command::Update { ticket: "X-1".into() },
        Command::Standup,
        Command::Log { file: None },
        Command::Close,
    ];
    for cmd in &write_commands {
        let p = cli_adapter::from_command(cmd);
        assert!(p.task_type.is_write(), "{:?} should be a write", p.task_type);
    }
}

#[test]
fn read_commands_report_is_write_false() {
    let read_commands = [
        Command::Ask { query: "test".into() },
        Command::Health,
        Command::Brief,
        Command::Scan,
        Command::Summarize,
        Command::Onboard { member: "bob".into() },
    ];
    for cmd in &read_commands {
        let p = cli_adapter::from_command(cmd);
        assert!(!p.task_type.is_write(), "{:?} should not be a write", p.task_type);
    }
}

// ── cli_adapter: skill assignment ─────────────────────────────────────────────

#[test]
fn ask_command_has_asks_skill() {
    let p = cli_adapter::from_command(&Command::Ask { query: "anything".into() });
    assert_eq!(p.skill.as_deref(), Some("ASKS"));
}

#[test]
fn init_command_has_project_initiator_skill() {
    let p = cli_adapter::from_command(&Command::Init { brief: "x".into() });
    assert_eq!(p.skill.as_deref(), Some("pre-project/project-initiator"));
}

#[test]
fn health_command_has_sprint_health_check_skill() {
    let p = cli_adapter::from_command(&Command::Health);
    assert_eq!(p.skill.as_deref(), Some("active-sprint/sprint-health-check"));
}

// ── cli_adapter: goal extraction ──────────────────────────────────────────────

#[test]
fn ask_goal_matches_query_first_sentence() {
    let p = cli_adapter::from_command(&Command::Ask {
        query: "Show me the backlog. Filter by priority.".into(),
    });
    assert_eq!(p.goal, "Show me the backlog");
}

#[test]
fn init_goal_contains_keyword() {
    let p = cli_adapter::from_command(&Command::Init {
        brief: "Build a customer portal".into(),
    });
    assert!(p.goal.contains("Initialize project") || p.goal.contains("customer portal"),
        "goal was: {}", p.goal);
}

// ── cli_adapter: raw query ────────────────────────────────────────────────────

#[test]
fn raw_query_normalizes_whitespace() {
    let p = cli_adapter::from_raw_query("  what   is   the  status  ");
    assert_eq!(p.normalized_prompt, "what is the status");
}

#[test]
fn raw_query_infers_insert_from_text() {
    let p = cli_adapter::from_raw_query("create a new epic for the platform team");
    assert_eq!(p.task_type, TaskType::Insert);
}

#[test]
fn raw_query_source_is_cli() {
    let p = cli_adapter::from_raw_query("anything");
    assert!(matches!(p.source, InterfaceSource::Cli));
}

// ── slack_adapter ─────────────────────────────────────────────────────────────

#[test]
fn slack_strips_bot_mention() {
    let msg = SlackMessage {
        user_id: "U001".into(),
        channel_id: "C001".into(),
        text: "<@UBOTID> scan the sprint".into(),
    };
    let p = from_message(&msg);
    assert_eq!(p.normalized_prompt, "scan the sprint");
}

#[test]
fn slack_no_mention_leaves_text_intact() {
    let msg = SlackMessage {
        user_id: "U001".into(),
        channel_id: "C001".into(),
        text: "health check please".into(),
    };
    let p = from_message(&msg);
    assert_eq!(p.normalized_prompt, "health check please");
}

#[test]
fn slack_update_prefix_is_update() {
    let msg = SlackMessage {
        user_id: "U001".into(),
        channel_id: "C001".into(),
        text: "update ticket ABC-5 to in-progress".into(),
    };
    let p = from_message(&msg);
    assert_eq!(p.task_type, TaskType::Update);
}

#[test]
fn slack_create_prefix_is_insert() {
    let msg = SlackMessage {
        user_id: "U001".into(),
        channel_id: "C001".into(),
        text: "create a task for the auth module".into(),
    };
    let p = from_message(&msg);
    assert_eq!(p.task_type, TaskType::Insert);
}

#[test]
fn slack_delete_prefix_is_delete() {
    let msg = SlackMessage {
        user_id: "U001".into(),
        channel_id: "C001".into(),
        text: "archive the old sprint board".into(),
    };
    let p = from_message(&msg);
    assert_eq!(p.task_type, TaskType::Delete);
}

#[test]
fn slack_source_carries_user_and_channel() {
    let msg = SlackMessage {
        user_id: "U999".into(),
        channel_id: "C123".into(),
        text: "show sprint health".into(),
    };
    let p = from_message(&msg);
    match p.source {
        InterfaceSource::Slack { user_id, channel_id } => {
            assert_eq!(user_id, "U999");
            assert_eq!(channel_id, "C123");
        }
        _ => panic!("expected Slack source"),
    }
}

#[test]
fn slack_context_includes_user_and_channel() {
    let msg = SlackMessage {
        user_id: "U777".into(),
        channel_id: "C456".into(),
        text: "anything".into(),
    };
    let p = from_message(&msg);
    let ctx = p.context.unwrap();
    assert!(ctx.contains("U777"));
    assert!(ctx.contains("C456"));
}

// ── types ────────────────────────────────────────────────────────────────────

#[test]
fn task_type_get_is_not_write() {
    assert!(!TaskType::Get.is_write());
}

#[test]
fn task_type_insert_is_write() {
    assert!(TaskType::Insert.is_write());
}

#[test]
fn task_type_update_is_write() {
    assert!(TaskType::Update.is_write());
}

#[test]
fn task_type_delete_is_write() {
    assert!(TaskType::Delete.is_write());
}

#[test]
fn task_type_as_str_values() {
    assert_eq!(TaskType::Get.as_str(), "get");
    assert_eq!(TaskType::Insert.as_str(), "insert");
    assert_eq!(TaskType::Update.as_str(), "update");
    assert_eq!(TaskType::Delete.as_str(), "delete");
}

#[test]
fn payload_serializes_to_json_with_all_fields() {
    let p = cli_adapter::from_command(&Command::Ask { query: "test".into() });
    let json = serde_json::to_string(&p).expect("serialization failed");
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(v.get("id").is_some());
    assert!(v.get("source").is_some());
    assert!(v.get("raw_input").is_some());
    assert!(v.get("normalized_prompt").is_some());
    assert!(v.get("goal").is_some());
    assert!(v.get("task_type").is_some());
}

#[test]
fn payload_round_trips_through_json() {
    let original = cli_adapter::from_command(&Command::Standup);
    let json = serde_json::to_string(&original).unwrap();
    let restored: super::types::PromptPayload = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.normalized_prompt, original.normalized_prompt);
    assert_eq!(restored.goal, original.goal);
    assert_eq!(restored.task_type, original.task_type);
    assert_eq!(restored.skill, original.skill);
}

#[test]
fn each_payload_gets_unique_id() {
    let p1 = cli_adapter::from_command(&Command::Health);
    let p2 = cli_adapter::from_command(&Command::Health);
    assert_ne!(p1.id, p2.id);
}

// ── cli_adapter: autopilot (non-agent command fallback) ───────────────────────

#[test]
fn autopilot_command_returns_get_type() {
    let p = cli_adapter::from_command(&Command::Autopilot {
        command: AutopilotCommand::Add { behavior: "be concise".into() },
    });
    assert_eq!(p.task_type, TaskType::Get);
}
