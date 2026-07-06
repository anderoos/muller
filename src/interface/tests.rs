#![cfg(test)]

use super::cli_adapter;
use super::normalizer;
use super::slack_adapter::{from_message, SlackMessage};
use super::types::{InterfaceSource, PromptError, TaskType};
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
fn normalize_collapses_tabs_within_a_line() {
    assert_eq!(normalizer::normalize("hello\tworld"), "hello world");
}

#[test]
fn normalize_preserves_newlines() {
    assert_eq!(normalizer::normalize("hello\t\nworld"), "hello\nworld");
}

#[test]
fn normalize_preserves_multiline_structure() {
    let brief = "Build a portal.\n\n- auth\n- reporting";
    assert_eq!(normalizer::normalize(brief), "Build a portal.\n\n- auth\n- reporting");
}

#[test]
fn normalize_collapses_blank_line_runs_and_trims_edges() {
    assert_eq!(normalizer::normalize("\n\na\n\n\n\nb\n\n"), "a\n\nb");
}

#[test]
fn normalize_strips_zero_width_and_control_chars() {
    assert_eq!(normalizer::normalize("cre\u{200B}ate\u{FEFF} epic\u{7}"), "create epic");
}

#[test]
fn normalize_empty_string() {
    assert_eq!(normalizer::normalize(""), "");
}

// ── validate ──────────────────────────────────────────────────────────────────

#[test]
fn validate_rejects_empty_prompt() {
    assert_eq!(normalizer::validate(""), Err(PromptError::Empty));
}

#[test]
fn validate_rejects_over_length_prompt() {
    let long = "x".repeat(normalizer::MAX_PROMPT_CHARS + 1);
    assert!(matches!(normalizer::validate(&long), Err(PromptError::TooLong { .. })));
}

