from fastapi import FastAPI, Depends, Header, HTTPException, Response
import sys
import platform
import sqlite3
import threading
from pydantic import BaseModel
import json
import os
import ctypes
from ctypes import c_char_p
import time

# Prometheus (optional)
try:
    from prometheus_client import Counter, Histogram, generate_latest, CONTENT_TYPE_LATEST
    _PROM_ENABLED = True
    MET_ANCHOR_REQ = Counter("panther_anchor_requests_total", "Total anchor requests")
    MET_ANCHOR_OK = Counter("panther_anchor_success_total", "Total successful anchor operations")
    MET_ANCHOR_LAT = Histogram("panther_anchor_latency_seconds", "Anchor latency (seconds)")
    MET_STATUS_REQ = Counter("panther_status_checks_total", "Total status check requests")
except Exception:
    _PROM_ENABLED = False

try:
    import pantherpy  # PyO3 module built with maturin
    _HAS_PYO3 = True
except Exception:
    pantherpy = None
    _HAS_PYO3 = False


class GenerateRequest(BaseModel):
    prompt: str


class EvaluateRequest(BaseModel):
    metric: str
    expected: str | None = None
    generated: str | None = None
    reference: str | None = None
    candidate: str | None = None
    text: str | None = None
    samples: list[str] | None = None


class BiasRequest(BaseModel):
    samples: list[str]


class ProviderConfig(BaseModel):
    type: str  # "openai" | "ollama"
    base_url: str | None = None
    model: str | None = None
    api_key: str | None = None


class ValidateRequest(BaseModel):
    prompt: str
    providers: list[ProviderConfig]
    guidelines_json: str | None = None

class ProofRequest(BaseModel):
    prompt: str
    providers: list[ProviderConfig]
    results: list[dict]
    guidelines_json: str | None = None
    salt: str | None = None


def _lib_names_for_platform() -> list[str]:
    system = platform.system().lower()
    if system.startswith("darwin") or system == "darwin":
        return ["libpanther_ffi.dylib"]
    if system.startswith("windows") or system == "windows":
        return ["panther_ffi.dll"]
    # default linux/unix
    return ["libpanther_ffi.so"]


def _load_rust_lib():
    paths = [
        # repo root (release/debug) and current working dir (release/debug)
        os.path.join(os.path.dirname(__file__), "..", "..", "target", "debug"),
        os.path.join(os.path.dirname(__file__), "..", "..", "target", "release"),
        os.path.join(os.getcwd(), "target", "debug"),
        os.path.join(os.getcwd(), "target", "release"),
    ]
    for base in paths:
        for name in _lib_names_for_platform():
            candidate = os.path.abspath(os.path.join(base, name))
            if not os.path.exists(candidate):
                continue
            try:
                lib = ctypes.CDLL(candidate)
                # signatures
                lib.panther_init.restype = ctypes.c_int
                lib.panther_generate.argtypes = [c_char_p]
                lib.panther_generate.restype = c_char_p
                # validation FFI (optional features)
                try:
                    lib.panther_validation_run_default.argtypes = [c_char_p]
                    lib.panther_validation_run_default.restype = c_char_p
                except Exception:
                    pass
                try:
                    lib.panther_validation_run_multi.argtypes = [c_char_p, c_char_p]
                    lib.panther_validation_run_multi.restype = c_char_p
                except Exception:
                    pass
                try:
                    lib.panther_validation_run_custom.argtypes = [c_char_p, c_char_p, c_char_p]
                    lib.panther_validation_run_custom.restype = c_char_p
                except Exception:
                    pass
                try:
                    lib.panther_validation_run_multi_with_proof.argtypes = [c_char_p, c_char_p]
                    lib.panther_validation_run_multi_with_proof.restype = c_char_p
                except Exception:
                    pass
                try:
                    lib.panther_proof_compute.argtypes = [c_char_p, c_char_p, c_char_p, c_char_p, c_char_p]
                    lib.panther_proof_compute.restype = c_char_p
                except Exception:
                    pass
                try:
                    lib.panther_proof_anchor_eth.argtypes = [c_char_p, c_char_p, c_char_p, c_char_p]
                    lib.panther_proof_anchor_eth.restype = c_char_p
                except Exception:
                    pass
                try:
                    lib.panther_proof_check_eth.argtypes = [c_char_p, c_char_p, c_char_p]
                    lib.panther_proof_check_eth.restype = c_char_p
                except Exception:
                    pass
                try:
                    lib.panther_proof_verify_local.argtypes = [c_char_p, c_char_p, c_char_p, c_char_p, c_char_p, c_char_p]
                    lib.panther_proof_verify_local.restype = ctypes.c_int
                except Exception:
                    pass
                try:
                    lib.panther_version_string.restype = c_char_p
                except Exception:
                    pass
                lib.panther_free_string.argtypes = [c_char_p]
                if lib.panther_init() == 0:
                    return lib
            except Exception:
                continue
    return None


