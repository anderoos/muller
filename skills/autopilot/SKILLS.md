---
name: autopilot
description: Receives user-suggested improvements and logs them as behavioral directives. Supports three commands — override, add, less — that modify how subsequent agents run without touching base skill code. Always read this file before invoking any other skill.
---

# Autopilot

Autopilot is a lightweight instruction layer that sits above all other skills. It captures user-suggested improvements as directives and applies them at runtime — before any subsequent skill executes. Base skill files are never modified.

**Always read this file first before running any other skill.**

## Commands

| Command | Effect |
|---------|--------|
| `override <behavior>` | Replaces a default behavior with the specified alternative |
| `add <behavior>` | Adds a new behavior on top of existing defaults |
| `less <behavior>` | Removes or suppresses a default behavior |

## Input

A user instruction in one of the three command forms:

```
override [what to change] → [new behavior]
add [new behavior to layer on]
less [behavior to suppress or remove]
```

## Output

The directive is appended to the **User Directives** section at the bottom of this file. No base skill code is modified. On the next run, all agents read this file first and apply the logged directives before executing.

## When to use

Invoke autopilot whenever the user wants to tune how the system behaves across sessions — without editing individual skill files. Acts as the persistent preference and override layer for all other skills.

---

## User Directives

<!-- Directives are appended below this line. Do not remove this comment. -->
