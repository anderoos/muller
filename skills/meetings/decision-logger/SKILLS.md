---
name: decision-logger
tier: Supplementary
description: Captures and assigns decisions from meetings with owner attribution, timestamp, and reasoning. Kept separate from action items so decisions are searchable and attributable long after the meeting ends.
---

# Decision Logger

Extracts decisions from meeting transcripts and stores them in a dedicated, searchable log — separate from action items. Ensures that what was decided, by whom, and why remains findable and attributable months later.

## Input

- Meeting transcript or transcriber output
- Attendee list with roles
- Meeting context (project, sprint, topic)

## Output

- Decision log entries, each with:
  - Decision text
  - Owner (who made or owns the decision)
  - Timestamp
  - Reasoning or context behind the decision
  - Meeting reference
- Pushed to the project decision log (Jira, Confluence, or Notion via MCP)

## When to use

Run after every meeting where significant decisions were made. Works best when fed the meeting-note-transcriber output directly. Kept strictly separate from action items to preserve decision history integrity.

## Connections

- Ladders up to: **meeting-note-transcriber**
- Consumed by: **docs-wiki-generator**, **new-member-onboarder**

## System Loop

Pre-project → Active sprint → **Meetings** → Communication → Risk & accountability → Project close

- Receives: Sprint activity and team context from **Active sprint**
- Feeds into: **Communication**