_RUST = _load_rust_lib()


_AUDIT: list[dict] = []
_PROOF_HISTORY: list[dict] = []

# SQLite persistence (Stage 3 — optional)
_DB_PATH = os.getenv("PANTHER_SQLITE_PATH", os.path.abspath(os.path.join(os.getcwd(), "panther_proofs.db")))
_DB: sqlite3.Connection | None = None
_DB_LOCK = threading.Lock()

# Prometheus metrics (Stage 3)
try:
    from prometheus_client import Counter, Histogram, generate_latest, CONTENT_TYPE_LATEST
    _PROM_ENABLED = True
    MET_ANCHOR_REQ = Counter('panther_anchor_requests_total', 'Total anchor requests')
    MET_ANCHOR_OK = Counter('panther_anchor_success_total', 'Total successful anchor operations')
    MET_ANCHOR_LAT = Histogram('panther_anchor_latency_seconds', 'Anchor latency (seconds)')
    MET_STATUS_REQ = Counter('panther_status_checks_total', 'Total status check requests')
except Exception:
    _PROM_ENABLED = False


def _init_db():
    global _DB
    try:
        _DB = sqlite3.connect(_DB_PATH, check_same_thread=False)
        _DB.execute(
            """
            CREATE TABLE IF NOT EXISTS proof_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                ts INTEGER NOT NULL,
                action TEXT NOT NULL,
                hash TEXT,
                tx_hash TEXT,
                anchored INTEGER,
                explorer_url TEXT,
                contract_url TEXT
            )
            """
        )
        _DB.execute("CREATE INDEX IF NOT EXISTS idx_proof_history_hash ON proof_history(hash)")
        _DB.commit()
    except Exception:
        _DB = None


def _db_insert_event(ev: dict):
    if _DB is None:
        return
    try:
        with _DB_LOCK:
            _DB.execute(
                "INSERT INTO proof_history(ts, action, hash, tx_hash, anchored, explorer_url, contract_url) VALUES(?,?,?,?,?,?,?)",
                (
                    int(ev.get("ts", 0)),
                    str(ev.get("action", "")),
                    ev.get("hash"),
                    ev.get("tx_hash"),
                    1 if ev.get("anchored") else 0 if ev.get("anchored") is not None else None,
                    ev.get("explorer_url"),
                    ev.get("contract_url"),
                ),
            )
            _DB.commit()
    except Exception:
        pass


def _auth_guard(x_api_key: str | None = Header(default=None)):
    required = os.getenv("PANTHER_API_KEY")
    if required and (x_api_key or "") != required:
        raise HTTPException(status_code=401, detail="invalid api key")


