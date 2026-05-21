---
name: standup-relay
tier: Core
description: MCP connection to Slack and other tools. Delivers daily updates, asks for progress, and captures blockers. Updates Jira automatically based on each user's response.
---

# Standup Relay

Drives the daily standup loop via Slack. Delivers prompts to each team member, captures progress and blockers in natural language, and writes updates back to Jira automatically — no manual ticket updates required.

## Input

- Slack MCP connection (authenticated)
- Jira board with active sprint and assignees
- Team member list with Slack handles

## Output

- Daily progress prompt delivered to each team member via Slack
- Parsed standup responses: what was done, what's next, blockers
- Jira ticket status updated automatically based on responses
- Escalated blockers surfaced for blocker-tracker and escalation-engine

## When to use

Every working day during an active sprint. Trigger at the start of the standup window. Receives pre-digested blocker and drift signals from supporting skills before delivery.

## Connections

- Receives input from: **drift-detector**, **blocker-tracker**, **escalation-engine**

## System Loop

Pre-project → **Active sprint** → Meetings → Communication → Risk & accountability → Project close

- Receives: Project structure and goals from **Pre-project**
- Receives: Escalation signals from **Risk & accountability** *(dashed — escalation path)*
- Feeds into: **Meetings**
