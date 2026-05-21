---
name: project-optimizer
tier: Supplementary
description: Handles mid-sprint re-planning, scope cuts, and pivot decisions. Triggered when the health check flags critical risk. Proposes trade-offs and updates Jira accordingly.
---

# Project Optimizer

Handles mid-sprint replanning when the health check signals the sprint is off-rails. Proposes concrete trade-offs — scope cuts, deprioritizations, or pivots — and executes the agreed changes in Jira.

## Input

- Sprint health check output with flagged items and verdicts
- Current sprint scope and remaining capacity
- Team availability for the remainder of the sprint

## Output

- Ranked list of scope cut or pivot options with trade-off analysis per option
- Recommended replanning path with rationale
- Updated Jira tickets reflecting agreed changes (status, scope, assignments)
- Summary of what was cut and why, for the retrospective record

## When to use

Only trigger when sprint-health-check flags a `critical` risk or `off-rails` verdict. Not for routine sprint management — this is a replanning intervention.

## Connections

- Triggered by: **sprint-health-check**

## System Loop

Pre-project → **Active sprint** → Meetings → Communication → Risk & accountability → Project close

- Receives: Project structure and goals from **Pre-project**
- Receives: Escalation signals from **Risk & accountability** *(dashed — escalation path)*
- Feeds into: **Meetings**
