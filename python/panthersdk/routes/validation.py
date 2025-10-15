from fastapi import APIRouter, Depends
from pydantic import BaseModel
import json, ctypes
from ctypes import c_char_p
from ..core import get_rust, auth_guard

router = APIRouter()


class ProviderConfig(BaseModel):
    type: str
    base_url: str | None = None
    model: str | None = None
    api_key: str | None = None


class ValidateRequest(BaseModel):
    prompt: str
    providers: list[ProviderConfig]
    guidelines_json: str | None = None


@router.post("/validation/run_multi")
def validation_run_multi(req: ValidateRequest, _auth=Depends(auth_guard)):
    lib = get_rust()
    import time as _t
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
    if lib and hasattr(lib, "panther_validation_run_multi"):
        try:
            prompt_c = req.prompt.encode("utf-8")
            prov_c = providers_json.encode("utf-8")
            if req.guidelines_json and hasattr(lib, "panther_validation_run_custom"):
                g_c = req.guidelines_json.encode("utf-8")
                s = lib.panther_validation_run_custom(prompt_c, prov_c, g_c)
            else:
                s = lib.panther_validation_run_multi(prompt_c, prov_c)
            try:
                data = ctypes.cast(s, c_char_p).value.decode("utf-8")
                out = json.loads(data)
                return out
            finally:
                lib.panther_free_string(s)
        except Exception as e:
            return {"error": str(e)}
    return {"error": "validation ffi unavailable"}

