# Interface layer

This module converts every inbound surface (CLI subcommands, free-text queries,
Slack messages) into a single `PromptPayload` that the agent consumes. `main.rs`
stays thin routing, and `agent.rs` never needs to know which surface a request
came from.

```
CLI subcommand ──► cli_adapter::from_command ──┐
CLI free text  ──► cli_adapter::from_raw_query ─┼─► PromptPayload ─► agent::run_payload
Slack message  ──► slack_adapter::from_message ─┘
```

Each payload carries the raw input, a normalized prompt, a one-line goal, a
CRUD task type, and the skill to inject into the system prompt. The task type
drives the write guards: dev builds block write commands entirely and force
read-only Jira access at the MCP layer (`READ_ONLY_MODE`); release builds run
the plan → review → confirm → execute flow for writes, while read tasks are
answered directly with no confirmation step.

## Agent commands

| Command | What it does | Task type | Skill activated |
|---|---|---|---|
| `mueller ask "<query>"` | Answers a direct question about tickets, sprint, or project status | Get | `ASKS` |
| `mueller "<query>"` (no subcommand) | Free text. A leading command verb (`ask …`, `standup`, `health`, …) activates the same prompt and skill as the subcommand; otherwise the task type is inferred from keywords | inferred | verb's skill, or `ASKS` for reads |
| `mueller init "<brief or path>"` | Starts a new project from a brief: breaks it into epics/stories/tasks and produces a full Jira structure ready for sprint planning. A file path is resolved and read as the brief | Insert | `pre-project/project-initiator` |
| `mueller update "<ticket>"` | Updates a ticket: status changes, meeting notes, scope adjustments, reassignment | Update | `ASKS` |
| `mueller standup` | Triggers the daily standup relay: prompts each team member for progress, captures blockers, updates ticket statuses | Update | `active-sprint` |
| `mueller health` | Runs a sprint health check: analyses statuses, dependencies, and velocity; returns an on-track / at-risk / off-rails verdict per flagged item | Get | `active-sprint/sprint-health-check` |
| `mueller log [--file <path>]` | Transcribes a meeting, extracts action items, decisions, and takeaways, and pushes tasks to Jira. Omit `--file` to paste the transcript | Insert | `meetings/meeting-note-transcriber` |
| `mueller brief` | Generates a pre-meeting brief for the next calendar event: open blockers, relevant tickets, outstanding decisions, suggested agenda | Get | `meetings/pre-meeting-briefer` |
| `mueller scan` | Scans the active sprint for drift, stale tickets, slippage patterns, and accountability gaps | Get | `risk-evaluation` |
| `mueller summarize` | Produces a plain-English project summary for technical and non-technical audiences | Get | `communication/summarizer` |
| `mueller onboard "<member>"` | Generates an onboarding brief for a new team member joining mid-project, delivered to Slack | Get | `communication/new-member-onboarder` |
| `mueller close` | Closes the project: schedules the retro, notifies stakeholders, archives the board, triggers knowledge transfer | Delete | `project-close/project-terminator` |

Skill names resolve against the `skills/` directory: `ASKS` loads
`skills/ASKS.md`; nested names like `active-sprint/sprint-health-check` load
`skills/<name>/SKILLS.md`. The `skills/autopilot/SKILLS.md` file is loaded
into **every** agent run in addition to the command's skill.

## Housekeeping commands (no agent dispatch, no skill)

| Command | What it does |
|---|---|
| `mueller login` | Authenticates with Claude, then runs setup |
| `mueller setup` | Configures Jira/Slack credentials and project settings |
| `mueller dashboard` | Serves the local LangSmith-compatible trace dashboard on port 6007 |
| `mueller autopilot add\|override\|less "<behavior>"` | Appends a behavioral directive to the always-loaded autopilot skill |
| `mueller --refresh-embeddings` | Re-embeds changed methodology files into ChromaDB |

`--dump-payload` (hidden flag) prints the constructed `PromptPayload` as JSON
and exits without running the agent — used by `scripts/test_interface.py`.

## Normalization and validation

`normalizer.rs` is surface-agnostic; adapters handle surface-specific cleanup
before calling it.

- **Normalize**: strips control and zero-width characters, collapses
  spaces/tabs within lines, trims lines, collapses blank-line runs. Newlines
  are preserved so multi-line briefs and transcripts keep their structure.
- **Validate** (free-text paths only — `from_raw_query`, `from_message`):
  rejects prompts that are empty after cleanup or over `MAX_PROMPT_CHARS`
  (10,000 chars) with a `PromptError`.
- **Command-word routing** (raw queries and Slack): free text starting with a
  command verb maps to the equivalent subcommand and activates its prompt
  template and skill. Verbs that need an argument (`ask`, `init`, `update`,
  `onboard`) match only when followed by text; bare verbs (`standup`,
  `health`, `brief`, `scan`, `summarize`, `close`, `log`) match only as the
  whole message, so "scan ticket ABC-1 for problems" keeps its specifics and
  falls through to inference.
- **Task-type inference** (free text with no command verb): whole-word keyword
  matching, checked most-destructive first (Delete → Update → Insert, falling
  back to Get). Read-classified text gets the `ASKS` skill; inferred writes
  get none. This is advisory routing only — write enforcement happens at the
  MCP layer and the release confirmation flow, so a misclassified write fails
  safe into read-only mode.
- **Slack cleanup** (`slack_adapter`): strips the leading bot mention, rewrites
  mentions/channel refs/links (`<@U123>` → `@U123`, `<#C456|general>` →
  `#general`, `<url|label>` → `label (url)`), and decodes Slack's HTML
  entities.
- **Prompt-injection hygiene**: user text interpolated into command templates
  is wrapped in delimiter tags (`<brief>`, `<ticket>`, `<file>`,
  `<new_member>`) so the model can distinguish data from instructions.

Unit tests for all of the above live in `tests.rs`; integration tests that
exercise the compiled binary live in `scripts/test_interface.py`.
