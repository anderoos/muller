---
name: stakeholder-briefer
tier: Supplementary
description: Auto-generates monthly status emails for non-technical audiences. Links to relevant Jira epics, translates technical progress into business language, and flags any risks requiring executive attention.
---

# Stakeholder Briefer

Generates polished monthly status emails that translate technical progress into business language. Ensures executives and non-technical stakeholders stay informed without requiring engineering time to produce updates.

## Input

- Summarizer output (project narrative and current status)
- Jira epics with status and completion percentages
- Risk register or flagged items from risk-scanner
- Milestone and delivery date data

## Output

- Monthly status email formatted for non-technical audiences:
  - Progress summary in business terms
  - Links to relevant Jira epics
  - Upcoming milestones and delivery dates
  - Risks or blockers requiring executive attention
- Ready to send or reviewed before dispatch

## When to use

Generate monthly during active projects. Also trigger before board reviews, investor updates, or any stakeholder touchpoint requiring a formal status report.

## Connections

- Consumes: **summarizer**
- Receives escalated items from: **escalation-engine**

## System Loop

Pre-project → Active sprint → Meetings → **Communication** → Risk & accountability → Project close

- Receives: Meeting notes, decisions, and project narrative from **Meetings**
- Feeds into: **Risk & accountability**
