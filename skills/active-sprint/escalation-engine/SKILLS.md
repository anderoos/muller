---
name: escalation-engine
tier: New
description: Time-based trigger that fires when blockers or risk flags go unacknowledged. Auto-pings the ticket owner, then their manager, then surfaces in standup — in sequence with configurable thresholds.
---

# Escalation Engine

Fires when blockers or risk flags go unacknowledged past a configurable time threshold. Walks an escalation chain — ticket owner first, then manager, then standup — so nothing stays buried due to inattention.

## Input

- Unacknowledged blocker flags from blocker-tracker
- Risk signals from risk-scanner and ownership-audit
- Escalation threshold configuration: hours before each escalation step triggers
- Team member hierarchy (owner → manager mapping)

## Output

- Sequenced escalation notifications via Slack: owner → manager → standup
- Escalation log with timestamps, acknowledgment status, and resolution outcome
- Unresolved items surfaced in standup-relay for team visibility

## When to use

Runs automatically on a time-based schedule during active sprints. Fires whenever a blocker or risk flag remains unacknowledged past the configured threshold. Thresholds should be set per project based on sprint cadence.

## Connections

- Ladders up to: **standup-relay**
- Receives signals from: **blocker-tracker**, **risk-scanner**, **ownership-audit**

## System Loop

Pre-project → **Active sprint** → Meetings → Communication → Risk & accountability → Project close

- Receives: Project structure and goals from **Pre-project**
- Receives: Escalation signals from **Risk & accountability** *(dashed — escalation path)*
- Feeds into: **Meetings**
