---
name: risk-scanner
tier: Supplementary
description: Identifies patterns that predict slippage — repeated ticket reassignment, ticket age, scope change rate — at the ticket, person, and project level. Generates a prioritized risk register with confidence scores.
---

# Risk Scanner

Analyzes historical ticket patterns to surface the early signals that predict project slippage. Looks across ticket age, reassignment frequency, and scope change rate to build a risk register with confidence scores before issues become delays.

## Input

- Jira ticket history: creation date, transitions, assignee changes, estimate revisions
- Scope change indicators: description edits, acceptance criteria changes, story point revisions
- Sprint completion history per team member

## Output

- Prioritized risk register with entries at three levels:
  - **Ticket level**: tickets showing slippage patterns
  - **Person level**: assignees with systematic estimation gaps or reassignment patterns
  - **Project level**: aggregate risk score with trend direction
- Confidence score per risk entry
- Recommended mitigations ranked by impact

## When to use

Run weekly during any active sprint. Always run before sprint planning when historical data is available. Feed output into escalation-engine when high-confidence risks are unacknowledged.

## Connections

- Feeds into: **escalation-engine**

## System Loop

Pre-project → Active sprint → Meetings → Communication → **Risk & accountability** → Project close

- Receives: Project narrative and status from **Communication**
- Feeds into: **Project close**
- Also sends: Escalation signals back to **Active sprint** *(dashed — escalation path)*
