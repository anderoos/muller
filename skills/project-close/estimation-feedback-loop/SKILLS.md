---
name: estimation-feedback-loop
tier: New
description: At project close, compares all sprint estimates to actuals by ticket type, owner, and complexity. Writes calibration data back into the velocity tracker so the project initiator makes better estimates on the next project.
---

# Estimation Feedback Loop

Closes the planning accuracy loop. At project end, compares every estimate against what actually happened — broken down by ticket type, owner, and complexity — and writes calibration data back so future projects start with better numbers.

## Input

- All sprint ticket data: original estimates, revised estimates, actual time logged
- Ticket metadata: type, complexity, epic, assignee
- Velocity tracker historical data for baseline comparison
- User input on what went well, what there needs to be more of, less of and what caused the most friction (if applicable).

## Output

- Estimation accuracy report broken down by:
  - Ticket type (feature, bug, chore, spike)
  - Team member (systematic over/underestimation patterns)
  - Complexity tier
- Calibration factors written back to velocity-tracker
- Specific recommendations for how project-initiator should adjust on the next similar project

## When to use

Triggered automatically by project-terminator at project close. Should run for every project to build calibration data over time — the more projects tracked, the more accurate future estimates become.

## Connections

- Triggered by: **project-terminator**
- Feeds back into: **velocity-tracker**

## System Loop

Pre-project → Active sprint → Meetings → Communication → Risk & accountability → **Project close**

- Receives: Risk signals and project completion status from **Risk & accountability**
- Feeds back into: **Pre-project** via learning loop-back *(dashed — estimation calibration)*
- Feeds back into: **velocity-tracker**
