#!/usr/bin/env python3
"""Local LangSmith-compatible trace server + dashboard for Mueller.

LangSmith's hosted platform is closed source, but its wire protocol and Python
SDK are open. This server implements the LangSmith runs API locally:

    POST  /runs            create a run            (Rust tracer + langsmith SDK)
    PATCH /runs/{id}       close/update a run
    POST  /runs/batch      batched create/update   (langsmith SDK background queue)
    GET   /info            SDK handshake

so anything that can log to LangSmith — the Rust CLI, or any Python code using
the open-source `langsmith` SDK (`@traceable`) — can log here instead, with no
API key and no cloud. Runs are stored in SQLite and rendered on a built-in
dashboard.

If a LangSmith API key is configured (`observability.langsmith_api_key` in
~/.mueller/config.json, or LANGSMITH_API_KEY), every trace is additionally
mirrored to hosted LangSmith via the `langsmith` Python SDK — local dashboard
for quick reference, smith.langchain.com for the full product.

Usage:
    mueller dashboard               (or: python3 scripts/observability.py)
    open http://127.0.0.1:6007

Pointing things at it:
    Rust CLI:    `mueller setup` → Observability → Local dashboard
    Python SDK:  LANGSMITH_ENDPOINT=http://127.0.0.1:6007  LANGSMITH_TRACING=true

Environment overrides:
    MUELLER_OBS_PORT      server port            (default 6007)
    MUELLER_OBS_DB        SQLite path            (default ~/.mueller/traces.db)
    LANGSMITH_API_KEY     mirror traces to hosted LangSmith (overrides config)
    MUELLER_NO_BROWSER    set to skip opening the dashboard on start
"""

import gzip
import json
import os
import sqlite3
import sys
import webbrowser
from contextlib import asynccontextmanager
from datetime import datetime, timezone
from pathlib import Path

try:
    import uvicorn
    from fastapi import FastAPI, Request
    from fastapi.responses import HTMLResponse, JSONResponse
except ImportError as e:
    sys.stderr.write(
        f"Missing dependency: {e}\n"
        "Install the observability stack with:\n"
        "    pip install -r scripts/requirements.txt\n"
    )
    sys.exit(1)

PORT = int(os.getenv("MUELLER_OBS_PORT", "6007"))
DB_PATH = Path(os.getenv("MUELLER_OBS_DB", Path.home() / ".mueller" / "traces.db"))
CONFIG_PATH = Path.home() / ".mueller" / "config.json"

DB = None  # sqlite3 connection, opened in the lifespan hook


# ── hosted LangSmith mirroring (open-source `langsmith` SDK) ─────────────────

def load_langsmith_client():
    key = os.getenv("LANGSMITH_API_KEY")
    if not key:
        try:
            cfg = json.loads(CONFIG_PATH.read_text())
            key = (cfg.get("observability") or {}).get("langsmith_api_key")
        except (OSError, ValueError):
            key = None
    if not key:
        return None
    try:
        from langsmith import Client
    except ImportError:
        sys.stderr.write(
            "LangSmith mirroring is configured but the `langsmith` package is "
            "missing — running local-only. Install it with:\n"
            "    pip install -r scripts/requirements.txt\n"
        )
        return None
    # The SDK queues runs and posts from a background thread, so mirroring
    # never blocks ingest.
    return Client(api_key=key)


LANGSMITH = load_langsmith_client()
_mirror_warned = False


def iso_to_dt(value):
    if not isinstance(value, str) or not value:
        return None
    try:
        dt = datetime.fromisoformat(value.replace("Z", "+00:00"))
        return dt if dt.tzinfo else dt.replace(tzinfo=timezone.utc)
    except ValueError:
        return None


def mirror(action, **kwargs):
    global _mirror_warned
    if LANGSMITH is None:
        return
    try:
        action(**{k: v for k, v in kwargs.items() if v is not None})
    except Exception as e:
        if not _mirror_warned:
            sys.stderr.write(f"Warning: could not mirror trace to LangSmith: {e}\n")
            _mirror_warned = True


