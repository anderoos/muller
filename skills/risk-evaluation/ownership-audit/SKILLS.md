---
name: ownership-audit
tier: New
description: Periodically reviews whether every open ticket has a clear, active owner. Flags orphaned work, reassigned-but-unacknowledged tickets, and accountability gaps before they become delays.
---

# Ownership Audit

Scans every open ticket to verify it has a clear, active owner. Catches orphaned work and quiet reassignments — accountability gaps that cause delays precisely because no one notices them until it's too late.

## Input

- All open Jira tickets with current assignee and recent activity
- Assignee change history per ticket
- Team roster to validate active membership (detects tickets assigned to people who have left or rolled off)

## Output

- List of orphaned tickets (no active assignee)
- List of reassigned-but-unacknowledged tickets (reassigned with no response from new owner)
- Accountability gap summary by team and project
- Recommended resolution per flagged ticket: reassign, escalate, or close

## When to use

Run weekly during active sprints. Always run before sprint planning and before any team member rolls off or changes projects.

## Connections

- Ladders up to: **escalation-engine**

## System Loop

Pre-project → Active sprint → Meetings → Communication → **Risk & accountability** → Project close

- Receives: Project narrative and status from **Communication**
- Feeds into: **Project close**
- Also sends: Escalation signals back to **Active sprint** *(dashed — escalation path)*
