---
name: project-initiator
tier: Core
description: Starts a new project from a brief. Breaks work down into epics and tickets, assigns owners based on role and capacity data, and creates the full Jira structure ready for sprint planning.
---

# Project Initiator

Starts a new project from a brief. Decomposes work into epics and tickets, assigns owners based on role and current capacity, and produces a Jira structure that is ready for sprint planning.

## Input

The brief must explicitly address all of the following:

- Purpose and business proposition
- Goals and objectives
- Scope of work
- Target audience and stakeholders
- Timeline and milestones
- Known constraints and assumptions
- Deliverables
- Measures of success

## Output

- Full Jira board structure: epics, stories, subtasks, and bugs as appropriate
- Owner assignments per ticket based on role fit and capacity data
- Sprint-ready backlog ordered by priority and dependency
- Flagged risks or gaps in the brief that need resolution before work begins

## Jira MCP Tools

Execute in this order:
1. `jira_get_project` — verify the target project exists and retrieve its issue type scheme
2. `jira_create_issue` with `issuetype: Epic` — create one epic per major workstream
3. `jira_create_issue` with `issuetype: Story` and `parent: EPIC-KEY` — create stories under each epic
4. `jira_create_issue` with `issuetype: Subtask` — break down complex stories where subtasks are needed
5. `jira_update_issue` — set `story_points`, `assignee`, `priority`, and `labels` on each created issue
6. `jira_search_issues` — verify the created board structure before reporting completion

Always set `assignee` based on role fit and capacity data from the brief.

## When to use

Always use when initializing a new project. Run capacity-planner and okr-aligner first to ensure assignments are realistic and work is strategically aligned.

## Connections

- Consumes: **capacity-planner**, **okr-aligner**
- Use **risk-evaluation** to flag risks in the brief when the project has significant unknowns

## System Loop

**Pre-project** → Active sprint → Meetings → Communication → Risk & accountability → Project close

- Receives: Calibration data from **Project close** via estimation feedback loop *(learning loop-back)*
- Feeds into: **Active sprint**
