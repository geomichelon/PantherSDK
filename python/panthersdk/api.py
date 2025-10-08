from fastapi import FastAPI
from pydantic import BaseModel
import json
import os
import ctypes
from ctypes import c_char_p

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


def _load_rust_lib():
    paths = [
        # macOS dev build default
        os.path.join(os.path.dirname(__file__), "..", "..", "target", "debug", "libpanther_ffi.dylib"),
        os.path.join(os.getcwd(), "target", "debug", "libpanther_ffi.dylib"),
    ]
    for p in paths:
        if os.path.exists(p):
            try:
                lib = ctypes.CDLL(p)
                # signatures
                lib.panther_init.restype = ctypes.c_int
                lib.panther_generate.argtypes = [c_char_p]
                lib.panther_generate.restype = c_char_p
                lib.panther_free_string.argtypes = [c_char_p]
                if lib.panther_init() == 0:
                    return lib
            except Exception:
                continue
    return None


_RUST = _load_rust_lib()


def create_app() -> FastAPI:
    app = FastAPI(title="PantherSDK API")

    @app.get("/health")
    def health():
        return {"status": "ok", "rust": bool(_RUST or _HAS_PYO3), "pyo3": _HAS_PYO3}

    @app.post("/v1/generate")
    def generate(req: GenerateRequest):
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
    def metrics_evaluate(req: EvaluateRequest):
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
    def metrics_history(metric: str):
        if _HAS_PYO3:
            try:
                return json.loads(pantherpy.get_history_py(metric))
            except Exception as e:
                return {"error": str(e)}
        return []

    @app.post("/bias/analyze")
    def bias_analyze(req: BiasRequest):
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

    return app
