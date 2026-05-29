# Project Management Styles: A Comparative Reference

## Executive Summary

This guide provides lean, structured descriptions of five major project management styles: **Waterfall, Agile, Scrum, Kanban, and Extreme Programming (XP)**. Each style is described across standardized categories — work structure, planning, cadence, scope changes, blockers, reporting, ceremonies, definition of done, roles, and best-fit scenarios — plus style-specific characteristics.

**Key relationships:**
- **Agile** is an umbrella philosophy (values + principles), not a specific process [5][6].
- **Scrum** and **XP** are Agile frameworks — they implement Agile principles with concrete practices [1][4].
- **Kanban** is a management method that can be applied to any existing process, including Agile ones [3].
- **Waterfall** is a sequential, plan-driven approach that predates and contrasts with Agile [7][8].

Each style has a dedicated reference file with full details. This document provides a cross-cutting comparison.

## Individual Style Files

| Style | File | One-Line Summary |
|-------|------|-----------------|
| Waterfall | [outputs/styles/waterfall.md](styles/waterfall.md) | Sequential phases, upfront planning, formal gates |
| Agile | [outputs/styles/agile.md](styles/agile.md) | Umbrella philosophy: iterative, collaborative, adaptive |
| Scrum | [outputs/styles/scrum.md](styles/scrum.md) | Fixed-length sprints, prescribed events, empirical process control |
| Kanban | [outputs/styles/kanban.md](styles/kanban.md) | Continuous flow, WIP limits, evolutionary improvement |
| XP | [outputs/styles/xp.md](styles/xp.md) | Engineering-first, TDD, pair programming, embrace change |

## Comparison Matrix

| Category | Waterfall | Agile (Philosophy) | Scrum | Kanban | XP |
|----------|-----------|-------------------|-------|--------|-----|
| **Type** | Methodology | Philosophy / values | Framework | Management method | Methodology |
| **Work Unit** | Phase deliverables | Increments | Product Backlog Items → Increments | Work items (any granularity) | User Stories → Tasks |
| **Planning** | Upfront, comprehensive | Continuous, adaptive | Per Sprint (Sprint Planning) | Continuous, demand-driven | Planning Game (release + iteration) |
| **Cadence** | Linear phases | Iterative (framework-dependent) | Fixed Sprints (1–4 weeks) | Continuous flow (no iterations) | Short iterations (1–2 weeks) |
| **Scope Changes** | Formal change control; discouraged | Welcomed (Principle #2) | Sprint Goal protected; backlog evolves | Absorbed naturally via pull system | Embraced; reprioritize at next planning |
| **Blockers** | Escalate to PM; halts downstream | Surface quickly; team resolves | Daily Scrum + Scrum Master removes | Visualized on board; pull signals | Standup + pair programming + coach |
| **Reporting** | Gantt charts, EVM, phase gates | Working software = progress | Sprint Review + optional metrics | Lead time, throughput, CFD | Velocity, test pass rates, build status |
| **Ceremonies** | Kickoff, phase gates, post-mortem | Framework-dependent | 5 events (Sprint, Planning, Daily, Review, Retro) | Cadences (daily, replenishment, reviews) | Planning, standup, demo, retro |
| **DoD** | Phase exit criteria + acceptance | Team-defined | Formal DoD per Increment | Pull criteria per workflow stage | All tests pass + paired + integrated + refactored |
| **Roles** | PM, BA, Dev, QA, Sponsor | Not prescribed | PO, Scrum Master, Developers | Not prescribed (overlay on existing) | Customer, Developer, Coach, Tracker, Tester |
| **Team Size** | Variable (often large) | Not prescribed | ≤ 10 | Not prescribed | Small (2–12 developers) |
| **Best For** | Stable requirements, regulated industries | Evolving requirements, fast feedback | Complex product development | Improving existing processes, ops/support | High-quality software, changing requirements |

## Key Differentiators

### Waterfall vs. Everything Else
Waterfall is the only purely **sequential** approach [8]. All others embrace some form of iteration or continuous flow. Waterfall assumes requirements can be fully known upfront; the others assume they cannot [7][8].

### Scrum vs. Kanban
Both are popular in software teams, but they differ fundamentally:
- **Scrum** uses fixed-length Sprints with committed scope [1]; **Kanban** uses continuous flow with no iterations [3].
- **Scrum** prescribes roles (PO, SM, Developers) [1]; **Kanban** preserves existing roles [3].
- **Scrum** has 5 mandatory events [1]; **Kanban** recommends cadences that evolve organically [3].
- Both can use WIP limits, but in Kanban they are the **core mechanism** [3]; in Scrum they are optional.

### XP vs. Scrum
Both are Agile frameworks with iterations, but:
- **XP** prescribes engineering practices (TDD, pair programming, CI, refactoring) [4][10]; **Scrum** is silent on engineering [1].
- **XP** requires an on-site customer [10]; **Scrum** requires a Product Owner [1].
- XP and Scrum are frequently **combined** — Scrum for project structure, XP for engineering discipline [10].

### Agile vs. Specific Frameworks
Agile is the **philosophy** [5]; Scrum, XP, Kanban, Crystal, DSDM, SAFe, etc. are **implementations**. Saying "we do Agile" without specifying a framework is incomplete — it's like saying "we eat healthy" without specifying a diet.

## Caveats & Open Questions

1. **Hybrid approaches are common.** Most real-world teams blend practices (e.g., Scrum + Kanban = "Scrumban," Scrum + XP practices). Pure implementations are rare.
2. **Agile scaling frameworks** (SAFe, LeSS, Nexus) were not covered here. They extend these base styles to large organizations.
3. **Other methodologies** exist (Crystal, DSDM, FDD, PRINCE2, Lean Software Development) but were outside the requested scope.
4. **Context matters.** No style is universally "best." The right choice depends on project type, team maturity, organizational culture, regulatory environment, and customer availability.

## Sources

1. Schwaber, K. & Sutherland, J. (2020). *The Scrum Guide.* https://scrumguides.org/scrum-guide.html
2. Anderson, D.J. (2010). *Kanban: Successful Evolutionary Change for Your Technology Business.*
3. Kanban University. "The Official Guide to The Kanban Method." https://kanban.university/kanban-guide/
4. Beck, K. (1999). *Extreme Programming Explained: Embrace Change.* Addison-Wesley.
5. Agile Manifesto (2001). https://agilemanifesto.org/
6. Agile Manifesto Principles. http://agilemanifesto.org/principles
7. Royce, W.W. (1970). "Managing the Development of Large Software Systems."
8. Asana. "Waterfall Project Management Methodology." https://asana.com/resources/waterfall-project-management-methodology
9. PMI. "Waterfall Methodology Agile Approach." https://www.pmi.org/learning/library/waterfall-methodology-agile-approach-5821
10. Wikipedia. "Extreme programming." https://en.wikipedia.org/wiki/Extreme_programming
11. Wikipedia. "Extreme programming practices." https://en.wikipedia.org/wiki/Extreme_Programming_Practices
12. Atlassian. "Agile Manifesto." https://www.atlassian.com/en/agile/manifesto
13. Kanban University. "Principles and General Practices." https://kanban.university/principles-general-practices-kanban-method/
