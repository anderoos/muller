# Kanban

## Overview & Philosophy

Kanban is a **management method** (not a methodology or framework) applied on top of an existing process or way of working. It uses visualization, work-in-progress limits, and flow management to improve service delivery. Kanban does not replace what you already do — it helps you improve it incrementally.

Originated from Lean Manufacturing (Toyota Production System). Adapted for knowledge work by **David J. Anderson** (*Kanban: Successful Evolutionary Change for Your Technology Business*, 2010). Governed by **Kanban University**.

**Core belief:** Optimize the flow of work through the system by making it visible, limiting overload, and evolving policies based on data.

## Principles

**Change Management Principles:**
1. Start with what you do now.
2. Agree to pursue improvement through evolutionary change.
3. Encourage acts of leadership at all levels.

**Service Delivery Principles:**
1. Understand and focus on customer needs and expectations.
2. Manage the work; let people self-organize around it.
3. Regularly review the network of services and its policies to improve outcomes.

## Work Structure

- Work items flow through a **Kanban board** that models the actual workflow (not an idealized one).
- Columns represent workflow stages (e.g., Backlog → Analysis → Development → Testing → Done).
- Work items can be any granularity: tasks, stories, features, campaigns, projects, etc.
- Different **work item types** may have different characteristics (size, speed, priority).
- **Classes of service** allow differentiated treatment (e.g., Expedite, Fixed Date, Standard, Intangible).

## Planning

- **No prescribed planning cadence.** Planning is continuous.
- Work is **pulled** into the system when capacity is available (pull system vs. push).
- **Replenishment meetings** decide which items enter the commitment point on the board.
- Planning is demand-driven and capacity-aware, not time-boxed to iterations.

## Cadence

- **No fixed iterations.** Work flows continuously.
- **Cadences** (recurring feedback loops) evolve over time. Common cadences at team level:
  - **Daily standup / Kanban meeting** — coordinate daily work.
  - **Replenishment meeting** — decide what new work enters the system.
  - **Delivery planning** — coordinate releases.
  - **Service delivery review** — review metrics and system performance.
  - **Risk review** — identify and mitigate delivery risks.
  - **Operations review** — broader organizational review.
  - **Strategy review** — alignment with business strategy.
- Frequency and duration are context-dependent. More frequent and shorter is generally preferred.

## Scope Changes

- **Handled naturally.** New work enters the backlog; it is pulled in when capacity allows.
- No iteration boundary to protect — reprioritization can happen at any time.
- **Classes of service** handle urgent items (Expedite class bypasses normal WIP limits).
- The system self-regulates: WIP limits prevent the team from being overwhelmed by scope additions.

## Blockers

- **Visualized on the board** (typically a blocker icon/tag on the card).
- Blocked items are highlighted during daily standups.
- The system's transparency makes blockers immediately visible.
- Persistent blockers are analyzed for root causes during **Service Delivery Reviews**.

## Reporting & Metrics

Core metrics:
- **Lead Time:** time from commitment point to delivery.
- **Delivery Rate (Throughput):** completed items per unit of time.
- **Work in Progress (WIP):** items in the system at a point in time.

Common charts:
- **Cumulative Flow Diagram (CFD):** visualizes flow across all stages over time.
- **Lead Time Distribution:** histogram of lead times — goal is narrow and shifted left.
- **Lead Time Run Chart:** sequential lead times over time to spot trends.

## Ceremonies / Meetings (Cadences)

Kanban does not prescribe ceremonies — it recommends **cadences** (feedback loops) that evolve with maturity:

| Cadence | Typical Frequency | Purpose |
|---------|-------------------|---------|
| Kanban Meeting (standup) | Daily | Coordinate work, surface blockers |
| Replenishment | Weekly or as needed | Select new work for the board |
| Delivery Planning | As needed | Coordinate what gets released |
| Service Delivery Review | Bi-weekly / monthly | Review metrics, identify improvements |
| Risk Review | Monthly | Assess and mitigate delivery risks |
| Operations Review | Monthly / quarterly | Cross-team / cross-service review |
| Strategy Review | Quarterly | Align services with business strategy |

## Roles

- **No prescribed roles.** Kanban is applied to existing organizational structures.
- Existing roles (project manager, team lead, developer, etc.) are preserved.
- The method emphasizes **"manage the work, let people self-organize around it."**
- As maturity grows, roles like **Service Delivery Manager** or **Service Request Manager** may emerge.

## Definition of Done

- Defined as **pull criteria** — explicit policies that state when a work item can move from one stage to the next.
- Each column transition on the board has its own "done" criteria.
- Policies are made explicit and posted visibly (ideally next to the board).
- There is no single "Definition of Done" event — done is defined per workflow stage.

## Best Suited For

- Teams that want to **improve an existing process** without wholesale change.
- Work with **high variability** in item size, priority, or arrival rate.
- **Support, operations, and maintenance** teams with continuous incoming work.
- Organizations that need to manage **multiple work types** with different service levels.
- Any context where flow optimization matters more than iteration planning.

## Style-Specific: WIP Limits as the Core Mechanism

The defining mechanism of Kanban is the **WIP limit** — a cap on the number of items allowed in a stage (or the whole system) at any time. WIP limits:
- Create a **pull system** (work is pulled when capacity exists, not pushed by a schedule).
- Expose bottlenecks (if a stage is always at its WIP limit, it's a constraint).
- Reduce context switching and multitasking.
- Improve flow predictability.

*"Stop starting, start finishing"* is the Kanban mantra. High utilization ≠ high throughput — just like a congested highway moves fewer cars per hour than a moderately loaded one.

## Style-Specific: STATIK

**Systems Thinking Approach To Introducing Kanban** — a repeatable process for designing a Kanban system:
1. Identify sources of dissatisfaction.
2. Analyze demand.
3. Analyze system capabilities.
4. Model the workflow.
5. Identify classes of service.
6. Design the Kanban system.

---

**Sources:**
- Anderson, D.J. (2010). *Kanban: Successful Evolutionary Change for Your Technology Business.*
- Kanban University. "The Official Guide to The Kanban Method." https://kanban.university/kanban-guide/
- Kanban University. "Principles and General Practices." https://kanban.university/principles-general-practices-kanban-method/
