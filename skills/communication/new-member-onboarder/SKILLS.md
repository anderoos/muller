---
name: new-member-onboarder
tier: New
description: When someone joins a project mid-flight, generates a structured brief covering project context, past key decisions, open blockers, their assigned tickets, and the team roster — delivered to Slack without consuming senior eng time.
---

# New Member Onboarder

Generates a structured onboarding brief for anyone joining an in-flight project. Delivers everything a new team member needs to get up to speed — without pulling senior engineers into lengthy knowledge-transfer calls.

## Input

- New member's Slack handle, role, and assigned Jira tickets
- Project summary (from summarizer)
- Decision log (from decision-logger)
- Current sprint board state and open blockers
- Team roster with roles and Slack handles

## Output

- Personalized onboarding brief delivered to the new member via Slack:
  - Project context and current status
  - Key decisions made and the reasoning behind them
  - Open blockers relevant to their assigned work
  - Their assigned tickets with context
  - Team roster and who to go to for what

## When to use

Trigger whenever a new person is added to a project that is already in flight. Works for full team members, contractors, and stakeholders joining mid-project.

## Connections

- Consumes: **summarizer**, **decision-logger**

## System Loop

Pre-project → Active sprint → Meetings → **Communication** → Risk & accountability → Project close

- Receives: Meeting notes, decisions, and project narrative from **Meetings**
- Feeds into: **Risk & accountability**
