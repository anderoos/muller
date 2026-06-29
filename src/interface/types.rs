use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
    Get,
    Insert,
    Update,
    Delete,
}

impl TaskType {
    pub fn is_write(&self) -> bool {
        !matches!(self, TaskType::Get)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            TaskType::Get    => "get",
            TaskType::Insert => "insert",
            TaskType::Update => "update",
            TaskType::Delete => "delete",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InterfaceSource {
    Cli,
    Slack { user_id: String, channel_id: String },
}

/// Structured prompt package passed from the interface layer to the orchestration layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptPayload {
    /// Unique request identifier, used for tracing.
    pub id: String,
    /// Surface that produced this payload.
    pub source: InterfaceSource,
    /// Unmodified text exactly as received from the user.
    pub raw_input: String,
    /// Cleaned, whitespace-normalised prompt ready for the LLM.
    pub normalized_prompt: String,
    /// One-sentence statement of what the user wants to achieve.
    pub goal: String,
    /// Optional background (e.g. Slack channel, PM style, prior session data).
    pub context: Option<String>,
    /// CRUD classification used to enforce dev/release write guards.
    pub task_type: TaskType,
    /// Skills/methodology file to inject into the system prompt.
    pub skill: Option<String>,
}

impl PromptPayload {
    pub fn new(
        source: InterfaceSource,
        raw_input: impl Into<String>,
        normalized_prompt: impl Into<String>,
        goal: impl Into<String>,
        context: Option<String>,
        task_type: TaskType,
        skill: Option<impl Into<String>>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            source,
            raw_input: raw_input.into(),
            normalized_prompt: normalized_prompt.into(),
            goal: goal.into(),
            context,
            task_type,
            skill: skill.map(Into::into),
        }
    }
}
