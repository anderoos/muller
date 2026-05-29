---
name: escalation-engine
tier: New
description: Receives signals from the risk scanner, blocker tracker, and ownership audit. Auto-escalates unresolved items through a configurable path — owner → manager → standup → stakeholder briefer.
---

# Escalation Engine

Receives risk signals from multiple upstream skills and escalates unresolved items through a configurable chain — ticket owner first, then manager, then standup, then stakeholder briefer — until the item is acknowledged or resolved.

## Input

- Risk flags from risk-scanner (with confidence scores)
- Unresolved blockers from blocker-tracker
- Accountability gaps from ownership-audit
- Escalation path configuration: thresholds (hours before each step), team hierarchy

## Output

- Escalation notifications delivered in sequence via Slack:
  1. Ticket owner pinged with context and requested action
  2. Manager notified if owner does not acknowledge within threshold
  3. Item surfaced in standup-relay for team visibility
  4. Stakeholder briefer notified if item remains unresolved past final threshold
- Escalation log: timestamps, acknowledgments, resolution status

## When to use

Runs automatically on a schedule during active projects. Fires whenever upstream signals indicate an unacknowledged risk or accountability gap. Thresholds should be calibrated to the project's sprint cadence and urgency level.

## Connections

- Consumes: **risk-scanner**, **blocker-tracker**, **ownership-audit**
- Ladders up to: **standup-relay**, **stakeholder-briefer**

## System Loop

Pre-project → Active sprint → Meetings → Communication → **Risk & accountability** → Project close

- Receives: Project narrative and status from **Communication**
- Feeds into: **Project close**
- Also sends: Escalation signals back to **Active sprint** *(dashed — escalation path)*
