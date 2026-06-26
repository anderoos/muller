# Muller Project Management Tool

## Description
_______________
George Muller was an associate administrator for NASA who kept everyone on track to ensure the Apollo program was on schedule to land on the moon. Referred to as the father of the Apollo, he saved money, time and resources. More time come on this project. 


## WIP Architecture
________________
I'm starting to realize this project is just turning into a RAG with fancy MCP connections so I 
#### Observability 
LangSmith — local LangSmith-compatible dashboard (`mueller dashboard`), with optional mirroring to smith.langchain.com via the open-source `langsmith` Python SDK

#### Database
ChromaDB

#### Agents
Langraph

#### Sparse search
BM25

Docs > Chunking > VectorDB 

Query > Vector/ Keyword Search > Query Refinement > Rewrite and retrieve > evaluate output 


Python - stateless retrieval and evaluation
Rust - transport layer

## Observability
________________
Every agent command is traced in three layers — **prompt refinement → agent processing → output** — using the LangSmith runs protocol.

```bash
pip install -r scripts/requirements.txt   # once
mueller dashboard                         # serves http://127.0.0.1:6007
```

`mueller dashboard` starts a local, open-source, LangSmith-compatible trace server (Python + SQLite at `~/.mueller/traces.db`) with a built-in dashboard: trace list, run-tree waterfall, per-run inputs/outputs/errors. The Rust CLI sends traces there with no API key.

If `observability.langsmith_api_key` is set in `~/.mueller/config.json` (or `LANGSMITH_API_KEY` is exported), the server also mirrors every trace to hosted LangSmith (smith.langchain.com) through the open-source `langsmith` Python SDK.

Python code joins the same traces with the standard SDK env vars:

```bash
LANGSMITH_TRACING=true LANGSMITH_ENDPOINT=http://127.0.0.1:6007 python your_script.py
```

Backends are configured under `mueller setup` → Observability (local dashboard, LangSmith cloud direct, or off).