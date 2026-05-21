---
name: sprint-health-check
tier: Supplementary
description: Mid-sprint analysis of highest-risk items and cross-ticket dependencies. Delivers an on-track / at-risk / off-rails verdict with a specific recommended action per flagged item.
---

# Sprint Health Check

Mid-sprint analysis that gives the team a clear read on delivery risk before it's too late to act. Examines the highest-risk tickets, cross-ticket dependencies, and current velocity to produce a verdict and specific action per concern.

## Input

- Current sprint board state: ticket statuses, remaining estimates, days left
- Cross-ticket dependency map
- Velocity data for the current sprint vs. historical baseline

## Output

- Per-ticket verdict: `on-track` | `at-risk` | `off-rails`
- Dependency conflict map showing which blocked items cascade
- One specific recommended action per flagged item
- Overall sprint verdict with confidence level

## When to use

Run at the sprint midpoint and again 2–3 days before sprint review. Trigger manually if standup signals accumulating risk. If any item is rated `off-rails`, automatically trigger project-optimizer.

## Connections

- Triggers: **project-optimizer** when critical risk is flagged

## System Loop

Pre-project → **Active sprint** → Meetings → Communication → Risk & accountability → Project close

- Receives: Project structure and goals from **Pre-project**
- Receives: Escalation signals from **Risk & accountability** *(dashed — escalation path)*
- Feeds into: **Meetings**