def mirror_create(run):
    if LANGSMITH is None or not run.get("id"):
        return
    # Batched SDK clients may deliver a finished run as a single create —
    # forward the completion fields too when they're already present.
    mirror(
        LANGSMITH.create_run,
        name=run.get("name", "run"),
        run_type=run.get("run_type", "chain"),
        inputs=run.get("inputs") or {},
        id=run.get("id"),
        trace_id=run.get("trace_id"),
        parent_run_id=run.get("parent_run_id"),
        dotted_order=run.get("dotted_order"),
        start_time=iso_to_dt(run.get("start_time")),
        end_time=iso_to_dt(run.get("end_time")),
        outputs=run.get("outputs"),
        error=run.get("error"),
        project_name=run.get("session_name"),
    )


def mirror_update(run_id, run):
    if LANGSMITH is None:
        return
    # PATCH bodies don't carry parent_run_id, but LangSmith rejects updates
    # whose dotted_order implies a child without one — recover it from the
    # stored create.
    parent_run_id = run.get("parent_run_id")
    if parent_run_id is None:
        row = DB.execute("SELECT parent_run_id FROM runs WHERE id = ?", (run_id,)).fetchone()
        parent_run_id = row["parent_run_id"] if row else None
    mirror(
        LANGSMITH.update_run,
        run_id=run_id,
        outputs=run.get("outputs"),
        error=run.get("error"),
        end_time=iso_to_dt(run.get("end_time")),
        trace_id=run.get("trace_id"),
        parent_run_id=parent_run_id,
        dotted_order=run.get("dotted_order"),
    )


# ── storage ──────────────────────────────────────────────────────────────────

SCHEMA = """
CREATE TABLE IF NOT EXISTS runs (
    id            TEXT PRIMARY KEY,
    trace_id      TEXT,
    parent_run_id TEXT,
    name          TEXT,
    run_type      TEXT,
    session_name  TEXT,
    start_time    TEXT,
    end_time      TEXT,
    start_ts      REAL,
    end_ts        REAL,
    inputs        TEXT,
    outputs       TEXT,
    error         TEXT,
    raw           TEXT
);
CREATE INDEX IF NOT EXISTS idx_runs_trace ON runs(trace_id);
CREATE INDEX IF NOT EXISTS idx_runs_start ON runs(start_ts);
"""

# Every ingest path funnels into the same upsert; COALESCE keeps whichever
# side has data, so create-then-patch and patch-before-create both work.
UPSERT = """
INSERT INTO runs (id, trace_id, parent_run_id, name, run_type, session_name,
                  start_time, end_time, start_ts, end_ts, inputs, outputs, error, raw)
VALUES (:id, :trace_id, :parent_run_id, :name, :run_type, :session_name,
        :start_time, :end_time, :start_ts, :end_ts, :inputs, :outputs, :error, :raw)
ON CONFLICT(id) DO UPDATE SET
    trace_id      = COALESCE(excluded.trace_id,      trace_id),
    parent_run_id = COALESCE(excluded.parent_run_id, parent_run_id),
    name          = COALESCE(excluded.name,          name),
    run_type      = COALESCE(excluded.run_type,      run_type),
    session_name  = COALESCE(excluded.session_name,  session_name),
    start_time    = COALESCE(excluded.start_time,    start_time),
    end_time      = COALESCE(excluded.end_time,      end_time),
    start_ts      = COALESCE(excluded.start_ts,      start_ts),
    end_ts        = COALESCE(excluded.end_ts,        end_ts),
    inputs        = COALESCE(excluded.inputs,        inputs),
    outputs       = COALESCE(excluded.outputs,       outputs),
    error         = COALESCE(excluded.error,         error),
    raw           = COALESCE(excluded.raw,           raw)
"""


def parse_ts(value):
    """ISO 8601 (Rust tracer and langsmith SDK variants) -> epoch milliseconds."""
    if not isinstance(value, str) or not value:
        return None
    try:
        normalized = value.replace("Z", "+00:00")
        dt = datetime.fromisoformat(normalized)
        if dt.tzinfo is None:
            dt = dt.replace(tzinfo=timezone.utc)  # SDK sometimes sends naive UTC
        return dt.timestamp() * 1000.0
    except ValueError:
        return None


