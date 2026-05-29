---
name: velocity-tracker
tier: New
description: Tracks actual vs estimated hours per ticket, per person, and per ticket type over time. Feeds the project initiator and estimation feedback loop to improve future planning accuracy.
---

# Velocity Tracker

Tracks the gap between estimated and actual effort across every ticket, segmented by person, ticket type, and complexity. Builds the calibration dataset that makes future estimates progressively more accurate.

## Input

- Jira ticket estimates (original and any revisions)
- Actual time logged per ticket and per assignee
- Ticket metadata: type (feature, bug, chore), complexity, epic

## Output

- Velocity report: actual vs. estimated by person, ticket type, and sprint
- Trend lines showing estimation accuracy over time
- Calibration factors per ticket type and per person for future planning
- Exported calibration data consumed by project-initiator and estimation-feedback-loop

## When to use

Run at the end of each sprint. Data accumulates over time — the more sprints tracked, the more accurate the calibration. Essential input for project-initiator when staffing and scoping a new project.

## Connections

- Feeds into: **project-initiator**, **estimation-feedback-loop**

## System Loop

Pre-project → **Active sprint** → Meetings → Communication → Risk & accountability → Project close

- Receives: Project structure and goals from **Pre-project**
- Receives: Escalation signals from **Risk & accountability** *(dashed — escalation path)*
- Feeds into: **Meetings**
