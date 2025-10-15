from fastapi import APIRouter, Depends, HTTPException, Response
import os, threading
from pydantic import BaseModel
import json
import time
from ..core import get_rust, auth_guard, DB, DB_LOCK, db_insert_event
from ctypes import c_char_p
import ctypes

router = APIRouter()


class AgentRunRequest(BaseModel):
    plan: dict | None = None
    input: dict
    async_run: bool | None = False


_AGENT_RUNS: dict[str, dict] = {}
_AGENT_LOCK = DB_LOCK  # reuse global lock for simplicity


@router.post("/agent/run")
def agent_run(req: AgentRunRequest, _auth=Depends(auth_guard)):
    plan = req.plan or {"type": "ValidateSealAnchor"}
    input_obj = req.input
    run_id = f"r{int(time.time()*1000)}-{os.getpid()}"

    def _exec_and_store():
        lib = get_rust()
        try:
            plan_s = json.dumps(plan)
            input_s = json.dumps(input_obj)
            if lib and hasattr(lib, "panther_agent_run"):
                s = lib.panther_agent_run(plan_s.encode("utf-8"), input_s.encode("utf-8"))
                try:
                    data = ctypes.cast(s, c_char_p).value.decode("utf-8")
                    out = json.loads(data)
                finally:
                    lib.panther_free_string(s)
            else:
                out = {"error": "agents FFI unavailable"}
        except Exception as e:
            out = {"error": str(e)}
        with _AGENT_LOCK:
            st = _AGENT_RUNS.get(run_id, {"status": "running", "events": []})
            st["status"] = "done"
            st["result"] = out
            try:
                st["events"] = list(out.get("events", []))
            except Exception:
                pass
            _AGENT_RUNS[run_id] = st

    with _AGENT_LOCK:
        _AGENT_RUNS[run_id] = {"status": "running", "events": []}

    if req.async_run:
        th = threading.Thread(target=_exec_and_store, daemon=True)
        th.start()
        return {"run_id": run_id, "status": "running"}
    else:
        _exec_and_store()
        with _AGENT_LOCK:
            return {"run_id": run_id, **_AGENT_RUNS.get(run_id, {})}


@router.get("/agent/status")
def agent_status(run_id: str, _auth=Depends(auth_guard)):
    with _AGENT_LOCK:
        st = _AGENT_RUNS.get(run_id)
    if st:
        return {"run_id": run_id, "status": st.get("status"), "done": st.get("status") == "done"}
    # Fallback to DB is not applicable for runs here; return 404
    raise HTTPException(status_code=404, detail="run not found")


@router.get("/agent/events")
def agent_events(run_id: str, _auth=Depends(auth_guard)):
    with _AGENT_LOCK:
        st = _AGENT_RUNS.get(run_id)
        if st:
            return {"run_id": run_id, "events": st.get("events", [])}
    raise HTTPException(status_code=404, detail="run not found")


@router.post("/agent/start")
def agent_start(req: AgentRunRequest, _auth=Depends(auth_guard)):
    plan = req.plan or {"type": "ValidateSealAnchor"}
    input_obj = req.input
    lib = get_rust()
    if lib and hasattr(lib, "panther_agent_start"):
        s = lib.panther_agent_start(json.dumps(plan).encode("utf-8"), json.dumps(input_obj).encode("utf-8"))
        try:
            data = ctypes.cast(s, c_char_p).value.decode("utf-8")
            return json.loads(data)
        finally:
            lib.panther_free_string(s)
    return {"error": "agents FFI unavailable"}


@router.get("/agent/poll")
def agent_poll(run_id: str, cursor: int = 0, _auth=Depends(auth_guard)):
    lib = get_rust()
    if lib and hasattr(lib, "panther_agent_poll"):
        s = lib.panther_agent_poll(run_id.encode("utf-8"), str(cursor).encode("utf-8"))
        try:
            data = ctypes.cast(s, c_char_p).value.decode("utf-8")
            return json.loads(data)
        finally:
            lib.panther_free_string(s)
    return {"error": "agents FFI unavailable"}


@router.get("/agent/events/stream")
def agent_events_stream(run_id: str, _auth=Depends(auth_guard)):
    # Incremental SSE using poll
    import time as _t
    from fastapi.responses import StreamingResponse

    def _gen():
        cursor = 0
        while True:
            try:
                res = agent_poll(run_id, cursor)  # type: ignore
                for ev in res.get("events", []):
                    yield f"data: {json.dumps(ev)}\n\n".encode()
                cursor = int(res.get("cursor", cursor))
                if res.get("done"):
                    break
            except Exception:
                pass
            yield b"event: ping\n\n"
            _t.sleep(0.2)
    return StreamingResponse(_gen(), media_type="text/event-stream")