#[test]
fn validate_accepts_normal_prompt() {
    assert_eq!(normalizer::validate("what is the sprint status"), Ok(()));
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

#[test]
fn extract_goal_ignores_version_numbers_and_ticket_ids() {
    let goal = normalizer::extract_goal("Update PROJ-1.2 estimates for v2.0 release");
    assert_eq!(goal, "Update PROJ-1.2 estimates for v2.0 release");
}

#[test]
fn extract_goal_uses_first_line_of_multiline_prompt() {
    let goal = normalizer::extract_goal("Build a portal\n- auth\n- reporting");
    assert_eq!(goal, "Build a portal");
}

#[test]
fn extract_goal_truncates_very_long_sentences() {
    let long = "word ".repeat(100);
    let goal = normalizer::extract_goal(long.trim());
    assert!(goal.chars().count() <= 121, "goal was {} chars", goal.chars().count());
    assert!(goal.ends_with('…'));
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

#[test]
fn infer_matches_whole_words_only() {
    // "address" must not match "add", "backlog" must not match "log",
    // "dropdown" must not match "drop", "prefix" must not match "fix".
    assert_eq!(normalizer::infer_task_type("address the backlog"), TaskType::Get);
    assert_eq!(normalizer::infer_task_type("what is in the dropdown"), TaskType::Get);
    assert_eq!(normalizer::infer_task_type("what does this prefix mean"), TaskType::Get);
    assert_eq!(normalizer::infer_task_type("show me the news"), TaskType::Get);
}

#[test]
fn infer_matches_multi_word_keywords() {
    assert_eq!(normalizer::infer_task_type("please close project alpha"), TaskType::Delete);
    assert_eq!(normalizer::infer_task_type("open ticket for the outage"), TaskType::Insert);
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
    let p = cli_adapter::from_raw_query("  what   is   the  status  ").unwrap();
    assert_eq!(p.normalized_prompt, "what is the status");
}

#[test]
fn raw_query_infers_insert_from_text() {
    let p = cli_adapter::from_raw_query("create a new epic for the platform team").unwrap();
    assert_eq!(p.task_type, TaskType::Insert);
}

#[test]
fn raw_query_source_is_cli() {
    let p = cli_adapter::from_raw_query("anything").unwrap();
    assert!(matches!(p.source, InterfaceSource::Cli));
}

#[test]
fn raw_query_ask_verb_routes_to_ask_command() {
    let p = cli_adapter::from_raw_query("ask what is blocked right now").unwrap();
    assert_eq!(p.task_type, TaskType::Get);
    assert_eq!(p.skill.as_deref(), Some("ASKS"));
    assert_eq!(p.normalized_prompt, "what is blocked right now");
    assert_eq!(p.raw_input, "ask what is blocked right now");
}

#[test]
fn raw_query_bare_standup_routes_to_standup_command() {
    let p = cli_adapter::from_raw_query("standup").unwrap();
    assert_eq!(p.task_type, TaskType::Update);
    assert_eq!(p.skill.as_deref(), Some("active-sprint"));
}

#[test]
fn raw_query_bare_verb_with_trailing_text_falls_through() {
    // "scan <details>" must not drop the details into the canned scan prompt.
    let p = cli_adapter::from_raw_query("scan ticket ABC-1 for problems").unwrap();
    assert_eq!(p.normalized_prompt, "scan ticket ABC-1 for problems");
}

#[test]
fn raw_query_plain_question_gets_asks_skill() {
    let p = cli_adapter::from_raw_query("who owns the auth epic").unwrap();
    assert_eq!(p.task_type, TaskType::Get);
    assert_eq!(p.skill.as_deref(), Some("ASKS"));
}

#[test]
fn raw_query_inferred_write_gets_no_skill() {
    let p = cli_adapter::from_raw_query("create a new epic for the platform team").unwrap();
    assert!(p.skill.is_none());
}

#[test]
fn raw_query_rejects_empty_input() {
    assert_eq!(cli_adapter::from_raw_query("   \n\t  ").unwrap_err(), PromptError::Empty);
}

#[test]
fn raw_query_rejects_over_length_input() {
    let long = "spam ".repeat(5_000);
    assert!(matches!(
        cli_adapter::from_raw_query(&long),
        Err(PromptError::TooLong { .. })
    ));
}

// ── slack_adapter ─────────────────────────────────────────────────────────────

fn slack_msg(text: &str) -> SlackMessage {
    SlackMessage {
        user_id: "U001".into(),
        channel_id: "C001".into(),
        text: text.into(),
    }
}

#[test]
fn slack_strips_bot_mention() {
    let p = from_message(&slack_msg("<@UBOTID> scan the sprint")).unwrap();
    assert_eq!(p.normalized_prompt, "scan the sprint");
}

#[test]
fn slack_no_mention_leaves_text_intact() {
    let p = from_message(&slack_msg("health check please")).unwrap();
    assert_eq!(p.normalized_prompt, "health check please");
}

#[test]
fn slack_mention_only_message_is_rejected() {
    assert_eq!(from_message(&slack_msg("<@UBOTID>")).unwrap_err(), PromptError::Empty);
}

#[test]
fn slack_decodes_html_entities() {
    let p = from_message(&slack_msg("track Q&amp;A tickets &lt;now&gt;")).unwrap();
    assert_eq!(p.normalized_prompt, "track Q&A tickets <now>");
}

#[test]
fn slack_rewrites_mid_text_mention() {
    let p = from_message(&slack_msg("reassign ABC-5 to <@U123>")).unwrap();
    assert_eq!(p.normalized_prompt, "reassign ABC-5 to @U123");
}

#[test]
fn slack_rewrites_channel_reference() {
    let p = from_message(&slack_msg("post the summary in <#C456|general>")).unwrap();
    assert_eq!(p.normalized_prompt, "post the summary in #general");
}

#[test]
fn slack_unwraps_labelled_link() {
    let p = from_message(&slack_msg("see <https://example.com/spec|the spec>")).unwrap();
    assert_eq!(p.normalized_prompt, "see the spec (https://example.com/spec)");
}

#[test]
fn slack_unwraps_bare_link() {
    let p = from_message(&slack_msg("review <https://example.com/doc> today")).unwrap();
    assert_eq!(p.normalized_prompt, "review https://example.com/doc today");
}

#[test]
fn slack_leading_link_is_not_stripped_as_mention() {
    let p = from_message(&slack_msg("<https://example.com/doc> needs review")).unwrap();
    assert_eq!(p.normalized_prompt, "https://example.com/doc needs review");
}

#[test]
fn slack_update_prefix_is_update() {
    let p = from_message(&slack_msg("update ticket ABC-5 to in-progress")).unwrap();
    assert_eq!(p.task_type, TaskType::Update);
}

#[test]
fn slack_bare_health_routes_to_health_command() {
    let p = from_message(&slack_msg("<@UBOTID> health")).unwrap();
    assert_eq!(p.task_type, TaskType::Get);
    assert_eq!(p.skill.as_deref(), Some("active-sprint/sprint-health-check"));
}

#[test]
fn slack_ask_verb_routes_to_ask_command() {
    let p = from_message(&slack_msg("ask who owns ABC-1")).unwrap();
    assert_eq!(p.task_type, TaskType::Get);
    assert_eq!(p.skill.as_deref(), Some("ASKS"));
    assert_eq!(p.normalized_prompt, "who owns ABC-1");
}

#[test]
fn slack_command_word_payload_keeps_slack_source_and_context() {
    let p = from_message(&slack_msg("standup")).unwrap();
    assert_eq!(p.skill.as_deref(), Some("active-sprint"));
    assert!(matches!(p.source, InterfaceSource::Slack { .. }));
    assert!(p.context.unwrap().contains("C001"));
    assert_eq!(p.raw_input, "standup");
}

#[test]
fn slack_plain_question_gets_asks_skill() {
    let p = from_message(&slack_msg("what is our velocity this sprint")).unwrap();
    assert_eq!(p.skill.as_deref(), Some("ASKS"));
}

#[test]
fn slack_create_prefix_is_insert() {
    let p = from_message(&slack_msg("create a task for the auth module")).unwrap();
    assert_eq!(p.task_type, TaskType::Insert);
}

#[test]
fn slack_delete_prefix_is_delete() {
    let p = from_message(&slack_msg("archive the old sprint board")).unwrap();
    assert_eq!(p.task_type, TaskType::Delete);
}

#[test]
fn slack_source_carries_user_and_channel() {
    let msg = SlackMessage {
        user_id: "U999".into(),
        channel_id: "C123".into(),
        text: "show sprint health".into(),
    };
    let p = from_message(&msg).unwrap();
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
    let p = from_message(&msg).unwrap();
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
