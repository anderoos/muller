// Tracing over the LangSmith runs REST protocol (POST /runs, PATCH /runs/{id}).
//
// The same protocol reaches two backends: the local open-source Phoenix
// dashboard (via the Python relay started by `mueller dashboard` — no API key)
// or hosted LangSmith (API key required). Which one is decided by config/env;
// the wire format never changes.
//
// HTTP posts happen on a dedicated worker thread so tracing never adds latency
// to the user-facing path — span calls just enqueue an event on a channel.
// reqwest's blocking client cannot run on a tokio runtime thread, which is the
// other reason the worker owns its own plain thread.
//
// A trace is a tree of runs linked by `trace_id` + `dotted_order` (the same
// scheme the LangSmith Python SDK uses: parent segments joined by '.', each
// segment being start-timestamp + run id).

use std::sync::mpsc::{self, Sender};
use std::thread::JoinHandle;

use chrono::Utc;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::config::MuellerConfig;

struct Backend {
    endpoint: String,
    api_key: Option<String>,
    project: String,
}

enum Event {
    Create(Value),
    Update(String, Value),
}

pub struct Span {
    id: String,
    dotted_order: String,
}

pub struct Tracer {
    tx: Option<Sender<Event>>,
    worker: Option<JoinHandle<()>>,
    project: String,
    trace_id: String,
    root: Option<Span>,
}

fn now_iso() -> String {
    Utc::now().format("%Y-%m-%dT%H:%M:%S%.6fZ").to_string()
}

fn dotted_segment(id: &str) -> String {
    format!("{}{}", Utc::now().format("%Y%m%dT%H%M%S%6fZ"), id)
}

// Config takes precedence; env vars (the LangSmith convention, shared with the
// Python layer) are the fallback. A keyless endpoint — LANGSMITH_ENDPOINT set
// with no api key — means the local relay.
fn resolve_backend(cfg: &MuellerConfig) -> Option<Backend> {
    let trim = |e: &str| e.trim_end_matches('/').to_string();

    if let Some(obs) = &cfg.observability {
        return Some(Backend {
            endpoint: trim(&obs.endpoint),
            api_key: obs.api_key.clone(),
            project: obs.project.clone(),
        });
    }

    let project = std::env::var("LANGSMITH_PROJECT").unwrap_or_else(|_| "mueller".to_string());

    if let Ok(api_key) = std::env::var("LANGSMITH_API_KEY")
        .or_else(|_| std::env::var("LANGCHAIN_API_KEY"))
    {
        let endpoint = std::env::var("LANGSMITH_ENDPOINT")
            .unwrap_or_else(|_| crate::config::LANGSMITH_ENDPOINT.to_string());
        return Some(Backend { endpoint: trim(&endpoint), api_key: Some(api_key), project });
    }

    if let Ok(endpoint) = std::env::var("LANGSMITH_ENDPOINT") {
        return Some(Backend { endpoint: trim(&endpoint), api_key: None, project });
    }

    None
}

fn spawn_worker(endpoint: String, api_key: Option<String>) -> (Sender<Event>, JoinHandle<()>) {
    let (tx, rx) = mpsc::channel::<Event>();
    let worker = std::thread::spawn(move || {
        let client = reqwest::blocking::Client::new();
        let mut warned = false;
        for event in rx {
            let request = match &event {
                Event::Create(body) => client.post(format!("{}/runs", endpoint)).json(body),
                Event::Update(id, body) => {
                    client.patch(format!("{}/runs/{}", endpoint, id)).json(body)
                }
            };
            let request = match &api_key {
                Some(key) => request.header("x-api-key", key),
                None => request,
            };
            // Observability must never break the CLI: warn once, then stay quiet.
            match request.send() {
                Ok(resp) if resp.status().is_success() => {}
                Ok(resp) => {
                    if !warned {
                        eprintln!(
                            "Warning: observability backend at {} returned {}",
                            endpoint,
                            resp.status()
                        );
                        warned = true;
                    }
                }
                Err(e) => {
                    if !warned {
                        eprintln!(
                            "Warning: could not reach observability backend at {} ({}). \
                            Is `mueller dashboard` running?",
                            endpoint, e
                        );
                        warned = true;
                    }
                }
            }
        }
    });
    (tx, worker)
}

