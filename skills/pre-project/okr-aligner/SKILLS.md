---
name: okr-aligner
tier: New
description: Maps proposed project work against company OKRs before kick-off. Surfaces misaligned tickets and ensures sprint goals ladder up to strategic priorities before a ticket is written.
---

# OKR Aligner

Maps proposed project work against company OKRs before kick-off. Surfaces tickets that don't support any strategic objective and ensures every sprint goal ladders up to a priority before work begins.

## Input

- Proposed project scope or draft ticket list
- Current company or team OKRs (key results and objectives)

## Output

- Alignment map: each proposed work item tagged to the OKR it supports
- List of misaligned or unanchored tickets with recommended disposition (cut, reframe, or park)
- Summary confidence score for overall strategic alignment of the proposed sprint

## When to use

Always run before kick-off and before any tickets are formally written. Required before project-initiator creates the Jira structure.

## Connections

- Feeds into: **project-initiator**

## System Loop

**Pre-project** → Active sprint → Meetings → Communication → Risk & accountability → Project close

- Receives: Calibration data from **Project close** via estimation feedback loop *(learning loop-back)*
- Feeds into: **Active sprint**
