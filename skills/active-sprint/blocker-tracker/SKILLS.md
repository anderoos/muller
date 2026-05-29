---
name: blocker-tracker
tier: Supplementary
description: Surfaces unresolved blockers before daily standup so delays are addressed proactively. Distinguishes technical blockers, dependency blockers, and decision blockers.
---

# Blocker Tracker

Surfaces all unresolved blockers before standup so nothing slips through without discussion. Categorizes each blocker by type so the team knows what kind of action is needed to unblock.

## Input

- Jira tickets flagged as blocked or with blocker-type linked issues
- Open dependency relationships between tickets
- Blocker comments or status history

## Output

- Prioritized list of unresolved blockers sorted by age and severity
- Blocker type per item: `technical` | `dependency` | `decision`
- Suggested owner and resolution path for each blocker
- Ready-to-surface summary for standup-relay

## When to use

Run before every daily standup. Also useful when a sprint is at risk and you need a rapid view of what's actually blocking delivery.

## Connections

- Ladders up to: **standup-relay**
- Feeds into: **escalation-engine** when blockers go unacknowledged

## System Loop

Pre-project → **Active sprint** → Meetings → Communication → Risk & accountability → Project close

- Receives: Project structure and goals from **Pre-project**
- Receives: Escalation signals from **Risk & accountability** *(dashed — escalation path)*
- Feeds into: **Meetings**