def create_app() -> FastAPI:
    app = FastAPI(title="PantherSDK API")
    # Initialize optional SQLite DB
    _init_db()

    @app.get("/health")
    def health():
        return {"status": "ok", "rust": bool(_RUST or _HAS_PYO3), "pyo3": _HAS_PYO3}

    @app.post("/v1/generate")
    def generate(req: GenerateRequest, _auth=Depends(_auth_guard)):
        if _HAS_PYO3:
            try:
                if hasattr(pantherpy, "init"):
                    # safe to call multiple times; OnceCell guards in Rust
                    pantherpy.init()
                return pantherpy.generate(req.prompt)
            except Exception as e:
                return {"error": str(e)}
        if _RUST:
            s = _RUST.panther_generate(req.prompt.encode("utf-8"))
            try:
                data = ctypes.cast(s, c_char_p).value.decode("utf-8")
                return json.loads(data)
            finally:
                _RUST.panther_free_string(s)
        # Fallback (dev mode)
        return {"text": f"echo: {req.prompt}", "model": "python-fallback"}

    @app.post("/metrics/evaluate")
    def metrics_evaluate(req: EvaluateRequest, _auth=Depends(_auth_guard)):
        m = req.metric.lower()
        if _HAS_PYO3 and m == "bleu" and req.reference and req.candidate:
            try:
                return {"score": pantherpy.evaluate_bleu_py(req.reference, req.candidate)}
            except Exception as e:
                return {"error": str(e)}
        # Python fallbacks for simple metrics
        if m == "accuracy" and req.expected is not None and req.generated is not None:
            et = req.expected.split()
            gt = req.generated.split()
            n = max(len(et), len(gt)) or 1
            matches = sum(1 for a, b in zip(et, gt) if a == b)
            return {"score": matches / n}
        if m == "coherence" and req.text is not None:
            t = req.text.split()
            if len(t) < 2:
                return {"score": 1.0}
            bigrams = set()
            repeats = 0
            for i in range(len(t) - 1):
                bg = (t[i], t[i + 1])
                if bg in bigrams:
                    repeats += 1
                else:
                    bigrams.add(bg)
            return {"score": 1.0 - repeats / max(1, len(t) - 1)}
        if m == "diversity" and req.samples is not None:
            tokens = []
            types = set()
            for s in req.samples:
                for tok in s.split():
                    tokens.append(tok)
                    types.add(tok)
            if not tokens:
                return {"score": 0.0}
            return {"score": len(types) / len(tokens)}
        if m == "fluency" and req.text is not None:
            toks = req.text.split()
            if not toks:
                return {"score": 0.0}
            vowels = set("aeiouAEIOU")
            good = sum(1 for t in toks if any(ch in vowels for ch in t))
            return {"score": good / len(toks)}
        return {"error": "unsupported or missing fields"}

    @app.get("/metrics/history")
    def metrics_history(metric: str, _auth=Depends(_auth_guard)):
        if _HAS_PYO3:
            try:
                return json.loads(pantherpy.get_history_py(metric))
            except Exception as e:
                return {"error": str(e)}
        return []

    @app.post("/bias/analyze")
    def bias_analyze(req: BiasRequest, _auth=Depends(_auth_guard)):
        if _HAS_PYO3:
            try:
                return json.loads(pantherpy.detect_bias_py(req.samples))
            except Exception as e:
                return {"error": str(e)}
        # Fallback: simple counts
        def count_group(samples: list[str], toks: list[str]) -> int:
            c = 0
            for s in samples:
                low = s.lower()
                for t in toks:
                    c += low.count(t)
            return c
        male = count_group(req.samples, ["he", "him", "his"])
        female = count_group(req.samples, ["she", "her", "hers"])
        neutral = count_group(req.samples, ["they", "them", "their"])
        mx = max(male, female, neutral, 1)
        mn = min(male, female, neutral)
        score = (mx - mn) / mx
        return {"group_counts": {"male": male, "female": female, "neutral": neutral}, "bias_score": score}

    # --- Guidelines endpoints ---
    @app.get("/guidelines/default")
    def guidelines_default(_auth=Depends(_auth_guard)):
        try:
            repo_root = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
            path = os.path.join(repo_root, "crates", "panther-validation", "guidelines", "anvisa.json")
            with open(path, "r", encoding="utf-8") as f:
                return json.load(f)
        except Exception as e:
            return {"error": str(e)}

    # --- Validation (multi-provider) ---
    # Prometheus validation metrics
    try:
        from prometheus_client import Counter, Histogram  # type: ignore
        MET_VAL_REQ = Counter('panther_validation_requests_total', 'Total validation requests')
        MET_VAL_ERR = Counter('panther_validation_errors_total', 'Total validation errors')
        MET_VAL_ERR_L = Counter('panther_validation_errors_labeled_total', 'Validation errors by provider/category', ['provider','category'])
        MET_VAL_LAT = Histogram('panther_validation_latency_seconds', 'Validation latency (seconds)')
    except Exception:
        MET_VAL_REQ = MET_VAL_ERR = MET_VAL_LAT = MET_VAL_ERR_L = None  # type: ignore

    @app.post("/validation/run_multi")
    def validation_run_multi(req: ValidateRequest, _auth=Depends(_auth_guard)):
        import time as _t
        if MET_VAL_REQ: MET_VAL_REQ.inc()
        t0 = _t.time()
        providers = [
            {
                "type": p.type,
                "base_url": p.base_url,
                "model": p.model,
                **({"api_key": p.api_key} if p.api_key else {}),
            }
            for p in req.providers
        ]
        providers_json = json.dumps(providers)
        if _RUST and hasattr(_RUST, "panther_validation_run_multi"):
            try:
                prompt_c = req.prompt.encode("utf-8")
                prov_c = providers_json.encode("utf-8")
                if req.guidelines_json and hasattr(_RUST, "panther_validation_run_custom"):
                    g_c = req.guidelines_json.encode("utf-8")
                    s = _RUST.panther_validation_run_custom(prompt_c, prov_c, g_c)
                else:
                    s = _RUST.panther_validation_run_multi(prompt_c, prov_c)
                try:
                    data = ctypes.cast(s, c_char_p).value.decode("utf-8")
                    out = json.loads(data)
                finally:
                    _RUST.panther_free_string(s)
                # Audit (best-effort)
                try:
                    top = out[0]["adherence_score"] if isinstance(out, list) and out else None
                    _AUDIT.append({
                        "ts": int(__import__("time").time() * 1000),
                        "prompt": req.prompt[:120],
                        "providers": len(providers),
                        "top_score": top,
                    })
                    if len(_AUDIT) > 200:
                        del _AUDIT[: len(_AUDIT) - 200]
                    if MET_VAL_LAT: MET_VAL_LAT.observe(max(0.0, _t.time() - t0))
                    # errors: adherence 0 + raw_text JSON {error:{category,...}}
                    try:
                        errc = 0
                        for r in out:
                            if r.get("adherence_score") == 0 and isinstance(r.get("raw_text"), str):
                                txt = r["raw_text"].strip()
                                if txt.startswith("{"):
                                    errc += 1
                                    if MET_VAL_ERR_L:
                                        try:
                                            data = json.loads(txt)
                                            cat = data.get('error',{}).get('category','unknown')
                                            MET_VAL_ERR_L.labels(r.get('provider_name','unknown'), cat).inc()
                                        except Exception:
                                            MET_VAL_ERR_L.labels(r.get('provider_name','unknown'), 'unknown').inc()
                        if MET_VAL_ERR and errc > 0:
                            MET_VAL_ERR.inc(errc)
                    except Exception:
                        pass
                except Exception:
                    pass
                return out
            except Exception as e:
                if MET_VAL_LAT: MET_VAL_LAT.observe(max(0.0, _t.time() - t0))
                if MET_VAL_ERR: MET_VAL_ERR.inc()
                return {"error": str(e)}
        return {"error": "validation ffi unavailable"}

    # --- Proof compute ---
    @app.post("/proof/compute")
    def proof_compute(req: ProofRequest, _auth=Depends(_auth_guard)):
        providers = [
            {
                "type": p.type,
                "base_url": p.base_url,
                "model": p.model,
                **({"api_key": p.api_key} if p.api_key else {}),
            }
            for p in req.providers
        ]
        providers_json = json.dumps(providers)
        guidelines_json = req.guidelines_json
        if not guidelines_json:
            try:
                repo_root = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
                path = os.path.join(repo_root, "crates", "panther-validation", "guidelines", "anvisa.json")
                with open(path, "r", encoding="utf-8") as f:
                    guidelines_json = json.dumps(json.load(f))
            except Exception:
                guidelines_json = "[]"
        results_json = json.dumps(req.results)
        salt = (req.salt or "").encode("utf-8") if req.salt else None
        if _RUST and hasattr(_RUST, "panther_proof_compute"):
            s = _RUST.panther_proof_compute(
                req.prompt.encode("utf-8"),
                providers_json.encode("utf-8"),
                guidelines_json.encode("utf-8"),
                results_json.encode("utf-8"),
                salt or None,
            )
            try:
                data = ctypes.cast(s, c_char_p).value.decode("utf-8")
                return json.loads(data)
            finally:
                _RUST.panther_free_string(s)
        # Fallback: Python compute with sha3_512 and canonical JSON
        import hashlib
        def canon(obj):
            return json.dumps(obj, sort_keys=True, separators=(",", ":")).encode("utf-8")
        def hbytes(b: bytes) -> str:
            return hashlib.sha3_512(b).hexdigest()
        providers_hash = hbytes(canon(providers))
        gjson = json.loads(guidelines_json)
        guidelines_hash = hbytes(canon(gjson))
        input_bundle = {"prompt": req.prompt, "providers": providers, "guidelines": gjson, "salt": req.salt}
        input_hash = hbytes(canon(input_bundle))
        results_hash = hbytes(canon(req.results))
        combined_hash = hbytes((input_hash + results_hash).encode("utf-8"))
        return {
            "scheme": "panther-proof-v1",
            "input_hash": input_hash,
            "results_hash": results_hash,
            "combined_hash": combined_hash,
            "guidelines_hash": guidelines_hash,
            "providers_hash": providers_hash,
            "timestamp_ms": int(__import__("time").time() * 1000),
            "sdk_version": "python-api",
            "salt_present": bool(req.salt),
        }

    @app.get("/audit/logs")
    def audit_logs(_auth=Depends(_auth_guard)):
        return list(_AUDIT)

    # --- Proof anchoring (Stage 2) ---
    @app.post("/proof/anchor")
    def proof_anchor(body: dict, _auth=Depends(_auth_guard)):
 feature/ProofSeal
        if _PROM_ENABLED:
            MET_ANCHOR_REQ.inc()
        t0 = time.time()

        import time as _t
        if _PROM_ENABLED:
            MET_ANCHOR_REQ.inc()
        start = _t.time()
