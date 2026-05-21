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

## When to use

Always use when initializing a new project. Run capacity-planner and okr-aligner first to ensure assignments are realistic and work is strategically aligned.

## Connections

- Consumes: **capacity-planner**, **okr-aligner**
- Use **risk-evaluation** to flag risks in the brief when the project has significant unknowns

## System Loop

**Pre-project** → Active sprint → Meetings → Communication → Risk & accountability → Project close

- Receives: Calibration data from **Project close** via estimation feedback loop *(learning loop-back)*
- Feeds into: **Active sprint**
