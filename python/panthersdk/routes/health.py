from fastapi import APIRouter, Depends, HTTPException
import os, json
from ..core import get_rust, auth_guard

router = APIRouter()


@router.get("/health")
def health():
    ok = bool(get_rust())
    try:
        import pantherpy  # type: ignore
        pyo3 = True
    except Exception:
        pyo3 = False
    return {"status": "ok", "rust": ok or pyo3, "pyo3": pyo3}


@router.get("/guidelines/default")
def guidelines_default(_auth=Depends(auth_guard)):
    try:
        repo_root = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", ".."))
        path = os.path.join(repo_root, "crates", "panther-validation", "guidelines", "anvisa.json")
        with open(path, "r", encoding="utf-8") as f:
            return json.load(f)
    except Exception as e:
        return {"error": str(e)}

