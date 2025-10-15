import os
import ctypes
from ctypes import c_char_p
import platform
import threading
import sqlite3
import json
from fastapi import Header, HTTPException

try:
    import pantherpy  # PyO3 module built with maturin
    _HAS_PYO3 = True
except Exception:
    pantherpy = None
    _HAS_PYO3 = False


def _lib_names_for_platform() -> list[str]:
    system = platform.system().lower()
    if system.startswith("darwin") or system == "darwin":
        return ["libpanther_ffi.dylib"]
    if system.startswith("windows") or system == "windows":
        return ["panther_ffi.dll"]
    return ["libpanther_ffi.so"]


def _load_rust_lib():
    paths = [
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
                lib.panther_init.restype = ctypes.c_int
                lib.panther_generate.argtypes = [c_char_p]
                lib.panther_generate.restype = c_char_p
                # Optionally present functions (guarded by features)
                for fname in [
                    "panther_validation_run_default",
                    "panther_validation_run_multi",
                    "panther_validation_run_custom",
                    "panther_validation_run_multi_with_proof",
                    "panther_proof_compute",
                    "panther_proof_anchor_eth",
                    "panther_proof_check_eth",
                    "panther_proof_verify_local",
                    "panther_version_string",
                    "panther_agent_run",
                    "panther_agent_start",
                    "panther_agent_poll",
                    "panther_agent_status",
                    "panther_agent_result",
                    "panther_metrics_rouge_l",
                    "panther_metrics_fact_coverage",
                    "panther_metrics_plagiarism",
                ]:
                    try:
                        fn = getattr(lib, fname)
                        # set defaults for pointer args and returns
                        if fname.startswith("panther_metrics_"):
                            fn.restype = ctypes.c_double
                        elif fname.startswith("panther_proof_verify_local"):
                            fn.restype = ctypes.c_int
                        else:
                            fn.restype = c_char_p
                    except Exception:
                        pass
                if lib.panther_init() == 0:
                    return lib
            except Exception:
                continue
    return None


_RUST = _load_rust_lib()


def get_rust():
    return _RUST


# SQLite persistence (Stage 3 — optional)
DB_PATH = os.getenv("PANTHER_SQLITE_PATH", os.path.abspath(os.path.join(os.getcwd(), "panther_proofs.db")))
DB: sqlite3.Connection | None = None
DB_LOCK = threading.Lock()


def init_db():
    global DB
    try:
        DB = sqlite3.connect(DB_PATH, check_same_thread=False)
        DB.execute(
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
        DB.execute("CREATE INDEX IF NOT EXISTS idx_proof_history_hash ON proof_history(hash)")
        DB.commit()
    except Exception:
        DB = None


def db_insert_event(ev: dict):
    if DB is None:
        return
    try:
        with DB_LOCK:
            DB.execute(
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
            DB.commit()
    except Exception:
        pass


def auth_guard(x_api_key: str | None = Header(default=None)):
    required = os.getenv("PANTHER_API_KEY")
    if required and (x_api_key or "") != required:
        raise HTTPException(status_code=401, detail="invalid api key")
