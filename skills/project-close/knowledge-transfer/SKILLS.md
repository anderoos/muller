---
name: knowledge-transfer
tier: New
description: Generates a comprehensive handoff document from the full project history — decisions made, outcomes, what broke, lessons learned — formatted for future teams or audits. Pushed to the wiki automatically.
---

# Knowledge Transfer

Synthesizes the complete project history into a handoff document that future teams, auditors, or anyone revisiting the work can use without needing to reconstruct context from scratch. Published to the wiki automatically.

## Input

- Full Jira ticket history for the project
- Decision log (from decision-logger)
- Sprint retrospective notes
- Post-mortems or incident records if applicable
- Project summary (from summarizer)

## Output

- Comprehensive handoff document covering:
  - Project context and original goals
  - Key decisions made and their outcomes
  - What worked and what broke
  - Lessons learned and recommendations for future teams
  - Links to relevant tickets, epics, and external references
- Published to Confluence or Notion via MCP

## When to use

Triggered automatically by project-terminator at project close. Can also run manually when a key team member rolls off mid-project or when documentation is needed for an audit.

## Connections

- Triggered by: **project-terminator**
- Consumes: **decision-logger**, **summarizer**
- Triggers: **confluence-docs-generator**

## System Loop

Pre-project → Active sprint → Meetings → Communication → Risk & accountability → **Project close**

- Receives: Risk signals and project completion status from **Risk & accountability**
- Feeds back into: **Pre-project** via learning loop-back *(dashed — estimation calibration)*
