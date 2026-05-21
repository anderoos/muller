---
name: meeting-note-transcriber
tier: Core
description: Event-triggered on meeting end. Transcribes audio to text, extracts action items, decisions, purpose, and takeaways. Pushes tasks to Jira and makes suggestions based on each attendee's role, experience, and performance history.
---

# Meeting Note Transcriber

Fires automatically at the end of a meeting. Converts audio to text, extracts the structured signal from the conversation, and pushes action items to Jira — removing the manual note-taking burden entirely.

## Input

- Meeting audio recording (or real-time transcript)
- Attendee list with roles and Jira usernames
- Optional: attendee performance history and open ticket context

## Output

- Full meeting transcript
- Extracted action items with suggested Jira assignees based on attendee role
- Decision list (handed to decision-logger for separate storage)
- Meeting purpose, key takeaways, and next steps
- Jira tasks created and assigned automatically

## When to use

Trigger automatically on meeting end via calendar integration or recording hook. Works for sprint ceremonies, stakeholder syncs, design reviews, and ad-hoc calls.

## Connections

- Feeds into: **decision-logger**, **docs-wiki-generator**, **new-member-onboarder**
- Context enriched by: **pre-meeting-briefer** when run beforehand

## System Loop

Pre-project → Active sprint → **Meetings** → Communication → Risk & accountability → Project close

- Receives: Sprint activity and team context from **Active sprint**
- Feeds into: **Communication**
