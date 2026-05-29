---
name: summarizer
tier: Core
description: Produces a plain-English summary of the project including timeline, key events, decisions made, and current status. Readable by any audience — technical or not.
---

# Summarizer

Produces a plain-English narrative of the project that any audience can read — no Jira access or domain knowledge required. Covers where the project started, what happened, what was decided, and where things stand now.

## Input

- Jira project board with full ticket history
- Decision log (from decision-logger)
- Sprint history and key milestone dates

## Output

- Plain-English project summary covering:
  - Timeline and milestones
  - Key events and inflection points
  - Decisions made and their outcomes
  - Current status and what's left
- Formatted for readability by both technical and non-technical audiences

## When to use

On-demand when a stakeholder, new team member, or executive needs project context. Runs automatically as input to stakeholder-briefer and new-member-onboarder. Useful at project close for knowledge transfer.

## Connections

- Feeds into: **stakeholder-briefer**, **new-member-onboarder**

## System Loop

Pre-project → Active sprint → Meetings → **Communication** → Risk & accountability → Project close

- Receives: Meeting notes, decisions, and project narrative from **Meetings**
- Feeds into: **Risk & accountability**
