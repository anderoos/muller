# Waterfall

## Overview & Philosophy

Waterfall is a sequential, linear project management methodology where work flows through distinct phases in a fixed order. Each phase must be fully completed and approved before the next begins. It prioritizes upfront planning, comprehensive documentation, and predictability. Coined by Dr. Winston W. Royce in a 1970 paper, it is one of the oldest formalized PM approaches.

**Core belief:** If you define requirements thoroughly enough upfront, you can plan and execute the entire project predictably.

## Work Structure

- Work is broken into **phases**: Requirements → Design → Implementation → Testing → Deployment → Maintenance.
- Within each phase, tasks are defined in detail before execution begins.
- Deliverables from one phase become inputs to the next.
- No overlap between phases (in pure Waterfall).

## Planning

- **Upfront, comprehensive planning.** The bulk of planning occurs before any execution.
- A detailed **project requirements document** is created first, specifying scope, resources, team assignments, dependencies, and timelines.
- The **Iron Triangle** (scope, time, cost) governs all trade-offs — scope is typically fixed first.
- Plans are baseline-locked; changes require formal change control.

## Cadence

- **No iterations.** Work proceeds linearly through phases.
- Timeline is determined during planning; each phase has a scheduled start/end date.
- Progress is measured by phase completion, often tracked on a **Gantt chart**.

## Scope Changes

- **Strongly discouraged.** Scope is defined and locked during the Requirements phase.
- Changes after a phase is complete require a formal **change request** process.
- Late-stage changes are expensive and disruptive — they may force rework across multiple completed phases.
- A **Change Control Board (CCB)** typically reviews and approves/rejects change requests.

## Blockers

- Blockers in any phase halt all downstream work, since phases are sequential.
- **Escalation path:** blockers are raised to the project manager, who coordinates resolution.
- Third-party delays (e.g., vendor deliveries) can shift the entire project timeline.
- Risk mitigation relies on thorough upfront planning and buffer time.

## Reporting & Metrics

- **Phase gate reviews:** formal checkpoints where deliverables are reviewed before proceeding.
- **Gantt charts** for timeline tracking.
- **Earned Value Management (EVM):** tracks cost and schedule performance (common in government/defense).
- Status reports focus on phase completion %, budget burn, and milestone adherence.
- Heavy documentation creates an audit trail.

## Ceremonies / Meetings

- **Kickoff meeting:** aligns stakeholders on scope, plan, and roles.
- **Phase gate reviews / stage-gate meetings:** formal sign-off at the end of each phase.
- **Status meetings:** periodic updates to stakeholders on progress vs. plan.
- **Change Control Board meetings:** as needed to review change requests.
- **Post-mortem / Lessons learned:** conducted after project completion.

## Roles

- **Project Manager:** central authority; owns the plan, schedule, and coordination.
- **Business Analyst:** gathers and documents requirements.
- **Architects / Designers:** produce system and detailed design.
- **Developers / Engineers:** build the product per specifications.
- **QA / Testers:** verify the product against requirements.
- **Sponsor / Stakeholders:** approve phase gates and fund the project.

## Definition of Done

- **Per phase:** each phase has explicit exit criteria (deliverables reviewed, approved, and signed off).
- **Project level:** the product meets all documented requirements, passes acceptance testing, and is deployed.
- Done = fully completed, documented, and formally approved at each gate.

## Best Suited For

- Projects with **well-defined, stable requirements** (e.g., construction, manufacturing, regulatory/compliance).
- Environments requiring **extensive documentation** and audit trails.
- Fixed-price contracts where scope, timeline, and budget are contractually locked.
- Hardware or physical product development where rework is extremely costly.

## Style-Specific: Phase Gates

The defining structural element of Waterfall. Each phase ends with a formal review and approval before proceeding. This creates clear accountability and documentation but makes backtracking expensive.

---

**Sources:**
- Royce, W.W. (1970). "Managing the Development of Large Software Systems."
- Asana. "Waterfall Project Management Methodology." https://asana.com/resources/waterfall-project-management-methodology
- PMI. "Waterfall Methodology Agile Approach." https://www.pmi.org/learning/library/waterfall-methodology-agile-approach-5821
