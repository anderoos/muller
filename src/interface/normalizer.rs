use super::types::{PromptError, TaskType};

/// Maximum accepted prompt length, in characters, for free-text entry points.
pub const MAX_PROMPT_CHARS: usize = 10_000;

/// Maximum length of the extracted goal, in characters.
const MAX_GOAL_CHARS: usize = 120;

/// Invisible characters that survive copy-paste from rich-text sources and
/// would otherwise defeat keyword matching.
const ZERO_WIDTH: &[char] = &['\u{200B}', '\u{200C}', '\u{200D}', '\u{2060}', '\u{FEFF}'];

// Words that signal a Jira write — ordered most-specific first so Delete
// is checked before Update (e.g. "close" maps to Delete, not Update).
const DELETE_KEYWORDS: &[&str] = &[
    "delete", "remove", "archive", "cancel", "drop", "close project",
];

const UPDATE_KEYWORDS: &[&str] = &[
    "update", "change", "modify", "edit", "move", "reassign", "standup",
    "complete", "transition", "fix", "resolve", "adjust", "close ticket",
];

const INSERT_KEYWORDS: &[&str] = &[
    "create", "add", "new", "init", "start", "log", "insert", "push", "open ticket",
];

/// Clean a prompt while preserving its line structure:
/// - strips control characters (except newlines and tabs) and zero-width characters
/// - collapses runs of spaces/tabs within each line and trims each line
/// - collapses runs of blank lines to a single blank line
/// - drops leading/trailing blank lines
pub fn normalize(raw: &str) -> String {
    let cleaned: String = raw
        .chars()
        .filter(|&c| !ZERO_WIDTH.contains(&c) && (!c.is_control() || c == '\n' || c == '\t'))
        .collect();

    let mut lines: Vec<String> = Vec::new();
    for line in cleaned.lines() {
        let line = line.split_whitespace().collect::<Vec<_>>().join(" ");
        if !line.is_empty() || lines.last().is_some_and(|prev| !prev.is_empty()) {
            lines.push(line);
        }
    }
    while lines.last().is_some_and(|l| l.is_empty()) {
        lines.pop();
    }
    lines.join("\n")
}

/// Reject prompts that are empty after normalisation or beyond the length cap.
pub fn validate(normalized: &str) -> Result<(), PromptError> {
    if normalized.is_empty() {
        return Err(PromptError::Empty);
    }
    let len = normalized.chars().count();
    if len > MAX_PROMPT_CHARS {
        return Err(PromptError::TooLong { len, max: MAX_PROMPT_CHARS });
    }
    Ok(())
}

/// Infer a CRUD classification from the prompt text.
/// Matches whole words only, so "address" does not trigger "add" and
/// "backlog" does not trigger "log". Falls back to `Get` when no write
/// keyword is found.
///
/// This is advisory routing, not the enforcement point: dev builds also
/// enforce read-only Jira access at the MCP layer (READ_ONLY_MODE).
pub fn infer_task_type(text: &str) -> TaskType {
    let lower = text.to_lowercase();
    let words: Vec<&str> = lower
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .collect();

    if contains_keyword(&words, DELETE_KEYWORDS) {
        return TaskType::Delete;
    }
    if contains_keyword(&words, UPDATE_KEYWORDS) {
        return TaskType::Update;
    }
    if contains_keyword(&words, INSERT_KEYWORDS) {
        return TaskType::Insert;
    }
    TaskType::Get
}

fn contains_keyword(words: &[&str], keywords: &[&str]) -> bool {
    keywords.iter().any(|kw| {
        let kw_words: Vec<&str> = kw.split_whitespace().collect();
        words
            .windows(kw_words.len())
            .any(|window| window == kw_words.as_slice())
    })
}

/// Return the first sentence of the first line of the normalised prompt as
/// the user's goal, capped at MAX_GOAL_CHARS. A sentence only ends at `.`,
/// `!`, or `?` followed by whitespace or end-of-line, so ticket IDs and
/// version numbers ("PROJ-1.2") do not split the goal.
pub fn extract_goal(normalized: &str) -> String {
    let first_line = normalized.lines().next().unwrap_or(normalized).trim();

    let mut end = first_line.len();
    for (i, c) in first_line.char_indices() {
        if matches!(c, '.' | '!' | '?') {
            let at_boundary = first_line[i + c.len_utf8()..]
                .chars()
                .next()
                .map_or(true, |next| next.is_whitespace());
            if at_boundary {
                end = i;
                break;
            }
        }
    }

    let sentence = first_line[..end].trim();
    let goal = if sentence.is_empty() { first_line } else { sentence };
    truncate_chars(goal, MAX_GOAL_CHARS)
}

fn truncate_chars(s: &str, max: usize) -> String {
    let mut result: String = s.chars().take(max).collect();
    if result.len() < s.len() {
        result.truncate(result.trim_end().len());
        result.push('…');
    }
    result
}