impl Tracer {
    /// Starts a trace with a root run, or a no-op tracer when no observability
    /// backend is configured. Every other method is safe to call either way.
    pub fn start(name: &str, inputs: Value, cfg: &MuellerConfig) -> Tracer {
        let Some(backend) = resolve_backend(cfg) else {
            return Tracer { tx: None, worker: None, project: String::new(), trace_id: String::new(), root: None };
        };
        let project = backend.project;

        let id = Uuid::new_v4().to_string();
        let dotted_order = dotted_segment(&id);
        let (tx, worker) = spawn_worker(backend.endpoint, backend.api_key);

        let _ = tx.send(Event::Create(json!({
            "id": id,
            "trace_id": id,
            "dotted_order": dotted_order,
            "name": name,
            "run_type": "chain",
            "start_time": now_iso(),
            "inputs": inputs,
            "session_name": project,
        })));

        Tracer {
            tx: Some(tx),
            worker: Some(worker),
            project,
            trace_id: id.clone(),
            root: Some(Span { id, dotted_order }),
        }
    }

    /// Opens a child run under the trace root. Returns None when disabled.
    pub fn span(&self, name: &str, run_type: &str, inputs: Value) -> Option<Span> {
        let tx = self.tx.as_ref()?;
        let root = self.root.as_ref()?;
        let id = Uuid::new_v4().to_string();
        let dotted_order = format!("{}.{}", root.dotted_order, dotted_segment(&id));
        let _ = tx.send(Event::Create(json!({
            "id": id,
            "trace_id": self.trace_id,
            "dotted_order": dotted_order,
            "parent_run_id": root.id,
            "name": name,
            "run_type": run_type,
            "start_time": now_iso(),
            "inputs": inputs,
            "session_name": self.project,
        })));
        Some(Span { id, dotted_order })
    }

    pub fn end_span(&self, span: Option<Span>, outputs: Value) {
        if let (Some(tx), Some(span)) = (self.tx.as_ref(), span) {
            let _ = tx.send(Event::Update(span.id, json!({
                "end_time": now_iso(),
                "outputs": outputs,
                "trace_id": self.trace_id,
                "dotted_order": span.dotted_order,
            })));
        }
    }

    pub fn fail_span(&self, span: Option<Span>, error: &str) {
        if let (Some(tx), Some(span)) = (self.tx.as_ref(), span) {
            let _ = tx.send(Event::Update(span.id, json!({
                "end_time": now_iso(),
                "error": error,
                "trace_id": self.trace_id,
                "dotted_order": span.dotted_order,
            })));
        }
    }

    /// Closes the root run. The Drop impl flushes the queue afterwards.
    pub fn finish(mut self, outputs: Value) {
        if let (Some(tx), Some(root)) = (self.tx.as_ref(), self.root.take()) {
            let _ = tx.send(Event::Update(root.id, json!({
                "end_time": now_iso(),
                "outputs": outputs,
                "trace_id": self.trace_id,
                "dotted_order": root.dotted_order,
            })));
        }
    }

    pub fn finish_error(mut self, error: &str) {
        if let (Some(tx), Some(root)) = (self.tx.as_ref(), self.root.take()) {
            let _ = tx.send(Event::Update(root.id, json!({
                "end_time": now_iso(),
                "error": error,
                "trace_id": self.trace_id,
                "dotted_order": root.dotted_order,
            })));
        }
    }
}

// Dropping the sender ends the worker's receive loop; joining drains whatever
// is still queued so short-lived CLI invocations don't lose trailing events.
impl Drop for Tracer {
    fn drop(&mut self) {
        drop(self.tx.take());
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
