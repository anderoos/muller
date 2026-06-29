use super::types::TaskType;

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

/// Collapse runs of whitespace and trim leading/trailing space.
pub fn normalize(raw: &str) -> String {
    raw.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Infer a CRUD classification from the prompt text.
/// Falls back to `Get` when no write keyword is found.
pub fn infer_task_type(text: &str) -> TaskType {
    let lower = text.to_lowercase();
    if DELETE_KEYWORDS.iter().any(|kw| lower.contains(kw)) {
        return TaskType::Delete;
    }
    if UPDATE_KEYWORDS.iter().any(|kw| lower.contains(kw)) {
        return TaskType::Update;
    }
    if INSERT_KEYWORDS.iter().any(|kw| lower.contains(kw)) {
        return TaskType::Insert;
    }
    TaskType::Get
}

/// Return the first sentence of the normalised prompt as the user's goal.
/// Falls back to the full text when no sentence boundary is found.
pub fn extract_goal(normalized: &str) -> String {
    let first = normalized
        .split(['.', '!', '?'])
        .next()
        .unwrap_or(normalized)
        .trim();

    if first.is_empty() {
        normalized.to_string()
    } else {
        first.to_string()
    }
}
