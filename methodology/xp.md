# Extreme Programming (XP)

## Overview & Philosophy

Extreme Programming (XP) is an **Agile software development methodology** that takes proven engineering practices to their logical extremes. If code review is good, review code constantly (pair programming). If testing is good, test every change automatically (TDD). If short iterations are good, make them very short.

Created by **Kent Beck** during the Chrysler C3 payroll project (late 1990s). Published in *Extreme Programming Explained: Embrace Change* (1999).

**Core belief:** The cost of changing software can be kept low through disciplined engineering practices, enabling teams to embrace rather than resist changing requirements.

**Values:** Communication, Simplicity, Feedback, Courage, Respect.

## Work Structure

- Work is expressed as **User Stories** — short descriptions of desired functionality written by the customer.
- Stories are estimated by the development team in relative units (story points or ideal days).
- Stories are broken into **engineering tasks** during iteration planning.
- Tasks are typically ≤ 1–2 days of work.
- All code is **collectively owned** — any developer can modify any part of the codebase.

## Planning

- **The Planning Game:** a structured negotiation between business and development.
  - **Release Planning:** customer selects stories for the next release; team estimates effort. Customer sets priorities and scope; team sets technical estimates and velocity.
  - **Iteration Planning:** team selects stories for the current iteration and breaks them into tasks.
- Planning is **adaptive** — the plan is updated every iteration based on actual velocity.
- **"Yesterday's weather":** use last iteration's velocity to predict next iteration's capacity.

## Cadence

- **Short iterations:** typically **1–2 weeks** (XP originally recommended 1–3 weeks).
- **Small, frequent releases** to production.
- Feedback loops at multiple timescales:
  - Seconds/minutes: pair programming, unit tests.
  - Hours/days: continuous integration, daily standup.
  - Weeks: iteration demo, customer acceptance tests.
  - Months: release planning.

## Scope Changes

- **Embraced.** XP Principle: *"Embrace change."*
- Customer can add, remove, or reprioritize stories at any release planning session.
- Within an iteration, the team commits to selected stories — mid-iteration scope changes are discouraged but the next iteration can absorb new priorities.
- Low cost of change is maintained through engineering practices (TDD, refactoring, simple design, CI).

## Blockers

- Surfaced during **daily standups** and through continuous pair programming (partners notice blockers immediately).
- The **XP Coach** helps remove impediments and maintain practices.
- **On-site customer** resolves requirement ambiguities in real time.
- Small iterations and continuous integration mean blockers have limited blast radius.

## Reporting & Metrics

- **Velocity:** stories (or story points) completed per iteration. Primary planning metric.
- **Acceptance test pass rate:** customer-written tests that validate stories.
- **Unit test pass rate:** all tests must pass at all times.
- **Build status:** continuous integration server shows red/green.
- XP favors **working software** over status reports. The running, tested system is the report.

## Ceremonies / Meetings

| Ceremony | Frequency | Purpose |
|----------|-----------|---------|
| Release Planning | Per release cycle | Customer selects stories, team estimates, agree on release scope |
| Iteration Planning | Start of each iteration | Select stories, break into tasks, commit to iteration scope |
| Daily Standup | Daily | Quick sync — what I did, what I'll do, blockers |
| Iteration Demo | End of each iteration | Show working software to customer, run acceptance tests |
| Retrospective | End of each iteration | Reflect on process, adjust practices |

## Roles

| Role | Responsibility |
|------|---------------|
| **Customer** | Writes user stories, sets priorities, defines acceptance tests. Available on-site. |
| **Developer** | Writes code, unit tests, estimates stories. Works in pairs. |
| **XP Coach** | Ensures practices are followed. Mentors the team. Removes impediments. |
| **Tracker** | Monitors team velocity and progress. Raises early warnings. |
| **Tester** | Helps customer write acceptance tests. Runs and reports test results. |

## Definition of Done

- A story is done when:
  1. All **unit tests** pass (written before the code — TDD).
  2. All **acceptance tests** pass (written by the customer).
  3. Code has been **pair-programmed** (or extensively reviewed).
  4. Code is **integrated** into the main branch and the build is green.
  5. Code follows the team's **coding standards**.
  6. Design has been **refactored** to remove duplication (Simple Design rules).

## The 12 Practices

Grouped into four areas:

**Fine-Scale Feedback:**
1. **Pair Programming** — two developers at one workstation.
2. **Planning Game** — structured business/dev negotiation.
3. **Test-Driven Development (TDD)** — write the test before the code.
4. **Whole Team** — customer is a full team member, available on-site.

**Continuous Process:**
5. **Continuous Integration** — integrate and test multiple times per day.
6. **Refactoring (Design Improvement)** — continuously improve code structure without changing behavior.
7. **Small Releases** — release to production frequently in small increments.

**Shared Understanding:**
8. **Coding Standards** — team agrees on consistent code style.
9. **Collective Code Ownership** — anyone can change any code.
10. **Simple Design** — the simplest thing that works. YAGNI (You Aren't Gonna Need It).
11. **System Metaphor** — shared story/analogy that describes how the system works.

**Programmer Welfare:**
12. **Sustainable Pace** — no overtime. 40-hour weeks. Rested developers produce better code.

## Best Suited For

- **Small to medium teams** (2–12 developers) working on software.
- Projects with **rapidly changing requirements**.
- Teams with access to an **engaged customer/product owner**.
- Organizations willing to invest in engineering discipline (TDD, CI, pair programming).
- Environments where **software quality** is a top priority.

## Style-Specific: Engineering-First

XP is unique among Agile frameworks in its heavy emphasis on **specific engineering practices**. While Scrum is silent on how developers should write code, XP prescribes TDD, pair programming, refactoring, CI, and simple design. This makes XP the most technically prescriptive Agile methodology — and the one most focused on keeping the cost of change low through code quality.

---

**Sources:**
- Beck, K. (1999). *Extreme Programming Explained: Embrace Change.* Addison-Wesley.
- Wikipedia. "Extreme programming." https://en.wikipedia.org/wiki/Extreme_programming
- Wikipedia. "Extreme programming practices." https://en.wikipedia.org/wiki/Extreme_Programming_Practices
- C2 Wiki. "Extreme Programming Core Practices." http://xp.c2.com/ExtremeProgrammingCorePractices.html