def as_json_text(value):
    if value is None:
        return None
    return json.dumps(value, indent=2, default=str)


def upsert_run(run, run_id=None):
    rid = run_id or run.get("id")
    if not rid:
        return False
    DB.execute(UPSERT, {
        "id": str(rid),
        "trace_id": run.get("trace_id"),
        "parent_run_id": run.get("parent_run_id"),
        "name": run.get("name"),
        "run_type": run.get("run_type"),
        "session_name": run.get("session_name") or run.get("session_id"),
        "start_time": run.get("start_time"),
        "end_time": run.get("end_time"),
        "start_ts": parse_ts(run.get("start_time")),
        "end_ts": parse_ts(run.get("end_time")),
        "inputs": as_json_text(run.get("inputs")),
        "outputs": as_json_text(run.get("outputs")),
        "error": run.get("error"),
        "raw": json.dumps(run, default=str),
    })
    DB.commit()
    return True


def run_status(row):
    if row["error"]:
        return "error"
    if row["end_time"] is None:
        return "running"
    return "ok"


def duration_ms(row):
    if row["start_ts"] is not None and row["end_ts"] is not None:
        return max(0.0, row["end_ts"] - row["start_ts"])
    return None


# ── ingest API (LangSmith runs protocol) ─────────────────────────────────────

@asynccontextmanager
async def lifespan(_app):
    global DB
    DB_PATH.parent.mkdir(parents=True, exist_ok=True)
    DB = sqlite3.connect(DB_PATH, check_same_thread=False)
    DB.row_factory = sqlite3.Row
    DB.executescript(SCHEMA)
    yield
    DB.close()


app = FastAPI(title="mueller observability", lifespan=lifespan)


async def read_json(request):
    body = await request.body()
    if request.headers.get("content-encoding", "").lower() == "gzip":
        body = gzip.decompress(body)
    return json.loads(body)


@app.get("/info")
async def info():
    # Handshake for the langsmith SDK: declare plain JSON batching (no multipart).
    return {
        "version": "mueller-local",
        "instance_flags": {},
        "batch_ingest_config": {
            "use_multipart_endpoint": False,
            "size_limit": 100,
            "size_limit_bytes": 20971520,
            "scale_up_nthreads_limit": 1,
            "scale_up_qsize_trigger": 1000,
            "scale_down_nempty_trigger": 4,
        },
    }


@app.post("/runs")
async def create_run(request: Request):
    run = await read_json(request)
    if not upsert_run(run):
        return JSONResponse({"detail": "missing run id"}, status_code=400)
    mirror_create(run)
    return {"ok": True}


@app.patch("/runs/{run_id}")
async def update_run(run_id: str, request: Request):
    run = await read_json(request)
    upsert_run(run, run_id=run_id)
    mirror_update(run_id, run)
    return {"ok": True}


@app.post("/runs/batch")
async def batch(request: Request):
    payload = await read_json(request)
    for run in payload.get("post", []) or []:
        upsert_run(run)
        mirror_create(run)
    for run in payload.get("patch", []) or []:
        upsert_run(run)
        if run.get("id"):
            mirror_update(run["id"], run)
    return {"ok": True}


# ── dashboard API ────────────────────────────────────────────────────────────

@app.get("/api/traces")
async def list_traces(limit: int = 50):
    rows = DB.execute(
        """
        SELECT r.*,
               (SELECT COUNT(*) FROM runs c WHERE c.trace_id = r.trace_id)              AS run_count,
               (SELECT COUNT(*) FROM runs c WHERE c.trace_id = r.trace_id AND c.error IS NOT NULL) AS error_count
        FROM runs r
        WHERE r.parent_run_id IS NULL
        ORDER BY r.start_ts DESC
        LIMIT ?
        """,
        (limit,),
    ).fetchall()

    traces = []
    for row in rows:
        status = "error" if row["error_count"] else run_status(row)
        traces.append({
            "id": row["id"],
            "trace_id": row["trace_id"] or row["id"],
            "name": row["name"],
            "session_name": row["session_name"],
            "start_time": row["start_time"],
            "start_ts": row["start_ts"],
            "duration_ms": duration_ms(row),
            "status": status,
            "run_count": row["run_count"],
        })
    return traces


