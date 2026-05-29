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

## Jira MCP Tools

Use these tools for standup execution:
1. `jira_search_issues` with JQL `project = PROJECT AND sprint in openSprints() AND assignee = MEMBER` — fetch each team member's open tickets before messaging them
2. `jira_get_transitions` — list the valid status transitions for a ticket
3. `jira_transition_issue` — move a ticket to Done / In Progress / Blocked based on the member's standup response
4. `jira_add_comment` — log the standup response as a comment on relevant tickets
5. `jira_update_issue` — update the `priority` field or add blockers when a member reports one

## When to use

Every working day during an active sprint. Trigger at the start of the standup window. Receives pre-digested blocker and drift signals from supporting skills before delivery.

## Connections

- Receives input from: **drift-detector**, **blocker-tracker**, **escalation-engine**

## System Loop

Pre-project → **Active sprint** → Meetings → Communication → Risk & accountability → Project close

- Receives: Project structure and goals from **Pre-project**
- Receives: Escalation signals from **Risk & accountability** *(dashed — escalation path)*
- Feeds into: **Meetings**
