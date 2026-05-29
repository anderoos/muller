#!/usr/bin/env python3
"""
Embed PM methodology documents into a ChromaDB instance.

Documents are split by ## subheaders. Vectors are produced by
openai/text-embedding-3-large routed through OpenRouter.

Usage:
    python3 scripts/embed_methodology.py            # incremental
    python3 scripts/embed_methodology.py --force    # drop collection + re-embed all
"""
import os
import sys
import json
import hashlib
import argparse
import time
from pathlib import Path

import chromadb

SCRIPT_DIR     = Path(__file__).parent.resolve()
REPO_ROOT      = SCRIPT_DIR.parent
METHODOLOGY_DIR = REPO_ROOT / "methodology"
CACHE_FILE     = METHODOLOGY_DIR / ".embeddings_cache.json"
EXPORT_FILE    = METHODOLOGY_DIR / "embeddings.json"

CHROMA_HOST    = "localhost"
CHROMA_PORT    = 8000
COLLECTION_NAME = "pm_methodology"
EMBED_MODEL    = "openai/text-embedding-3-large"


# ---------------------------------------------------------------------------
# OpenRouter embedding function
# ---------------------------------------------------------------------------

class OpenRouterEmbeddingFunction:
    """Custom ChromaDB embedding function backed by OpenRouter."""

    def __init__(self, api_key: str, model: str = EMBED_MODEL):
        from openai import OpenAI
        self._client = OpenAI(
            base_url="https://openrouter.ai/api/v1",
            api_key=api_key,
        )
        self._model = model

    def name(self) -> str:
        return f"openrouter-{self._model.replace('/', '-')}"

    def __call__(self, input: list) -> list:
        response = self._client.embeddings.create(
            input=input,
            model=self._model,
        )
        return [item.embedding for item in response.data]


def resolve_api_key() -> str:
    """Return the OpenRouter API key from env or ~/.mueller/config.json."""
    key = os.environ.get("OPENROUTER_API_KEY", "")
    if key:
        return key

    config_path = Path.home() / ".mueller" / "config.json"
    if config_path.exists():
        cfg = json.loads(config_path.read_text(encoding="utf-8"))
        emb = cfg.get("embedding") or {}
        if emb.get("provider") == "OpenRouter":
            key = emb.get("api_key", "")
            if key:
                return key

    print(
        "Error: OPENROUTER_API_KEY not found.\n"
        "Set the env var or run `mueller setup` to save it.",
        file=sys.stderr,
    )
    sys.exit(1)


# ---------------------------------------------------------------------------
# Markdown → chunks (subheader splitting — no LLM needed)
# ---------------------------------------------------------------------------

def split_by_subheaders(content: str, source: str) -> list:
    lines = content.splitlines()

    doc_title = ""
    for line in lines:
        if line.startswith("# ") and not line.startswith("## "):
            doc_title = line[2:].strip()
            break

    chunks: list = []
    current_header = None
    current_body:  list = []

    def flush():
        if current_header is None:
            return
        body = "\n".join(current_body).strip()
        if not body:
            return
        chunks.append({
            "text":     f"# {doc_title}\n## {current_header}\n\n{body}",
            "section":  current_header,
            "document": doc_title,
            "concepts": [w.strip("():/") for w in current_header.split() if len(w) > 3],
        })

    for line in lines:
        if line.startswith("## "):
            flush()
            current_header = line[3:].strip()
            current_body   = []
        elif line.startswith("# ") and not line.startswith("## "):
            pass  # H1 title captured above
        else:
            current_body.append(line)

    flush()

    return chunks or [{"text": content, "section": source, "document": source, "concepts": []}]


# ---------------------------------------------------------------------------
# ChromaDB helpers
# ---------------------------------------------------------------------------

def wait_for_chroma(retries: int = 20, delay: float = 1.0) -> chromadb.HttpClient:
    for attempt in range(retries):
        try:
            client = chromadb.HttpClient(host=CHROMA_HOST, port=CHROMA_PORT)
            client.heartbeat()
            return client
        except Exception:
            if attempt == retries - 1:
                print(f"Error: ChromaDB not reachable at {CHROMA_HOST}:{CHROMA_PORT}", file=sys.stderr)
                sys.exit(1)
            time.sleep(delay)