@app.get("/api/traces/{trace_id}")
async def trace_detail(trace_id: str):
    rows = DB.execute(
        "SELECT * FROM runs WHERE trace_id = ? OR id = ? ORDER BY start_ts",
        (trace_id, trace_id),
    ).fetchall()
    runs = []
    for row in rows:
        runs.append({
            "id": row["id"],
            "parent_run_id": row["parent_run_id"],
            "name": row["name"],
            "run_type": row["run_type"],
            "session_name": row["session_name"],
            "start_time": row["start_time"],
            "end_time": row["end_time"],
            "start_ts": row["start_ts"],
            "end_ts": row["end_ts"],
            "duration_ms": duration_ms(row),
            "status": run_status(row),
            "error": row["error"],
            "inputs": row["inputs"],
            "outputs": row["outputs"],
        })
    return {"trace_id": trace_id, "runs": runs}


# ── dashboard UI ─────────────────────────────────────────────────────────────

DASHBOARD_HTML = """<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>mueller traces</title>
<style>
  :root {
    --bg: #0d1117; --panel: #161b22; --border: #30363d; --text: #c9d1d9;
    --dim: #8b949e; --accent: #58a6ff; --ok: #3fb950; --err: #f85149;
    --running: #d29922; --llm: #a371f7; --chain: #58a6ff; --tool: #f0883e;
  }
  * { box-sizing: border-box; margin: 0; }
  body {
    background: var(--bg); color: var(--text);
    font: 13px/1.5 ui-monospace, SFMono-Regular, Menlo, monospace;
    height: 100vh; display: flex; flex-direction: column;
  }
  header {
    padding: 10px 16px; border-bottom: 1px solid var(--border);
    display: flex; align-items: baseline; gap: 12px;
  }
  header h1 { font-size: 15px; color: var(--accent); }
  header span { color: var(--dim); font-size: 12px; }
  main { flex: 1; display: grid; grid-template-columns: 330px 1fr 400px; min-height: 0; }
  section { border-right: 1px solid var(--border); overflow-y: auto; min-height: 0; }
  section:last-child { border-right: none; }
  h2 {
    font-size: 11px; text-transform: uppercase; letter-spacing: 1px; color: var(--dim);
    padding: 10px 14px 6px; position: sticky; top: 0; background: var(--bg);
  }
  .trace {
    padding: 9px 14px; border-bottom: 1px solid var(--border); cursor: pointer;
  }
  .trace:hover { background: var(--panel); }
  .trace.selected { background: var(--panel); border-left: 2px solid var(--accent); }
  .trace .row1 { display: flex; justify-content: space-between; gap: 8px; }
  .trace .name { white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .trace .meta { color: var(--dim); font-size: 11px; display: flex; gap: 10px; margin-top: 2px; }
  .dot { display: inline-block; width: 8px; height: 8px; border-radius: 50%; flex: none; margin-top: 5px; }
  .dot.ok { background: var(--ok); } .dot.error { background: var(--err); }
  .dot.running { background: var(--running); animation: pulse 1.2s infinite; }
  @keyframes pulse { 50% { opacity: .35; } }
  .span-row {
    display: grid; grid-template-columns: minmax(180px, 38%) 1fr 70px;
    gap: 10px; padding: 7px 14px; cursor: pointer; align-items: center;
    border-bottom: 1px solid #1c2128;
  }
  .span-row:hover { background: var(--panel); }
  .span-row.selected { background: var(--panel); box-shadow: inset 2px 0 0 var(--accent); }
  .span-name { display: flex; align-items: center; gap: 7px; min-width: 0; }
  .span-name em {
    font-style: normal; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
  }
  .kind {
    font-size: 9px; padding: 1px 5px; border-radius: 3px; flex: none;
    text-transform: uppercase; letter-spacing: .5px; color: var(--bg); font-weight: 700;
  }
  .kind.llm { background: var(--llm); } .kind.chain { background: var(--chain); }
  .kind.tool { background: var(--tool); } .kind.retriever { background: var(--ok); }
  .bar-lane { position: relative; height: 14px; background: #1c2128; border-radius: 3px; }
  .bar { position: absolute; top: 0; height: 100%; border-radius: 3px; min-width: 3px; opacity: .9; }
  .dur { text-align: right; color: var(--dim); font-size: 11px; }
  .detail { padding: 12px 14px; }
  .detail .field { margin-bottom: 14px; }
  .detail .label { font-size: 10px; text-transform: uppercase; letter-spacing: 1px; color: var(--dim); margin-bottom: 4px; }
  pre {
    background: var(--panel); border: 1px solid var(--border); border-radius: 6px;
    padding: 10px; overflow-x: auto; white-space: pre-wrap; word-break: break-word;
    font-size: 12px; max-height: 320px; overflow-y: auto;
  }
  pre.error { color: var(--err); border-color: var(--err); }
  .status-chip { padding: 1px 8px; border-radius: 10px; font-size: 11px; color: var(--bg); font-weight: 700; }
  .status-chip.ok { background: var(--ok); } .status-chip.error { background: var(--err); }
  .status-chip.running { background: var(--running); }
  .empty { color: var(--dim); padding: 24px 14px; text-align: center; }
</style>
</head>
<body>
<header>
  <h1>mueller traces</h1>
  <span>local LangSmith-compatible store · ingest at POST /runs · refreshes live</span>
</header>
<main>
  <section id="traces"><h2>Traces</h2><div id="trace-list"><div class="empty">no traces yet — run a mueller command</div></div></section>
  <section id="tree"><h2>Run tree</h2><div id="span-list"><div class="empty">select a trace</div></div></section>
  <section id="details"><h2>Details</h2><div id="detail" class="detail"><div class="empty">select a run</div></div></section>
</main>
<script>
let selectedTrace = null, selectedRun = null, runsCache = [];

const esc = s => String(s ?? "").replace(/[&<>"]/g, c => ({"&":"&amp;","<":"&lt;",">":"&gt;",'"':"&quot;"}[c]));
const fmtDur = ms => ms == null ? "—" : ms < 1000 ? Math.round(ms) + " ms"
  : ms < 60000 ? (ms / 1000).toFixed(1) + " s"
  : Math.floor(ms / 60000) + "m " + Math.round((ms % 60000) / 1000) + "s";
const fmtAge = ts => {
  if (!ts) return "—";
  const s = (Date.now() - ts) / 1000;
  return s < 60 ? Math.round(s) + "s ago" : s < 3600 ? Math.round(s / 60) + "m ago"
    : s < 86400 ? Math.round(s / 3600) + "h ago" : new Date(ts).toLocaleString();
};
const pretty = t => { try { return JSON.stringify(JSON.parse(t), null, 2); } catch { return t; } };

async function loadTraces() {
  const traces = await fetch("/api/traces").then(r => r.json()).catch(() => []);
  const el = document.getElementById("trace-list");
  if (!traces.length) return;
  el.innerHTML = traces.map(t => `
    <div class="trace ${t.trace_id === selectedTrace ? "selected" : ""}" onclick="selectTrace('${t.trace_id}')">
      <div class="row1">
        <span class="name">${esc(t.name)}</span>
        <span class="dot ${t.status}"></span>
      </div>
      <div class="meta">
        <span>${fmtDur(t.duration_ms)}</span><span>${t.run_count} runs</span>
        <span>${esc(t.session_name || "")}</span><span>${fmtAge(t.start_ts)}</span>
      </div>
    </div>`).join("");
  if (!selectedTrace && traces.length) selectTrace(traces[0].trace_id);
}

async function selectTrace(id) {
  selectedTrace = id;
  const detail = await fetch("/api/traces/" + id).then(r => r.json()).catch(() => null);
  if (!detail) return;
  runsCache = detail.runs;
  renderTree();
  document.querySelectorAll(".trace").forEach(e => e.classList.remove("selected"));
  loadTraces();
  if (!selectedRun || !runsCache.find(r => r.id === selectedRun)) {
    const root = runsCache.find(r => !r.parent_run_id);
    if (root) selectRun(root.id);
  } else renderDetail();
}

function treeOrder(runs) {
  const byParent = {};
  runs.forEach(r => (byParent[r.parent_run_id || ""] = byParent[r.parent_run_id || ""] || []).push(r));
  Object.values(byParent).forEach(l => l.sort((a, b) => (a.start_ts || 0) - (b.start_ts || 0)));
  const out = [];
  (function walk(pid, depth) {
    (byParent[pid] || []).forEach(r => { out.push([r, depth]); walk(r.id, depth + 1); });
  })("", 0);
  return out.length ? out : runs.map(r => [r, 0]);
}

function renderTree() {
  const el = document.getElementById("span-list");
  if (!runsCache.length) { el.innerHTML = '<div class="empty">no runs in trace</div>'; return; }
  const t0 = Math.min(...runsCache.map(r => r.start_ts || Infinity));
  const t1 = Math.max(...runsCache.map(r => r.end_ts || Date.now()), t0 + 1);
  el.innerHTML = treeOrder(runsCache).map(([r, depth]) => {
    const left = ((r.start_ts || t0) - t0) / (t1 - t0) * 100;
    const width = Math.max(0.8, (((r.end_ts || Date.now()) - (r.start_ts || t0)) / (t1 - t0)) * 100);
    const kind = (r.run_type || "chain").toLowerCase();
    const color = r.status === "error" ? "var(--err)" : `var(--${["llm","chain","tool","retriever"].includes(kind) ? kind : "chain"})`;
    return `
    <div class="span-row ${r.id === selectedRun ? "selected" : ""}" onclick="selectRun('${r.id}')">
      <div class="span-name" style="padding-left:${depth * 16}px">
        <span class="kind ${kind}">${esc(kind)}</span><em>${esc(r.name)}</em>
      </div>
      <div class="bar-lane"><div class="bar" style="left:${left}%;width:${width}%;background:${color}"></div></div>
      <div class="dur">${fmtDur(r.duration_ms)}</div>
    </div>`;
  }).join("");
}

function selectRun(id) { selectedRun = id; renderTree(); renderDetail(); }

function renderDetail() {
  const r = runsCache.find(x => x.id === selectedRun);
  const el = document.getElementById("detail");
  if (!r) { el.innerHTML = '<div class="empty">select a run</div>'; return; }
  el.innerHTML = `
    <div class="field"><div class="label">Run</div><div>${esc(r.name)}
      <span class="status-chip ${r.status}">${r.status}</span></div></div>
    <div class="field"><div class="label">Timing</div>
      <div>${esc(r.start_time || "—")}<br>→ ${esc(r.end_time || "still running")} (${fmtDur(r.duration_ms)})</div></div>
    ${r.error ? `<div class="field"><div class="label">Error</div><pre class="error">${esc(r.error)}</pre></div>` : ""}
    <div class="field"><div class="label">Inputs</div><pre>${esc(pretty(r.inputs) || "—")}</pre></div>
    <div class="field"><div class="label">Outputs</div><pre>${esc(pretty(r.outputs) || "—")}</pre></div>`;
}

loadTraces();
setInterval(loadTraces, 2500);
setInterval(() => { if (selectedTrace && runsCache.some(r => !r.end_time)) selectTrace(selectedTrace); }, 1500);
</script>
</body>
</html>
"""


@app.get("/")
async def dashboard():
    return HTMLResponse(DASHBOARD_HTML)


# ── entry point ──────────────────────────────────────────────────────────────

def main():
    url = f"http://127.0.0.1:{PORT}"
    mirroring = "on → smith.langchain.com" if LANGSMITH else "off (local only)"
    print(f"mueller observability — LangSmith-compatible local trace server")
    print(f"  Dashboard:        {url}")
    print(f"  Ingest:           {url}/runs   (LangSmith runs protocol, no API key)")
    print(f"  Store:            {DB_PATH}")
    print(f"  Cloud mirroring:  {mirroring}")
    print(f"  Ctrl-C to stop\n")

    if not os.getenv("MUELLER_NO_BROWSER"):
        webbrowser.open(url)

    try:
        uvicorn.run(app, host="127.0.0.1", port=PORT, log_level="warning")
    finally:
        if LANGSMITH is not None:
            try:
                LANGSMITH.flush()  # drain the SDK's background queue
            except Exception:
                pass


if __name__ == "__main__":
    main()
