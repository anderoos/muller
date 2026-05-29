---
name: docs-wiki-generator
tier: New
description: Synthesizes transcriber output, decision log, and summarizer into a structured project wiki page or runbook. Pushes to Confluence or Notion via MCP. Keeps living documentation current without manual effort.
---

# Docs / Wiki Generator

Synthesizes the outputs of the transcriber, decision logger, and summarizer into a structured wiki page or runbook — then pushes it to Confluence or Notion automatically. Living documentation stays current without anyone manually writing it.

## Input

- Meeting transcriber output (meeting notes and action items)
- Decision log entries (from decision-logger)
- Project summary (from summarizer)
- Target destination: Confluence space key or Notion page ID

## Output

- Structured wiki page or runbook containing:
  - Project overview and purpose
  - Key decisions with attribution and reasoning
  - Meeting summaries and outcomes
  - Current status and open items
- Published to Confluence or Notion via MCP; existing page updated in place if one exists

## When to use

Run after significant milestones (sprint review, major decision, architecture change). Also trigger at project close to produce the final project record. Set up on a recurring schedule to keep docs current throughout the project.

## Connections

- Consumes: **meeting-note-transcriber**, **decision-logger**, **summarizer**

## System Loop

Pre-project → Active sprint → Meetings → **Communication** → Risk & accountability → Project close

- Receives: Meeting notes, decisions, and project narrative from **Meetings**
- Feeds into: **Risk & accountability**