# ---------------------------------------------------------------------------
# Cache helpers
# ---------------------------------------------------------------------------

def get_file_hash(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load_cache() -> dict:
    if CACHE_FILE.exists():
        return json.loads(CACHE_FILE.read_text(encoding="utf-8"))
    return {}


def save_cache(cache: dict):
    CACHE_FILE.write_text(json.dumps(cache, indent=2), encoding="utf-8")


# ---------------------------------------------------------------------------
# Main pipeline
# ---------------------------------------------------------------------------

def embed_documents(force: bool = False):
    api_key = resolve_api_key()
    ef      = OpenRouterEmbeddingFunction(api_key)

    print("Connecting to ChromaDB…")
    chroma = wait_for_chroma()

    if force:
        try:
            chroma.delete_collection(COLLECTION_NAME)
            print(f"  dropped existing '{COLLECTION_NAME}' collection")
        except Exception:
            pass

    collection = chroma.get_or_create_collection(
        name=COLLECTION_NAME,
        embedding_function=ef,
        metadata={"hnsw:space": "cosine"},
    )

    cache  = load_cache()
    files  = sorted(METHODOLOGY_DIR.rglob("*.md"))

    if not files:
        print(f"No .md files found in {METHODOLOGY_DIR}", file=sys.stderr)
        sys.exit(1)

    new_cache:  dict = {}
    all_chunks: list = []
    any_changed = False

    for filepath in files:
        rel        = str(filepath.relative_to(METHODOLOGY_DIR))
        file_hash  = get_file_hash(filepath)
        cached     = cache.get(rel, {})

        if not force and cached.get("hash") == file_hash:
            print(f"  skip (unchanged): {rel}")
            new_cache[rel] = cached
            all_chunks.extend(cached.get("chunks", []))
            continue

        any_changed = True
        content = filepath.read_text(encoding="utf-8")
        chunks  = split_by_subheaders(content, rel)
        print(f"  embedding: {rel}  ({len(chunks)} sections)")

        ids, docs, metas, records = [], [], [], []
        for i, chunk in enumerate(chunks):
            doc_id = f"{rel}::{i}::{chunk['section']}"
            ids.append(doc_id)
            docs.append(chunk["text"])
            metas.append({
                "source":   rel,
                "document": chunk["document"],
                "section":  chunk["section"],
                "concepts": ", ".join(chunk["concepts"]),
            })
            record = {"id": doc_id, **chunk, "source": rel}
            records.append(record)
            all_chunks.append(record)

        collection.upsert(ids=ids, documents=docs, metadatas=metas)
        new_cache[rel] = {"hash": file_hash, "chunks": records}

    if any_changed or force:
        save_cache(new_cache)
        _write_export(all_chunks)
    else:
        print("\n  All files up to date.")


def _write_export(chunks: list):
    by_doc: dict = {}
    for c in chunks:
        by_doc.setdefault(c["document"], []).append({
            "id":       c["id"],
            "section":  c["section"],
            "concepts": c.get("concepts", []),
            "source":   c["source"],
            "preview":  c["text"][:120].replace("\n", " ") + "…",
        })

    export = {
        "collection":   COLLECTION_NAME,
        "embed_model":  EMBED_MODEL,
        "total_chunks": len(chunks),
        "documents": [
            {"title": title, "sections": sections}
            for title, sections in sorted(by_doc.items())
        ],
    }

    EXPORT_FILE.write_text(json.dumps(export, indent=2), encoding="utf-8")

    print(f"\n  {len(chunks)} chunks → {EXPORT_FILE.relative_to(REPO_ROOT)}")
    print(f"  model: {EMBED_MODEL}\n")
    print(f"  {'Document':<40} {'Sections':>8}")
    print(f"  {'-'*40} {'-'*8}")
    for doc in export["documents"]:
        print(f"  {doc['title']:<40} {len(doc['sections']):>8}")
    print(f"  {'TOTAL':<40} {len(chunks):>8}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Embed PM methodology docs into ChromaDB via OpenRouter")
    parser.add_argument("--force", action="store_true", help="Drop collection and re-embed all files")
    args = parser.parse_args()

    print(f"Mueller Embeddings {'(force)' if args.force else '(incremental)'}")
    embed_documents(force=args.force)
