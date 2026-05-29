---
name: confluence-docs-generator
tier: New
description: Generates and publishes a structured Confluence page for a project, product, outcome, or policy. Covers description, members involved, project close date, key decisions, risks, and all pertinent context — formatted for Confluence and pushed via MCP.
---

# Confluence Docs Generator

Generates a complete, structured Confluence page from project data. Runs after knowledge-transfer so the canonical page is grounded in the full project history — not just current state. Works for projects, products, outcomes, and policies alike. Can be commanded ad-hoc to append and link newer information and documentation to existing doc to avoid duplicity.

## Input

From **knowledge-transfer** output:
- Project context and original goals
- Key decisions made with owner attribution and reasoning
- What worked, what broke, and lessons learned
- Links to relevant tickets, epics, and external references

Additional inputs:
- Project name and type (`project` | `product` | `outcome` | `policy`)
- Description of the product, outcome, or policy — what it is and why it exists
- Team roster: names, roles, and Confluence/Jira usernames
- Project close date and key milestone dates
- Jira epic or board reference (used to pull final status and linked tickets)
- Any additional context: risks, dependencies, success metrics, related pages

## Output

A published Confluence page with the following layout:

| Section | Content |
|---------|---------|
| **Overview** | Plain-English description of the product, outcome, or policy and its purpose |
| **Team** | Members involved with name, role, and @mention |
| **Timeline** | Project start, key milestones, and project close date |
| **Objectives & Success Metrics** | What success looks like and how it was measured |
| **Key Decisions** | Decisions made with owner attribution and reasoning (from knowledge-transfer) |
| **Outcomes & Lessons Learned** | What worked, what broke, and recommendations for future teams |
| **Risks & Dependencies** | Known risks encountered and how they were resolved |
| **Resources** | Links to Jira epic, handoff document, related pages, and tickets |

Page is created or updated in-place via MCP Confluence integration. Page metadata (owner, space, parent page) is set based on project context.

## When to use

Triggered after knowledge-transfer completes at project close. Also run manually when documenting a new policy or product outcome, or when a stakeholder needs a single canonical link for the project record.

## Connections

- Triggered by: **knowledge-transfer**
- Consumes: **knowledge-transfer**, **decision-logger**, **summarizer**

## System Loop

Pre-project → Active sprint → Meetings → Communication → Risk & accountability → **Project close**

Within Project close: project-terminator → knowledge-transfer → **confluence-docs-generator** → estimation-feedback-loop

- Receives: Full project history and handoff document from **knowledge-transfer**
- Feeds back into: **Pre-project** via learning loop-back *(dashed — estimation calibration)*
