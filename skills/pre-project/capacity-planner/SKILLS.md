---
name: capacity-planner
tier: New
description: Checks team bandwidth before work is assigned. Reads open tickets, PTO, and current velocity per person to flag overallocation before sprint planning begins. Feeds into project-initiator.
---

# Capacity Planner

Checks team bandwidth before any work is assigned. Reads open tickets, PTO, and current velocity per person to flag overallocation risks before sprint planning begins.

## Input

- Open Jira tickets with assignees and estimates
- PTO calendar or time-off data per team member
- Historical velocity per person (from velocity-tracker if available)

## Output

- Per-person bandwidth report showing available capacity vs. assigned load
- List of overallocated team members with specific conflict details
- Recommended capacity buffer per person for the upcoming sprint

## When to use

Always run before sprint planning or before assigning new work to the team. Required input step for project-initiator when staffing a new project.

## Connections

- Feeds into: **project-initiator**
- Consumes: **velocity-tracker** data when available

## System Loop

**Pre-project** → Active sprint → Meetings → Communication → Risk & accountability → Project close

- Receives: Calibration data from **Project close** via estimation feedback loop *(learning loop-back)*
- Feeds into: **Active sprint**
