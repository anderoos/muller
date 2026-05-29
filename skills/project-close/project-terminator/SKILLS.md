---
name: project-terminator
tier: Core
description: Triggered when all tickets reach done. Schedules the retrospective, notifies stakeholders, archives the Jira board, and initiates the knowledge transfer and estimation feedback loop.
---

# Project Terminator

Handles the full project close sequence automatically once every ticket reaches done. Ensures nothing is left dangling — retrospective is scheduled, stakeholders are notified, the board is archived, and the knowledge transfer and calibration loops are kicked off.

## Input

- Jira board with all tickets in `Done` status
- Stakeholder list and Slack handles
- Project metadata: name, dates, team roster, linked epics

## Output

- Retrospective meeting scheduled in the team calendar
- Stakeholder notification sent via Slack with project close summary
- Jira board archived
- knowledge-transfer triggered to produce the handoff document
- estimation-feedback-loop triggered to process estimate vs. actual data

## When to use

Trigger automatically when Jira detects all tickets in a project have reached `Done`. Can also be triggered manually when a project is wound down before full completion.

## Connections

- Triggers: **knowledge-transfer**, **estimation-feedback-loop**

## System Loop

Pre-project → Active sprint → Meetings → Communication → Risk & accountability → **Project close**

- Receives: Risk signals and project completion status from **Risk & accountability**
- Feeds back into: **Pre-project** via learning loop-back *(dashed — estimation calibration)*