main
        h = body.get("hash") or body.get("combined_hash")
        if not h:
            return {"error": "missing 'hash' (combined_hash)"}
        rpc = os.getenv("PANTHER_ETH_RPC")
        ctr = os.getenv("PANTHER_PROOF_CONTRACT")
        key = os.getenv("PANTHER_ETH_PRIVKEY")
 feature/ProofSeal
        exp = os.getenv("PANTHER_EXPLORER_BASE")

        exp = os.getenv("PANTHER_EXPLORER_BASE")  # e.g., https://sepolia.etherscan.io
main
        if not (rpc and ctr and key):
            return {"error": "server not configured: set PANTHER_ETH_RPC, PANTHER_PROOF_CONTRACT, PANTHER_ETH_PRIVKEY"}
        if _RUST and hasattr(_RUST, "panther_proof_anchor_eth"):
            s = _RUST.panther_proof_anchor_eth(h.encode("utf-8"), rpc.encode("utf-8"), ctr.encode("utf-8"), key.encode("utf-8"))
            try:
                data = ctypes.cast(s, c_char_p).value.decode("utf-8")
                out = json.loads(data)
                tx = out.get("tx_hash")
                if exp and isinstance(tx, str):
                    out["explorer_url"] = f"{exp.rstrip('/')}/tx/{tx}"
