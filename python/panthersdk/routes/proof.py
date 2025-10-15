from fastapi import APIRouter, Depends, HTTPException
from pydantic import BaseModel
import os, json, ctypes, time
from ctypes import c_char_p
from ..core import get_rust, auth_guard, DB, DB_LOCK, db_insert_event

router = APIRouter()


class ProviderConfig(BaseModel):
    type: str
    base_url: str | None = None
    model: str | None = None
    api_key: str | None = None


class ProofRequest(BaseModel):
    prompt: str
    providers: list[ProviderConfig]
    results: list[dict]
    guidelines_json: str | None = None
    salt: str | None = None


@router.post("/proof/compute")
def proof_compute(req: ProofRequest, _auth=Depends(auth_guard)):
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
            repo_root = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", ".."))
            path = os.path.join(repo_root, "crates", "panther-validation", "guidelines", "anvisa.json")
            with open(path, "r", encoding="utf-8") as f:
                guidelines_json = json.dumps(json.load(f))
        except Exception:
            guidelines_json = "[]"
    results_json = json.dumps(req.results)
    salt = (req.salt or "").encode("utf-8") if req.salt else None
    lib = get_rust()
    if lib and hasattr(lib, "panther_proof_compute"):
        s = lib.panther_proof_compute(
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
            lib.panther_free_string(s)
    # Python fallback: compute with sha3_512 and canonical JSON
    try:
        import hashlib
        def canon(obj):
            return json.dumps(obj, sort_keys=True, separators=(",", ":")).encode("utf-8")
        def hbytes(b: bytes) -> str:
            return hashlib.sha3_512(b).hexdigest()
        providers = json.loads(providers_json)
        gjson = json.loads(guidelines_json)
        results = json.loads(results_json)
        providers_hash = hbytes(canon(providers))
        guidelines_hash = hbytes(canon(gjson))
        input_bundle = {"prompt": req.prompt, "providers": providers, "guidelines": gjson, "salt": req.salt}
        input_hash = hbytes(canon(input_bundle))
        results_hash = hbytes(canon(results))
        combined_hash = hbytes((input_hash + results_hash).encode("utf-8"))
        return {
            "scheme": "panther-proof-v1",
            "input_hash": input_hash,
            "results_hash": results_hash,
            "combined_hash": combined_hash,
            "guidelines_hash": guidelines_hash,
            "providers_hash": providers_hash,
            "timestamp_ms": int(time.time() * 1000),
            "sdk_version": "python-api",
            "salt_present": bool(req.salt),
        }
    except Exception as e:
        return {"error": str(e)}


@router.post("/proof/anchor")
def proof_anchor(body: dict, _auth=Depends(auth_guard)):
    import time as _t
    from time import time as now
    h = body.get("hash") or body.get("combined_hash")
    if not h:
        return {"error": "missing 'hash' (combined_hash)"}
    rpc = os.getenv("PANTHER_ETH_RPC")
    ctr = os.getenv("PANTHER_PROOF_CONTRACT")
    key = os.getenv("PANTHER_ETH_PRIVKEY")
    exp = os.getenv("PANTHER_EXPLORER_BASE")
    if not (rpc and ctr and key):
        return {"error": "server not configured: set PANTHER_ETH_RPC, PANTHER_PROOF_CONTRACT, PANTHER_ETH_PRIVKEY"}
    lib = get_rust()
    if lib and hasattr(lib, "panther_proof_anchor_eth"):
        s = lib.panther_proof_anchor_eth(h.encode("utf-8"), rpc.encode("utf-8"), ctr.encode("utf-8"), key.encode("utf-8"))
        try:
            data = ctypes.cast(s, c_char_p).value.decode("utf-8")
            out = json.loads(data)
            tx = out.get("tx_hash")
            if exp and isinstance(tx, str):
                out["explorer_url"] = f"{exp.rstrip('/')}/tx/{tx}"
            try:
                if DB is not None:
                    with DB_LOCK:
                        DB.execute(
                            "INSERT INTO proof_history(ts, action, hash, tx_hash, explorer_url) VALUES(?,?,?,?,?)",
                            (int(now()*1000), "anchor", h, tx, out.get("explorer_url")),
                        ); DB.commit()
            except Exception:
                pass
            return out
        finally:
            lib.panther_free_string(s)
    return {"error": "blockchain FFI unavailable"}


@router.get("/proof/status")
def proof_status(hash: str, _auth=Depends(auth_guard)):
    rpc = os.getenv("PANTHER_ETH_RPC"); ctr = os.getenv("PANTHER_PROOF_CONTRACT"); exp = os.getenv("PANTHER_EXPLORER_BASE")
    if not (rpc and ctr):
        return {"error": "server not configured: set PANTHER_ETH_RPC, PANTHER_PROOF_CONTRACT"}
    lib = get_rust()
    if lib and hasattr(lib, "panther_proof_check_eth"):
        s = lib.panther_proof_check_eth(hash.encode("utf-8"), rpc.encode("utf-8"), ctr.encode("utf-8"))
        try:
            data = ctypes.cast(s, c_char_p).value.decode("utf-8")
            out = json.loads(data)
            if exp and isinstance(ctr, str):
                out["contract_url"] = f"{exp.rstrip('/')}/address/{ctr}"
            try:
                if DB is not None:
                    with DB_LOCK:
                        DB.execute(
                            "INSERT INTO proof_history(ts, action, hash, anchored, contract_url) VALUES(?,?,?,?,?)",
                            (int(time.time()*1000), "status", hash, 1 if out.get("anchored") else 0, out.get("contract_url")),
                        ); DB.commit()
            except Exception:
                pass
            return out
        finally:
            lib.panther_free_string(s)
    return {"error": "blockchain FFI unavailable"}


@router.get("/proof/history")
def proof_history(hash: str | None = None, limit: int = 100, _auth=Depends(auth_guard)):
    if DB is not None:
        try:
            q = "SELECT ts, action, hash, tx_hash, anchored, explorer_url, contract_url FROM proof_history"
            params: list = []
            if hash:
                q += " WHERE hash = ?"; params.append(hash)
            q += " ORDER BY ts DESC LIMIT ?"; params.append(max(0, min(1000, limit)))
            with DB_LOCK:
                rows = list(DB.execute(q, tuple(params)))
            out = []
            for ts, action, h, tx, an, ex, cu in rows:
                out.append({"ts": ts, "action": action, "hash": h, "tx_hash": tx, "anchored": bool(an) if an is not None else None, "explorer_url": ex, "contract_url": cu})
            return out
        except Exception as e:
            return {"error": str(e)}
    return []


@router.post("/proof/verify")
def proof_verify(body: dict, _auth=Depends(auth_guard)):
    prompt = body.get("prompt", ""); providers = body.get("providers", []); guidelines = body.get("guidelines_json"); results = body.get("results", []); salt = body.get("salt"); proof = body.get("proof") or {}
    if guidelines is None:
        try:
            repo_root = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", ".."))
            path = os.path.join(repo_root, "crates", "panther-validation", "guidelines", "anvisa.json")
            with open(path, "r", encoding="utf-8") as f:
                guidelines = json.dumps(json.load(f))
        except Exception:
            guidelines = "[]"
    providers_json = json.dumps(providers); results_json = json.dumps(results); proof_json = json.dumps(proof)
    lib = get_rust()
    if lib and hasattr(lib, "panther_proof_verify_local"):
        ok = lib.panther_proof_verify_local(
            prompt.encode("utf-8"),
            providers_json.encode("utf-8"),
            guidelines.encode("utf-8"),
            results_json.encode("utf-8"),
            (salt or "").encode("utf-8") if salt is not None else None,
            proof_json.encode("utf-8"),
        )
        return {"valid": bool(ok)}
    # Fallback: recompute combined hash
    try:
        def canon(obj): return json.dumps(obj, sort_keys=True, separators=(",", ":")).encode("utf-8")
        import hashlib
        gjson = json.loads(guidelines)
        input_bundle = {"prompt": prompt, "providers": providers, "guidelines": gjson, "salt": salt}
        input_hash = hashlib.sha3_512(canon(input_bundle)).hexdigest()
        results_hash = hashlib.sha3_512(canon(results)).hexdigest()
        combined = hashlib.sha3_512((input_hash + results_hash).encode("utf-8")).hexdigest()
        return {"valid": (str(proof.get("combined_hash", "")).lower().replace("0x", "") == combined.lower())}
    except Exception as e:
        return {"error": str(e)}
