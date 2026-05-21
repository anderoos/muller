---
name: pre-meeting-briefer
tier: New
description: Fires 15 minutes before a calendar event. Surfaces open blockers relevant to attendees, tickets last discussed by the group, outstanding decisions from the prior session, and a suggested agenda.
---

# Pre-Meeting Briefer

Fires 15 minutes before a calendar event with a compact brief that lets attendees walk in prepared. Surfaces the unresolved threads that actually need the group's attention so the meeting can start in context.

## Input

- Calendar event with attendee list
- Jira tickets assigned to or recently touched by attendees
- Decision log from the prior session with this group
- Open blockers relevant to attendees

## Output

- Briefing delivered to all attendees via Slack:
  - Open blockers relevant to this group
  - Tickets last discussed in a prior session with these attendees
  - Outstanding decisions from the previous meeting
  - Suggested agenda based on open items

## When to use

Trigger automatically 15 minutes before any calendar event that includes tracked team members. Most valuable for recurring ceremonies (sprint planning, retros, design reviews) and stakeholder syncs.

## Connections

- Feeds into: **meeting-note-transcriber** (as pre-loaded context)

## System Loop

Pre-project → Active sprint → **Meetings** → Communication → Risk & accountability → Project close

- Receives: Sprint activity and team context from **Active sprint**
- Feeds into: **Communication**