feature/ProofSeal
                if _PROM_ENABLED and isinstance(tx, str):
                    MET_ANCHOR_OK.inc()

                # history event
                try:
                    _PROOF_HISTORY.append({
                        "ts": int(__import__("time").time() * 1000),
                        "action": "anchor",
                        "hash": h,
                        "tx_hash": tx,
                        "explorer_url": out.get("explorer_url"),
                    })
                    if len(_PROOF_HISTORY) > 500:
                        del _PROOF_HISTORY[: len(_PROOF_HISTORY) - 500]
                    _db_insert_event({
                        "ts": int(__import__("time").time() * 1000),
                        "action": "anchor",
                        "hash": h,
                        "tx_hash": tx,
                        "explorer_url": out.get("explorer_url"),
                    })
                except Exception:
                    pass
 main
                return out
            finally:
                _RUST.panther_free_string(s)
        if _PROM_ENABLED:
feature/ProofSeal
            MET_ANCHOR_LAT.observe(max(0.0, time.time() - t0))
            MET_ANCHOR_LAT.observe(max(0.0, _t.time() - start))
 main
        return {"error": "blockchain FFI unavailable"}

    @app.get("/proof/status")
    def proof_status(hash: str, _auth=Depends(_auth_guard)):
        if _PROM_ENABLED:
            MET_STATUS_REQ.inc()
        rpc = os.getenv("PANTHER_ETH_RPC")
        ctr = os.getenv("PANTHER_PROOF_CONTRACT")
        exp = os.getenv("PANTHER_EXPLORER_BASE")
        if not (rpc and ctr):
            return {"error": "server not configured: set PANTHER_ETH_RPC, PANTHER_PROOF_CONTRACT"}
        if _RUST and hasattr(_RUST, "panther_proof_check_eth"):
            s = _RUST.panther_proof_check_eth(hash.encode("utf-8"), rpc.encode("utf-8"), ctr.encode("utf-8"))
            try:
                data = ctypes.cast(s, c_char_p).value.decode("utf-8")
                out = json.loads(data)
                if exp and isinstance(ctr, str):
                    out["contract_url"] = f"{exp.rstrip('/')}/address/{ctr}"
 feature/ProofSeal

                # history event
                try:
                    _PROOF_HISTORY.append({
                        "ts": int(__import__("time").time() * 1000),
                        "action": "status",
                        "hash": hash,
                        "anchored": bool(out.get("anchored", False)),
                        "contract_url": out.get("contract_url"),
                    })
                    if len(_PROOF_HISTORY) > 500:
                        del _PROOF_HISTORY[: len(_PROOF_HISTORY) - 500]
                    _db_insert_event({
                        "ts": int(__import__("time").time() * 1000),
                        "action": "status",
                        "hash": hash,
                        "anchored": bool(out.get("anchored", False)),
                        "contract_url": out.get("contract_url"),
                    })
                except Exception:
                    pass
 main
                return out
            finally:
                _RUST.panther_free_string(s)
        return {"error": "blockchain FFI unavailable"}

    @app.get("/metrics")
    def metrics():
        if not _PROM_ENABLED:
            raise HTTPException(status_code=503, detail="prometheus not enabled")
