---
name: drift-detector
tier: Supplementary
description: Flags tickets that haven't moved or have changed scope silently. Detects reassignments, stale estimates, and creeping requirements before they compound.
---

# Drift Detector

Scans the active sprint for tickets that have drifted without being explicitly flagged. Catches silent scope changes, stale estimates, and quiet reassignments before they surface as surprises in standup or sprint review.

## Input

- Jira ticket history: status transitions, comment activity, assignee changes
- Estimate fields (original vs. current)
- Description and acceptance criteria change history

## Output

- List of drifted tickets with drift type: `stale` | `reassigned` | `scope-creeping`
- Severity score per ticket based on how long drift has gone unaddressed
- Brief summary of what changed and when, ready to surface in standup

## When to use

Run automatically before each daily standup. Also useful mid-sprint if a ticket feels off. Output feeds directly into standup-relay so drift is surfaced proactively.

## Connections

- Ladders up to: **standup-relay**

## System Loop

Pre-project → **Active sprint** → Meetings → Communication → Risk & accountability → Project close

- Receives: Project structure and goals from **Pre-project**
- Receives: Escalation signals from **Risk & accountability** *(dashed — escalation path)*
- Feeds into: **Meetings**
