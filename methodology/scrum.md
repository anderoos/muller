# Scrum

## Overview & Philosophy

Scrum is a **lightweight framework** for developing and sustaining complex products. It is the most widely adopted Agile framework. Scrum uses fixed-length iterations (Sprints) and prescribed events to create a regular rhythm of inspection and adaptation.

Created by Ken Schwaber and Jeff Sutherland in the early 1990s. The current definitive reference is the **Scrum Guide (November 2020)**.

**Core belief:** Knowledge comes from experience; decisions should be based on what is observed. Scrum's empirical approach rests on three pillars: **Transparency, Inspection, and Adaptation.**

**Values:** Commitment, Focus, Openness, Respect, Courage.

## Work Structure

- **Product Backlog:** an emergent, ordered list of everything needed to improve the product. Single source of work. Each item has a description, order, and size.
- **Sprint Backlog:** the Sprint Goal (why) + selected Product Backlog items (what) + the plan for delivering them (how).
- **Increment:** a concrete, usable stepping stone toward the Product Goal. Multiple Increments may be created within a Sprint.
- Product Backlog items are refined into smaller items (typically ≤ 1 day of work) during **Backlog Refinement** (ongoing activity, not a formal event).

## Planning

- **Sprint Planning** kicks off each Sprint. Timeboxed to 8 hours max (for a 1-month Sprint).
  - Topic 1: *Why is this Sprint valuable?* → Sprint Goal
  - Topic 2: *What can be done?* → select items from Product Backlog
  - Topic 3: *How will it get done?* → decompose into tasks
- Planning horizon = one Sprint at a time, guided by the **Product Goal** (longer-term objective).
- Developers estimate based on past performance and capacity.

## Cadence

- **Sprints:** fixed-length, 1 month or less. A new Sprint starts immediately after the previous one ends.
- All events occur within the Sprint container.
- Sprint length is consistent (not variable sprint-to-sprint).

## Scope Changes

- **Sprint Goal is protected.** No changes that would endanger the Sprint Goal.
- Scope within a Sprint may be **clarified and renegotiated** with the Product Owner as more is learned — but the Sprint Goal does not change.
- The Product Backlog is continuously refined; new items can be added at any time for future Sprints.
- Only the Product Owner can **cancel a Sprint** (if the Sprint Goal becomes obsolete).

## Blockers

- Surfaced during the **Daily Scrum** (and throughout the day as needed).
- The **Scrum Master** is accountable for causing the removal of impediments to the team's progress.
- Team self-manages to resolve issues; Scrum Master escalates organizational impediments.

## Reporting & Metrics

- **Sprint Review:** inspect the Increment and adapt the Product Backlog. Working session with stakeholders (not just a presentation). Max 4 hours for a 1-month Sprint.
- Common metrics (not prescribed by the Scrum Guide): velocity, burndown charts, burnup charts, cumulative flow diagrams.
- Progress toward the Product Goal is inspected at least every Sprint.

## Ceremonies / Meetings (Scrum Events)

| Event | Timebox (1-month Sprint) | Purpose |
|-------|--------------------------|---------|
| Sprint Planning | 8 hours max | Define Sprint Goal, select items, plan work |
| Daily Scrum | 15 minutes | Inspect progress toward Sprint Goal, adapt plan for the day |
| Sprint Review | 4 hours max | Inspect Increment, discuss progress, adapt Product Backlog |
| Sprint Retrospective | 3 hours max | Reflect on process, identify improvements |
| The Sprint | 1 month max (container) | Contain all events and work |

For shorter Sprints, events are proportionally shorter.

## Roles (Accountabilities)

| Role | Accountability |
|------|---------------|
| **Product Owner** | Maximize product value. Owns the Product Backlog (ordering, clarity, communicating Product Goal). One person, not a committee. |
| **Scrum Master** | Establish and maintain Scrum. Enable team effectiveness. Remove impediments. Coach the team, PO, and organization. |
| **Developers** | Create the Increment. Own the Sprint Backlog. Adhere to the Definition of Done. Self-manage day-to-day work. |

Team size: **10 or fewer** people. No sub-teams or hierarchies within the Scrum Team.

## Definition of Done

- A **formal description** of the state of the Increment when it meets the quality measures required for the product.
- Agreed upon by the Scrum Team (or defined by organizational standards as a minimum).
- If a Product Backlog item does not meet the DoD, it **cannot be released or presented** at the Sprint Review — it returns to the Product Backlog.
- If multiple Scrum Teams work on the same product, they share the same DoD.

## Artifacts & Commitments

| Artifact | Commitment |
|----------|-----------|
| Product Backlog | Product Goal |
| Sprint Backlog | Sprint Goal |
| Increment | Definition of Done |

## Best Suited For

- Complex product development where requirements evolve.
- Teams of 10 or fewer people.
- Environments where stakeholder feedback is available at least every Sprint.
- Organizations willing to adopt the full Scrum framework (events, artifacts, accountabilities).

## Style-Specific: Empiricism + Purposeful Incompleteness

The Scrum Guide explicitly states: *"The Scrum framework is purposefully incomplete, only defining the parts required to implement Scrum theory."* Scrum provides the scaffolding; teams fill in the specific engineering practices, tools, and techniques. This is why Scrum is often combined with XP practices (TDD, pair programming) or Kanban practices (WIP limits, flow metrics).

---

**Sources:**
- Schwaber, K. & Sutherland, J. (2020). *The Scrum Guide.* https://scrumguides.org/scrum-guide.html