feature/ProofSeal
        return Response(content=generate_latest(), media_type=CONTENT_TYPE_LATEST)  # type: ignore

        data = generate_latest()  # type: ignore
        from fastapi import Response
        return Response(content=data, media_type=CONTENT_TYPE_LATEST)

    @app.get("/proof/history")
    def proof_history(hash: str | None = None, limit: int = 100, _auth=Depends(_auth_guard)):
        # Prefer DB if available
        if _DB is not None:
            try:
                q = "SELECT ts, action, hash, tx_hash, anchored, explorer_url, contract_url FROM proof_history"
                params: list = []
                if hash:
                    q += " WHERE hash = ?"
                    params.append(hash)
                q += " ORDER BY ts DESC LIMIT ?"
                params.append(max(0, min(1000, limit)))
                with _DB_LOCK:
                    rows = list(_DB.execute(q, tuple(params)))
                out = []
                for ts, action, h, tx, an, ex, cu in rows:
                    out.append({
                        "ts": ts,
                        "action": action,
                        "hash": h,
                        "tx_hash": tx,
                        "anchored": bool(an) if an is not None else None,
                        "explorer_url": ex,
                        "contract_url": cu,
                    })
                return out
            except Exception as e:
                return {"error": str(e)}
        # Fallback in-memory
        try:
            items = list(_PROOF_HISTORY)
            if hash:
                items = [e for e in items if e.get("hash") == hash]
            items.sort(key=lambda e: e.get("ts", 0), reverse=True)
            return items[: max(0, min(1000, limit))]
        except Exception as e:
            return {"error": str(e)}
 main

    @app.post("/proof/verify")
    def proof_verify(body: dict, _auth=Depends(_auth_guard)):
        prompt = body.get("prompt", "")
        providers = body.get("providers", [])
        guidelines = body.get("guidelines_json")
        results = body.get("results", [])
        salt = body.get("salt")
        proof = body.get("proof") or {}
        if guidelines is None:
            try:
                repo_root = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
                path = os.path.join(repo_root, "crates", "panther-validation", "guidelines", "anvisa.json")
                with open(path, "r", encoding="utf-8") as f:
                    guidelines = json.dumps(json.load(f))
            except Exception:
                guidelines = "[]"
        providers_json = json.dumps(providers)
        results_json = json.dumps(results)
        proof_json = json.dumps(proof)
        if _RUST and hasattr(_RUST, "panther_proof_verify_local"):
            ok = _RUST.panther_proof_verify_local(
                prompt.encode("utf-8"),
                providers_json.encode("utf-8"),
                guidelines.encode("utf-8"),
                results_json.encode("utf-8"),
                (salt or "").encode("utf-8") if salt is not None else None,
                proof_json.encode("utf-8"),
            )
            return {"valid": bool(ok)}
        # Fallback: recompute proof and compare combined_hash
        try:
            def canon(obj):
                return json.dumps(obj, sort_keys=True, separators=(",", ":")).encode("utf-8")
            import hashlib
            providers_hash = hashlib.sha3_512(canon(providers)).hexdigest()
            gjson = json.loads(guidelines)
            guidelines_hash = hashlib.sha3_512(canon(gjson)).hexdigest()
            input_bundle = {"prompt": prompt, "providers": providers, "guidelines": gjson, "salt": salt}
            input_hash = hashlib.sha3_512(canon(input_bundle)).hexdigest()
            results_hash = hashlib.sha3_512(canon(results)).hexdigest()
            combined = hashlib.sha3_512((input_hash + results_hash).encode("utf-8")).hexdigest()
            return {"valid": (str(proof.get("combined_hash", "")).lower().replace("0x", "") == combined.lower())}
        except Exception as e:
            return {"error": str(e)}

    return app
