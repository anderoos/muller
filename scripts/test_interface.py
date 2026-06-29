#!/usr/bin/env python3
"""
Interface-layer integration tests for Mueller.

Two evaluation passes per test case:
  1. Deterministic — exact checks on task_type, source, skill, JSON schema.
  2. LLM-as-judge  — LangChain chains evaluate whether the extracted goal and
                     normalised prompt semantically match user intent.

A third "variance" pass uses the LLM to generate 2–3 paraphrases of each raw
input and verifies that all paraphrases produce the same task_type (consistency
under surface-level input modulation).

Results are traced through LangSmith when LANGSMITH_TRACING=true is set.

Usage:
    python3 scripts/test_interface.py

Requirements:
    pip install -r scripts/requirements.txt
    cargo build  (produces target/debug/mueller)
    export ANTHROPIC_API_KEY=sk-ant-...
    # optional: export LANGSMITH_API_KEY=lsv2_... LANGSMITH_TRACING=true
"""

import json
import os
import shutil
import subprocess
import sys
from typing import Any

# ── dependency check ──────────────────────────────────────────────────────────

try:
    from langchain_anthropic import ChatAnthropic
    from langchain_core.output_parsers import JsonOutputParser
    from langchain_core.prompts import ChatPromptTemplate
    from langsmith import traceable
except ImportError as exc:
    sys.stderr.write(
        f"Missing dependency: {exc}\n"
        "Install with:\n"
        "    pip install -r scripts/requirements.txt\n"
    )
    sys.exit(1)

# ── LLM setup ─────────────────────────────────────────────────────────────────

_LLM = ChatAnthropic(model="claude-haiku-4-5-20251001", temperature=0)

# ── binary location ───────────────────────────────────────────────────────────

def _find_binary() -> str:
    """Locate the mueller binary: PATH first, then local build outputs."""
    if shutil.which("mueller"):
        return "mueller"
    for candidate in [
        "target/debug/mueller",
        "target/release/mueller",
        "../target/debug/mueller",
    ]:
        if os.path.isfile(candidate):
            return candidate
    sys.exit(
        "mueller binary not found. Run `cargo build` first, or add it to PATH."
    )

BINARY = _find_binary()


def get_payload(args: list[str]) -> dict[str, Any]:
    """Run `mueller --dump-payload <args>` and return the parsed JSON payload."""
    result = subprocess.run(
        [BINARY, "--dump-payload"] + args,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        raise RuntimeError(
            f"mueller exited {result.returncode}:\n"
            f"stdout: {result.stdout}\nstderr: {result.stderr}"
        )
    return json.loads(result.stdout)


# ── test dataset ──────────────────────────────────────────────────────────────
#
# Each entry:
#   args          — CLI args passed to mueller (after --dump-payload)
#   expected_type — exact task_type string
#   expected_skill — exact skill path (None means any/no skill is fine)
#   intent_hint   — short phrase describing user intent, used by LLM evaluators

TEST_CASES = [
    {
        "id": "ask_sprint_status",
        "args": ["ask", "What is the current sprint status?"],
        "expected_type": "get",
        "expected_skill": "ASKS",
        "intent_hint": "query current sprint status",
    },
    {
        "id": "ask_blocker_summary",
        "args": ["ask", "List all open blockers and who owns them"],
        "expected_type": "get",
        "expected_skill": "ASKS",
        "intent_hint": "list open blockers with owners",
    },
    {
        "id": "init_simple_project",
        "args": ["init", "Build a customer portal with authentication and reporting"],
        "expected_type": "insert",
        "expected_skill": "pre-project/project-initiator",
        "intent_hint": "create a new project with specific features",
    },
    {
        "id": "update_ticket",
        "args": ["update", "ABC-42: move to in-progress and assign to Alice"],
        "expected_type": "update",
        "expected_skill": "ASKS",
        "intent_hint": "change ticket status and assignee",
    },
    {
        "id": "standup",
        "args": ["standup"],
        "expected_type": "update",
        "expected_skill": "active-sprint",
        "intent_hint": "run daily standup and collect progress",
    },
    {
        "id": "health",
        "args": ["health"],
        "expected_type": "get",
        "expected_skill": "active-sprint/sprint-health-check",
        "intent_hint": "evaluate sprint health and flag at-risk items",
    },
    {
        "id": "log_no_file",
        "args": ["log"],
        "expected_type": "insert",
        "expected_skill": "meetings/meeting-note-transcriber",
        "intent_hint": "transcribe meeting and create Jira tasks",
    },
    {
        "id": "log_with_file",
        "args": ["log", "--file", "meeting_notes.txt"],
        "expected_type": "insert",
        "expected_skill": "meetings/meeting-note-transcriber",
        "intent_hint": "transcribe a meeting file and push action items",
    },
    {
        "id": "brief",
        "args": ["brief"],
        "expected_type": "get",
        "expected_skill": "meetings/pre-meeting-briefer",
        "intent_hint": "generate pre-meeting context and agenda",
    },
    {
        "id": "scan",
        "args": ["scan"],
        "expected_type": "get",
        "expected_skill": "risk-evaluation",
        "intent_hint": "find drift and at-risk tickets in the sprint",
    },
    {
        "id": "summarize",
        "args": ["summarize"],
        "expected_type": "get",
        "expected_skill": "communication/summarizer",
        "intent_hint": "summarise the project for any audience",
    },
    {
        "id": "onboard_member",
        "args": ["onboard", "alice"],
        "expected_type": "get",
        "expected_skill": "communication/new-member-onboarder",
        "intent_hint": "onboard a new team member named alice",
    },
    {
        "id": "close_project",
        "args": ["close"],
        "expected_type": "delete",
        "expected_skill": "project-close/project-terminator",
        "intent_hint": "archive and close the project",
    },
    {
        "id": "raw_query_get",
        "args": ["What is the velocity for the last three sprints?"],
        "expected_type": "get",
        "expected_skill": None,
        "intent_hint": "retrieve historical sprint velocity",
    },
    {
        "id": "raw_query_insert",
        "args": ["Create a new epic for Q3 platform migration"],
        "expected_type": "insert",
        "expected_skill": None,
        "intent_hint": "create a new epic for a migration effort",
    },
]

# ── schema validation ─────────────────────────────────────────────────────────

REQUIRED_FIELDS = {"id", "source", "raw_input", "normalized_prompt", "goal", "task_type"}


def check_schema(payload: dict) -> list[str]:
    missing = REQUIRED_FIELDS - payload.keys()
    errors = [f"missing field: {f}" for f in sorted(missing)]
    if not isinstance(payload.get("id"), str) or not payload["id"]:
        errors.append("id must be a non-empty string")
    if not isinstance(payload.get("goal"), str) or not payload["goal"].strip():
        errors.append("goal must be a non-empty string")
    if not isinstance(payload.get("normalized_prompt"), str):
        errors.append("normalized_prompt must be a string")
    if payload.get("task_type") not in ("get", "insert", "update", "delete"):
        errors.append(f"task_type '{payload.get('task_type')}' is not a valid value")
    return errors


# ── deterministic checks ──────────────────────────────────────────────────────

def check_deterministic(case: dict, payload: dict) -> dict[str, Any]:
    errors = check_schema(payload)

    if payload.get("task_type") != case["expected_type"]:
        errors.append(
            f"task_type: got '{payload.get('task_type')}', "
            f"expected '{case['expected_type']}'"
        )

    expected_skill = case["expected_skill"]
    if expected_skill is not None and payload.get("skill") != expected_skill:
        errors.append(
            f"skill: got '{payload.get('skill')}', expected '{expected_skill}'"
        )

    # normalized_prompt must not have leading/trailing whitespace or double spaces
    norm = payload.get("normalized_prompt", "")
    if norm != norm.strip():
        errors.append("normalized_prompt has leading/trailing whitespace")
    if "  " in norm:
        errors.append("normalized_prompt contains double spaces")

    return {"passed": not errors, "errors": errors}


# ── LLM evaluators ────────────────────────────────────────────────────────────

_GOAL_EVAL_PROMPT = ChatPromptTemplate.from_messages([
    (
        "system",
        "You are evaluating an AI prompt-normalisation system.\n"
        "Given a raw user input and the extracted goal field, decide whether the "
        "goal accurately captures the user's primary intent in a single concise statement.\n"
        "Return ONLY valid JSON with two keys:\n"
        "  score: a float 0.0–1.0 (1.0 = perfect match, 0.0 = completely wrong)\n"
        "  reasoning: one sentence explaining the score\n"
        "Do not include markdown fences or any other text.",
    ),
    (
        "user",
        "Intent hint: {intent_hint}\n"
        "Raw input: {raw_input}\n"
        "Extracted goal: {goal}",
    ),
])

_NORM_EVAL_PROMPT = ChatPromptTemplate.from_messages([
    (
        "system",
        "You are evaluating an AI prompt-normalisation system.\n"
        "Decide whether the normalised_prompt is a clean, coherent version of the raw "
        "input — no extraneous whitespace, preserves the full intent, and reads naturally.\n"
        "Return ONLY valid JSON:\n"
        "  score: float 0.0–1.0\n"
        "  reasoning: one sentence\n"
        "Do not include markdown fences.",
    ),
    (
        "user",
        "Raw input: {raw_input}\n"
        "Normalised prompt: {normalized_prompt}",
    ),
])

_PARAPHRASE_PROMPT = ChatPromptTemplate.from_messages([
    (
        "system",
        "Generate exactly 3 different phrasings of the user request below that express "
        "the same intent. Vary tone, phrasing, and sentence structure but keep the core "
        "action identical.\n"
        "Return ONLY a JSON array of 3 strings — no markdown, no extra keys.",
    ),
    ("user", "{input}"),
])

_goal_chain = _GOAL_EVAL_PROMPT | _LLM | JsonOutputParser()
_norm_chain = _NORM_EVAL_PROMPT | _LLM | JsonOutputParser()
_paraphrase_chain = _PARAPHRASE_PROMPT | _LLM | JsonOutputParser()


@traceable(name="eval_goal_accuracy")
def eval_goal(raw_input: str, goal: str, intent_hint: str) -> dict:
    return _goal_chain.invoke(
        {"raw_input": raw_input, "goal": goal, "intent_hint": intent_hint}
    )


@traceable(name="eval_normalization_quality")
def eval_normalization(raw_input: str, normalized_prompt: str) -> dict:
    return _norm_chain.invoke(
        {"raw_input": raw_input, "normalized_prompt": normalized_prompt}
    )


@traceable(name="eval_input_variance")
def eval_variance(raw_input: str, expected_type: str) -> dict:
    """
    Generate paraphrases of raw_input and verify they all produce the same
    task_type.  Uses the raw-query path (no subcommand) so the normaliser
    and classifier are exercised on natural language directly.
    """
    try:
        paraphrases = _paraphrase_chain.invoke({"input": raw_input})
        if not isinstance(paraphrases, list):
            return {"passed": False, "error": "paraphrase chain did not return a list"}
    except Exception as exc:
        return {"passed": False, "error": str(exc)}

    results = []
    for phrase in paraphrases[:3]:
        try:
            p = get_payload([str(phrase)])
            results.append(
                {
                    "phrase": phrase,
                    "task_type": p.get("task_type"),
                    "match": p.get("task_type") == expected_type,
                }
            )
        except Exception as exc:
            results.append({"phrase": phrase, "error": str(exc), "match": False})

    consistent = all(r.get("match") for r in results)
    return {"passed": consistent, "paraphrases": results}


# ── test runner ───────────────────────────────────────────────────────────────

@traceable(name="run_interface_tests")
def run_tests() -> None:
    passed = 0
    failed = 0
    total = len(TEST_CASES)

    print(f"\nMueller interface-layer tests ({total} cases)\n{'─' * 60}")

    for case in TEST_CASES:
        case_id = case["id"]
        print(f"\n▸ {case_id}")

        # 1. Fetch payload from the binary
        try:
            payload = get_payload(case["args"])
        except Exception as exc:
            print(f"  [FAIL] could not get payload: {exc}")
            failed += 1
            continue

        # 2. Deterministic checks
        det = check_deterministic(case, payload)
        if det["passed"]:
            print("  [PASS] deterministic checks")
        else:
            for err in det["errors"]:
                print(f"  [FAIL] {err}")
            failed += 1
            continue

        # 3. LLM: goal accuracy
        try:
            goal_result = eval_goal(
                raw_input=payload["raw_input"],
                goal=payload["goal"],
                intent_hint=case["intent_hint"],
            )
            score = float(goal_result.get("score", 0))
            reasoning = goal_result.get("reasoning", "")
            threshold = 0.7
            if score >= threshold:
                print(f"  [PASS] goal accuracy  score={score:.2f}  {reasoning}")
            else:
                print(f"  [FAIL] goal accuracy  score={score:.2f} (< {threshold})  {reasoning}")
                failed += 1
                continue
        except Exception as exc:
            print(f"  [WARN] goal eval failed: {exc}")

        # 4. LLM: normalization quality
        try:
            norm_result = eval_normalization(
                raw_input=payload["raw_input"],
                normalized_prompt=payload["normalized_prompt"],
            )
            score = float(norm_result.get("score", 0))
            reasoning = norm_result.get("reasoning", "")
            if score >= 0.7:
                print(f"  [PASS] normalization  score={score:.2f}  {reasoning}")
            else:
                print(f"  [FAIL] normalization  score={score:.2f}  {reasoning}")
                failed += 1
                continue
        except Exception as exc:
            print(f"  [WARN] normalization eval failed: {exc}")

        # 5. Input variance (only for cases with enough natural-language surface area)
        if len(payload["raw_input"]) > 20:
            try:
                var = eval_variance(
                    raw_input=payload["raw_input"],
                    expected_type=case["expected_type"],
                )
                if var.get("passed"):
                    print("  [PASS] input variance — all paraphrases consistent")
                else:
                    print(f"  [WARN] input variance — inconsistencies detected")
                    for r in var.get("paraphrases", []):
                        mark = "✓" if r.get("match") else "✗"
                        phrase = r.get("phrase", r.get("error", "?"))
                        print(f"         {mark} {phrase!r} → {r.get('task_type', 'error')}")
            except Exception as exc:
                print(f"  [WARN] variance eval failed: {exc}")

        passed += 1

    # ── summary ──────────────────────────────────────────────────────────────
    print(f"\n{'─' * 60}")
    print(f"Results: {passed} passed, {failed} failed, {total} total")
    if failed:
        sys.exit(1)


if __name__ == "__main__":
    run_tests()
